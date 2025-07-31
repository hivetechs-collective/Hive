//! Real-time Updates for Problems Panel
//!
//! Manages real-time updates of problems from various sources including:
//! - File system watching
//! - Build system monitoring
//! - LSP/language server integration
//! - Git status changes

use super::build_integration::{BuildSystemIntegration, BuildTool};
use super::navigation_handler::ProblemNavigationHandler;
use super::problems_panel::{ProblemItem, ProblemsState};
use crate::desktop::events::{Event, EventHandler, EventType};
use anyhow::Result;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Real-time problems update manager
pub struct ProblemsUpdateManager {
    workspace_path: PathBuf,
    build_integration: Arc<RwLock<BuildSystemIntegration>>,
    navigation_handler: Arc<ProblemNavigationHandler>,
    file_watcher: Option<RecommendedWatcher>,
    update_sender: mpsc::UnboundedSender<UpdateEvent>,
    settings: Arc<RwLock<UpdateSettings>>,
    debounce_timers: Arc<RwLock<HashMap<PathBuf, Instant>>>,
    active_builds: Arc<RwLock<HashSet<BuildTool>>>,
}

/// Update events from various sources
#[derive(Debug, Clone)]
pub enum UpdateEvent {
    FileChanged {
        paths: Vec<PathBuf>,
        change_type: FileChangeType,
    },
    BuildStarted {
        tool: BuildTool,
    },
    BuildCompleted {
        tool: BuildTool,
        problems: Vec<ProblemItem>,
        success: bool,
        duration: Duration,
    },
    GitStatusChanged {
        modified_files: Vec<PathBuf>,
        conflicts: Vec<PathBuf>,
    },
    LspDiagnosticsReceived {
        file_path: PathBuf,
        problems: Vec<ProblemItem>,
    },
    ProblemsCleared {
        source: String,
    },
    RefreshRequested,
}

/// Types of file changes that can trigger updates
#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeType {
    SourceCode,
    Configuration,
    Test,
    Documentation,
    GitFile,
    Other,
}

/// Update frequency settings
#[derive(Debug, Clone)]
pub struct UpdateFrequency {
    pub file_watch_debounce: Duration,
    pub build_debounce: Duration,
    pub git_check_interval: Duration,
    pub auto_refresh_interval: Duration,
}

/// Update settings
#[derive(Debug, Clone)]
pub struct UpdateSettings {
    pub auto_build_on_save: bool,
    pub auto_refresh_enabled: bool,
    pub show_build_progress: bool,
    pub debounce_updates: bool,
    pub frequency: UpdateFrequency,
    pub enabled_sources: HashSet<UpdateSource>,
}

/// Sources of updates
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdateSource {
    FileSystem,
    BuildSystem,
    Git,
    LanguageServer,
    Manual,
}

impl Default for UpdateFrequency {
    fn default() -> Self {
        Self {
            file_watch_debounce: Duration::from_millis(500),
            build_debounce: Duration::from_millis(1000),
            git_check_interval: Duration::from_secs(5),
            auto_refresh_interval: Duration::from_secs(30),
        }
    }
}

impl Default for UpdateSettings {
    fn default() -> Self {
        let mut enabled_sources = HashSet::new();
        enabled_sources.insert(UpdateSource::FileSystem);
        enabled_sources.insert(UpdateSource::BuildSystem);
        enabled_sources.insert(UpdateSource::Git);
        enabled_sources.insert(UpdateSource::Manual);

        Self {
            auto_build_on_save: true,
            auto_refresh_enabled: true,
            show_build_progress: true,
            debounce_updates: true,
            frequency: UpdateFrequency::default(),
            enabled_sources,
        }
    }
}

impl ProblemsUpdateManager {
    /// Create new problems update manager
    pub async fn new(
        workspace_path: PathBuf,
        build_integration: Arc<RwLock<BuildSystemIntegration>>,
        navigation_handler: Arc<ProblemNavigationHandler>,
    ) -> Result<(Self, mpsc::UnboundedReceiver<UpdateEvent>)> {
        let (update_sender, update_receiver) = mpsc::unbounded_channel();

        let manager = Self {
            workspace_path,
            build_integration,
            navigation_handler,
            file_watcher: None,
            update_sender,
            settings: Arc::new(RwLock::new(UpdateSettings::default())),
            debounce_timers: Arc::new(RwLock::new(HashMap::new())),
            active_builds: Arc::new(RwLock::new(HashSet::new())),
        };

        Ok((manager, update_receiver))
    }

    /// Initialize the update manager
    pub async fn initialize(&mut self) -> Result<()> {
        info!("🔄 Initializing problems update manager for: {}", self.workspace_path.display());

        // Set up file system watching
        self.setup_file_watcher().await?;

        // Set up periodic tasks
        self.setup_periodic_tasks().await?;

        info!("✅ Problems update manager initialized");
        Ok(())
    }

    /// Set up file system watching
    async fn setup_file_watcher(&mut self) -> Result<()> {
        let (fs_sender, fs_receiver) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(fs_sender)?;

        // Watch source directories
        let watch_paths = [
            "src",
            "tests",
            "examples",
            "benches",
        ];

        for watch_path in &watch_paths {
            let path = self.workspace_path.join(watch_path);
            if path.exists() {
                watcher.watch(&path, RecursiveMode::Recursive)?;
                debug!("👀 Watching directory: {}", path.display());
            }
        }

        // Watch configuration files
        let config_files = [
            "Cargo.toml",
            "Cargo.lock",
            "package.json",
            "package-lock.json",
            "tsconfig.json",
            ".eslintrc.json",
            ".eslintrc.js",
            "rustfmt.toml",
            "clippy.toml",
        ];

        for config_file in &config_files {
            let path = self.workspace_path.join(config_file);
            if path.exists() {
                watcher.watch(&path, RecursiveMode::NonRecursive)?;
                debug!("👀 Watching config file: {}", path.display());
            }
        }

        // Watch .git directory for git status changes
        let git_path = self.workspace_path.join(".git");
        if git_path.exists() {
            watcher.watch(&git_path.join("HEAD"), RecursiveMode::NonRecursive)?;
            watcher.watch(&git_path.join("index"), RecursiveMode::NonRecursive)?;
            debug!("👀 Watching git status files");
        }

        self.file_watcher = Some(watcher);

        // Start file watcher thread
        let workspace_clone = self.workspace_path.clone();
        let sender_clone = self.update_sender.clone();
        let settings_clone = self.settings.clone();
        let debounce_timers_clone = self.debounce_timers.clone();

        tokio::spawn(async move {
            Self::file_watcher_loop(fs_receiver, sender_clone, workspace_clone, settings_clone, debounce_timers_clone).await;
        });

        Ok(())
    }

    /// File watcher loop
    async fn file_watcher_loop(
        receiver: std::sync::mpsc::Receiver<notify::Result<notify::Event>>,
        sender: mpsc::UnboundedSender<UpdateEvent>,
        workspace_path: PathBuf,
        settings: Arc<RwLock<UpdateSettings>>,
        debounce_timers: Arc<RwLock<HashMap<PathBuf, Instant>>>,
    ) {
        loop {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    if let Some((paths, change_type)) = Self::process_file_event(&event, &workspace_path).await {
                        // Check debouncing
                        let should_send = {
                            let settings_read = settings.read().await;
                            if settings_read.debounce_updates {
                                let mut timers = debounce_timers.write().await;
                                let now = Instant::now();
                                let debounce_duration = settings_read.frequency.file_watch_debounce;
                                
                                let mut should_send = false;
                                for path in &paths {
                                    if let Some(last_time) = timers.get(path) {
                                        if now.duration_since(*last_time) >= debounce_duration {
                                            should_send = true;
                                            timers.insert(path.clone(), now);
                                        }
                                    } else {
                                        should_send = true;
                                        timers.insert(path.clone(), now);
                                    }
                                }
                                should_send
                            } else {
                                true
                            }
                        };

                        if should_send {
                            let _ = sender.send(UpdateEvent::FileChanged { paths, change_type });
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("File watcher error: {}", e);
                }
                Err(_) => {
                    // Timeout - continue
                }
            }

            // Clean up old debounce timers
            {
                let mut timers = debounce_timers.write().await;
                let cutoff = Instant::now() - Duration::from_secs(60);
                timers.retain(|_, timestamp| *timestamp >= cutoff);
            }
        }
    }

    /// Process file system event
    async fn process_file_event(
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

                        // Determine change type and relevance
                        if path_str.ends_with(".rs") {
                            change_type = if path_str.contains("/tests/") || path_str.starts_with("tests/") {
                                FileChangeType::Test
                            } else {
                                FileChangeType::SourceCode
                            };
                            relevant_paths.push(path.clone());
                        } else if path_str.ends_with(".js") || path_str.ends_with(".ts") || path_str.ends_with(".tsx") || path_str.ends_with(".jsx") {
                            change_type = FileChangeType::SourceCode;
                            relevant_paths.push(path.clone());
                        } else if Self::is_config_file(&path_str) {
                            change_type = FileChangeType::Configuration;
                            relevant_paths.push(path.clone());
                        } else if path_str.contains("/.git/") || path_str.starts_with(".git/") {
                            change_type = FileChangeType::GitFile;
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

    /// Check if a file is a configuration file
    fn is_config_file(path_str: &str) -> bool {
        let config_files = [
            "Cargo.toml", "Cargo.lock", "package.json", "package-lock.json",
            "tsconfig.json", ".eslintrc.json", ".eslintrc.js", "eslint.config.js",
            "rustfmt.toml", "clippy.toml", ".gitignore", "README.md",
        ];

        config_files.iter().any(|&config| path_str.ends_with(config))
    }

    /// Set up periodic tasks
    async fn setup_periodic_tasks(&self) -> Result<()> {
        // Git status checking task
        let sender_clone = self.update_sender.clone();
        let workspace_clone = self.workspace_path.clone();
        let settings_clone = self.settings.clone();

        tokio::spawn(async move {
            Self::periodic_git_check_task(sender_clone, workspace_clone, settings_clone).await;
        });

        // Auto-refresh task
        let sender_clone = self.update_sender.clone();
        let settings_clone = self.settings.clone();

        tokio::spawn(async move {
            Self::auto_refresh_task(sender_clone, settings_clone).await;
        });

        Ok(())
    }

    /// Periodic git status check task
    async fn periodic_git_check_task(
        sender: mpsc::UnboundedSender<UpdateEvent>,
        workspace_path: PathBuf,
        settings: Arc<RwLock<UpdateSettings>>,
    ) {
        let mut last_check = Instant::now();

        loop {
            let interval = {
                let settings_read = settings.read().await;
                settings_read.frequency.git_check_interval
            };

            tokio::time::sleep(interval).await;

            if last_check.elapsed() >= interval {
                if let Ok(git_status) = Self::check_git_status(&workspace_path).await {
                    let _ = sender.send(UpdateEvent::GitStatusChanged {
                        modified_files: git_status.modified_files,
                        conflicts: git_status.conflicts,
                    });
                }
                last_check = Instant::now();
            }
        }
    }

    /// Auto-refresh task
    async fn auto_refresh_task(
        sender: mpsc::UnboundedSender<UpdateEvent>,
        settings: Arc<RwLock<UpdateSettings>>,
    ) {
        loop {
            let (enabled, interval) = {
                let settings_read = settings.read().await;
                (settings_read.auto_refresh_enabled, settings_read.frequency.auto_refresh_interval)
            };

            if enabled {
                tokio::time::sleep(interval).await;
                let _ = sender.send(UpdateEvent::RefreshRequested);
            } else {
                tokio::time::sleep(Duration::from_secs(5)).await; // Check settings every 5 seconds
            }
        }
    }

    /// Check git status
    async fn check_git_status(workspace_path: &Path) -> Result<GitStatus> {
        let output = tokio::process::Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(workspace_path)
            .output()
            .await?;

        let mut git_status = GitStatus::default();
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.len() >= 3 {
                let status_chars = &line[..2];
                let file_path = PathBuf::from(&line[3..]);

                match status_chars {
                    "UU" | "AA" | "DD" => {
                        git_status.conflicts.push(workspace_path.join(file_path));
                    }
                    _ if status_chars.chars().any(|c| c != ' ' && c != '?') => {
                        git_status.modified_files.push(workspace_path.join(file_path));
                    }
                }
            }
        }

        Ok(git_status)
    }

    /// Handle update event
    pub async fn handle_update_event(&mut self, event: UpdateEvent) -> Result<()> {
        debug!("🔄 Handling update event: {:?}", event);

        match event {
            UpdateEvent::FileChanged { paths, change_type } => {
                self.handle_file_changed(paths, change_type).await?;
            }
            UpdateEvent::BuildStarted { tool } => {
                self.handle_build_started(tool).await?;
            }
            UpdateEvent::BuildCompleted { tool, problems, success, duration } => {
                self.handle_build_completed(tool, problems, success, duration).await?;
            }
            UpdateEvent::GitStatusChanged { modified_files, conflicts } => {
                self.handle_git_status_changed(modified_files, conflicts).await?;
            }
            UpdateEvent::LspDiagnosticsReceived { file_path, problems } => {
                self.handle_lsp_diagnostics(file_path, problems).await?;
            }
            UpdateEvent::ProblemsCleared { source } => {
                self.handle_problems_cleared(source).await?;
            }
            UpdateEvent::RefreshRequested => {
                self.handle_refresh_requested().await?;
            }
        }

        Ok(())
    }

    /// Handle file changed event
    async fn handle_file_changed(&mut self, paths: Vec<PathBuf>, change_type: FileChangeType) -> Result<()> {
        debug!("📝 File changed: {:?} (type: {:?})", paths, change_type);

        let settings = self.settings.read().await;
        
        if settings.auto_build_on_save && matches!(change_type, FileChangeType::SourceCode | FileChangeType::Configuration) {
            // Trigger appropriate build tools
            let mut build_integration = self.build_integration.write().await;
            
            match change_type {
                FileChangeType::SourceCode => {
                    // Trigger relevant build tools
                    if paths.iter().any(|p| p.extension().map_or(false, |ext| ext == "rs")) {
                        build_integration.run_build_check(&BuildTool::Cargo).await?;
                        build_integration.run_build_check(&BuildTool::Clippy).await?;
                    }
                    
                    if paths.iter().any(|p| {
                        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                        matches!(ext, "js" | "ts" | "tsx" | "jsx")
                    }) {
                        build_integration.run_build_check(&BuildTool::TypeScript).await?;
                        build_integration.run_build_check(&BuildTool::ESLint).await?;
                    }
                }
                FileChangeType::Configuration => {
                    // Configuration changes might affect everything
                    build_integration.refresh_all().await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Handle build started event
    async fn handle_build_started(&mut self, tool: BuildTool) -> Result<()> {
        debug!("🔧 Build started: {:?}", tool);
        
        {
            let mut active_builds = self.active_builds.write().await;
            active_builds.insert(tool);
        }

        Ok(())
    }

    /// Handle build completed event
    async fn handle_build_completed(&mut self, tool: BuildTool, problems: Vec<ProblemItem>, success: bool, duration: Duration) -> Result<()> {
        info!("🔧 Build completed: {:?} - {} problems in {:?}", tool, problems.len(), duration);
        
        {
            let mut active_builds = self.active_builds.write().await;
            active_builds.remove(&tool);
        }

        // The build integration should already have updated the problems state
        // This event is more for UI updates and logging

        Ok(())
    }

    /// Handle git status changed event
    async fn handle_git_status_changed(&mut self, modified_files: Vec<PathBuf>, conflicts: Vec<PathBuf>) -> Result<()> {
        debug!("📋 Git status changed: {} modified, {} conflicts", modified_files.len(), conflicts.len());
        
        // Update git-related problems in the build integration
        // This would typically involve updating conflict markers and unstaged changes
        
        Ok(())
    }

    /// Handle LSP diagnostics received
    async fn handle_lsp_diagnostics(&mut self, file_path: PathBuf, problems: Vec<ProblemItem>) -> Result<()> {
        debug!("🔍 LSP diagnostics received for {}: {} problems", file_path.display(), problems.len());
        
        // This would integrate with language server protocol diagnostics
        // For now, just log the event
        
        Ok(())
    }

    /// Handle problems cleared event
    async fn handle_problems_cleared(&mut self, source: String) -> Result<()> {
        debug!("🧹 Problems cleared for source: {}", source);
        Ok(())
    }

    /// Handle refresh requested event
    async fn handle_refresh_requested(&mut self) -> Result<()> {
        info!("🔄 Manual refresh requested");
        
        let mut build_integration = self.build_integration.write().await;
        build_integration.refresh_all().await?;
        
        Ok(())
    }

    /// Get current update settings
    pub async fn get_settings(&self) -> UpdateSettings {
        self.settings.read().await.clone()
    }

    /// Update settings
    pub async fn update_settings(&self, new_settings: UpdateSettings) -> Result<()> {
        {
            let mut settings = self.settings.write().await;
            *settings = new_settings;
        }
        info!("⚙️ Update settings updated");
        Ok(())
    }

    /// Get current active builds
    pub async fn get_active_builds(&self) -> HashSet<BuildTool> {
        self.active_builds.read().await.clone()
    }

    /// Force refresh all problems
    pub async fn force_refresh(&self) -> Result<()> {
        let _ = self.update_sender.send(UpdateEvent::RefreshRequested);
        Ok(())
    }
}

/// Git status information
#[derive(Debug, Default)]
struct GitStatus {
    modified_files: Vec<PathBuf>,
    conflicts: Vec<PathBuf>,
}