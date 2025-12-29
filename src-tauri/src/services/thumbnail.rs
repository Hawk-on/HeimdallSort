//! Thumbnail-generering og caching for galleri-visning
//!
//! Genererer thumbnails på forespørsel og cacher dem for raskere lasting.

// use image::GenericImageView;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Standard thumbnail-størrelse
pub const THUMBNAIL_SIZE: u32 = 200;

/// Henter eller genererer en thumbnail for et bilde
/// Returnerer stien til thumbnail-filen
pub fn get_or_create_thumbnail(
    image_path: &Path,
    cache_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    // Generer unik cache-nøkkel basert på filsti og mtime
    let cache_key = generate_cache_key(image_path)?;
    let thumbnail_path = cache_dir.join(format!("{}.jpg", cache_key));

    // Returner cached thumbnail hvis den finnes
    if thumbnail_path.exists() {
        return Ok(thumbnail_path);
    }

    // Sørg for at cache-mappen finnes
    fs::create_dir_all(cache_dir)?;

    let ext = image_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let video_extensions = ["mp4", "mov", "avi", "mkv", "webm", "wmv", "m4v"];

    if video_extensions.contains(&ext.as_str()) {
        generate_video_thumbnail(image_path, &thumbnail_path)?;
    } else {
        // Last og resize bildet (Opprinnelig logikk)
        let img = load_image(image_path)?;
        let thumbnail = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
        // Lagre som JPEG med god komprimering
        thumbnail.save(&thumbnail_path)?;
    }

    Ok(thumbnail_path)
}

fn generate_video_thumbnail(input: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::process::Command;
    
    // Bruk ffmpeg til å hente ut en frame
    // -y: overskriv
    // -ss: seek til 1 sekund (unngå svart start-frame)
    // -i: input
    // -vframes 1: kun ett bilde
    // -q:v 2: god kvalitet jpeg
    
    let status = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-ss", "00:00:01",
            "-i", input.to_str().unwrap_or_default(), // todo: handle formatting error?
            "-vframes", "1",
            "-q:v", "2",
            output.to_str().unwrap_or_default(),
        ])
        .status()?;

    if !status.success() {
        return Err("Feil ved generering av video-thumbnail (ffmpeg feilet)".into());
    }
    
    Ok(())
}

/// Laster et bilde fra fil
fn load_image(path: &Path) -> Result<image::DynamicImage, Box<dyn std::error::Error + Send + Sync>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let img = image::load_from_memory(&buffer)?;
    Ok(img)
}

/// Genererer en unik cache-nøkkel for et bilde basert på sti og mtime
pub fn generate_cache_key(path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata
        .modified()
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(0);

    let mut hasher = Sha256::new();
    hasher.update(path.to_string_lossy().as_bytes());
    hasher.update(mtime.to_le_bytes());
    let result = hasher.finalize();

    Ok(hex::encode(&result[..16])) // Bruk kun første 16 bytes for kortere filnavn
}

/// Sletter alle thumbnails i cache-mappen
pub fn clear_cache(cache_dir: &Path) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    if !cache_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;
    for entry in fs::read_dir(cache_dir)? {
        if let Ok(entry) = entry {
            if entry.path().extension().map(|e| e == "jpg").unwrap_or(false) {
                if fs::remove_file(entry.path()).is_ok() {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use tempfile::tempdir;

    /// Lager et test-bilde som kan lagres til disk
    fn create_test_image(width: u32, height: u32) -> image::DynamicImage {
        let mut img = RgbaImage::new(width, height);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = Rgba([
                (x % 256) as u8,
                (y % 256) as u8,
                ((x + y) % 256) as u8,
                255,
            ]);
        }
        image::DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_thumbnail_size_constant() {
        assert_eq!(THUMBNAIL_SIZE, 200);
    }

    #[test]
    fn test_generate_cache_key_deterministic() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let key1 = generate_cache_key(&test_file).unwrap();
        let key2 = generate_cache_key(&test_file).unwrap();

        assert_eq!(key1, key2, "Samme fil skal gi samme cache-nøkkel");
        assert_eq!(key1.len(), 32, "Cache-nøkkel skal være 32 hex-tegn (16 bytes)");
    }

    #[test]
    fn test_generate_cache_key_different_files() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let key1 = generate_cache_key(&file1).unwrap();
        let key2 = generate_cache_key(&file2).unwrap();

        assert_ne!(key1, key2, "Forskjellige filer skal gi forskjellige nøkler");
    }

    #[test]
    fn test_get_or_create_thumbnail() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().join("cache");
        let image_path = dir.path().join("test_image.png");

        // Lag et test-bilde
        let img = create_test_image(400, 400);
        img.save(&image_path).unwrap();

        // Generer thumbnail
        let result = get_or_create_thumbnail(&image_path, &cache_dir);
        assert!(result.is_ok());

        let thumbnail_path = result.unwrap();
        assert!(thumbnail_path.exists());
        assert!(thumbnail_path.to_string_lossy().ends_with(".jpg"));

        // Verifiser at thumbnail er mindre enn originalen
        let thumb_img = image::open(&thumbnail_path).unwrap();
        let (width, height) = thumb_img.dimensions();
        assert!(width <= THUMBNAIL_SIZE);
        assert!(height <= THUMBNAIL_SIZE);
    }

    #[test]
    fn test_thumbnail_caching() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().join("cache");
        let image_path = dir.path().join("test_image.png");

        let img = create_test_image(300, 300);
        img.save(&image_path).unwrap();

        // Første kall - genererer thumbnail
        let path1 = get_or_create_thumbnail(&image_path, &cache_dir).unwrap();
        let mtime1 = fs::metadata(&path1).unwrap().modified().unwrap();

        // Andre kall - skal returnere cached versjon (samme fil)
        let path2 = get_or_create_thumbnail(&image_path, &cache_dir).unwrap();
        let mtime2 = fs::metadata(&path2).unwrap().modified().unwrap();

        assert_eq!(path1, path2);
        assert_eq!(mtime1, mtime2, "Cached thumbnail skal ikke regenereres");
    }

    #[test]
    fn test_clear_cache_empty() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().join("nonexistent");

        let result = clear_cache(&cache_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_clear_cache_with_files() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().join("cache");
        fs::create_dir_all(&cache_dir).unwrap();

        // Lag noen test-thumbnails
        fs::write(cache_dir.join("thumb1.jpg"), "fake").unwrap();
        fs::write(cache_dir.join("thumb2.jpg"), "fake").unwrap();
        fs::write(cache_dir.join("other.txt"), "not a thumbnail").unwrap();

        let result = clear_cache(&cache_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2); // Kun .jpg filer slettet

        // Verifiser at .txt filen fortsatt finnes
        assert!(cache_dir.join("other.txt").exists());
    }

    #[test]
    fn test_thumbnail_maintains_aspect_ratio() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().join("cache");
        let image_path = dir.path().join("wide_image.png");

        // Lag et bredt bilde (800x200)
        let img = create_test_image(800, 200);
        img.save(&image_path).unwrap();

        let thumbnail_path = get_or_create_thumbnail(&image_path, &cache_dir).unwrap();
        let thumb_img = image::open(&thumbnail_path).unwrap();
        let (width, height) = thumb_img.dimensions();

        // Thumbnail skal bevare aspect ratio
        assert_eq!(width, THUMBNAIL_SIZE);
        assert!(height < width, "Bredt bilde skal gi thumbnail som er bredere enn det er høyt");
    }

    #[test]
    fn test_nonexistent_image() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().join("cache");
        let nonexistent = dir.path().join("does_not_exist.jpg");

        let result = get_or_create_thumbnail(&nonexistent, &cache_dir);
        assert!(result.is_err());
    }
}
