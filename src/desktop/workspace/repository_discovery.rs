//! Repository Discovery Service
//!
//! Advanced repository discovery and scanning functionality that extends
//! the basic git repository detection with configurable scanning modes,
//! caching, and comprehensive metadata extraction.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::core::config::HiveConfig;
use crate::desktop::git::repository::{GitRepository, RepositoryInfo as BasicRepositoryInfo};

/// Extended repository information with discovery metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryMetadata {
    /// Basic repository information
    pub basic_info: BasicRepositoryInfo,
    
    /// Discovery metadata
    pub discovery: DiscoveryMetadata,
    
    /// Git status information
    pub git_status: GitStatusInfo,
    
    /// Repository statistics
    pub stats: RepositoryStats,
    
    /// Project information (detected from files)
    pub project_info: Option<ProjectInfo>,
}

/// Discovery metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveryMetadata {
    /// When this repository was first discovered
    pub discovered_at: u64,
    
    /// Last time metadata was refreshed
    pub last_refreshed: u64,
    
    /// How the repository was discovered
    pub discovery_method: DiscoveryMethod,
    
    /// Confidence level of detection (0.0 - 1.0)
    pub confidence: f32,
    
    /// Whether repository is accessible
    pub is_accessible: bool,
    
    /// Any errors encountered during discovery
    pub errors: Vec<String>,
}

/// How the repository was discovered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryMethod {
    /// Found via direct .git directory scan
    DirectScan,
    
    /// Found via git discovery from parent path
    GitDiscovery,
    
    /// Found via configuration or cache
    Cached,
    
    /// Manually added by user
    Manual,
    
    /// Found via symlink resolution
    Symlink,
}

/// Git status information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitStatusInfo {
    /// Current branch name
    pub current_branch: Option<String>,
    
    /// Upstream branch information
    pub upstream: Option<UpstreamInfo>,
    
    /// Working directory status
    pub working_dir_status: WorkingDirStatus,
    
    /// Remote repositories
    pub remotes: Vec<RemoteInfo>,
    
    /// Recent commits (limited number)
    pub recent_commits: Vec<CommitInfo>,
}

/// Upstream branch information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpstreamInfo {
    /// Name of upstream branch
    pub branch_name: String,
    
    /// Remote name
    pub remote_name: String,
    
    /// Commits ahead of upstream
    pub ahead: usize,
    
    /// Commits behind upstream
    pub behind: usize,
}

/// Working directory status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkingDirStatus {
    /// Files with changes
    pub modified_files: usize,
    
    /// Staged files
    pub staged_files: usize,
    
    /// Untracked files
    pub untracked_files: usize,
    
    /// Files with conflicts
    pub conflicted_files: usize,
    
    /// Whether repository is clean
    pub is_clean: bool,
}

/// Remote repository information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteInfo {
    /// Remote name (e.g., "origin")
    pub name: String,
    
    /// Remote URL
    pub url: String,
    
    /// Whether this is the fetch URL
    pub is_fetch: bool,
    
    /// Whether this is the push URL
    pub is_push: bool,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitInfo {
    /// Commit hash (short)
    pub hash: String,
    
    /// Commit message (first line)
    pub message: String,
    
    /// Author name
    pub author: String,
    
    /// Commit timestamp
    pub timestamp: u64,
}

/// Repository statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryStats {
    /// Total number of files in repository (estimated)
    pub file_count: Option<usize>,
    
    /// Repository size on disk (bytes)
    pub size_bytes: Option<u64>,
    
    /// Primary language detected
    pub primary_language: Option<String>,
    
    /// Languages distribution (language -> percentage)
    pub languages: HashMap<String, f32>,
    
    /// Last commit timestamp
    pub last_commit: Option<u64>,
    
    /// Number of contributors
    pub contributor_count: Option<usize>,
}

/// Project information detected from repository files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectInfo {
    /// Project type (rust, javascript, python, etc.)
    pub project_type: ProjectType,
    
    /// Project name (from manifest files)
    pub name: Option<String>,
    
    /// Project version
    pub version: Option<String>,
    
    /// Project description
    pub description: Option<String>,
    
    /// Build system detected
    pub build_system: Option<String>,
    
    /// Key files found
    pub key_files: Vec<String>,
}

/// Detected project type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectType {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    CSharp,
    Cpp,
    C,
    Swift,
    Kotlin,
    Ruby,
    PHP,
    Shell,
    Mixed,
    Unknown,
}

/// Repository discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Scanning mode to use
    pub scanning_mode: ScanningMode,
    
    /// Directories to scan for repositories
    pub scan_paths: Vec<PathBuf>,
    
    /// Maximum depth for recursive scanning
    pub max_depth: usize,
    
    /// Whether to follow symlinks
    pub follow_symlinks: bool,
    
    /// Patterns to exclude from scanning
    pub exclude_patterns: Vec<String>,
    
    /// Maximum repositories to discover
    pub max_repositories: Option<usize>,
    
    /// Cache settings
    pub cache: CacheConfig,
    
    /// Whether to extract detailed project information
    pub extract_project_info: bool,
    
    /// Timeout for individual repository operations
    pub operation_timeout: Duration,
}

/// Scanning mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScanningMode {
    /// Only scan immediate directories for .git folders
    Shallow,
    
    /// Recursively scan directories up to max_depth
    Deep,
    
    /// Use git discovery to find repositories
    GitDiscovery,
    
    /// Hybrid approach: shallow scan with git discovery fallback
    Hybrid,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,
    
    /// Cache duration in seconds
    pub ttl_seconds: u64,
    
    /// Maximum entries in cache
    pub max_entries: usize,
    
    /// Whether to persist cache to disk
    pub persist: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            scanning_mode: ScanningMode::Hybrid,
            scan_paths: vec![
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Developer"),
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Projects"),
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Code"),
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Workspace"),
                PathBuf::from("."),
            ],
            max_depth: 3,
            follow_symlinks: false,
            exclude_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                "build".to_string(),
                ".vscode".to_string(),
                ".idea".to_string(),
                "*.app".to_string(),
                "System Volume Information".to_string(),
                "$RECYCLE.BIN".to_string(),
            ],
            max_repositories: Some(100),
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 3600, // 1 hour
                max_entries: 500,
                persist: true,
            },
            extract_project_info: true,
            operation_timeout: Duration::from_secs(10),
        }
    }
}

/// Cache entry for repository discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    metadata: RepositoryMetadata,
    cached_at: u64,
}

impl CacheEntry {
    fn is_expired(&self, ttl_seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now - self.cached_at > ttl_seconds
    }
}

/// Repository Discovery Service
pub struct RepositoryDiscoveryService {
    config: DiscoveryConfig,
    cache: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    scan_stats: Arc<RwLock<ScanStats>>,
}

/// Statistics about scanning operations
#[derive(Debug, Default, Clone)]
struct ScanStats {
    total_scans: u64,
    total_repositories_found: u64,
    total_errors: u64,
    last_scan_duration: Option<Duration>,
    cache_hits: u64,
    cache_misses: u64,
}

impl RepositoryDiscoveryService {
    /// Create a new repository discovery service
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            scan_stats: Arc::new(RwLock::new(ScanStats::default())),
        }
    }
    
    /// Create service from hive configuration
    pub fn from_hive_config(hive_config: &HiveConfig) -> Self {
        let config = Self::convert_hive_config_to_discovery_config(&hive_config.repository_discovery);
        Self::new(config)
    }
    
    /// Convert HiveConfig's RepositoryDiscoveryConfig to our DiscoveryConfig
    fn convert_hive_config_to_discovery_config(hive_config: &crate::core::config::RepositoryDiscoveryConfig) -> DiscoveryConfig {
        let scanning_mode = match hive_config.scanning_mode.as_str() {
            "shallow" => ScanningMode::Shallow,
            "deep" => ScanningMode::Deep,
            "git_discovery" => ScanningMode::GitDiscovery,
            "hybrid" => ScanningMode::Hybrid,
            _ => ScanningMode::Hybrid,
        };
        
        DiscoveryConfig {
            scanning_mode,
            scan_paths: hive_config.scan_paths.clone(),
            max_depth: hive_config.max_depth,
            follow_symlinks: hive_config.follow_symlinks,
            exclude_patterns: hive_config.exclude_patterns.clone(),
            max_repositories: hive_config.max_repositories,
            cache: CacheConfig {
                enabled: hive_config.cache_enabled,
                ttl_seconds: hive_config.cache_ttl_seconds,
                max_entries: hive_config.cache_max_entries,
                persist: hive_config.cache_persist,
            },
            extract_project_info: hive_config.extract_project_info,
            operation_timeout: Duration::from_secs(hive_config.operation_timeout_seconds),
        }
    }
    
    /// Discover repositories in the configured scan paths
    pub async fn discover_repositories(&self) -> Result<Vec<RepositoryMetadata>> {
        let start_time = std::time::Instant::now();
        
        {
            let mut stats = self.scan_stats.write().await;
            stats.total_scans += 1;
        }
        
        info!("Starting repository discovery with mode: {:?}", self.config.scanning_mode);
        
        let mut all_repositories = Vec::new();
        let mut seen_paths = HashSet::new();
        
        for scan_path in &self.config.scan_paths {
            if !scan_path.exists() {
                debug!("Scan path does not exist: {:?}", scan_path);
                continue;
            }
            
            match self.scan_path(scan_path, &mut seen_paths).await {
                Ok(mut repos) => {
                    info!("Found {} repositories in {:?}", repos.len(), scan_path);
                    all_repositories.append(&mut repos);
                }
                Err(e) => {
                    warn!("Failed to scan path {:?}: {}", scan_path, e);
                    let mut stats = self.scan_stats.write().await;
                    stats.total_errors += 1;
                }
            }
            
            // Check if we've reached the maximum
            if let Some(max) = self.config.max_repositories {
                if all_repositories.len() >= max {
                    info!("Reached maximum repository limit: {}", max);
                    all_repositories.truncate(max);
                    break;
                }
            }
        }
        
        let duration = start_time.elapsed();
        {
            let mut stats = self.scan_stats.write().await;
            stats.last_scan_duration = Some(duration);
            stats.total_repositories_found += all_repositories.len() as u64;
        }
        
        info!(
            "Repository discovery completed: found {} repositories in {:?}",
            all_repositories.len(),
            duration
        );
        
        Ok(all_repositories)
    }
    
    /// Scan a specific path for repositories
    async fn scan_path(
        &self,
        path: &Path,
        seen_paths: &mut HashSet<PathBuf>,
    ) -> Result<Vec<RepositoryMetadata>> {
        if seen_paths.contains(path) {
            return Ok(Vec::new());
        }
        
        seen_paths.insert(path.to_path_buf());
        
        match self.config.scanning_mode {
            ScanningMode::Shallow => self.shallow_scan(path).await,
            ScanningMode::Deep => self.deep_scan(path).await,
            ScanningMode::GitDiscovery => self.git_discovery_scan(path).await,
            ScanningMode::Hybrid => self.hybrid_scan(path).await,
        }
    }
    
    /// Perform a shallow scan (only immediate directories)
    async fn shallow_scan(&self, path: &Path) -> Result<Vec<RepositoryMetadata>> {
        let mut repositories = Vec::new();
        
        // Check if the path itself is a repository
        if let Some(repo) = self.try_discover_repository(path).await {
            repositories.push(repo);
        }
        
        // Scan immediate subdirectories
        let mut entries = fs::read_dir(path).await
            .context("Failed to read directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            
            if !entry_path.is_dir() {
                continue;
            }
            
            if self.should_exclude_path(&entry_path) {
                continue;
            }
            
            if let Some(repo) = self.try_discover_repository(&entry_path).await {
                repositories.push(repo);
            }
        }
        
        Ok(repositories)
    }
    
    /// Perform a deep scan using iterative approach to avoid recursion issues
    async fn deep_scan(&self, path: &Path) -> Result<Vec<RepositoryMetadata>> {
        use std::collections::VecDeque;
        
        let mut repositories = Vec::new();
        let mut queue = VecDeque::new();
        
        // Initialize with root path
        queue.push_back((path.to_path_buf(), 0));
        
        while let Some((current_path, depth)) = queue.pop_front() {
            if depth > self.config.max_depth {
                continue;
            }
            
            // Check if the path itself is a repository
            if let Some(repo) = self.try_discover_repository(&current_path).await {
                repositories.push(repo);
                // If we found a repository, don't scan its subdirectories
                continue;
            }
            
            // Scan subdirectories
            let entries = match fs::read_dir(&current_path).await {
                Ok(entries) => entries,
                Err(e) => {
                    debug!("Cannot read directory {:?}: {}", current_path, e);
                    continue;
                }
            };
            
            let mut entries = entries;
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                
                if !entry_path.is_dir() {
                    continue;
                }
                
                if self.should_exclude_path(&entry_path) {
                    continue;
                }
                
                // Handle symlinks
                if entry_path.is_symlink() && !self.config.follow_symlinks {
                    continue;
                }
                
                // Add to queue for processing
                queue.push_back((entry_path, depth + 1));
            }
        }
        
        Ok(repositories)
    }
    
    /// Perform git discovery scan
    async fn git_discovery_scan(&self, path: &Path) -> Result<Vec<RepositoryMetadata>> {
        // Use git2's discovery mechanism
        if let Some(repo) = self.try_discover_repository(path).await {
            Ok(vec![repo])
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Perform hybrid scan (shallow + git discovery)
    async fn hybrid_scan(&self, path: &Path) -> Result<Vec<RepositoryMetadata>> {
        let mut repositories = Vec::new();
        
        // First try git discovery
        if let Some(repo) = self.try_discover_repository(path).await {
            repositories.push(repo);
        }
        
        // Then do shallow scan of subdirectories
        let mut shallow_repos = self.shallow_scan(path).await?;
        
        // Deduplicate based on repository path
        let existing_paths: HashSet<PathBuf> = repositories
            .iter()
            .map(|r| r.basic_info.path.clone())
            .collect();
        
        shallow_repos.retain(|r| !existing_paths.contains(&r.basic_info.path));
        repositories.append(&mut shallow_repos);
        
        Ok(repositories)
    }
    
    /// Try to discover a repository at the given path
    async fn try_discover_repository(&self, path: &Path) -> Option<RepositoryMetadata> {
        // Check cache first
        if self.config.cache.enabled {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(path) {
                if !entry.is_expired(self.config.cache.ttl_seconds) {
                    {
                        let mut stats = self.scan_stats.write().await;
                        stats.cache_hits += 1;
                    }
                    return Some(entry.metadata.clone());
                }
            }
        }
        
        {
            let mut stats = self.scan_stats.write().await;
            stats.cache_misses += 1;
        }
        
        // Try to discover repository
        let metadata = match self.discover_repository_metadata(path).await {
            Ok(metadata) => metadata,
            Err(e) => {
                debug!("Failed to discover repository at {:?}: {}", path, e);
                return None;
            }
        };
        
        // Cache the result
        if self.config.cache.enabled {
            let entry = CacheEntry {
                metadata: metadata.clone(),
                cached_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            
            let mut cache = self.cache.write().await;
            cache.insert(path.to_path_buf(), entry);
            
            // Trim cache if it's too large
            if cache.len() > self.config.cache.max_entries {
                // Remove oldest entries (simple LRU would be better)
                let paths_to_remove: Vec<PathBuf> = cache
                    .iter()
                    .take(cache.len() - self.config.cache.max_entries)
                    .map(|(path, _)| path.clone())
                    .collect();
                
                for path in paths_to_remove {
                    cache.remove(&path);
                }
            }
        }
        
        Some(metadata)
    }
    
    /// Discover comprehensive metadata for a repository
    async fn discover_repository_metadata(&self, path: &Path) -> Result<RepositoryMetadata> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // First check if there's a .git directory
        let git_dir = path.join(".git");
        let has_git_dir = git_dir.exists();
        
        // Try to open with git2
        let git_repo = GitRepository::open(path);
        
        let (basic_info, git_status, discovery_method, confidence, errors) =
            match (has_git_dir, git_repo) {
                (true, Ok(repo)) => {
                    // Full git repository
                    let basic_info = BasicRepositoryInfo {
                        path: path.to_path_buf(),
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        is_bare: false, // TODO: detect bare repositories
                        has_changes: true, // TODO: detect changes
                    };
                    
                    let git_status = self.extract_git_status(&repo).await;
                    (basic_info, git_status, DiscoveryMethod::DirectScan, 1.0, Vec::new())
                }
                (false, Ok(repo)) => {
                    // Git repository discovered via git2 (might be in parent)
                    let repo_path = repo.path().to_path_buf();
                    let basic_info = BasicRepositoryInfo {
                        path: repo_path.clone(),
                        name: repo_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        is_bare: false,
                        has_changes: true,
                    };
                    
                    let git_status = self.extract_git_status(&repo).await;
                    (basic_info, git_status, DiscoveryMethod::GitDiscovery, 0.8, Vec::new())
                }
                (true, Err(e)) => {
                    // Has .git directory but git2 failed
                    let basic_info = BasicRepositoryInfo {
                        path: path.to_path_buf(),
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        is_bare: false,
                        has_changes: false,
                    };
                    
                    let git_status = GitStatusInfo {
                        current_branch: None,
                        upstream: None,
                        working_dir_status: WorkingDirStatus {
                            modified_files: 0,
                            staged_files: 0,
                            untracked_files: 0,
                            conflicted_files: 0,
                            is_clean: true,
                        },
                        remotes: Vec::new(),
                        recent_commits: Vec::new(),
                    };
                    
                    (basic_info, git_status, DiscoveryMethod::DirectScan, 0.5, vec![e.to_string()])
                }
                (false, Err(e)) => {
                    return Err(e);
                }
            };
        
        let stats = if self.config.extract_project_info {
            self.extract_repository_stats(path).await
        } else {
            RepositoryStats {
                file_count: None,
                size_bytes: None,
                primary_language: None,
                languages: HashMap::new(),
                last_commit: None,
                contributor_count: None,
            }
        };
        
        let project_info = if self.config.extract_project_info {
            self.extract_project_info(path).await
        } else {
            None
        };
        
        Ok(RepositoryMetadata {
            basic_info,
            discovery: DiscoveryMetadata {
                discovered_at: now,
                last_refreshed: now,
                discovery_method,
                confidence,
                is_accessible: true,
                errors,
            },
            git_status,
            stats,
            project_info,
        })
    }
    
    /// Extract git status information from a repository
    async fn extract_git_status(&self, repo: &GitRepository) -> GitStatusInfo {
        let current_branch = repo.current_branch().ok();
        
        let upstream = if let (Ok(Some(upstream_name)), Ok((ahead, behind))) =
            (repo.upstream_branch(), repo.ahead_behind())
        {
            // Parse remote and branch from upstream name
            if let Some((remote, branch)) = upstream_name.split_once('/') {
                Some(UpstreamInfo {
                    branch_name: branch.to_string(),
                    remote_name: remote.to_string(),
                    ahead,
                    behind,
                })
            } else {
                None
            }
        } else {
            None
        };
        
        let working_dir_status = WorkingDirStatus {
            modified_files: 0,    // TODO: extract from git status
            staged_files: 0,      // TODO: extract from git status
            untracked_files: 0,   // TODO: extract from git status
            conflicted_files: 0,  // TODO: extract from git status
            is_clean: true,       // TODO: calculate from above
        };
        
        let remotes = repo.remotes()
            .unwrap_or_default()
            .into_iter()
            .map(|name| RemoteInfo {
                name: name.clone(),
                url: format!("unknown"), // TODO: get actual URL
                is_fetch: true,
                is_push: true,
            })
            .collect();
        
        GitStatusInfo {
            current_branch,
            upstream,
            working_dir_status,
            remotes,
            recent_commits: Vec::new(), // TODO: extract recent commits
        }
    }
    
    /// Extract repository statistics
    async fn extract_repository_stats(&self, path: &Path) -> RepositoryStats {
        // TODO: Implement comprehensive stats extraction
        // This is a placeholder implementation
        RepositoryStats {
            file_count: None,
            size_bytes: None,
            primary_language: None,
            languages: HashMap::new(),
            last_commit: None,
            contributor_count: None,
        }
    }
    
    /// Extract project information from repository files
    async fn extract_project_info(&self, path: &Path) -> Option<ProjectInfo> {
        // Check for common project files
        let mut key_files = Vec::new();
        let mut project_type = ProjectType::Unknown;
        let mut name = None;
        let mut version = None;
        let mut description = None;
        let mut build_system = None;
        
        // Rust project
        if path.join("Cargo.toml").exists() {
            key_files.push("Cargo.toml".to_string());
            project_type = ProjectType::Rust;
            build_system = Some("Cargo".to_string());
            
            // Try to parse Cargo.toml for metadata
            if let Ok(content) = fs::read_to_string(path.join("Cargo.toml")).await {
                // Simple parsing - could use proper TOML parser
                for line in content.lines() {
                    if line.starts_with("name = ") {
                        name = line.split('"').nth(1).map(|s| s.to_string());
                    } else if line.starts_with("version = ") {
                        version = line.split('"').nth(1).map(|s| s.to_string());
                    } else if line.starts_with("description = ") {
                        description = line.split('"').nth(1).map(|s| s.to_string());
                    }
                }
            }
        }
        
        // JavaScript/TypeScript project
        if path.join("package.json").exists() {
            key_files.push("package.json".to_string());
            if path.join("tsconfig.json").exists() {
                project_type = ProjectType::TypeScript;
                key_files.push("tsconfig.json".to_string());
            } else {
                project_type = ProjectType::JavaScript;
            }
            build_system = Some("npm".to_string());
        }
        
        // Python project
        if path.join("setup.py").exists() || path.join("pyproject.toml").exists() {
            project_type = ProjectType::Python;
            if path.join("setup.py").exists() {
                key_files.push("setup.py".to_string());
            }
            if path.join("pyproject.toml").exists() {
                key_files.push("pyproject.toml".to_string());
            }
        }
        
        // Go project
        if path.join("go.mod").exists() {
            key_files.push("go.mod".to_string());
            project_type = ProjectType::Go;
            build_system = Some("Go Modules".to_string());
        }
        
        // Only return project info if we detected something
        if project_type != ProjectType::Unknown || !key_files.is_empty() {
            Some(ProjectInfo {
                project_type,
                name,
                version,
                description,
                build_system,
                key_files,
            })
        } else {
            None
        }
    }
    
    /// Check if a path should be excluded from scanning
    fn should_exclude_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        self.config.exclude_patterns.iter().any(|pattern| {
            // Simple pattern matching - could be enhanced with glob patterns
            if pattern.starts_with("*.") {
                // Extension pattern
                let ext = &pattern[2..];
                path_str.ends_with(ext)
            } else {
                // Directory name pattern
                path_str.contains(pattern)
            }
        })
    }
    
    /// Get discovery statistics
    pub async fn get_stats(&self) -> ScanStats {
        self.scan_stats.read().await.clone()
    }
    
    /// Clear the discovery cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
    
    /// Refresh metadata for a specific repository
    pub async fn refresh_repository(&self, path: &Path) -> Result<RepositoryMetadata> {
        // Remove from cache to force refresh
        if self.config.cache.enabled {
            let mut cache = self.cache.write().await;
            cache.remove(path);
        }
        
        // Discover fresh metadata
        self.discover_repository_metadata(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    
    #[tokio::test]
    async fn test_discovery_service_creation() {
        let config = DiscoveryConfig::default();
        let service = RepositoryDiscoveryService::new(config);
        
        let stats = service.get_stats().await;
        assert_eq!(stats.total_scans, 0);
    }
    
    #[tokio::test]
    async fn test_shallow_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = DiscoveryConfig::default();
        let service = RepositoryDiscoveryService::new(config);
        
        let repos = service.shallow_scan(temp_dir.path()).await.unwrap();
        assert!(repos.is_empty());
    }
    
    #[tokio::test]
    async fn test_project_info_detection_rust() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        
        fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"
description = "A test project"
"#).await.unwrap();
        
        let config = DiscoveryConfig::default();
        let service = RepositoryDiscoveryService::new(config);
        
        let project_info = service.extract_project_info(temp_dir.path()).await.unwrap();
        assert_eq!(project_info.project_type, ProjectType::Rust);
        assert_eq!(project_info.name, Some("test-project".to_string()));
        assert_eq!(project_info.version, Some("0.1.0".to_string()));
        assert!(project_info.key_files.contains(&"Cargo.toml".to_string()));
    }
    
    #[tokio::test]
    async fn test_exclude_patterns() {
        let config = DiscoveryConfig::default();
        let service = RepositoryDiscoveryService::new(config);
        
        assert!(service.should_exclude_path(&PathBuf::from("/test/.git")));
        assert!(service.should_exclude_path(&PathBuf::from("/test/node_modules")));
        assert!(service.should_exclude_path(&PathBuf::from("/test/target")));
        assert!(!service.should_exclude_path(&PathBuf::from("/test/src")));
    }
}