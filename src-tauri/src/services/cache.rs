use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CachedHash {
    pub hash: String,
    pub mtime: u64,
}

pub struct HashCache {
    file_path: PathBuf,
    entries: HashMap<String, CachedHash>,
}

impl HashCache {
    pub fn new(cache_dir: &Path) -> Self {
        let file_path = cache_dir.join("hash_cache.json");
        let entries = if file_path.exists() {
            Self::load_cache(&file_path).unwrap_or_else(|e| {
                println!("Kunne ikke laste cache: {}", e);
                HashMap::new()
            })
        } else {
            HashMap::new()
        };

        HashCache {
            file_path,
            entries,
        }
    }

    fn load_cache(path: &Path) -> Result<HashMap<String, CachedHash>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let cache = serde_json::from_str(&content)?;
        Ok(cache)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string(&self.entries)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn get(&self, path: &str, current_mtime: SystemTime) -> Option<String> {
        if let Some(entry) = self.entries.get(path) {
            if let Ok(mtime_secs) = current_mtime.duration_since(UNIX_EPOCH) {
                if entry.mtime == mtime_secs.as_secs() {
                    return Some(entry.hash.clone());
                }
            }
        }
        None
    }

    pub fn insert(&mut self, path: String, mtime: SystemTime, hash: String) {
        if let Ok(mtime_secs) = mtime.duration_since(UNIX_EPOCH) {
            self.entries.insert(
                path,
                CachedHash {
                    hash,
                    mtime: mtime_secs.as_secs(),
                },
            );
        }
    }
}
