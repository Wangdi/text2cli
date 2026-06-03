use crate::context::Context;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Cache entry for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The request (natural language)
    pub request: String,
    /// The generated command(s)
    pub commands: Vec<String>,
    /// Working directory when cached
    pub working_dir: PathBuf,
    /// Timestamp when cached
    pub cached_at: SystemTime,
    /// Time-to-live in microseconds (for sub-second precision)
    #[serde(default)]
    pub ttl_micros: u64,
    /// Legacy field: Time-to-live in seconds (deprecated, use ttl_micros)
    #[serde(default, rename = "ttl_secs")]
    ttl_secs_legacy: u64,
}

impl CacheEntry {
    pub fn new(request: String, commands: Vec<String>, working_dir: PathBuf, ttl: Duration) -> Self {
        Self {
            request,
            commands,
            working_dir,
            cached_at: SystemTime::now(),
            ttl_micros: ttl.as_micros() as u64,
            ttl_secs_legacy: 0,
        }
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.cached_at.elapsed() {
            // Use ttl_micros if set, otherwise fall back to legacy ttl_secs
            let ttl_micros = if self.ttl_micros > 0 {
                self.ttl_micros
            } else {
                self.ttl_secs_legacy * 1_000_000
            };
            elapsed.as_micros() > ttl_micros as u128
        } else {
            true
        }
    }

    /// Check if context matches
    pub fn matches_context(&self, context: &Context) -> bool {
        self.working_dir == context.working_dir
    }
}

/// Command cache manager
pub struct CommandCache {
    /// Cache storage
    cache: HashMap<String, CacheEntry>,
    /// Cache file path
    cache_file: PathBuf,
    /// Default TTL
    default_ttl: Duration,
}

impl CommandCache {
    /// Create a new cache manager
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| crate::Error::Parse("Cannot determine home directory".to_string()))?
            .join(".text2cli")
            .join("cache");

        // Ensure directory exists
        std::fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("commands.json");

        let mut cache = Self {
            cache: HashMap::new(),
            cache_file,
            default_ttl: Duration::from_secs(3600), // 1 hour default
        };

        // Load existing cache
        cache.load()?;

        Ok(cache)
    }

    /// Generate cache key
    fn cache_key(request: &str, context: &Context) -> String {
        // Use working_dir + request as key
        format!("{}:{}", context.working_dir.display(), request)
    }

    /// Get cached command
    pub fn get(&self, request: &str, context: &Context) -> Option<&CacheEntry> {
        let key = Self::cache_key(request, context);
        self.cache.get(&key).filter(|entry| !entry.is_expired())
    }

    /// Store command in cache
    pub fn put(&mut self, request: String, commands: Vec<String>, context: &Context) -> Result<()> {
        let key = Self::cache_key(&request, context);
        let entry = CacheEntry::new(request, commands, context.working_dir.clone(), self.default_ttl);
        self.cache.insert(key, entry);
        self.save()
    }

    /// Store command with custom TTL
    pub fn put_with_ttl(&mut self, request: String, commands: Vec<String>, context: &Context, ttl: Duration) -> Result<()> {
        let key = Self::cache_key(&request, context);
        let entry = CacheEntry::new(request, commands, context.working_dir.clone(), ttl);
        self.cache.insert(key, entry);
        self.save()
    }

    /// Clear expired entries
    pub fn cleanup(&mut self) -> Result<usize> {
        let before = self.cache.len();
        self.cache.retain(|_, entry| !entry.is_expired());
        let removed = before - self.cache.len();

        if removed > 0 {
            self.save()?;
        }

        Ok(removed)
    }

    /// Clear all cache
    pub fn clear(&mut self) -> Result<()> {
        self.cache.clear();
        self.save()
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Load cache from disk
    fn load(&mut self) -> Result<()> {
        if !self.cache_file.exists() {
            return Ok(());
        }

        let json = std::fs::read_to_string(&self.cache_file)?;
        let entries: HashMap<String, CacheEntry> = serde_json::from_str(&json)?;

        // Filter out expired entries
        self.cache = entries.into_iter().filter(|(_, entry)| !entry.is_expired()).collect();

        Ok(())
    }

    /// Save cache to disk
    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.cache)?;
        std::fs::write(&self.cache_file, json)?;
        Ok(())
    }
}

impl Default for CommandCache {
    fn default() -> Self {
        Self::new().expect("Failed to create CommandCache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new(
            "test".to_string(),
            vec!["cmd".to_string()],
            PathBuf::from("/tmp"),
            Duration::from_millis(1), // Very short TTL
        );

        // Should be expired after a short delay
        std::thread::sleep(Duration::from_millis(10));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_entry_not_expired() {
        let entry = CacheEntry::new(
            "test".to_string(),
            vec!["cmd".to_string()],
            PathBuf::from("/tmp"),
            Duration::from_secs(3600),
        );

        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_put_and_get() {
        let mut cache = CommandCache::new().expect("Failed to create cache");
        let context = Context {
            working_dir: PathBuf::from("/tmp"),
            ..Default::default()
        };

        cache.put("test request".to_string(), vec!["ls -la".to_string()], &context).unwrap();

        let entry = cache.get("test request", &context);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().commands, vec!["ls -la".to_string()]);
    }
}
