//! High-performance caching system for Hive AI
//!
//! This module provides comprehensive caching capabilities including:
//! - AST cache for parsed code structures
//! - Semantic index cache for search operations
//! - Model response cache for consensus operations
//! - LRU eviction with size-based constraints
//! - Disk persistence for long-term storage

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::num::NonZeroUsize;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use lru::LruCache;
use sha2::{Sha256, Digest};
use anyhow::{Result, anyhow};
use tokio::fs;
use once_cell::sync::Lazy;

use crate::core::error::{HiveError, ErrorCategory};

/// Global cache instance
static CACHE_MANAGER: Lazy<Arc<CacheManager>> = Lazy::new(|| {
    Arc::new(CacheManager::new(CacheConfig::default()))
});

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum memory cache size in bytes
    pub max_memory_size: usize,
    /// Maximum disk cache size in bytes
    pub max_disk_size: u64,
    /// Cache directory path
    pub cache_dir: PathBuf,
    /// Enable disk persistence
    pub enable_disk_cache: bool,
    /// Cache expiration duration
    pub expiration: Duration,
    /// Compression for disk cache
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let cache_dir = home_dir.join(".hive").join("cache");
        
        Self {
            max_memory_size: 256 * 1024 * 1024, // 256MB
            max_disk_size: 1024 * 1024 * 1024,  // 1GB
            cache_dir,
            enable_disk_cache: true,
            expiration: Duration::from_secs(3600 * 24), // 24 hours
            enable_compression: true,
        }
    }
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntryMeta {
    /// Creation timestamp
    created_at: SystemTime,
    /// Last access timestamp
    last_accessed: SystemTime,
    /// Access count
    access_count: u64,
    /// Entry size in bytes
    size: usize,
    /// Entry category
    category: CacheCategory,
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Entry metadata
    meta: CacheEntryMeta,
    /// Cached data
    data: Arc<Vec<u8>>,
}

/// Cache categories for different types of data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheCategory {
    /// AST parsing results
    Ast,
    /// Semantic analysis results
    Semantic,
    /// Model responses
    ModelResponse,
    /// Repository analysis
    Repository,
    /// Search indices
    SearchIndex,
    /// Configuration data
    Config,
    /// General purpose
    General,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Current memory usage
    pub memory_usage: usize,
    /// Current disk usage
    pub disk_usage: u64,
    /// Number of entries in memory
    pub memory_entries: usize,
    /// Number of entries on disk
    pub disk_entries: usize,
    /// Hit rate percentage
    pub hit_rate: f64,
    /// Category-specific stats
    pub category_stats: HashMap<CacheCategory, CategoryStats>,
}

/// Category-specific cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub size: usize,
}

/// Main cache manager
pub struct CacheManager {
    /// Configuration
    config: CacheConfig,
    /// In-memory LRU cache
    memory_cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
    /// Disk cache index
    disk_index: Arc<RwLock<HashMap<String, CacheEntryMeta>>>,
    /// Background task handle
    cleanup_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl CacheManager {
    /// Create a new cache manager
    fn new(config: CacheConfig) -> Self {
        let max_entries = config.max_memory_size / (64 * 1024); // Assume 64KB average entry
        let memory_cache = Arc::new(RwLock::new(LruCache::new(
            std::num::NonZeroUsize::new(max_entries).unwrap()
        )));
        
        let stats = Arc::new(RwLock::new(CacheStats {
            hits: 0,
            misses: 0,
            memory_usage: 0,
            disk_usage: 0,
            memory_entries: 0,
            disk_entries: 0,
            hit_rate: 0.0,
            category_stats: HashMap::new(),
        }));
        
        Self {
            config,
            memory_cache,
            stats,
            disk_index: Arc::new(RwLock::new(HashMap::new())),
            cleanup_handle: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Initialize the cache system
    async fn initialize(&self) -> Result<()> {
        // Create cache directory
        if self.config.enable_disk_cache {
            fs::create_dir_all(&self.config.cache_dir).await
                .map_err(|e| HiveError::internal("cache_init", format!("Failed to create cache directory: {}", e)))?;
            
            // Load disk cache index
            self.load_disk_index().await?;
        }
        
        // Start background cleanup task
        self.start_cleanup_task().await;
        
        tracing::info!(
            memory_size_mb = self.config.max_memory_size / (1024 * 1024),
            disk_size_mb = self.config.max_disk_size / (1024 * 1024),
            cache_dir = %self.config.cache_dir.display(),
            "Cache system initialized"
        );
        
        Ok(())
    }
    
    /// Get a value from the cache
    pub async fn get(&self, key: &str, category: CacheCategory) -> Option<Vec<u8>> {
        // Try memory cache first
        {
            let mut cache = self.memory_cache.write().await;
            if let Some(entry) = cache.get_mut(key) {
                entry.meta.last_accessed = SystemTime::now();
                entry.meta.access_count += 1;
                
                self.record_hit(category).await;
                return Some(entry.data.as_ref().clone());
            }
        }
        
        // Try disk cache if enabled
        if self.config.enable_disk_cache {
            if let Ok(Some(data)) = self.load_from_disk(key, category).await {
                self.record_hit(category).await;
                return Some(data);
            }
        }
        
        self.record_miss(category).await;
        None
    }
    
    /// Put a value in the cache
    pub async fn put(&self, key: String, data: Vec<u8>, category: CacheCategory) -> Result<()> {
        let size = data.len();
        
        // Check if data is too large
        if size > self.config.max_memory_size / 4 {
            return Err(anyhow!("Cache entry too large: {} bytes", size));
        }
        
        let entry = CacheEntry {
            meta: CacheEntryMeta {
                created_at: SystemTime::now(),
                last_accessed: SystemTime::now(),
                access_count: 0,
                size,
                category,
            },
            data: Arc::new(data),
        };
        
        // Add to memory cache
        {
            let mut cache = self.memory_cache.write().await;
            if let Some((_key, old_entry)) = cache.push(key.clone(), entry.clone()) {
                self.update_memory_stats(-(old_entry.meta.size as isize)).await;
            }
            self.update_memory_stats(size as isize).await;
        }
        
        // Optionally persist to disk
        if self.config.enable_disk_cache {
            self.save_to_disk(&key, &entry).await?;
        }
        
        Ok(())
    }
    
    /// Remove a value from the cache
    pub async fn remove(&self, key: &str) -> Result<()> {
        // Remove from memory
        {
            let mut cache = self.memory_cache.write().await;
            if let Some(entry) = cache.pop(key) {
                self.update_memory_stats(-(entry.meta.size as isize)).await;
            }
        }
        
        // Remove from disk
        if self.config.enable_disk_cache {
            self.remove_from_disk(key).await?;
        }
        
        Ok(())
    }
    
    /// Clear all cache entries
    pub async fn clear(&self) -> Result<()> {
        // Clear memory cache
        {
            let mut cache = self.memory_cache.write().await;
            cache.clear();
        }
        
        // Clear disk cache
        if self.config.enable_disk_cache {
            self.clear_disk_cache().await?;
        }
        
        // Reset stats
        {
            let mut stats = self.stats.write().await;
            *stats = CacheStats {
                hits: 0,
                misses: 0,
                memory_usage: 0,
                disk_usage: 0,
                memory_entries: 0,
                disk_entries: 0,
                hit_rate: 0.0,
                category_stats: HashMap::new(),
            };
        }
        
        Ok(())
    }
    
    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
    
    /// Load disk cache index
    async fn load_disk_index(&self) -> Result<()> {
        let index_path = self.config.cache_dir.join("index.json");
        if index_path.exists() {
            let data = fs::read(&index_path).await?;
            let index: HashMap<String, CacheEntryMeta> = serde_json::from_slice(&data)?;
            
            let mut disk_index = self.disk_index.write().await;
            *disk_index = index;
        }
        Ok(())
    }
    
    /// Save disk cache index
    async fn save_disk_index(&self) -> Result<()> {
        let index_path = self.config.cache_dir.join("index.json");
        let index = self.disk_index.read().await;
        let data = serde_json::to_vec_pretty(&*index)?;
        fs::write(&index_path, data).await?;
        Ok(())
    }
    
    /// Load entry from disk
    async fn load_from_disk(&self, key: &str, _category: CacheCategory) -> Result<Option<Vec<u8>>> {
        let disk_index = self.disk_index.read().await;
        if let Some(meta) = disk_index.get(key) {
            // Check expiration
            if SystemTime::now().duration_since(meta.created_at).unwrap_or_default() > self.config.expiration {
                return Ok(None);
            }
            
            let file_path = self.get_disk_path(key);
            if file_path.exists() {
                let data = fs::read(&file_path).await?;
                
                // Optionally decompress
                let data = if self.config.enable_compression {
                    self.decompress(&data)?
                } else {
                    data
                };
                
                // Update memory cache
                let entry = CacheEntry {
                    meta: meta.clone(),
                    data: Arc::new(data.clone()),
                };
                
                let mut cache = self.memory_cache.write().await;
                cache.push(key.to_string(), entry);
                
                return Ok(Some(data));
            }
        }
        Ok(None)
    }
    
    /// Save entry to disk
    async fn save_to_disk(&self, key: &str, entry: &CacheEntry) -> Result<()> {
        let file_path = self.get_disk_path(key);
        
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Optionally compress data
        let data = if self.config.enable_compression {
            self.compress(entry.data.as_ref())?
        } else {
            entry.data.as_ref().clone()
        };
        
        // Write to disk
        fs::write(&file_path, &data).await?;
        
        // Update index
        {
            let mut disk_index = self.disk_index.write().await;
            disk_index.insert(key.to_string(), entry.meta.clone());
        }
        
        // Save index
        self.save_disk_index().await?;
        
        Ok(())
    }
    
    /// Remove entry from disk
    async fn remove_from_disk(&self, key: &str) -> Result<()> {
        let file_path = self.get_disk_path(key);
        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }
        
        let mut disk_index = self.disk_index.write().await;
        disk_index.remove(key);
        
        self.save_disk_index().await?;
        Ok(())
    }
    
    /// Clear all disk cache
    async fn clear_disk_cache(&self) -> Result<()> {
        if self.config.cache_dir.exists() {
            fs::remove_dir_all(&self.config.cache_dir).await?;
            fs::create_dir_all(&self.config.cache_dir).await?;
        }
        
        let mut disk_index = self.disk_index.write().await;
        disk_index.clear();
        
        Ok(())
    }
    
    /// Get disk path for a cache key
    fn get_disk_path(&self, key: &str) -> PathBuf {
        let hash = self.hash_key(key);
        let dir1 = &hash[0..2];
        let dir2 = &hash[2..4];
        self.config.cache_dir.join(dir1).join(dir2).join(&hash[4..])
    }
    
    /// Hash a cache key
    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Compress data
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }
    
    /// Decompress data
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
    
    /// Start background cleanup task
    async fn start_cleanup_task(&self) {
        let cache_dir = self.config.cache_dir.clone();
        let expiration = self.config.expiration;
        let disk_index = Arc::clone(&self.disk_index);
        
        let handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await; // Run hourly
                
                // Clean up expired entries
                let now = SystemTime::now();
                let mut expired_keys = Vec::new();
                
                {
                    let index = disk_index.read().await;
                    for (key, meta) in index.iter() {
                        if now.duration_since(meta.created_at).unwrap_or_default() > expiration {
                            expired_keys.push(key.clone());
                        }
                    }
                }
                
                // Remove expired entries
                for key in expired_keys {
                    let hash = format!("{:x}", Sha256::digest(key.as_bytes()));
                    let dir1 = &hash[0..2];
                    let dir2 = &hash[2..4];
                    let file_path = cache_dir.join(dir1).join(dir2).join(&hash[4..]);
                    
                    if file_path.exists() {
                        let _ = fs::remove_file(&file_path).await;
                    }
                    
                    let mut index = disk_index.write().await;
                    index.remove(&key);
                }
            }
        });
        
        let mut cleanup_handle = self.cleanup_handle.lock().await;
        *cleanup_handle = Some(handle);
    }
    
    /// Record a cache hit
    async fn record_hit(&self, category: CacheCategory) {
        let mut stats = self.stats.write().await;
        stats.hits += 1;
        
        let cat_stats = stats.category_stats.entry(category).or_insert(CategoryStats {
            hits: 0,
            misses: 0,
            entries: 0,
            size: 0,
        });
        cat_stats.hits += 1;
        
        // Update hit rate
        let total = stats.hits + stats.misses;
        if total > 0 {
            stats.hit_rate = (stats.hits as f64 / total as f64) * 100.0;
        }
    }
    
    /// Record a cache miss
    async fn record_miss(&self, category: CacheCategory) {
        let mut stats = self.stats.write().await;
        stats.misses += 1;
        
        let cat_stats = stats.category_stats.entry(category).or_insert(CategoryStats {
            hits: 0,
            misses: 0,
            entries: 0,
            size: 0,
        });
        cat_stats.misses += 1;
        
        // Update hit rate
        let total = stats.hits + stats.misses;
        if total > 0 {
            stats.hit_rate = (stats.hits as f64 / total as f64) * 100.0;
        }
    }
    
    /// Update memory statistics
    async fn update_memory_stats(&self, size_delta: isize) {
        let mut stats = self.stats.write().await;
        if size_delta > 0 {
            stats.memory_usage += size_delta as usize;
            stats.memory_entries += 1;
        } else {
            stats.memory_usage = stats.memory_usage.saturating_sub((-size_delta) as usize);
            stats.memory_entries = stats.memory_entries.saturating_sub(1);
        }
    }
}

/// Initialize the global cache system
pub async fn initialize() -> Result<()> {
    CACHE_MANAGER.initialize().await
}

/// Get a value from the cache
pub async fn get(key: &str, category: CacheCategory) -> Option<Vec<u8>> {
    CACHE_MANAGER.get(key, category).await
}

/// Put a value in the cache
pub async fn put(key: String, data: Vec<u8>, category: CacheCategory) -> Result<()> {
    CACHE_MANAGER.put(key, data, category).await
}

/// Remove a value from the cache
pub async fn remove(key: &str) -> Result<()> {
    CACHE_MANAGER.remove(key).await
}

/// Clear all cache entries
pub async fn clear() -> Result<()> {
    CACHE_MANAGER.clear().await
}

/// Get cache statistics
pub async fn get_stats() -> CacheStats {
    CACHE_MANAGER.get_stats().await
}

/// Cache key builder utilities
pub struct CacheKey;

impl CacheKey {
    /// Build AST cache key
    pub fn ast(file_path: &Path) -> String {
        format!("ast:{}", file_path.display())
    }
    
    /// Build semantic cache key
    pub fn semantic(file_path: &Path) -> String {
        format!("semantic:{}", file_path.display())
    }
    
    /// Build model response cache key
    pub fn model_response(model: &str, prompt_hash: &str) -> String {
        format!("model:{}:{}", model, prompt_hash)
    }
    
    /// Build repository cache key
    pub fn repository(repo_path: &Path) -> String {
        format!("repo:{}", repo_path.display())
    }
    
    /// Build search index cache key
    pub fn search_index(index_name: &str) -> String {
        format!("index:{}", index_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_cache_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = CacheConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let cache = CacheManager::new(config);
        cache.initialize().await?;
        
        // Test put and get
        let key = "test_key";
        let data = b"test data".to_vec();
        cache.put(key.to_string(), data.clone(), CacheCategory::General).await?;
        
        let retrieved = cache.get(key, CacheCategory::General).await;
        assert_eq!(retrieved, Some(data));
        
        // Test remove
        cache.remove(key).await?;
        let retrieved = cache.get(key, CacheCategory::General).await;
        assert_eq!(retrieved, None);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_cache_stats() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = CacheConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            enable_disk_cache: false,
            ..Default::default()
        };
        
        let cache = CacheManager::new(config);
        cache.initialize().await?;
        
        // Generate some hits and misses
        cache.put("key1".to_string(), vec![1, 2, 3], CacheCategory::Ast).await?;
        cache.get("key1", CacheCategory::Ast).await; // Hit
        cache.get("key2", CacheCategory::Ast).await; // Miss
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 50.0);
        
        Ok(())
    }
    
    #[test]
    fn test_cache_key_builder() {
        let path = Path::new("/tmp/test.rs");
        assert_eq!(CacheKey::ast(path), "ast:/tmp/test.rs");
        assert_eq!(CacheKey::semantic(path), "semantic:/tmp/test.rs");
        assert_eq!(CacheKey::model_response("gpt-4", "hash123"), "model:gpt-4:hash123");
    }
}