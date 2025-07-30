//! Advanced Build System Monitor
//!
//! Provides real-time monitoring of Rust build systems with:
//! - File watching with intelligent debouncing
//! - Multi-tool integration (cargo, clippy, fmt, test)
//! - Background compilation and analysis
//! - Problem aggregation and categorization
//! - Performance optimization with caching

use anyhow::Result;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

use super::problems::{Problem, ProblemCategory, ProblemSeverity};

/// Advanced build system monitor with real-time capabilities
pub struct AdvancedBuildMonitor {
    workspace_path: PathBuf,
    _file_watcher: RecommendedWatcher,
    event_sender: mpsc::UnboundedSender<BuildMonitorEvent>,
    settings: Arc<RwLock<BuildSettings>>,
    build_cache: Arc<RwLock<BuildCache>>,
}

/// Build monitor events
#[derive(Debug, Clone)]
pub enum BuildMonitorEvent {
    FileChanged {
        paths: Vec<PathBuf>,
        change_type: FileChangeType,
    },
    BuildStarted {
        tool: BuildTool,
    },
    BuildCompleted {
        tool: BuildTool,
        success: bool,
        duration: Duration,
        problems: Vec<Problem>,
    },
    CacheUpdated {
        tool: BuildTool,
        cache_hit: bool,
    },
}

/// Types of file changes
#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeType {
    SourceCode,
    Configuration,
    Test,
    Documentation,
    Other,
}

/// Build tools supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildTool {
    CargoCheck,
    CargoClippy,
    CargoTest,
    CargoFmt,
    CargoDoc,
}

/// Build monitor settings
#[derive(Debug, Clone)]
pub struct BuildSettings {
    pub debounce_duration: Duration,
    pub auto_build_enabled: bool,
    pub cache_enabled: bool,
    pub parallel_builds: bool,
    pub tools_enabled: HashSet<BuildTool>,
    pub max_parallel_jobs: usize,
}

/// Build result cache
#[derive(Debug, Default)]
struct BuildCache {
    results: HashMap<BuildCacheKey, BuildCacheEntry>,
    file_hashes: HashMap<PathBuf, u64>,
}

/// Cache key for build results
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BuildCacheKey {
    tool: BuildTool,
    files_hash: u64,
    config_hash: u64,
}

/// Cached build entry
#[derive(Debug, Clone)]
struct BuildCacheEntry {
    problems: Vec<Problem>,
    timestamp: Instant,
    success: bool,
}

impl Default for BuildSettings {
    fn default() -> Self {
        let mut tools_enabled = HashSet::new();
        tools_enabled.insert(BuildTool::CargoCheck);
        tools_enabled.insert(BuildTool::CargoClippy);
        tools_enabled.insert(BuildTool::CargoFmt);
        
        Self {
            debounce_duration: Duration::from_millis(750),
            auto_build_enabled: true,
            cache_enabled: true,
            parallel_builds: true,
            tools_enabled,
            max_parallel_jobs: 4,
        }
    }
}

impl AdvancedBuildMonitor {
    /// Create new advanced build monitor
    pub async fn new(workspace_path: &Path) -> Result<(Self, mpsc::UnboundedReceiver<BuildMonitorEvent>)> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let (fs_sender, fs_receiver) = std::sync::mpsc::channel();
        
        let mut watcher = notify::recommended_watcher(fs_sender)?;
        
        // Set up comprehensive file watching
        Self::setup_file_watching(&mut watcher, workspace_path)?;
        
        let settings = Arc::new(RwLock::new(BuildSettings::default()));
        let build_cache = Arc::new(RwLock::new(BuildCache::default()));
        
        // Start file monitoring thread
        let workspace_clone = workspace_path.to_path_buf();
        let event_sender_clone = event_sender.clone();
        let settings_clone = settings.clone();
        let cache_clone = build_cache.clone();
        
        std::thread::spawn(move || {
            Self::monitor_file_changes(fs_receiver, event_sender_clone, workspace_clone, settings_clone, cache_clone);
        });
        
        let monitor = Self {
            workspace_path: workspace_path.to_path_buf(),
            _file_watcher: watcher,
            event_sender,
            settings,
            build_cache,
        };
        
        info!("🔍 Advanced build monitor initialized for: {}", workspace_path.display());
        
        Ok((monitor, event_receiver))
    }
    
    /// Set up comprehensive file watching
    fn setup_file_watching(watcher: &mut RecommendedWatcher, workspace_path: &Path) -> Result<()> {
        // Core source directories
        if workspace_path.join("src").exists() {
            watcher.watch(&workspace_path.join("src"), RecursiveMode::Recursive)?;
            debug!("Watching src/ directory");
        }
        
        // Test directories
        if workspace_path.join("tests").exists() {
            watcher.watch(&workspace_path.join("tests"), RecursiveMode::Recursive)?;
            debug!("Watching tests/ directory");
        }
        
        if workspace_path.join("benches").exists() {
            watcher.watch(&workspace_path.join("benches"), RecursiveMode::Recursive)?;
            debug!("Watching benches/ directory");
        }
        
        if workspace_path.join("examples").exists() {
            watcher.watch(&workspace_path.join("examples"), RecursiveMode::Recursive)?;
            debug!("Watching examples/ directory");
        }
        
        // Configuration files
        let config_files = [
            "Cargo.toml",
            "Cargo.lock",
            ".cargo/config.toml",
            "rust-toolchain.toml",
            "clippy.toml",
            "rustfmt.toml",
        ];
        
        for config_file in &config_files {
            let config_path = workspace_path.join(config_file);
            if config_path.exists() {
                watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
                debug!("Watching config file: {}", config_file);
            }
        }
        
        Ok(())
    }
    
    /// Monitor file changes and trigger builds
    fn monitor_file_changes(
        receiver: std::sync::mpsc::Receiver<notify::Result<notify::Event>>,
        event_sender: mpsc::UnboundedSender<BuildMonitorEvent>,
        workspace_path: PathBuf,
        settings: Arc<RwLock<BuildSettings>>,
        cache: Arc<RwLock<BuildCache>>,
    ) {
        let mut last_build_time = Instant::now();
        let mut pending_changes: HashMap<PathBuf, Instant> = HashMap::new();
        
        loop {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    if let Some((paths, change_type)) = Self::process_file_event(&event, &workspace_path) {
                        let now = Instant::now();
                        
                        // Update pending changes
                        for path in &paths {
                            pending_changes.insert(path.clone(), now);
                        }
                        
                        // Send file change event
                        let _ = event_sender.send(BuildMonitorEvent::FileChanged {
                            paths: paths.clone(),
                            change_type: change_type.clone(),
                        });
                        
                        // Check if we should trigger a build
                        if let Some(debounce_duration) = Self::get_debounce_duration(&settings) {
                            if last_build_time.elapsed() > debounce_duration && !pending_changes.is_empty() {
                                // Collect all pending changes
                                let changed_files: Vec<PathBuf> = pending_changes.keys().cloned().collect();
                                pending_changes.clear();
                                last_build_time = now;
                                
                                // Trigger builds in background
                                let workspace_clone = workspace_path.clone();
                                let sender_clone = event_sender.clone();
                                let settings_clone = settings.clone();
                                let cache_clone = cache.clone();
                                
                                tokio::spawn(async move {
                                    Self::run_incremental_builds(
                                        workspace_clone,
                                        sender_clone,
                                        settings_clone,
                                        cache_clone,
                                        changed_files,
                                        change_type,
                                    ).await;
                                });
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("File watcher error: {}", e);
                }
                Err(_) => {
                    // Timeout - clean up old pending changes
                    let cutoff = Instant::now() - Duration::from_secs(5);
                    pending_changes.retain(|_, timestamp| *timestamp >= cutoff);
                }
            }
        }
    }
    
    /// Process file system event
    fn process_file_event(
        event: &notify::Event,
        workspace_path: &Path,
    ) -> Option<(Vec<PathBuf>, FileChangeType)> {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                let mut relevant_paths = Vec::new();
                let mut change_type = FileChangeType::Other;
                
                for path in &event.paths {
                    if let Ok(relative_path) = path.strip_prefix(workspace_path) {
                        let path_str = relative_path.to_string_lossy();
                        
                        // Determine change type
                        if path_str.ends_with(".rs") {
                            if path_str.contains("/tests/") || path_str.starts_with("tests/") {
                                change_type = FileChangeType::Test;
                            } else {
                                change_type = FileChangeType::SourceCode;
                            }
                            relevant_paths.push(path.clone());
                        } else if path_str.ends_with("Cargo.toml") 
                            || path_str.ends_with("Cargo.lock")
                            || path_str.contains(".cargo/")
                            || path_str.ends_with("rust-toolchain.toml") {
                            change_type = FileChangeType::Configuration;
                            relevant_paths.push(path.clone());
                        } else if path_str.ends_with(".md") || path_str.contains("/docs/") {
                            change_type = FileChangeType::Documentation;
                            relevant_paths.push(path.clone());
                        }
                    }
                }
                
                if !relevant_paths.is_empty() {
                    Some((relevant_paths, change_type))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Get debounce duration from settings
    fn get_debounce_duration(settings: &Arc<RwLock<BuildSettings>>) -> Option<Duration> {
        if let Ok(settings) = settings.try_read() {
            if settings.auto_build_enabled {
                Some(settings.debounce_duration)
            } else {
                None
            }
        } else {
            Some(Duration::from_millis(750)) // Default fallback
        }
    }
    
    /// Run incremental builds based on file changes
    async fn run_incremental_builds(
        workspace_path: PathBuf,
        event_sender: mpsc::UnboundedSender<BuildMonitorEvent>,
        settings: Arc<RwLock<BuildSettings>>,
        cache: Arc<RwLock<BuildCache>>,
        changed_files: Vec<PathBuf>,
        change_type: FileChangeType,
    ) {
        let enabled_tools = {
            let settings = settings.read().await;
            settings.tools_enabled.clone()
        };
        
        // Determine which tools to run based on change type
        let mut tools_to_run = Vec::new();
        
        match change_type {
            FileChangeType::SourceCode => {
                if enabled_tools.contains(&BuildTool::CargoCheck) {
                    tools_to_run.push(BuildTool::CargoCheck);
                }
                if enabled_tools.contains(&BuildTool::CargoClippy) {
                    tools_to_run.push(BuildTool::CargoClippy);
                }
                if enabled_tools.contains(&BuildTool::CargoFmt) {
                    tools_to_run.push(BuildTool::CargoFmt);
                }
            }
            FileChangeType::Test => {
                if enabled_tools.contains(&BuildTool::CargoCheck) {
                    tools_to_run.push(BuildTool::CargoCheck);
                }
                if enabled_tools.contains(&BuildTool::CargoTest) {
                    tools_to_run.push(BuildTool::CargoTest);
                }
            }
            FileChangeType::Configuration => {
                // Configuration changes might affect everything
                tools_to_run.extend(enabled_tools.iter().cloned());
            }
            FileChangeType::Documentation => {
                if enabled_tools.contains(&BuildTool::CargoDoc) {
                    tools_to_run.push(BuildTool::CargoDoc);
                }
            }
            _ => {}
        }
        
        // Run tools in parallel if enabled
        let parallel_builds = {
            let settings = settings.read().await;
            settings.parallel_builds
        };
        
        if parallel_builds {
            // Run tools in parallel
            let mut tasks = Vec::new();
            
            for tool in tools_to_run {
                let workspace_clone = workspace_path.clone();
                let sender_clone = event_sender.clone();
                let cache_clone = cache.clone();
                let files_clone = changed_files.clone();
                
                let task = tokio::spawn(async move {
                    Self::run_single_tool(workspace_clone, sender_clone, cache_clone, tool, files_clone).await
                });
                
                tasks.push(task);
            }
            
            // Wait for all tasks to complete
            for task in tasks {
                if let Err(e) = task.await {
                    error!("Build task failed: {}", e);
                }
            }
        } else {
            // Run tools sequentially
            for tool in tools_to_run {
                Self::run_single_tool(
                    workspace_path.clone(),
                    event_sender.clone(),
                    cache.clone(),
                    tool,
                    changed_files.clone(),
                ).await;
            }
        }
    }
    
    /// Run a single build tool
    async fn run_single_tool(
        workspace_path: PathBuf,
        event_sender: mpsc::UnboundedSender<BuildMonitorEvent>,
        cache: Arc<RwLock<BuildCache>>,
        tool: BuildTool,
        _changed_files: Vec<PathBuf>,
    ) {
        let start_time = Instant::now();
        
        // Send build started event
        let _ = event_sender.send(BuildMonitorEvent::BuildStarted { tool: tool.clone() });
        
        // Check cache first
        if let Some(cached_result) = Self::check_cache(&cache, &tool, &workspace_path).await {
            let _ = event_sender.send(BuildMonitorEvent::CacheUpdated {
                tool: tool.clone(),
                cache_hit: true,
            });
            
            let _ = event_sender.send(BuildMonitorEvent::BuildCompleted {
                tool,
                success: cached_result.success,
                duration: Duration::from_millis(0), // Cached result
                problems: cached_result.problems,
            });
            
            return;
        }
        
        // Run the actual tool
        let result = Self::execute_tool(&tool, &workspace_path).await;
        let duration = start_time.elapsed();
        
        match result {
            Ok((success, problems)) => {
                // Update cache
                {
                    let mut cache = cache.write().await;
                    Self::update_cache(&mut cache, &tool, &workspace_path, success, &problems);
                }
                
                let _ = event_sender.send(BuildMonitorEvent::BuildCompleted {
                    tool: tool.clone(),
                    success,
                    duration,
                    problems,
                });
            }
            Err(e) => {
                error!("Failed to execute tool {:?}: {}", tool, e);
                
                let _ = event_sender.send(BuildMonitorEvent::BuildCompleted {
                    tool: tool.clone(),
                    success: false,
                    duration,
                    problems: vec![Problem {
                        severity: ProblemSeverity::Error,
                        category: ProblemCategory::BuildError,
                        message: format!("Tool execution failed: {}", e),
                        file_path: None,
                        line: None,
                        column: None,
                        source: format!("{:?}", tool),
                        context: None,
                    }],
                });
            }
        }
    }
    
    /// Execute a build tool and parse results
    async fn execute_tool(tool: &BuildTool, workspace_path: &Path) -> Result<(bool, Vec<Problem>)> {
        let (command, args) = match tool {
            BuildTool::CargoCheck => ("cargo", vec!["check", "--message-format=json", "--all-targets"]),
            BuildTool::CargoClippy => ("cargo", vec!["clippy", "--message-format=json", "--all-targets", "--", "-W", "clippy::all"]),
            BuildTool::CargoTest => ("cargo", vec!["test", "--no-run", "--message-format=json"]),
            BuildTool::CargoFmt => ("cargo", vec!["fmt", "--check"]),
            BuildTool::CargoDoc => ("cargo", vec!["doc", "--no-deps", "--message-format=json"]),
        };
        
        debug!("Executing: {} {}", command, args.join(" "));
        
        let output = Command::new(command)
            .args(&args)
            .current_dir(workspace_path)
            .output()
            .await?;
        
        let success = output.status.success();
        let problems = Self::parse_tool_output(tool, &output.stdout, &output.stderr, workspace_path)?;
        
        Ok((success, problems))
    }
    
    /// Parse tool output into problems
    fn parse_tool_output(
        tool: &BuildTool,
        stdout: &[u8],
        stderr: &[u8],
        workspace_path: &Path,
    ) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();
        
        match tool {
            BuildTool::CargoCheck | BuildTool::CargoClippy | BuildTool::CargoTest | BuildTool::CargoDoc => {
                let stdout_str = String::from_utf8_lossy(stdout);
                
                for line in stdout_str.lines() {
                    if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(problem) = Self::parse_cargo_json_message(&message, tool, workspace_path) {
                            problems.push(problem);
                        }
                    }
                }
            }
            BuildTool::CargoFmt => {
                let stderr_str = String::from_utf8_lossy(stderr);
                
                for line in stderr_str.lines() {
                    if line.starts_with("Diff in ") {
                        if let Some(file_path) = line.strip_prefix("Diff in ") {
                            let path = PathBuf::from(file_path.trim());
                            
                            problems.push(Problem {
                                severity: ProblemSeverity::Warning,
                                category: ProblemCategory::FormatError,
                                message: "File needs formatting".to_string(),
                                file_path: Some(workspace_path.join(path)),
                                line: None,
                                column: None,
                                source: "cargo-fmt".to_string(),
                                context: Some("Run `cargo fmt` to fix formatting".to_string()),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(problems)
    }
    
    /// Parse Cargo JSON message into problem
    fn parse_cargo_json_message(
        message: &serde_json::Value,
        tool: &BuildTool,
        workspace_path: &Path,
    ) -> Option<Problem> {
        let message_obj = message.get("message")?;
        let level = message_obj.get("level")?.as_str()?;
        
        let severity = match level {
            "error" => ProblemSeverity::Error,
            "warning" => ProblemSeverity::Warning,
            "note" | "help" => ProblemSeverity::Info,
            _ => return None,
        };
        
        let category = match (tool, level) {
            (BuildTool::CargoCheck, "error") => ProblemCategory::BuildError,
            (BuildTool::CargoCheck, "warning") => ProblemCategory::BuildWarning,
            (BuildTool::CargoClippy, "error") => ProblemCategory::LintError,
            (BuildTool::CargoClippy, "warning") => ProblemCategory::LintWarning,
            (BuildTool::CargoTest, "error") => ProblemCategory::TestFailure,
            _ => ProblemCategory::BuildWarning,
        };
        
        let message_text = message_obj
            .get("message")?
            .as_str()?
            .to_string();
        
        // Extract location and context
        let (file_path, line, column, context) = Self::extract_location_info(message_obj, workspace_path);
        
        Some(Problem {
            severity,
            category,
            message: message_text,
            file_path,
            line,
            column,
            source: format!("{:?}", tool).to_lowercase(),
            context,
        })
    }
    
    /// Extract location information from Cargo message
    fn extract_location_info(
        message_obj: &serde_json::Value, 
        workspace_path: &Path
    ) -> (Option<PathBuf>, Option<usize>, Option<usize>, Option<String>) {
        if let Some(spans) = message_obj.get("spans") {
            if let Some(span) = spans.as_array().and_then(|arr| arr.first()) {
                let file = span.get("file_name")
                    .and_then(|f| f.as_str())
                    .map(|f| {
                        let path = PathBuf::from(f);
                        if path.is_absolute() {
                            path
                        } else {
                            workspace_path.join(path)
                        }
                    });
                let line = span.get("line_start")
                    .and_then(|l| l.as_u64())
                    .map(|l| l as usize);
                let column = span.get("column_start")
                    .and_then(|c| c.as_u64())
                    .map(|c| c as usize);
                
                // Extract context
                let mut context_parts = Vec::new();
                
                if let Some(label) = span.get("label").and_then(|l| l.as_str()) {
                    context_parts.push(label.to_string());
                }
                
                if let Some(suggestion) = span.get("suggested_replacement").and_then(|s| s.as_str()) {
                    context_parts.push(format!("Suggestion: {}", suggestion));
                }
                
                let context = if context_parts.is_empty() {
                    None
                } else {
                    Some(context_parts.join(" | "))
                };
                    
                return (file, line, column, context);
            }
        }
        (None, None, None, None)
    }
    
    /// Check cache for existing build result
    async fn check_cache(
        cache: &Arc<RwLock<BuildCache>>,
        tool: &BuildTool,
        workspace_path: &Path,
    ) -> Option<BuildCacheEntry> {
        {
            let cache = cache.read().await;
            let cache_key = Self::compute_cache_key(tool, workspace_path, &cache)?;
            
            if let Some(entry) = cache.results.get(&cache_key) {
                // Check if cache entry is still fresh (e.g., within 5 minutes)
                if entry.timestamp.elapsed() < Duration::from_secs(300) {
                    debug!("Cache hit for {:?}", tool);
                    return Some(entry.clone());
                }
            }
        }
        None
    }
    
    /// Update cache with new build result
    fn update_cache(
        cache: &mut BuildCache,
        tool: &BuildTool,
        workspace_path: &Path,
        success: bool,
        problems: &[Problem],
    ) {
        if let Some(cache_key) = Self::compute_cache_key(tool, workspace_path, cache) {
            let entry = BuildCacheEntry {
                problems: problems.to_vec(),
                timestamp: Instant::now(),
                success,
            };
            
            cache.results.insert(cache_key, entry);
            
            // Clean up old cache entries (keep only last 100)
            if cache.results.len() > 100 {
                let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
                cache.results.retain(|_, entry| entry.timestamp >= cutoff);
            }
        }
    }
    
    /// Compute cache key for build result
    fn compute_cache_key(
        tool: &BuildTool,
        workspace_path: &Path,
        cache: &BuildCache,
    ) -> Option<BuildCacheKey> {
        // For simplicity, use a hash of relevant file modification times
        let mut files_hash = 0u64;
        let mut config_hash = 0u64;
        
        // Hash source files
        if let Ok(entries) = std::fs::read_dir(workspace_path.join("src")) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        files_hash ^= modified.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
                    }
                }
            }
        }
        
        // Hash config files
        let config_files = ["Cargo.toml", "Cargo.lock"];
        for config_file in &config_files {
            let config_path = workspace_path.join(config_file);
            if let Ok(metadata) = std::fs::metadata(&config_path) {
                if let Ok(modified) = metadata.modified() {
                    config_hash ^= modified.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
                }
            }
        }
        
        Some(BuildCacheKey {
            tool: tool.clone(),
            files_hash,
            config_hash,
        })
    }
    
    /// Trigger manual build for specific tool
    pub async fn trigger_manual_build(&self, tool: BuildTool) -> Result<()> {
        let workspace_clone = self.workspace_path.clone();
        let sender_clone = self.event_sender.clone();
        let cache_clone = self.build_cache.clone();
        
        tokio::spawn(async move {
            Self::run_single_tool(workspace_clone, sender_clone, cache_clone, tool, Vec::new()).await;
        });
        
        Ok(())
    }
    
    /// Update build settings
    pub async fn update_settings(&self, new_settings: BuildSettings) -> Result<()> {
        {
            let mut settings = self.settings.write().await;
            *settings = new_settings;
            info!("Build monitor settings updated");
        }
        Ok(())
    }
    
    /// Get current settings
    pub async fn get_settings(&self) -> Result<BuildSettings> {
        let settings = self.settings.read().await;
        Ok(settings.clone())
    }
    
    /// Clear build cache
    pub async fn clear_cache(&self) -> Result<()> {
        {
            let mut cache = self.build_cache.write().await;
            cache.results.clear();
            cache.file_hashes.clear();
            info!("Build cache cleared");
        }
        Ok(())
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        let cache = self.build_cache.read().await;
        Ok(CacheStats {
            entries: cache.results.len(),
            total_size: cache.results.len() * std::mem::size_of::<BuildCacheEntry>(),
            oldest_entry: cache.results.values()
                .map(|entry| entry.timestamp)
                .min(),
        })
    }
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub entries: usize,
    pub total_size: usize,
    pub oldest_entry: Option<Instant>,
}

impl BuildTool {
    /// Get display name for build tool
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CargoCheck => "Cargo Check",
            Self::CargoClippy => "Clippy",
            Self::CargoTest => "Tests",
            Self::CargoFmt => "Format",
            Self::CargoDoc => "Documentation",
        }
    }
    
    /// Get icon for build tool
    pub fn icon(&self) -> &'static str {
        match self {
            Self::CargoCheck => "🔧",
            Self::CargoClippy => "📎",
            Self::CargoTest => "🧪",
            Self::CargoFmt => "🎨",
            Self::CargoDoc => "📚",
        }
    }
}