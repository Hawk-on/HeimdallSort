use std::path::{Path, PathBuf};
use std::fs;
use crate::services::metadata;
use chrono::Datelike;
use serde::{Serialize, Deserialize};
use trash;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub processed: usize,
    pub success: usize,
    pub errors: usize,
    pub error_messages: Vec<String>,
}

impl OperationResult {
    pub fn new() -> Self {
        Self {
            processed: 0,
            success: 0,
            errors: 0,
            error_messages: Vec::new(),
        }
    }

    pub fn add_success(&mut self) {
        self.success += 1;
    }

    pub fn add_error(&mut self, msg: String) {
        self.errors += 1;
        self.error_messages.push(msg);
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SortConfig {
    pub use_day_folder: bool,
    pub use_month_names: bool,
}

pub fn sort_images(
    paths: Vec<String>,
    target_dir: &str,
    method: &str, // "copy" eller "move"
    config: SortConfig
) -> OperationResult {
    let mut result = OperationResult::new();
    result.processed = paths.len();
    let target_path = Path::new(target_dir);

    if !target_path.exists() {
        result.add_error(format!("Målmappen finnes ikke: {}", target_dir));
        return result;
    }

    let month_names = [
        "Januar", "Februar", "Mars", "April", "Mai", "Juni",
        "Juli", "August", "September", "Oktober", "November", "Desember"
    ];

    for path_str in paths {
        let source_path = Path::new(&path_str);
        
        if !source_path.exists() {
             result.add_error(format!("Fil finnes ikke: {}", path_str));
             continue;
        }

        // VIKTIG: Endret etter brukerønske. Alltid strict mode (ingen fallback til mtime).
        let date_opt = metadata::read_creation_date_with_fallback(source_path, false);

        let dest_dir = match date_opt {
            Some(date) => {
                let year = date.year();
                let month = date.month();
                let day = date.day();

                let month_folder = if config.use_month_names {
                    format!("{:02} - {}", month, month_names[(month - 1) as usize])
                } else {
                    format!("{:02}", month)
                };

                let mut dir = target_path.join(format!("{}", year)).join(month_folder);
                
                if config.use_day_folder {
                    dir = dir.join(format!("{:02}", day));
                }
                dir
            },
            None => {
                // Ingen dato funnet -> "Uten dato" mappe
                target_path.join("Uten dato")
            }
        };
        


        if let Err(e) = fs::create_dir_all(&dest_dir) {
             result.add_error(format!("Kunne ikke opprette mappe {:?}: {}", dest_dir, e));
             continue;
        }

        let filename = source_path.file_name().unwrap_or_default();
        let mut dest_path = dest_dir.join(filename);

        // Håndter filnavn-kollisjoner: img.jpg -> img_1.jpg
        let mut counter = 1;
        while dest_path.exists() {
            let stem = source_path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = source_path.extension().unwrap_or_default().to_string_lossy();
            let new_filename = if ext.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, ext)
            };
            dest_path = dest_dir.join(new_filename);
            counter += 1;
        }

        let op_result = if method == "move" {
            fs::rename(source_path, &dest_path)
        } else {
            fs::copy(source_path, &dest_path).map(|_| ())
        };

        match op_result {
            Ok(_) => {
                result.add_success();
                
                // Håndter sidecar-filer (kun hvis hovedfil ble flyttet/kopiert OK)
                let sidecars = crate::services::sidecar::find_sidecars(source_path);
                for sidecar in sidecars {
                    // Bestem nytt navn for sidecar basert på dest_path (for å matche evt rename av hovedfil)
                    if let Some(sidecar_ext) = sidecar.extension() {
                         let sidecar_ext_str = sidecar_ext.to_string_lossy();
                         
                         let sidecar_filename_original = sidecar.file_name().unwrap_or_default().to_string_lossy();
                         let source_filename_original = source_path.file_name().unwrap_or_default().to_string_lossy();
                         
                         let dest_sidecar_path = if sidecar_filename_original.starts_with(&*source_filename_original) {
                             // Case: image.jpg.json (sidecar inneholder hele originalnavnet)
                             // Da bør vi bygge nytt navn basert på dest_path filnavn + extension
                             let dest_filename = dest_path.file_name().unwrap_or_default().to_string_lossy();
                             dest_dir.join(format!("{}.{}", dest_filename, sidecar_ext_str))
                         } else {
                             // Case: image.xmp (sidecar har bare samme stem)
                             dest_path.with_extension(&*sidecar_ext_str)
                         };

                         if method == "move" {
                             let _ = fs::rename(&sidecar, &dest_sidecar_path);
                         } else {
                             let _ = fs::copy(&sidecar, &dest_sidecar_path);
                         }
                    }
                }
            },
            Err(e) => result.add_error(format!("Kunne ikke {} fil {}: {}", method, path_str, e)),
        }
    }

    result
}

pub fn delete_images(paths: Vec<String>) -> OperationResult {
    let mut result = OperationResult::new();
    result.processed = paths.len();

    for path_str in paths {
        let path = Path::new(&path_str);
        if !path.exists() {
             result.add_error(format!("Fil finnes ikke: {}", path_str));
             continue;
        }

        // Prøv å bruke trash først
        match trash::delete(path) {
            Ok(_) => {
                result.add_success();
                // Slett også sidecars
                let sidecars = crate::services::sidecar::find_sidecars(path);
                for sidecar in sidecars {
                    let _ = trash::delete(sidecar); // Ignorer feil for sidecars
                }
            },
            Err(e) => {
                // Hvis trash feiler, logg feilen - vi sletter IKKE permanent automatisk som fallback
                // for sikkerhets skyld.
                result.add_error(format!("Kunne ikke flytte til papirkurv: {}. Permanent sletting ikke utført av sikkerhetshensyn.", e));
            }
        }
    }
    result
}

pub fn move_images(paths: Vec<String>, target_dir: &str) -> OperationResult {
    let mut result = OperationResult::new();
    result.processed = paths.len();
    let target_path = Path::new(target_dir);

    // Klonet logikk fra sort_images (håndterer kollisjoner), uten dato-mappe opprettelse
    if !target_path.exists() {
         result.add_error(format!("Målmappen finnes ikke: {}", target_dir));
         return result;
    }

    for path_str in paths {
        let source_path = Path::new(&path_str);
        if !source_path.exists() {
            result.add_error(format!("Fil finnes ikke: {}", path_str));
            continue;
        }

        let filename = source_path.file_name().unwrap_or_default();
        let mut dest_path = target_path.join(filename);

        // Kollisjonshåndtering
        let mut counter = 1;
        while dest_path.exists() {
            let stem = source_path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = source_path.extension().unwrap_or_default().to_string_lossy();
             let new_filename = if ext.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, ext)
            };
            dest_path = target_path.join(new_filename);
            counter += 1;
        }

        match fs::rename(source_path, &dest_path) {
            Ok(_) => {
                result.add_success();
                
                // Håndter sidecar-filer
                let sidecars = crate::services::sidecar::find_sidecars(source_path);
                for sidecar in sidecars {
                     if let Some(sidecar_ext) = sidecar.extension() {
                         let sidecar_ext_str = sidecar_ext.to_string_lossy();
                         
                         let sidecar_filename_original = sidecar.file_name().unwrap_or_default().to_string_lossy();
                         let source_filename_original = source_path.file_name().unwrap_or_default().to_string_lossy();
                         
                         let dest_sidecar_path = if sidecar_filename_original.starts_with(&*source_filename_original) {
                             let dest_filename = dest_path.file_name().unwrap_or_default().to_string_lossy();
                             target_path.join(format!("{}.{}", dest_filename, sidecar_ext_str))
                         } else {
                             dest_path.with_extension(&*sidecar_ext_str)
                         };
                         
                         let _ = fs::rename(&sidecar, &dest_sidecar_path);
                    }
                }
            },
            Err(e) => result.add_error(format!("Kunne ikke flytte fil {}: {}", path_str, e)),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_dummy_file(dir: &Path, name: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        File::create(&path).unwrap();
        path
    }

    #[test]
    fn test_move_images() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");
        fs::create_dir(&source_dir).unwrap();
        fs::create_dir(&target_dir).unwrap();

        let file1 = create_dummy_file(&source_dir, "test1.jpg");
        let file2 = create_dummy_file(&source_dir, "test2.jpg");
        
        // Test move
        let paths = vec![
            file1.to_string_lossy().to_string(), 
            file2.to_string_lossy().to_string()
        ];
        
        let result = move_images(paths, target_dir.to_str().unwrap());
        
        assert_eq!(result.success, 2);
        assert_eq!(result.errors, 0);
        assert!(target_dir.join("test1.jpg").exists());
        assert!(target_dir.join("test2.jpg").exists());
        assert!(!source_dir.join("test1.jpg").exists());
    }

    #[test]
    fn test_move_images_collision() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();

        let src_file = create_dummy_file(&source, "image.jpg");
        let _existing = create_dummy_file(&target, "image.jpg"); // Create collision

        let result = move_images(
            vec![src_file.to_string_lossy().to_string()], 
            target.to_str().unwrap()
        );

        assert!(target.join("image.jpg").exists());
        assert!(target.join("image_1.jpg").exists()); // Should be renamed
    }

    #[test]
    fn test_move_with_sidecar() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();

        let img = create_dummy_file(&source, "photo.jpg");
        let xmp = create_dummy_file(&source, "photo.xmp");
        
        // Test normal move
        move_images(vec![img.to_string_lossy().to_string()], target.to_str().unwrap());
        
        assert!(target.join("photo.jpg").exists());
        assert!(target.join("photo.xmp").exists());
        assert!(!source.join("photo.jpg").exists());
        assert!(!source.join("photo.xmp").exists());
    }

    #[test]
    fn test_move_with_sidecar_rename() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();

        // Create existing file in target to force rename
        create_dummy_file(&target, "photo.jpg");
        create_dummy_file(&target, "photo.xmp"); // Existing sidecar too

        let img = create_dummy_file(&source, "photo.jpg");
        let xmp = create_dummy_file(&source, "photo.xmp");
        
        // Move should rename both to photo_1.jpg and photo_1.xmp
        move_images(vec![img.to_string_lossy().to_string()], target.to_str().unwrap());
        
        assert!(target.join("photo_1.jpg").exists());
        assert!(target.join("photo_1.xmp").exists());
    }
    
    #[test]
    fn test_sort_google_json_sidecar_rename() {
         // Verify proper handling of "image.jpg.json" when main file is renamed to "image_1.jpg"
         // Expected result: "image_1.jpg.json"
         
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source");
        let target = temp_dir.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();
        
         // Create existing collision
        create_dummy_file(&target, "img.jpg");

        let img = create_dummy_file(&source, "img.jpg");
        let json = create_dummy_file(&source, "img.jpg.json");
        
        move_images(vec![img.to_string_lossy().to_string()], target.to_str().unwrap());
        
        // Main file renamed to img_1.jpg
        assert!(target.join("img_1.jpg").exists());
        // Sidecar should be img_1.jpg.json
        assert!(target.join("img_1.jpg.json").exists());
    }

    #[test]
    fn test_sort_no_exif_no_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");
        fs::create_dir(&source_dir).unwrap();
        fs::create_dir(&target_dir).unwrap();
        
        // Lag en fil uten EXIF (bare random bytes)
        let file_path = create_dummy_file(&source_dir, "no_exif.jpg");
        
        let paths = vec![file_path.to_string_lossy().to_string()];
        
        let config = SortConfig {
            use_day_folder: false,
            use_month_names: false,
        };
        
        let result = sort_images(paths, target_dir.to_str().unwrap(), "copy", config);
        
        assert_eq!(result.success, 1);
        
        // Skal ligge i "Uten dato" mappe
        let expected_path = target_dir.join("Uten dato").join("no_exif.jpg");
        assert!(expected_path.exists(), "Filen skal flyttes til 'Uten dato' mappe når EXIF mangler og fallback er av");
    }

    // Merk: Vi tester ikke delete_images med trash crate her da det krever GUI environment
    // og kan være flaky i test-miljøer.
    // Vi tester heller ikke move_images_collision her da den er dekket over.
    // Siste test: Collision i "Uten dato" mappe - kollisjonshåndtering er generell så det bør funke.
}
