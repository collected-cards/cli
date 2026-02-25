use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

pub struct Cache {
    cache_dir: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    data: Value,
    timestamp: u64,
}

#[allow(dead_code)]
impl Cache {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("collected")
            .join("cache");
        
        fs::create_dir_all(&cache_dir)?;
        
        Ok(Self { cache_dir })
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let cache_file = self.cache_dir.join(format!("{}.json", Self::hash_key(key)));
        
        if !cache_file.exists() {
            return None;
        }
        
        let content = fs::read_to_string(&cache_file).ok()?;
        let entry: CacheEntry = serde_json::from_str(&content).ok()?;
        
        // Check if expired
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
        if now.saturating_sub(entry.timestamp) > CACHE_TTL.as_secs() {
            // Remove expired entry
            let _ = fs::remove_file(&cache_file);
            return None;
        }
        
        Some(entry.data)
    }
    
    pub fn set(&self, key: &str, data: &Value) -> Result<()> {
        let cache_file = self.cache_dir.join(format!("{}.json", Self::hash_key(key)));
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let entry = CacheEntry {
            data: data.clone(),
            timestamp,
        };
        
        let content = serde_json::to_string(&entry)?;
        fs::write(&cache_file, content)?;
        
        Ok(())
    }
    
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }
    
    fn hash_key(key: &str) -> String {
        // Simple hash for filename - in production might want to use a real hash
        key.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .chars()
            .take(50)
            .collect()
    }
}