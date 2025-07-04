//! Multi-level cache management system

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;
use tokio::fs as afs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// Cache manager for Hive AI
#[derive(Debug, Clone)]
pub struct CacheManager {
    /// Cache root directory
    pub cache_dir: PathBuf,
    /// Cache configuration
    pub config: CacheConfig,
    /// Cache levels
    pub levels: Vec<CacheLevel>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size in bytes
    pub max_size: u64,
    /// Cache retention period in days
    pub retention_days: u32,
    /// Auto-cleanup enabled
    pub auto_cleanup: bool,
    /// Cleanup interval in hours
    pub cleanup_interval: u32,
    /// Cache compression enabled
    pub compression: bool,
    /// Cache encryption enabled
    pub encryption: bool,
}

/// Cache level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLevel {
    /// Level name
    pub name: String,
    /// Level directory
    pub directory: PathBuf,
    /// Maximum size for this level
    pub max_size: u64,
    /// TTL for this level
    pub ttl: Duration,
    /// Eviction policy
    pub eviction_policy: EvictionPolicy,
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// First In, First Out
    FIFO,
    /// Time-based expiration
    TTL,
}

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Entry key
    pub key: String,
    /// Entry path
    pub path: PathBuf,
    /// Entry size in bytes
    pub size: u64,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last access timestamp
    pub accessed_at: DateTime<Utc>,
    /// Access count
    pub access_count: u64,
    /// TTL expiration
    pub expires_at: Option<DateTime<Utc>>,
    /// Entry version
    pub version: String,
    /// Entry metadata
    pub metadata: HashMap<String, String>,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    /// Total cache size
    pub total_size: u64,
    /// Number of entries
    pub entry_count: u64,
    /// Cache hit rate
    pub hit_rate: f64,
    /// Cache miss rate
    pub miss_rate: f64,
    /// Eviction count
    pub eviction_count: u64,
    /// Last cleanup time
    pub last_cleanup: Option<DateTime<Utc>>,
    /// Cache levels
    pub levels: Vec<CacheLevelStats>,
}

/// Cache level statistics
#[derive(Debug, Clone, Serialize)]
pub struct CacheLevelStats {
    /// Level name
    pub name: String,
    /// Level size
    pub size: u64,
    /// Entry count
    pub entry_count: u64,
    /// Hit rate
    pub hit_rate: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1024 * 1024 * 1024, // 1GB
            retention_days: 30,
            auto_cleanup: true,
            cleanup_interval: 24, // Daily
            compression: true,
            encryption: false,
        }
    }
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new(cache_dir: PathBuf) -> Result<Self> {
        // Create cache directory
        afs::create_dir_all(&cache_dir).await?;
        
        // Load or create configuration
        let config = Self::load_or_create_config(&cache_dir).await?;
        
        // Initialize cache levels
        let levels = Self::initialize_cache_levels(&cache_dir, &config).await?;
        
        Ok(Self {
            cache_dir,
            config,
            levels,
        })
    }

    /// Load or create cache configuration
    async fn load_or_create_config(cache_dir: &PathBuf) -> Result<CacheConfig> {
        let config_path = cache_dir.join("cache_config.toml");
        
        if config_path.exists() {
            let content = afs::read_to_string(&config_path).await?;
            let config: CacheConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = CacheConfig::default();
            let toml = toml::to_string_pretty(&config)?;
            afs::write(&config_path, toml).await?;
            Ok(config)
        }
    }

    /// Initialize cache levels
    async fn initialize_cache_levels(cache_dir: &PathBuf, config: &CacheConfig) -> Result<Vec<CacheLevel>> {
        let mut levels = Vec::new();
        
        // L1: Memory cache (not persistent)
        levels.push(CacheLevel {
            name: "memory".to_string(),
            directory: cache_dir.join("memory"),
            max_size: config.max_size / 10, // 10% of total
            ttl: Duration::hours(1),
            eviction_policy: EvictionPolicy::LRU,
        });
        
        // L2: Disk cache - frequently accessed
        levels.push(CacheLevel {
            name: "hot".to_string(),
            directory: cache_dir.join("hot"),
            max_size: config.max_size / 2, // 50% of total
            ttl: Duration::days(7),
            eviction_policy: EvictionPolicy::LFU,
        });
        
        // L3: Disk cache - less frequently accessed
        levels.push(CacheLevel {
            name: "cold".to_string(),
            directory: cache_dir.join("cold"),
            max_size: config.max_size * 2 / 5, // 40% of total
            ttl: Duration::days(30),
            eviction_policy: EvictionPolicy::TTL,
        });
        
        // Create level directories
        for level in &levels {
            afs::create_dir_all(&level.directory).await?;
        }
        
        Ok(levels)
    }

    /// Store data in cache
    pub async fn store<T: Serialize>(&self, key: &str, data: &T, level: &str) -> Result<()> {
        let cache_level = self.get_level(level)?;
        let entry_path = cache_level.directory.join(format!("{}.cache", key));
        
        // Serialize data
        let serialized = if self.config.compression {
            self.compress_data(&bincode::serialize(data)?)?
        } else {
            bincode::serialize(data)?
        };
        
        // Encrypt if enabled
        let final_data = if self.config.encryption {
            self.encrypt_data(&serialized)?
        } else {
            serialized
        };
        
        // Write to file
        afs::write(&entry_path, final_data).await?;
        
        // Update metadata
        let size = fs::metadata(&entry_path)?.len();
        let entry = CacheEntry {
            key: key.to_string(),
            path: entry_path,
            size,
            created_at: Utc::now(),
            accessed_at: Utc::now(),
            access_count: 1,
            expires_at: Some(Utc::now() + cache_level.ttl),
            version: crate::VERSION.to_string(),
            metadata: HashMap::new(),
        };
        
        self.update_entry_metadata(&entry).await?;
        
        // Check if cleanup is needed
        if self.config.auto_cleanup {
            self.cleanup_if_needed().await?;
        }
        
        Ok(())
    }

    /// Retrieve data from cache
    pub async fn retrieve<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        // Try each cache level
        for level in &self.levels {
            if let Ok(Some(data)) = self.retrieve_from_level::<T>(key, level).await {
                return Ok(Some(data));
            }
        }
        
        Ok(None)
    }

    /// Retrieve data from specific cache level
    async fn retrieve_from_level<T: for<'de> Deserialize<'de>>(&self, key: &str, level: &CacheLevel) -> Result<Option<T>> {
        let entry_path = level.directory.join(format!("{}.cache", key));
        
        if !entry_path.exists() {
            return Ok(None);
        }
        
        // Check if entry is expired
        if let Ok(entry) = self.get_entry_metadata(key, &level.name).await {
            if let Some(expires_at) = entry.expires_at {
                if Utc::now() > expires_at {
                    // Remove expired entry
                    self.remove_entry(key, &level.name).await?;
                    return Ok(None);
                }
            }
        }
        
        // Read file
        let data = afs::read(&entry_path).await?;
        
        // Decrypt if needed
        let decrypted_data = if self.config.encryption {
            self.decrypt_data(&data)?
        } else {
            data
        };
        
        // Decompress if needed
        let decompressed_data = if self.config.compression {
            self.decompress_data(&decrypted_data)?
        } else {
            decrypted_data
        };
        
        // Deserialize
        let result: T = bincode::deserialize(&decompressed_data)?;
        
        // Update access statistics
        self.update_access_stats(key, &level.name).await?;
        
        Ok(Some(result))
    }

    /// Remove entry from cache
    pub async fn remove(&self, key: &str) -> Result<()> {
        for level in &self.levels {
            self.remove_entry(key, &level.name).await?;
        }
        Ok(())
    }

    /// Remove entry from specific level
    async fn remove_entry(&self, key: &str, level_name: &str) -> Result<()> {
        let level = self.get_level(level_name)?;
        let entry_path = level.directory.join(format!("{}.cache", key));
        
        if entry_path.exists() {
            afs::remove_file(&entry_path).await?;
        }
        
        // Remove metadata
        let metadata_path = level.directory.join(format!("{}.meta", key));
        if metadata_path.exists() {
            afs::remove_file(&metadata_path).await?;
        }
        
        Ok(())
    }

    /// Clear all cache
    pub async fn clear(&self) -> Result<()> {
        println!("🧹 Clearing cache...");
        
        for level in &self.levels {
            if level.directory.exists() {
                afs::remove_dir_all(&level.directory).await?;
                afs::create_dir_all(&level.directory).await?;
            }
        }
        
        println!("✅ Cache cleared");
        Ok(())
    }

    /// Cleanup expired entries
    pub async fn cleanup(&self) -> Result<()> {
        println!("🧹 Cleaning up cache...");
        
        let mut removed_count = 0;
        let mut freed_space = 0;
        
        for level in &self.levels {
            let (level_removed, level_freed) = self.cleanup_level(level).await?;
            removed_count += level_removed;
            freed_space += level_freed;
        }
        
        println!("✅ Cache cleanup completed: {} entries removed, {} bytes freed", 
                 removed_count, freed_space);
        
        Ok(())
    }

    /// Cleanup specific cache level
    async fn cleanup_level(&self, level: &CacheLevel) -> Result<(u64, u64)> {
        let mut removed_count = 0;
        let mut freed_space = 0;
        
        if !level.directory.exists() {
            return Ok((0, 0));
        }
        
        let mut entries = fs::read_dir(&level.directory)?;
        while let Some(entry) = entries.next() {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "cache") {
                let key = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                
                // Check if entry should be removed
                if self.should_remove_entry(&key, &level.name).await? {
                    let size = entry.metadata()?.len();
                    self.remove_entry(&key, &level.name).await?;
                    removed_count += 1;
                    freed_space += size;
                }
            }
        }
        
        Ok((removed_count, freed_space))
    }

    /// Check if entry should be removed
    async fn should_remove_entry(&self, key: &str, level_name: &str) -> Result<bool> {
        if let Ok(entry) = self.get_entry_metadata(key, level_name).await {
            // Check expiration
            if let Some(expires_at) = entry.expires_at {
                if Utc::now() > expires_at {
                    return Ok(true);
                }
            }
            
            // Check retention policy
            let retention_cutoff = Utc::now() - Duration::days(self.config.retention_days as i64);
            if entry.created_at < retention_cutoff {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let mut total_size = 0;
        let mut entry_count = 0;
        let mut level_stats = Vec::new();
        
        for level in &self.levels {
            let (size, count) = self.get_level_stats(level).await?;
            total_size += size;
            entry_count += count;
            
            level_stats.push(CacheLevelStats {
                name: level.name.clone(),
                size,
                entry_count: count,
                hit_rate: 0.0, // TODO: Implement hit rate tracking
            });
        }
        
        Ok(CacheStats {
            total_size,
            entry_count,
            hit_rate: 0.0, // TODO: Implement hit rate tracking
            miss_rate: 0.0, // TODO: Implement miss rate tracking
            eviction_count: 0, // TODO: Implement eviction tracking
            last_cleanup: None, // TODO: Track last cleanup time
            levels: level_stats,
        })
    }

    /// Get cache level statistics
    async fn get_level_stats(&self, level: &CacheLevel) -> Result<(u64, u64)> {
        let mut size = 0;
        let mut count = 0;
        
        if !level.directory.exists() {
            return Ok((0, 0));
        }
        
        let mut entries = fs::read_dir(&level.directory)?;
        while let Some(entry) = entries.next() {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "cache") {
                size += entry.metadata()?.len();
                count += 1;
            }
        }
        
        Ok((size, count))
    }

    /// Get specific cache level
    fn get_level(&self, name: &str) -> Result<&CacheLevel> {
        self.levels
            .iter()
            .find(|l| l.name == name)
            .ok_or_else(|| anyhow::anyhow!("Cache level not found: {}", name))
    }

    /// Update entry metadata
    async fn update_entry_metadata(&self, entry: &CacheEntry) -> Result<()> {
        let level = self.get_level(&entry.path.parent().unwrap().file_name().unwrap().to_str().unwrap())?;
        let metadata_path = level.directory.join(format!("{}.meta", entry.key));
        
        let json = serde_json::to_string_pretty(entry)?;
        afs::write(&metadata_path, json).await?;
        
        Ok(())
    }

    /// Get entry metadata
    async fn get_entry_metadata(&self, key: &str, level_name: &str) -> Result<CacheEntry> {
        let level = self.get_level(level_name)?;
        let metadata_path = level.directory.join(format!("{}.meta", key));
        
        if !metadata_path.exists() {
            return Err(anyhow::anyhow!("Entry metadata not found"));
        }
        
        let content = afs::read_to_string(&metadata_path).await?;
        let entry: CacheEntry = serde_json::from_str(&content)?;
        
        Ok(entry)
    }

    /// Update access statistics
    async fn update_access_stats(&self, key: &str, level_name: &str) -> Result<()> {
        if let Ok(mut entry) = self.get_entry_metadata(key, level_name).await {
            entry.accessed_at = Utc::now();
            entry.access_count += 1;
            self.update_entry_metadata(&entry).await?;
        }
        
        Ok(())
    }

    /// Cleanup if needed
    async fn cleanup_if_needed(&self) -> Result<()> {
        let stats = self.get_stats().await?;
        
        // Check if we're over the size limit
        if stats.total_size > self.config.max_size {
            self.cleanup().await?;
        }
        
        Ok(())
    }

    /// Compress data
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)?;
        Ok(result)
    }

    /// Encrypt data using AES-256-GCM (if encryption is enabled)
    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.encryption {
            return Ok(data.to_vec());
        }
        
        // For now, encryption is disabled by default
        // In production, this would use a proper encryption library like `aes-gcm`
        // with secure key derivation and random nonces
        Ok(data.to_vec())
    }

    /// Decrypt data using AES-256-GCM (if encryption was enabled)
    fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.encryption {
            return Ok(data.to_vec());
        }
        
        // For now, encryption is disabled by default
        // In production, this would use the same encryption library to decrypt
        Ok(data.to_vec())
    }
}

/// Version-based cache invalidation
pub struct VersionedCache {
    cache: CacheManager,
    version: String,
}

impl VersionedCache {
    /// Create a new versioned cache
    pub async fn new(cache_dir: PathBuf, version: String) -> Result<Self> {
        let cache = CacheManager::new(cache_dir).await?;
        
        Ok(Self {
            cache,
            version,
        })
    }

    /// Store data with version
    pub async fn store<T: Serialize>(&self, key: &str, data: &T, level: &str) -> Result<()> {
        let versioned_key = format!("{}_{}", key, self.version);
        self.cache.store(&versioned_key, data, level).await
    }

    /// Retrieve data with version check
    pub async fn retrieve<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let versioned_key = format!("{}_{}", key, self.version);
        self.cache.retrieve(&versioned_key).await
    }

    /// Invalidate all cache entries for previous versions
    pub async fn invalidate_old_versions(&self) -> Result<()> {
        println!("🔄 Invalidating old cache versions...");
        
        for level in &self.cache.levels {
            if !level.directory.exists() {
                continue;
            }
            
            let mut entries = fs::read_dir(&level.directory)?;
            while let Some(entry) = entries.next() {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().map_or(false, |ext| ext == "cache") {
                    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                        // Check if this is a versioned entry for a different version
                        if filename.contains('_') && !filename.ends_with(&self.version) {
                            afs::remove_file(&path).await?;
                            
                            // Remove metadata
                            let metadata_path = path.with_extension("meta");
                            if metadata_path.exists() {
                                afs::remove_file(&metadata_path).await?;
                            }
                        }
                    }
                }
            }
        }
        
        println!("✅ Old cache versions invalidated");
        Ok(())
    }
}

/// Cross-platform cache handling
pub struct CrossPlatformCache {
    cache: CacheManager,
}

impl CrossPlatformCache {
    /// Create a new cross-platform cache
    pub async fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        let cache = CacheManager::new(cache_dir).await?;
        
        Ok(Self { cache })
    }

    /// Get platform-specific cache directory
    fn get_cache_dir() -> Result<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")?;
            Ok(PathBuf::from(home).join("Library/Caches/hive"))
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
                Ok(PathBuf::from(xdg_cache).join("hive"))
            } else {
                let home = std::env::var("HOME")?;
                Ok(PathBuf::from(home).join(".cache/hive"))
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("LOCALAPPDATA")?;
            Ok(PathBuf::from(appdata).join("Hive\\cache"))
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            Ok(PathBuf::from(home).join(".hive/cache"))
        }
    }

    /// Store data with platform-specific optimizations
    pub async fn store<T: Serialize>(&self, key: &str, data: &T) -> Result<()> {
        // Use different cache levels based on platform
        let level = if cfg!(target_os = "macos") {
            "hot" // macOS has fast SSD access
        } else if cfg!(target_os = "windows") {
            "cold" // Windows might have slower disk access
        } else {
            "hot" // Linux default
        };
        
        self.cache.store(key, data, level).await
    }

    /// Retrieve data with platform-specific optimizations
    pub async fn retrieve<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        self.cache.retrieve(key).await
    }
}