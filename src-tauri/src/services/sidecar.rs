//! Hjelpemodul for å håndtere sidecar-filer (metadata)
//! Støtter: .xmp, .aae, .json, .thm

use std::path::{Path, PathBuf};

const SIDECAR_EXTENSIONS: &[&str] = &["xmp", "aae", "json", "thm"];

/// Finner alle sidecar-filer som hører til gitte filsti
pub fn find_sidecars(image_path: &Path) -> Vec<PathBuf> {
    let mut sidecars = Vec::new();
    
    // Sjekk at vi har en gyldig filsti
    if let Some(stem) = image_path.file_stem() {
        if let Some(parent) = image_path.parent() {
            // Sjekk for hver støttet filendelse
            for ext in SIDECAR_EXTENSIONS {
                // Prøv med nøyaktig samme stem (image.jpg -> image.xmp)
                let sidecar_path = parent.join(format!("{}.{}", stem.to_string_lossy(), ext));
                if sidecar_path.exists() {
                     sidecars.push(sidecar_path);
                     continue; 
                }
                
                // Prøv med uppercase extension (image.XMP)
                let sidecar_path_upper = parent.join(format!("{}.{}", stem.to_string_lossy(), ext.to_uppercase()));
                if sidecar_path_upper.exists() {
                    sidecars.push(sidecar_path_upper);
                    continue;
                }
                
                // TODO: Google Photos JSON kan ha formatet image.jpg.json
                // Sjekk også "image.jpg.json" (fullt filnavn + ext)
                if let Some(filename) = image_path.file_name() {
                     let sidecar_path_full = parent.join(format!("{}.{}", filename.to_string_lossy(), ext));
                     if sidecar_path_full.exists() {
                         sidecars.push(sidecar_path_full);
                     }
                }
            }
        }
    }
    
    sidecars
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_find_sidecars_standard() {
        let dir = tempdir().unwrap();
        let image = dir.path().join("test.jpg");
        let xmp = dir.path().join("test.xmp");
        
        File::create(&image).unwrap();
        File::create(&xmp).unwrap();
        
        let sidecars = find_sidecars(&image);
        assert_eq!(sidecars.len(), 1);
        assert_eq!(sidecars[0], xmp);
    }

    #[test]
    fn test_find_sidecars_google_takeout() {
        let dir = tempdir().unwrap();
        let image = dir.path().join("IMG_1234.JPG");
        let json = dir.path().join("IMG_1234.JPG.json"); // Google Photos style
        
        File::create(&image).unwrap();
        File::create(&json).unwrap();
        
        let sidecars = find_sidecars(&image);
        assert_eq!(sidecars.len(), 1);
        assert_eq!(sidecars[0], json);
    }

    #[test]
    fn test_find_multiple_sidecars() {
        let dir = tempdir().unwrap();
        let image = dir.path().join("test.jpg");
        let xmp = dir.path().join("test.xmp");
        let aae = dir.path().join("test.aae");
        
        File::create(&image).unwrap();
        File::create(&xmp).unwrap();
        File::create(&aae).unwrap();
        
        let sidecars = find_sidecars(&image);
        assert_eq!(sidecars.len(), 2);
    }
}
