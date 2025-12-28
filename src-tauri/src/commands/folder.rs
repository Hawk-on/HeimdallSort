//! Kommandoer for mappehåndtering og duplikatdeteksjon

use crate::services::{hashing, scanner, thumbnail, sorter};
use crate::services::sorter::{OperationResult, SortConfig};
use crate::services::hashing::ComparableHash;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use crate::services::cache::HashCache;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub image_count: usize,
    pub total_size_bytes: u64,
    pub images: Vec<ImageInfo>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageWithHash {
    pub info: ImageInfo,
    pub hash: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub images: Vec<ImageInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateResult {
    pub groups: Vec<DuplicateGroup>,
    pub total_duplicates: usize,
    pub processed: usize,
    pub errors: usize,
}

/// Henter cache-mappe for thumbnails
/// Bruker systemets midlertidige mappe for OS-agnostisk støtte (Windows/Linux/macOS)
fn get_thumbnail_cache_dir() -> PathBuf {
    std::env::temp_dir().join("imagesorter-thumbnails")
}

/// Skanner en mappe og returnerer informasjon om bildene som ble funnet
#[tauri::command]
pub async fn scan_folder(path: String) -> Result<ScanResult, String> {
    let images = scanner::scan_directory(&path).map_err(|e| e.to_string())?;

    let total_size: u64 = images.iter().map(|img| img.size_bytes).sum();
    
    let image_infos: Vec<ImageInfo> = images
        .into_iter()
        .map(|img| ImageInfo {
            path: img.path,
            filename: img.filename,
            size_bytes: img.size_bytes,
        })
        .collect();

    Ok(ScanResult {
        image_count: image_infos.len(),
        total_size_bytes: total_size,
        images: image_infos,
    })
}

/// Henter eller genererer en thumbnail for et bilde
/// Returnerer stien til thumbnail-filen
#[tauri::command]
pub async fn get_thumbnail(path: String) -> Result<String, String> {
    let image_path = Path::new(&path);
    let cache_dir = get_thumbnail_cache_dir();
    
    let thumbnail_path = thumbnail::get_or_create_thumbnail(image_path, &cache_dir)
        .map_err(|e| e.to_string())?;
    
    Ok(thumbnail_path.to_string_lossy().to_string())
}

/// Åpner et bilde i standard bildeviser
#[tauri::command]
pub async fn open_image(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| e.to_string())
}

/// Finner duplikater blant gitte bildestier ved hjelp av perceptuell hashing
/// Optimalisert for store bildesamlinger med parallell prosessering
#[tauri::command]
pub async fn find_duplicates(app: tauri::AppHandle, paths: Vec<String>, threshold: u32) -> Result<DuplicateResult, String> {
    use tauri::Emitter;
    let error_count = Arc::new(Mutex::new(0usize));
    
    // --------------- STAGE 1: EXACT DUPLICATES (Rask filtrering) ---------------
    // Grupperer filer basert på størrelse først, så partial hash for kandidater.
    
    let app_handle = app.clone();
    let paths_len = paths.len();
    
    // 1.1 Samle filinfo (størrelse) raskt
    let mut file_sizes: HashMap<u64, Vec<String>> = HashMap::new();
    for path in &paths {
         if let Ok(metadata) = std::fs::metadata(path) {
             file_sizes.entry(metadata.len()).or_default().push(path.clone());
         }
    }
    
    // 1.2 Identifiser kandidater for eksakt match (samme størrelse)
    let potential_exact_dupes: Vec<String> = file_sizes
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .flat_map(|(_, files)| files)
        .collect();

    // 1.3 Beregn partial hash for kandidater parallelt
    let exact_dupe_cache = Arc::new(Mutex::new(HashMap::new()));
    let exact_pool = rayon::ThreadPoolBuilder::new().num_threads(16).build().unwrap();
    
    let potential_ids: Vec<String> = potential_exact_dupes.clone();
    
    exact_pool.install(|| {
        potential_ids.par_iter().for_each(|path_str| {
            let path = Path::new(path_str);
            if let Ok(p_hash) = hashing::compute_partial_hash(path) {
                 exact_dupe_cache.lock().unwrap().insert(path_str.clone(), p_hash);
            }
        });
    });
    
    // 1.4 Grupper eksakte duplikater
    let mut exact_groups: HashMap<String, Vec<ImageInfo>> = HashMap::new();
    let exact_cache_lock = exact_dupe_cache.lock().unwrap();
    
    for path_str in &potential_exact_dupes {
        if let Some(hash) = exact_cache_lock.get(path_str) {
             let path = Path::new(path_str);
             let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
             let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
             
             let output_key = format!("{}_{}", size, hash); // Unik nøkkel for eksakt gruppe
             
             exact_groups.entry(output_key).or_default().push(ImageInfo {
                 path: path_str.clone(),
                 filename,
                 size_bytes: size
             });
        }
    }
    
    // --------------- STAGE 2: VISUAL DUPLICATES (Perceptuell Hash) ---------------
    // For alle bilder som IKKE er en del av en eksakt gruppe (eller vi velger 1 representant fra hver eksakt gruppe)
    // kjører vi den tunge analysen.
    
    // Vi velger å kjøre visuell sjekk på ALLE unike bilder. 
    // Hvis vi har 3 eksakte kopier av Bilde A, trenger vi bare å visuelt sjekke én av dem mot Bilde B.
    
    let mut files_to_visual_scan: Vec<String> = Vec::new();
    let mut _handled_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Legg til unike filer (de som ikke var i potential_exact_dupes)
    let potential_set: std::collections::HashSet<_> = potential_exact_dupes.iter().collect();
    for path in &paths {
        if !potential_set.contains(path) {
            files_to_visual_scan.push(path.clone());
        }
    }
    
    // For eksakte grupper, legg til den første som representant
    for (_, group) in &exact_groups {
        if let Some(first) = group.first() {
            files_to_visual_scan.push(first.path.clone());
            // Marker alle i gruppen som 'håndtert' i første omgang, 
            // men vi må huske å merge dem tilbake i resultatet til slutt
        }
    }
    
    // Last inn cache for visuell hash
    let cache_dir = get_thumbnail_cache_dir();
    let cache = Arc::new(RwLock::new(HashCache::new(&cache_dir)));
    
    let visual_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(8)  // Lavere antall for å spare minne ved bilde-dekoding
        .build()
        .map_err(|e| format!("Kunne ikke starte trådpool: {}", e))?;

    let hashed_images: Vec<ImageWithHash> = visual_pool.install(|| {
        files_to_visual_scan
        .par_iter()
        .filter_map(|path_str| {
            let path = Path::new(path_str);
            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => {
                    *error_count.lock().unwrap() += 1;
                    return None;
                }
            };
            
            let mtime = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
            let size_bytes = metadata.len();
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();

            // Sjekk cache
            {
                let read_guard = cache.read().unwrap();
                if let Some(cached_hash_str) = read_guard.get(path_str, mtime) {
                    let _ = app_handle.emit("progress", serde_json::json!({ "tick": true }));
                    return Some(ImageWithHash {
                        info: ImageInfo { path: path_str.clone(), filename, size_bytes },
                        hash: cached_hash_str,
                    });
                }
            }

            // Beregn hash
            match hashing::load_image(path) {
                Ok(img) => {
                    match hashing::compute_perceptual_hash(&img, hashing::HashType::Difference) {
                        Ok(hash) => {
                            let hash_str = hash.to_base64();
                            {
                                let mut write_guard = cache.write().unwrap();
                                write_guard.insert(path_str.clone(), mtime, hash_str.clone());
                            }
                            let _ = app_handle.emit("progress", serde_json::json!({ "tick": true }));
                            Some(ImageWithHash {
                                info: ImageInfo { path: path_str.clone(), filename, size_bytes },
                                hash: hash_str,
                            })
                        }
                        Err(_) => {
                            *error_count.lock().unwrap() += 1;
                            None
                        }
                    }
                }
                Err(_) => {
                    *error_count.lock().unwrap() += 1;
                    None
                }
            }
        })
        .collect()
    });

    // Lagre cache
    if let Ok(read_guard) = cache.read() {
        let _ = read_guard.save();
    }
    
    // Bygg BK-Tree for visuelt søk
    let mut tree = bk_tree::BKTree::new(hashing::PerceptualMetric);
    let mut hash_to_indices: HashMap<ComparableHash, Vec<usize>> = HashMap::new();

    for (idx, img) in hashed_images.iter().enumerate() {
        if let Ok(hash) = img_hash::ImageHash::<Box<[u8]>>::from_base64(&img.hash) {
             let comp_hash = ComparableHash(hash);
             tree.add(comp_hash.clone());
             hash_to_indices.entry(comp_hash).or_default().push(idx);
        }
    }

    // Finn visuelle grupper
    let mut final_groups: Vec<Vec<ImageInfo>> = Vec::new();
    let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (i, img) in hashed_images.iter().enumerate() {
        if visited.contains(&i) { continue; }

        if let Ok(hash) = img_hash::ImageHash::<Box<[u8]>>::from_base64(&img.hash) {
            let comp_hash = ComparableHash(hash);
            let matches = tree.find(&comp_hash, threshold);
            
            let mut group_members: Vec<ImageInfo> = Vec::new();
            
            // Hvis vi finner matcher, må vi utvide resultatet med evt eksakte kopier
            // som vi filtrerte ut tidligere.
            for (_dist, found_hash) in matches {
                if let Some(indices) = hash_to_indices.get(found_hash) {
                    for &idx in indices {
                        if !visited.contains(&idx) {
                            visited.insert(idx);
                            
                            // 1. Legg til den visuelle matchen (representanten)
                            let rep = &hashed_images[idx];
                            group_members.push(rep.info.clone());
                            
                            // 2. Sjekk om denne representanten har eksakte kopier
                            // Vi må finne dem ved å søke gjennom exact_groups
                            // Dette er litt tregt (lineært søk), men antall grupper er forhåpentligvis håndterbart.
                            // Optimalisering: Kunne lagd en map: path -> group_id
                            
                            for group in exact_groups.values() {
                                // Hvis representanten finnes i en eksakt gruppe...
                                if group.iter().any(|g| g.path == rep.info.path) {
                                    // ...legg til resten av gruppen også
                                    for member in group {
                                        if member.path != rep.info.path {
                                            group_members.push(member.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if group_members.len() > 1 {
                final_groups.push(group_members);
            }
        }
    }
    
    // Legg til eventuelle "rene" eksakte grupper som ikke ble fanget opp av visuelt søk? 
    // (Det burde ikke skje, siden representanten er med i visuelt søk, og vil matche seg selv med distanse 0).
    
    let duplicate_groups: Vec<DuplicateGroup> = final_groups
        .into_iter()
        .map(|images| DuplicateGroup { images })
        .collect();

    let total_duplicates: usize = duplicate_groups.iter().map(|g| g.images.len() - 1).sum();
    let errors = *error_count.lock().unwrap();

    Ok(DuplicateResult {
        groups: duplicate_groups,
        total_duplicates,
        processed: paths_len,
        errors,
    })
}



/// Sorterer bilder basert på dato til en målsti (År/Måned)
#[tauri::command]
pub async fn sort_images_by_date(
    paths: Vec<String>,
    method: String, // "copy" eller "move"
    target_dir: String,
    options: Option<SortConfig>,
) -> Result<OperationResult, String> {
    
    let config = options.unwrap_or(SortConfig {
        use_day_folder: false,
        use_month_names: false,
    });

    let result = sorter::sort_images(paths, &target_dir, &method, config);
    Ok(result)
}

/// Sletter bilder (flytter til papirkurv hvis mulig)
#[tauri::command]
pub async fn delete_images(paths: Vec<String>) -> Result<OperationResult, String> {
    let result = sorter::delete_images(paths);
    Ok(result)
}

/// Flytter bilder til valgt mappe (uten datosortering)
#[tauri::command]
pub async fn move_images(paths: Vec<String>, target_dir: String) -> Result<OperationResult, String> {
    let result = sorter::move_images(paths, &target_dir);
    Ok(result)
}
