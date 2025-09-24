//! Workspace state management for multi-repository workspace
//!
//! Central state management for tracking multiple repositories, open files,
//! and workspace settings in the VS Code-style desktop UI.

use super::git::repository::RepositoryInfo;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

pub mod repository_discovery;
pub use repository_discovery::{
    DiscoveryConfig, GitStatusInfo, ProjectType, RepositoryDiscoveryService, RepositoryMetadata,
    ScanningMode,
};

/// Central workspace state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    /// Root path of the current workspace
    pub root_path: PathBuf,

    /// All discovered repositories in the workspace (basic info)
    pub repositories: Vec<RepositoryInfo>,

    /// Extended repository metadata from discovery service
    pub repository_metadata: HashMap<PathBuf, RepositoryMetadata>,

    /// Currently active repository
    pub active_repository: Option<PathBuf>,

    /// Open files and their states
    pub open_files: HashMap<PathBuf, FileState>,

    /// Workspace-specific settings
    pub workspace_settings: WorkspaceSettings,

    /// Recently opened workspaces
    pub recent_workspaces: Vec<PathBuf>,

    /// Last discovery scan timestamp
    pub last_discovery_scan: Option<u64>,
}

/// State of an open file in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    /// File path
    pub path: PathBuf,

    /// Last known modified time
    pub last_modified: Option<u64>,

    /// Current cursor position (line, column)
    pub cursor_position: (usize, usize),

    /// Scroll position
    pub scroll_position: usize,

    /// Whether the file has unsaved changes
    pub is_dirty: bool,

    /// Selection range if any
    pub selection: Option<SelectionRange>,
}

/// Selection range in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRange {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

/// Workspace-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    /// Default branch name for new repositories
    pub default_branch: String,

    /// Whether to auto-save files
    pub auto_save: bool,

    /// Auto-save delay in milliseconds
    pub auto_save_delay: u64,

    /// Whether to show hidden files
    pub show_hidden_files: bool,

    /// File patterns to exclude from the explorer
    pub exclude_patterns: Vec<String>,

    /// Whether to follow symlinks
    pub follow_symlinks: bool,

    /// Maximum number of recent workspaces to remember
    pub max_recent_workspaces: usize,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            default_branch: "main".to_string(),
            auto_save: false,
            auto_save_delay: 1000,
            show_hidden_files: false,
            exclude_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".DS_Store".to_string(),
            ],
            follow_symlinks: false,
            max_recent_workspaces: 10,
        }
    }
}

impl WorkspaceState {
    /// Create a new workspace state for the given root path
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            repositories: Vec::new(),
            repository_metadata: HashMap::new(),
            active_repository: None,
            open_files: HashMap::new(),
            workspace_settings: WorkspaceSettings::default(),
            recent_workspaces: Vec::new(),
            last_discovery_scan: None,
        }
    }

    /// Scan the workspace for git repositories using basic discovery
    pub fn scan_for_repositories(&mut self) -> Result<()> {
        info!("Scanning workspace for repositories: {:?}", self.root_path);

        // Clear existing repositories
        self.repositories.clear();

        // Use the existing git repository discovery functionality
        use super::git::repository::GitRepository;
        let discovered = GitRepository::discover_repositories(&self.root_path);

        info!("Discovered {} repositories", discovered.len());
        self.repositories = discovered;

        // If we have repositories but no active one, set the first as active
        if !self.repositories.is_empty() && self.active_repository.is_none() {
            self.active_repository = Some(self.repositories[0].path.clone());
        }

        Ok(())
    }

    /// Advanced repository discovery using the discovery service
    pub async fn discover_repositories_advanced(
        &mut self,
        discovery_service: &RepositoryDiscoveryService,
    ) -> Result<()> {
        info!(
            "Starting advanced repository discovery for workspace: {:?}",
            self.root_path
        );

        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Discover repositories with full metadata
        let discovered_metadata = discovery_service.discover_repositories().await?;

        info!(
            "Advanced discovery found {} repositories with metadata",
            discovered_metadata.len()
        );

        // Update state with discovered metadata
        self.repository_metadata.clear();
        self.repositories.clear();

        for metadata in discovered_metadata {
            let path = metadata.basic_info.path.clone();

            // Add basic info to repositories list
            self.repositories.push(metadata.basic_info.clone());

            // Store full metadata
            self.repository_metadata.insert(path, metadata);
        }

        // If we have repositories but no active one, set the first as active
        if !self.repositories.is_empty() && self.active_repository.is_none() {
            self.active_repository = Some(self.repositories[0].path.clone());
        }

        // Update scan timestamp
        self.last_discovery_scan = Some(start_time);

        info!(
            "Advanced repository discovery completed: {} repositories",
            self.repositories.len()
        );
        Ok(())
    }

    /// Refresh metadata for a specific repository
    pub async fn refresh_repository_metadata(
        &mut self,
        repository_path: &Path,
        discovery_service: &RepositoryDiscoveryService,
    ) -> Result<()> {
        info!("Refreshing metadata for repository: {:?}", repository_path);

        let metadata = discovery_service
            .refresh_repository(repository_path)
            .await?;

        // Update the metadata
        self.repository_metadata
            .insert(repository_path.to_path_buf(), metadata.clone());

        // Update basic info in repositories list if needed
        if let Some(repo) = self
            .repositories
            .iter_mut()
            .find(|r| r.path == repository_path)
        {
            *repo = metadata.basic_info;
        }

        Ok(())
    }

    /// Get extended metadata for a repository
    pub fn get_repository_metadata(&self, path: &Path) -> Option<&RepositoryMetadata> {
        self.repository_metadata.get(path)
    }

    /// Get all repositories with their metadata
    pub fn get_repositories_with_metadata(
        &self,
    ) -> Vec<(&RepositoryInfo, Option<&RepositoryMetadata>)> {
        self.repositories
            .iter()
            .map(|repo| {
                let metadata = self.repository_metadata.get(&repo.path);
                (repo, metadata)
            })
            .collect()
    }

    /// Filter repositories by project type
    pub fn filter_repositories_by_type(
        &self,
        project_type: ProjectType,
    ) -> Vec<&RepositoryMetadata> {
        self.repository_metadata
            .values()
            .filter(|metadata| {
                metadata
                    .project_info
                    .as_ref()
                    .map(|info| info.project_type == project_type)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get repositories sorted by various criteria
    pub fn get_repositories_sorted_by_activity(&self) -> Vec<&RepositoryMetadata> {
        let mut repos: Vec<&RepositoryMetadata> = self.repository_metadata.values().collect();

        repos.sort_by(|a, b| {
            // Sort by last commit timestamp (most recent first)
            let a_time = a.stats.last_commit.unwrap_or(0);
            let b_time = b.stats.last_commit.unwrap_or(0);
            b_time.cmp(&a_time)
        });

        repos
    }

    /// Check if discovery cache is stale and needs refresh
    pub fn needs_discovery_refresh(&self, max_age_seconds: u64) -> bool {
        if let Some(last_scan) = self.last_discovery_scan {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            now - last_scan > max_age_seconds
        } else {
            true // Never scanned
        }
    }

    /// Switch to a different repository
    pub fn switch_repository(&mut self, path: PathBuf) -> Result<()> {
        // Verify the repository exists in our list
        let exists = self.repositories.iter().any(|r| r.path == path);

        if !exists {
            return Err(anyhow::anyhow!(
                "Repository not found in workspace: {:?}",
                path
            ));
        }

        info!("Switching to repository: {:?}", path);
        self.active_repository = Some(path);

        Ok(())
    }

    /// Add a workspace to the recent workspaces list
    pub fn add_recent_workspace(&mut self, path: PathBuf) {
        // Remove if already exists to move it to the front
        self.recent_workspaces.retain(|p| p != &path);

        // Add to the front
        self.recent_workspaces.insert(0, path);

        // Trim to max size
        let max_size = self.workspace_settings.max_recent_workspaces;
        if self.recent_workspaces.len() > max_size {
            self.recent_workspaces.truncate(max_size);
        }
    }

    /// Save the workspace state to disk
    pub fn save_state(&self) -> Result<()> {
        let state_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("hive")
            .join("workspaces");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&state_dir)
            .context("Failed to create workspace state directory")?;

        // Generate a safe filename from the workspace path
        let workspace_hash = format!(
            "{:x}",
            md5::compute(self.root_path.to_string_lossy().as_bytes())
        );
        let state_file = state_dir.join(format!("{}.json", workspace_hash));

        // Serialize and save
        let json =
            serde_json::to_string_pretty(self).context("Failed to serialize workspace state")?;

        std::fs::write(&state_file, json).context("Failed to write workspace state")?;

        info!("Saved workspace state to: {:?}", state_file);
        Ok(())
    }

    /// Restore workspace state from disk
    pub fn restore_state(workspace_path: &Path) -> Result<Self> {
        let state_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("hive")
            .join("workspaces");

        // Generate the expected filename
        let workspace_hash = format!(
            "{:x}",
            md5::compute(workspace_path.to_string_lossy().as_bytes())
        );
        let state_file = state_dir.join(format!("{}.json", workspace_hash));

        if !state_file.exists() {
            info!("No saved state found for workspace: {:?}", workspace_path);
            return Ok(Self::new(workspace_path.to_path_buf()));
        }

        // Read and deserialize
        let json =
            std::fs::read_to_string(&state_file).context("Failed to read workspace state")?;

        let mut state: Self =
            serde_json::from_str(&json).context("Failed to deserialize workspace state")?;

        // Update the root path in case it changed
        state.root_path = workspace_path.to_path_buf();

        info!("Restored workspace state from: {:?}", state_file);
        Ok(state)
    }

    /// Update file state when a file is opened or modified
    pub fn update_file_state(&mut self, path: PathBuf, state: FileState) {
        self.open_files.insert(path, state);
    }

    /// Remove file state when a file is closed
    pub fn close_file(&mut self, path: &Path) {
        self.open_files.remove(path);
    }

    /// Get the current active repository info
    pub fn get_active_repository(&self) -> Option<&RepositoryInfo> {
        self.active_repository
            .as_ref()
            .and_then(|path| self.repositories.iter().find(|r| &r.path == path))
    }

    /// Get all open files for the active repository
    pub fn get_repository_files(&self) -> Vec<&FileState> {
        if let Some(repo_path) = &self.active_repository {
            self.open_files
                .iter()
                .filter(|(path, _)| path.starts_with(repo_path))
                .map(|(_, state)| state)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a path is excluded by the workspace settings
    pub fn is_path_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        self.workspace_settings
            .exclude_patterns
            .iter()
            .any(|pattern| {
                // Simple pattern matching - could be enhanced with glob patterns
                path_str.contains(pattern)
            })
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_state_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = WorkspaceState::new(temp_dir.path().to_path_buf());

        assert_eq!(workspace.root_path, temp_dir.path());
        assert!(workspace.repositories.is_empty());
        assert!(workspace.active_repository.is_none());
        assert!(workspace.open_files.is_empty());
    }

    #[test]
    fn test_recent_workspaces() {
        let mut workspace = WorkspaceState::new(PathBuf::from("/test"));

        // Add some recent workspaces
        workspace.add_recent_workspace(PathBuf::from("/workspace1"));
        workspace.add_recent_workspace(PathBuf::from("/workspace2"));
        workspace.add_recent_workspace(PathBuf::from("/workspace1")); // Duplicate

        // Should have workspace1 at the front (most recent)
        assert_eq!(workspace.recent_workspaces.len(), 2);
        assert_eq!(workspace.recent_workspaces[0], PathBuf::from("/workspace1"));
        assert_eq!(workspace.recent_workspaces[1], PathBuf::from("/workspace2"));
    }

    #[test]
    fn test_file_state_management() {
        let mut workspace = WorkspaceState::new(PathBuf::from("/test"));

        let file_state = FileState {
            path: PathBuf::from("/test/file.rs"),
            last_modified: Some(1234567890),
            cursor_position: (10, 5),
            scroll_position: 0,
            is_dirty: true,
            selection: None,
        };

        workspace.update_file_state(PathBuf::from("/test/file.rs"), file_state.clone());
        assert_eq!(workspace.open_files.len(), 1);

        workspace.close_file(&PathBuf::from("/test/file.rs"));
        assert!(workspace.open_files.is_empty());
    }

    #[test]
    fn test_path_exclusion() {
        let workspace = WorkspaceState::new(PathBuf::from("/test"));

        assert!(workspace.is_path_excluded(&PathBuf::from("/test/.git/config")));
        assert!(workspace.is_path_excluded(&PathBuf::from("/test/node_modules/package")));
        assert!(!workspace.is_path_excluded(&PathBuf::from("/test/src/main.rs")));
    }
}
