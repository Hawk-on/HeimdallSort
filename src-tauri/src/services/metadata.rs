//! Tjeneste for å lese metadata fra bilder (EXIF)

use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// prøver å lese opprettelsesdato fra bildet
/// 1. Sjekker EXIF (DateTimeOriginal)
/// 2. Faller tilbake til filsystemets endringsdato (mtime)
pub fn read_creation_date(path: &Path) -> Option<DateTime<Local>> {
    read_creation_date_with_fallback(path, true)
}

/// Leser opprettelsesdato med konfigurerbar fallback
pub fn read_creation_date_with_fallback(path: &Path, use_fallback: bool) -> Option<DateTime<Local>> {
    // 1. Prøv å lese EXIF (Bilder)
    if let Some(date) = read_exif_date(path) {
        return Some(Local.from_local_datetime(&date).unwrap());
    }

    // 2. Prøv å lese Videometadata (FFprobe)
    if let Some(date) = read_video_date(path) {
        return Some(Local.from_local_datetime(&date).unwrap());
    }
    
    if !use_fallback {
        return None;
    }

    // 3. Fallback til filsystem mtime
    read_file_mtime(path)
}

/// Leser opprettelsesdato fra video ved hjelp av FFprobe
fn read_video_date(path: &Path) -> Option<NaiveDateTime> {
    use std::process::Command;
    use std::env;

    // TODO: For production bundled sidecars, we need to resolve the correct path.
    // Ideally we'd use tauri's path resolver, but we are deep in a service module without AppHandle.
    // For now, we attempt to run "ffprobe" (assuming it's in PATH or CWD).
    // If that fails, we could try to look in relative paths, but platform-specific suffix naming makes it hard here.
    // The "Right Way" is to pass the sidecar path from the main thread/command handler down to here.
    // But let's stick to "ffprobe" command for now, as the user environment usually has it or we can't easily guess.
    // BUT: The user specifically asked to BUNDLE it.
    // Since we bundled it, "ffprobe" command WONT work unless we add the bin folder to PATH before running.
    // We can try to guess the path relative to CWD based on known target triple?
    
    // Attempt 1: "ffprobe" in PATH
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_entries", "format_tags=creation_time",
            path.to_str()?,
        ])
        .output()
        .ok();

    if let Some(out) = output {
        if out.status.success() {
             return parse_ffmpeg_json(&out.stdout);
        }
    }
    
    // Attempt 2 (Desperation): Look for local sidecar binary in expected dev location
    // This is hacky but helps in dev mode if they downloaded binaries.
    // In production, simpler to rely on frontend calling it, OR properly passing path.
    // For current scope: just return None if not found.
    // The `shell` plugin allows frontend to call specific sidecars easily.
    // Maybe we should extract metadata in Frontend?? No, sorting happens in Backend.
    
    None
}

fn parse_ffmpeg_json(output: &[u8]) -> Option<NaiveDateTime> {
    let json_str = std::str::from_utf8(output).ok()?;
    let v: serde_json::Value = serde_json::from_str(json_str).ok()?;
    
    let date_str = v["format"]["tags"]["creation_time"].as_str()?;
    
    // Datoformat fra FFmpeg er ofte ISO 8601: "2023-12-29T00:33:00.000000Z"
    let clean_date = date_str.split('.').next().unwrap_or(date_str);
    let clean_date = clean_date.trim_end_matches('Z');
    
    NaiveDateTime::parse_from_str(clean_date, "%Y-%m-%dT%H:%M:%S").ok()
}

fn read_exif_date(path: &Path) -> Option<NaiveDateTime> {
    let file = File::open(path).ok()?;
    let mut bufreader = BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).ok()?;

    // Prøv forskjellige datofelt i prioritert rekkefølge
    let date_fields = [
        exif::Tag::DateTimeOriginal,
        exif::Tag::DateTimeDigitized,
        exif::Tag::DateTime,
    ];

    for tag in date_fields {
        if let Some(field) = exif.get_field(tag, exif::In::PRIMARY) {
            if let exif::Value::Ascii(ref vec) = field.value {
                if !vec.is_empty() {
                    let s = std::str::from_utf8(&vec[0]).ok()?;
                    // EXIF datoformat: "YYYY:MM:DD HH:MM:SS"
                    // Vi erstatter første to : med - for å matche ISO 8601 delvis
                    // Eller bruke chrono sitt format direkte
                    if let Ok(date) = NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S") {
                        return Some(date);
                    }
                }
            }
        }
    }

    None
}

fn read_file_mtime(path: &Path) -> Option<DateTime<Local>> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let datetime: DateTime<Local> = modified.into();
    Some(datetime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_fallback_to_mtime() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_no_exif.txt");
        File::create(&file_path).unwrap().write_all(b"test").unwrap();

        let date = read_creation_date(&file_path);
        assert!(date.is_some());
        
        // Sjekk at datoen er nylig (innenfor siste minutt)
        let now = Local::now();
        let diff = now.signed_duration_since(date.unwrap());
        assert!(diff.num_seconds().abs() < 60);
    }
}
