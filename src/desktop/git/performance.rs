//! Performance optimizations for git operations in large repositories
//!
//! This module provides various optimizations to handle large-scale repositories:
//! - Lazy loading and pagination
//! - Background processing and caching  
//! - Memory-efficient operations
//! - Incremental updates and batch operations

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{mpsc, Semaphore};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, instrument, warn};

use super::{BranchInfo, GitRepository, RepositoryInfo};

/// Configuration for performance optimizations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable background processing
    pub background_processing: bool,
    /// Enable caching of git operations
    pub caching_enabled: bool,
    /// Cache expiry time in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum number of concurrent git operations
    pub max_concurrent_operations: usize,
    /// Maximum number of files to process in a single batch
    pub max_batch_size: usize,
    /// Enable lazy loading for large datasets
    pub lazy_loading_enabled: bool,
    /// Page size for paginated operations
    pub page_size: usize,
    /// Timeout for git operations in milliseconds
    pub operation_timeout_ms: u64,
    /// Enable memory usage optimization
    pub memory_optimization: bool,
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            background_processing: true,
            caching_enabled: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_concurrent_operations: 8,
            max_batch_size: 100,
            lazy_loading_enabled: true,
            page_size: 50,
            operation_timeout_ms: 30000, // 30 seconds
            memory_optimization: true,
            max_memory_mb: 512,
        }
    }
}

/// Cache entry for git operations
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    timestamp: SystemTime,
    access_count: usize,
}

impl<T> CacheEntry<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            timestamp: SystemTime::now(),
            access_count: 0,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed().unwrap_or(Duration::ZERO) > ttl
    }

    fn access(&mut self) -> &T {
        self.access_count += 1;
        &self.data
    }
}

/// Optimized git repository manager for large repositories
pub struct OptimizedGitManager {
    config: PerformanceConfig,
    // Caches
    repository_cache: Arc<RwLock<HashMap<PathBuf, CacheEntry<Arc<GitRepository>>>>>,
    branch_cache: Arc<RwLock<HashMap<PathBuf, CacheEntry<Vec<BranchInfo>>>>>,
    status_cache: Arc<RwLock<HashMap<PathBuf, CacheEntry<Vec<(PathBuf, git2::Status)>>>>>,
    repository_list_cache: Arc<RwLock<Option<CacheEntry<Vec<RepositoryInfo>>>>>,

    // Background processing
    operation_semaphore: Arc<Semaphore>,
    background_tx: mpsc::UnboundedSender<BackgroundTask>,

    // Statistics
    stats: Arc<Mutex<PerformanceStats>>,
}

/// Background task types
#[derive(Debug)]
enum BackgroundTask {
    RefreshRepositoryList { workspace_path: PathBuf },
    RefreshBranches { repo_path: PathBuf },
    RefreshStatus { repo_path: PathBuf },
    PreloadRepository { repo_path: PathBuf },
    CacheCleanup,
}

/// Performance statistics
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PerformanceStats {
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub background_tasks_completed: usize,
    pub operations_timed_out: usize,
    pub memory_usage_mb: usize,
    pub average_operation_time_ms: f64,
    pub total_operations: usize,
}

impl PerformanceStats {
    pub fn cache_hit_rate(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
        }
    }
}

/// Paginated result for large datasets
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub page: usize,
    pub total_pages: usize,
    pub total_items: usize,
    pub has_next: bool,
    pub has_previous: bool,
}

impl<T: Clone> PaginatedResult<T> {
    pub fn new(all_items: Vec<T>, page: usize, page_size: usize) -> Self {
        let total_items = all_items.len();
        let total_pages = (total_items + page_size - 1) / page_size;
        let start_idx = page * page_size;
        let end_idx = std::cmp::min(start_idx + page_size, total_items);

        let items = if start_idx < total_items {
            all_items[start_idx..end_idx].to_vec()
        } else {
            Vec::new()
        };

        Self {
            items,
            page,
            total_pages,
            total_items,
            has_next: page + 1 < total_pages,
            has_previous: page > 0,
        }
    }
}

impl OptimizedGitManager {
    /// Create a new optimized git manager
    pub fn new(config: PerformanceConfig) -> Self {
        let (background_tx, mut background_rx) = mpsc::unbounded_channel();
        let operation_semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));

        let manager = Self {
            config: config.clone(),
            repository_cache: Arc::new(RwLock::new(HashMap::new())),
            branch_cache: Arc::new(RwLock::new(HashMap::new())),
            status_cache: Arc::new(RwLock::new(HashMap::new())),
            repository_list_cache: Arc::new(RwLock::new(None)),
            operation_semaphore,
            background_tx,
            stats: Arc::new(Mutex::new(PerformanceStats::default())),
        };

        // Start background task processor
        if config.background_processing {
            let branch_cache = manager.branch_cache.clone();
            let status_cache = manager.status_cache.clone();
            let repository_list_cache = manager.repository_list_cache.clone();
            let stats = manager.stats.clone();
            let config = config.clone();

            tokio::spawn(async move {
                while let Some(task) = background_rx.recv().await {
                    Self::process_background_task(
                        task,
                        &branch_cache,
                        &status_cache,
                        &repository_list_cache,
                        &stats,
                        &config,
                    )
                    .await;
                }
            });

            // Start cache cleanup task
            let cleanup_tx = manager.background_tx.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
                loop {
                    interval.tick().await;
                    let _ = cleanup_tx.send(BackgroundTask::CacheCleanup);
                }
            });
        }

        manager
    }

    /// Get repository with caching and lazy loading
    #[instrument(skip(self), fields(path = %path.display()))]
    pub async fn get_repository(&self, path: &Path) -> Result<Arc<GitRepository>> {
        let cache_key = path.to_path_buf();

        // Check cache first
        if self.config.caching_enabled {
            let cache = self.repository_cache.read().unwrap();
            if let Some(entry) = cache.get(&cache_key) {
                let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
                if !entry.is_expired(ttl) {
                    // Cache hit
                    self.record_cache_hit();
                    debug!("Repository cache hit for: {:?}", path);
                    return Ok(entry.data.clone());
                }
            }
        }

        // Cache miss - load repository
        self.record_cache_miss();
        debug!("Repository cache miss for: {:?}", path);

        let start_time = Instant::now();
        let permit = self.operation_semaphore.acquire().await.unwrap();

        let repo = timeout(
            Duration::from_millis(self.config.operation_timeout_ms),
            async move {
                let _permit = permit; // Keep permit until operation completes
                GitRepository::open(path)
            },
        )
        .await;

        let elapsed = start_time.elapsed();
        self.record_operation_time(elapsed);

        match repo {
            Ok(Ok(git_repo)) => {
                let arc_repo = Arc::new(git_repo);

                // Cache the result
                if self.config.caching_enabled {
                    let mut cache = self.repository_cache.write().unwrap();
                    cache.insert(cache_key, CacheEntry::new(arc_repo.clone()));
                }

                Ok(arc_repo)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                self.record_timeout();
                Err(anyhow::anyhow!("Repository operation timed out"))
            }
        }
    }

    /// Discover repositories with performance optimizations
    #[instrument(skip(self), fields(workspace = %workspace_path.display()))]
    pub async fn discover_repositories_optimized(
        &self,
        workspace_path: &Path,
    ) -> Result<Vec<RepositoryInfo>> {
        // Check cache first
        if self.config.caching_enabled {
            let cache = self.repository_list_cache.read().unwrap();
            if let Some(entry) = &*cache {
                let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
                if !entry.is_expired(ttl) {
                    self.record_cache_hit();
                    debug!("Repository list cache hit");
                    return Ok(entry.data.clone());
                }
            }
        }

        self.record_cache_miss();
        debug!("Repository list cache miss - scanning workspace");

        let start_time = Instant::now();
        let repositories = self.scan_workspace_efficiently(workspace_path).await?;
        let elapsed = start_time.elapsed();

        self.record_operation_time(elapsed);
        info!(
            "Discovered {} repositories in {:?}",
            repositories.len(),
            elapsed
        );

        // Cache the result
        if self.config.caching_enabled {
            let mut cache = self.repository_list_cache.write().unwrap();
            *cache = Some(CacheEntry::new(repositories.clone()));
        }

        // Start background refresh
        if self.config.background_processing {
            let _ = self
                .background_tx
                .send(BackgroundTask::RefreshRepositoryList {
                    workspace_path: workspace_path.to_path_buf(),
                });
        }

        Ok(repositories)
    }

    /// Get branches with pagination and caching
    #[instrument(skip(self), fields(repo = %repo_path.display()))]
    pub async fn get_branches_paginated(
        &self,
        repo_path: &Path,
        page: usize,
    ) -> Result<PaginatedResult<BranchInfo>> {
        let cache_key = repo_path.to_path_buf();

        // Check cache first
        let branches = if self.config.caching_enabled {
            let cache = self.branch_cache.read().unwrap();
            if let Some(entry) = cache.get(&cache_key) {
                let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
                if !entry.is_expired(ttl) {
                    self.record_cache_hit();
                    debug!("Branch cache hit for: {:?}", repo_path);
                    entry.data.clone()
                } else {
                    drop(cache);
                    self.load_branches_fresh(repo_path).await?
                }
            } else {
                drop(cache);
                self.load_branches_fresh(repo_path).await?
            }
        } else {
            self.load_branches_fresh(repo_path).await?
        };

        // Return paginated result
        Ok(PaginatedResult::new(branches, page, self.config.page_size))
    }

    /// Get file statuses with batching and caching
    #[instrument(skip(self), fields(repo = %repo_path.display()))]
    pub async fn get_file_statuses_batched(
        &self,
        repo_path: &Path,
    ) -> Result<Vec<(PathBuf, git2::Status)>> {
        let cache_key = repo_path.to_path_buf();

        // Check cache first
        if self.config.caching_enabled {
            let cache = self.status_cache.read().unwrap();
            if let Some(entry) = cache.get(&cache_key) {
                let ttl = Duration::from_secs(self.config.cache_ttl_seconds / 4); // Shorter TTL for status
                if !entry.is_expired(ttl) {
                    self.record_cache_hit();
                    debug!("Status cache hit for: {:?}", repo_path);
                    return Ok(entry.data.clone());
                }
            }
        }

        self.record_cache_miss();

        let start_time = Instant::now();
        let permit = self.operation_semaphore.acquire().await.unwrap();

        let statuses = timeout(
            Duration::from_millis(self.config.operation_timeout_ms),
            async move {
                let _permit = permit;
                let repo = GitRepository::open(repo_path)?;
                repo.file_statuses()
            },
        )
        .await;

        let elapsed = start_time.elapsed();
        self.record_operation_time(elapsed);

        match statuses {
            Ok(Ok(statuses)) => {
                // Cache the result
                if self.config.caching_enabled {
                    let mut cache = self.status_cache.write().unwrap();
                    cache.insert(cache_key, CacheEntry::new(statuses.clone()));
                }

                // Start background refresh
                if self.config.background_processing {
                    let _ = self.background_tx.send(BackgroundTask::RefreshStatus {
                        repo_path: repo_path.to_path_buf(),
                    });
                }

                Ok(statuses)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                self.record_timeout();
                Err(anyhow::anyhow!("File status operation timed out"))
            }
        }
    }

    /// Preload repositories in background
    pub fn preload_repositories(&self, repo_paths: Vec<PathBuf>) {
        if !self.config.background_processing {
            return;
        }

        for repo_path in repo_paths {
            let _ = self
                .background_tx
                .send(BackgroundTask::PreloadRepository { repo_path });
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> PerformanceStats {
        self.stats.lock().unwrap().clone()
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        if !self.config.caching_enabled {
            return;
        }

        self.repository_cache.write().unwrap().clear();
        self.branch_cache.write().unwrap().clear();
        self.status_cache.write().unwrap().clear();
        *self.repository_list_cache.write().unwrap() = None;

        info!("Cleared all git operation caches");
    }

    /// Efficient workspace scanning with memory optimization
    async fn scan_workspace_efficiently(
        &self,
        workspace_path: &Path,
    ) -> Result<Vec<RepositoryInfo>> {
        let mut repositories = Vec::new();
        let mut visited_paths = HashSet::new();
        let max_depth = 5; // Prevent deep recursion

        // Use a work queue to control memory usage
        let mut work_queue = vec![(workspace_path.to_path_buf(), 0)];

        while let Some((current_path, depth)) = work_queue.pop() {
            if depth > max_depth || visited_paths.contains(&current_path) {
                continue;
            }

            visited_paths.insert(current_path.clone());

            // Check if this directory is a git repository
            if let Ok(repo) = GitRepository::open(&current_path) {
                let name = current_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                repositories.push(RepositoryInfo {
                    path: current_path.clone(),
                    name,
                    is_bare: repo.is_bare(),
                    has_changes: false, // Will be updated in background
                });

                // Don't scan subdirectories of git repositories
                continue;
            }

            // Scan subdirectories (but limit to avoid memory issues)
            if let Ok(entries) = std::fs::read_dir(&current_path) {
                let mut subdir_count = 0;
                for entry in entries {
                    if subdir_count >= 100 {
                        // Limit subdirectories to scan
                        break;
                    }

                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_dir()
                            && !path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .starts_with('.')
                        {
                            work_queue.push((path, depth + 1));
                            subdir_count += 1;
                        }
                    }
                }
            }

            // Yield periodically to prevent blocking
            if repositories.len() % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }

        info!(
            "Scanned workspace, found {} repositories",
            repositories.len()
        );
        Ok(repositories)
    }

    /// Load branches without caching (for fresh data)
    async fn load_branches_fresh(&self, repo_path: &Path) -> Result<Vec<BranchInfo>> {
        self.record_cache_miss();

        let start_time = Instant::now();
        let permit = self.operation_semaphore.acquire().await.unwrap();

        let branches = timeout(
            Duration::from_millis(self.config.operation_timeout_ms),
            async move {
                let _permit = permit;
                let repo = GitRepository::open(repo_path)?;
                repo.list_branches()
            },
        )
        .await;

        let elapsed = start_time.elapsed();
        self.record_operation_time(elapsed);

        match branches {
            Ok(Ok(branches)) => {
                // Cache the result
                if self.config.caching_enabled {
                    let mut cache = self.branch_cache.write().unwrap();
                    cache.insert(repo_path.to_path_buf(), CacheEntry::new(branches.clone()));
                }

                Ok(branches)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                self.record_timeout();
                Err(anyhow::anyhow!("Branch loading operation timed out"))
            }
        }
    }

    /// Process background tasks
    async fn process_background_task(
        task: BackgroundTask,
        branch_cache: &Arc<RwLock<HashMap<PathBuf, CacheEntry<Vec<BranchInfo>>>>>,
        status_cache: &Arc<RwLock<HashMap<PathBuf, CacheEntry<Vec<(PathBuf, git2::Status)>>>>>,
        repository_list_cache: &Arc<RwLock<Option<CacheEntry<Vec<RepositoryInfo>>>>>,
        stats: &Arc<Mutex<PerformanceStats>>,
        config: &PerformanceConfig,
    ) {
        debug!("Processing background task: {:?}", task);

        match task {
            BackgroundTask::RefreshRepositoryList { workspace_path } => {
                let repositories = GitRepository::discover_repositories(&workspace_path);
                if !repositories.is_empty() {
                    let mut cache = repository_list_cache.write().unwrap();
                    *cache = Some(CacheEntry::new(repositories));
                    debug!("Background: Refreshed repository list");
                }
            }

            BackgroundTask::RefreshBranches { repo_path } => {
                if let Ok(repo) = GitRepository::open(&repo_path) {
                    if let Ok(branches) = repo.list_branches() {
                        let mut cache = branch_cache.write().unwrap();
                        cache.insert(repo_path, CacheEntry::new(branches));
                        debug!("Background: Refreshed branches");
                    }
                }
            }

            BackgroundTask::RefreshStatus { repo_path } => {
                if let Ok(repo) = GitRepository::open(&repo_path) {
                    if let Ok(statuses) = repo.file_statuses() {
                        let mut cache = status_cache.write().unwrap();
                        cache.insert(repo_path, CacheEntry::new(statuses));
                        debug!("Background: Refreshed status");
                    }
                }
            }

            BackgroundTask::PreloadRepository { repo_path: _ } => {
                // Repository preloading disabled due to thread safety constraints
                // with git2::Repository containing non-Sync raw pointers
                debug!("Background: Repository preloading skipped (thread safety)");
            }

            BackgroundTask::CacheCleanup => {
                Self::cleanup_expired_caches(
                    branch_cache,
                    status_cache,
                    repository_list_cache,
                    config,
                );
                debug!("Background: Cleaned up expired caches");
            }
        }

        // Update stats
        let mut stats = stats.lock().unwrap();
        stats.background_tasks_completed += 1;
    }

    /// Clean up expired cache entries
    fn cleanup_expired_caches(
        branch_cache: &Arc<RwLock<HashMap<PathBuf, CacheEntry<Vec<BranchInfo>>>>>,
        status_cache: &Arc<RwLock<HashMap<PathBuf, CacheEntry<Vec<(PathBuf, git2::Status)>>>>>,
        repository_list_cache: &Arc<RwLock<Option<CacheEntry<Vec<RepositoryInfo>>>>>,
        config: &PerformanceConfig,
    ) {
        let ttl = Duration::from_secs(config.cache_ttl_seconds);

        // Clean branch cache
        {
            let mut cache = branch_cache.write().unwrap();
            cache.retain(|_, entry| !entry.is_expired(ttl));
        }

        // Clean status cache (shorter TTL)
        {
            let mut cache = status_cache.write().unwrap();
            let status_ttl = Duration::from_secs(config.cache_ttl_seconds / 4);
            cache.retain(|_, entry| !entry.is_expired(status_ttl));
        }

        // Clean repository list cache
        {
            let mut cache = repository_list_cache.write().unwrap();
            if let Some(entry) = &*cache {
                if entry.is_expired(ttl) {
                    *cache = None;
                }
            }
        }

        debug!("Cleaned up expired cache entries");
    }

    // Statistics recording methods
    fn record_cache_hit(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.cache_hits += 1;
    }

    fn record_cache_miss(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.cache_misses += 1;
    }

    fn record_operation_time(&self, elapsed: Duration) {
        let mut stats = self.stats.lock().unwrap();
        let elapsed_ms = elapsed.as_millis() as f64;

        // Update running average
        let total_ops = stats.total_operations as f64;
        stats.average_operation_time_ms =
            (stats.average_operation_time_ms * total_ops + elapsed_ms) / (total_ops + 1.0);

        stats.total_operations += 1;
    }

    fn record_timeout(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.operations_timed_out += 1;
    }
}

/// Memory-efficient batch processor for git operations
pub struct BatchProcessor {
    batch_size: usize,
    semaphore: Arc<Semaphore>,
}

impl BatchProcessor {
    pub fn new(batch_size: usize, max_concurrent: usize) -> Self {
        Self {
            batch_size,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// Process items in batches with controlled concurrency
    pub async fn process_batches<T, F, R>(&self, items: Vec<T>, processor: F) -> Vec<Result<R>>
    where
        T: Send + Clone + 'static,
        F: Fn(T) -> Result<R> + Send + Sync + Clone + 'static,
        R: Send + 'static,
    {
        let mut results = Vec::new();
        let chunks: Vec<_> = items.chunks(self.batch_size).collect();

        for chunk in chunks {
            let chunk_results = self.process_chunk(chunk.to_vec(), processor.clone()).await;
            results.extend(chunk_results);
        }

        results
    }

    async fn process_chunk<T, F, R>(&self, items: Vec<T>, processor: F) -> Vec<Result<R>>
    where
        T: Send + Clone + 'static,
        F: Fn(T) -> Result<R> + Send + Sync + Clone + 'static,
        R: Send + 'static,
    {
        let semaphore = self.semaphore.clone();
        let tasks: Vec<_> = items
            .into_iter()
            .map(|item| {
                let processor = processor.clone();
                let semaphore = semaphore.clone();
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    processor(item)
                })
            })
            .collect();

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(_) => results.push(Err(anyhow::anyhow!("Task panicked"))),
            }
        }

        results
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_performance_config() {
        let config = PerformanceConfig::default();
        assert!(config.background_processing);
        assert!(config.caching_enabled);
        assert_eq!(config.cache_ttl_seconds, 300);
    }

    #[tokio::test]
    async fn test_paginated_result() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = PaginatedResult::new(items, 0, 3);

        assert_eq!(result.items, vec![1, 2, 3]);
        assert_eq!(result.page, 0);
        assert_eq!(result.total_pages, 4);
        assert_eq!(result.total_items, 10);
        assert!(result.has_next);
        assert!(!result.has_previous);
    }

    #[tokio::test]
    async fn test_batch_processor() {
        let processor = BatchProcessor::new(2, 4);
        let items = vec![1, 2, 3, 4, 5];

        let results = processor.process_batches(items, |x| Ok(x * 2)).await;

        assert_eq!(results.len(), 5);
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok());
            assert_eq!(result.as_ref().unwrap(), &((i + 1) * 2));
        }
    }
}
