//! Filskanner for å finne bilder i mapper

use std::path::Path;
use walkdir::WalkDir;

/// Representerer et bilde funnet under skanning
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub path: String,
    pub filename: String,
    pub extension: String,
    pub size_bytes: u64,
}

/// Støttede bildeformater
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif", "ico", "heic", "heif",
];

/// Sjekker om en filendelse er støttet
pub fn is_supported_extension(ext: &str) -> bool {
    SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

/// Skanner en mappe rekursivt og returnerer alle bilder
pub fn scan_directory(path: &str) -> Result<Vec<ImageInfo>, Box<dyn std::error::Error>> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(format!("Mappen finnes ikke: {}", path.display()).into());
    }

    if !path.is_dir() {
        return Err(format!("Stien er ikke en mappe: {}", path.display()).into());
    }

    let mut images = Vec::new();

    for entry in WalkDir::new(path).follow_links(true).into_iter().flatten() {
        let entry_path = entry.path();

        if entry_path.is_file() {
            if let Some(ext) = entry_path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();

                if SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
                    if let Ok(metadata) = entry.metadata() {
                        let filename = entry_path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();

                        images.push(ImageInfo {
                            path: entry_path.to_string_lossy().to_string(),
                            filename,
                            extension: ext_lower,
                            size_bytes: metadata.len(),
                        });
                    }
                }
            }
        }
    }

    Ok(images)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_supported_extensions() {
        assert!(SUPPORTED_EXTENSIONS.contains(&"jpg"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"jpeg"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"png"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"gif"));
        assert!(SUPPORTED_EXTENSIONS.contains(&"webp"));
        assert!(!SUPPORTED_EXTENSIONS.contains(&"txt"));
        assert!(!SUPPORTED_EXTENSIONS.contains(&"pdf"));
        assert!(!SUPPORTED_EXTENSIONS.contains(&"mp4"));
    }

    #[test]
    fn test_is_supported_extension() {
        assert!(is_supported_extension("jpg"));
        assert!(is_supported_extension("JPG")); // Case insensitive
        assert!(is_supported_extension("JPEG"));
        assert!(is_supported_extension("png"));
        assert!(!is_supported_extension("txt"));
        assert!(!is_supported_extension(""));
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let result = scan_directory("/nonexistent/path/12345");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("finnes ikke"));
    }

    #[test]
    fn test_scan_empty_directory() {
        let dir = tempdir().unwrap();
        let result = scan_directory(dir.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_scan_directory_with_images() {
        let dir = tempdir().unwrap();
        
        // Lag noen test-filer (ikke ekte bilder, men filene vil bli funnet)
        let jpg_path = dir.path().join("test.jpg");
        let png_path = dir.path().join("test.png");
        let txt_path = dir.path().join("test.txt");
        
        File::create(&jpg_path).unwrap().write_all(b"fake jpg").unwrap();
        File::create(&png_path).unwrap().write_all(b"fake png").unwrap();
        File::create(&txt_path).unwrap().write_all(b"text file").unwrap();
        
        let result = scan_directory(dir.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let images = result.unwrap();
        assert_eq!(images.len(), 2); // Kun jpg og png
        
        let filenames: Vec<&str> = images.iter().map(|i| i.filename.as_str()).collect();
        assert!(filenames.contains(&"test.jpg"));
        assert!(filenames.contains(&"test.png"));
        assert!(!filenames.contains(&"test.txt"));
    }

    #[test]
    fn test_scan_recursive() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        
        // Lag bilder i hovedmappe og undermappe
        File::create(dir.path().join("image1.jpg")).unwrap();
        File::create(subdir.join("image2.png")).unwrap();
        
        let result = scan_directory(dir.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2); // Begge bildene funnet
    }

    #[test]
    fn test_image_info_fields() {
        let dir = tempdir().unwrap();
        let jpg_path = dir.path().join("testfile.jpg");
        File::create(&jpg_path).unwrap().write_all(b"12345678").unwrap();
        
        let result = scan_directory(dir.path().to_str().unwrap());
        let images = result.unwrap();
        
        assert_eq!(images.len(), 1);
        let img = &images[0];
        
        assert_eq!(img.filename, "testfile.jpg");
        assert_eq!(img.extension, "jpg");
        assert_eq!(img.size_bytes, 8);
        assert!(img.path.ends_with("testfile.jpg"));
    }

    #[test]
    fn test_scan_file_not_directory() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        File::create(&file_path).unwrap();
        
        let result = scan_directory(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ikke en mappe"));
    }
}
