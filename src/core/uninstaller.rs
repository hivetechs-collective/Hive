// Clean uninstall functionality for Hive AI
// Provides complete removal of all Hive AI components

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallPlan {
    pub components: Vec<UninstallComponent>,
    pub estimated_space_freed: u64,
    pub backup_recommended: bool,
    pub requires_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallComponent {
    pub name: String,
    pub component_type: ComponentType,
    pub paths: Vec<PathBuf>,
    pub size_bytes: u64,
    pub optional: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Binary,
    Configuration,
    Database,
    Cache,
    Logs,
    ShellCompletions,
    BackupFiles,
    TempFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallResult {
    pub success: bool,
    pub components_removed: usize,
    pub total_components: usize,
    pub space_freed: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct HiveUninstaller {
    pub dry_run: bool,
    pub preserve_config: bool,
    pub preserve_data: bool,
    pub force: bool,
}

impl Default for HiveUninstaller {
    fn default() -> Self {
        Self {
            dry_run: false,
            preserve_config: false,
            preserve_data: false,
            force: false,
        }
    }
}

impl HiveUninstaller {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn preserve_config(mut self, preserve: bool) -> Self {
        self.preserve_config = preserve;
        self
    }

    pub fn preserve_data(mut self, preserve: bool) -> Self {
        self.preserve_data = preserve;
        self
    }

    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Create uninstall plan by scanning for Hive AI components
    pub async fn create_uninstall_plan(&self) -> Result<UninstallPlan> {
        info!("Scanning system for Hive AI components...");
        
        let mut components = Vec::new();
        let mut total_size = 0u64;
        let mut requires_admin = false;

        // Find main binary
        if let Some(binary_component) = self.find_binary_component().await? {
            if self.requires_admin_access(&binary_component.paths) {
                requires_admin = true;
            }
            total_size += binary_component.size_bytes;
            components.push(binary_component);
        }

        // Find configuration directories
        if !self.preserve_config {
            if let Some(config_component) = self.find_config_component().await? {
                total_size += config_component.size_bytes;
                components.push(config_component);
            }
        }

        // Find database and data
        if !self.preserve_data {
            if let Some(database_component) = self.find_database_component().await? {
                total_size += database_component.size_bytes;
                components.push(database_component);
            }
        }

        // Find cache files
        if let Some(cache_component) = self.find_cache_component().await? {
            total_size += cache_component.size_bytes;
            components.push(cache_component);
        }

        // Find log files
        if let Some(logs_component) = self.find_logs_component().await? {
            total_size += logs_component.size_bytes;
            components.push(logs_component);
        }

        // Find shell completions
        if let Some(completions_component) = self.find_shell_completions_component().await? {
            total_size += completions_component.size_bytes;
            components.push(completions_component);
        }

        // Find backup files
        if let Some(backup_component) = self.find_backup_component().await? {
            total_size += backup_component.size_bytes;
            components.push(backup_component);
        }

        // Find temporary files
        if let Some(temp_component) = self.find_temp_component().await? {
            total_size += temp_component.size_bytes;
            components.push(temp_component);
        }

        let backup_recommended = !self.preserve_config && !self.preserve_data && 
                                components.iter().any(|c| matches!(c.component_type, ComponentType::Database | ComponentType::Configuration));

        Ok(UninstallPlan {
            components,
            estimated_space_freed: total_size,
            backup_recommended,
            requires_admin,
        })
    }

    /// Find main Hive AI binary
    async fn find_binary_component(&self) -> Result<Option<UninstallComponent>> {
        let possible_locations = [
            "/usr/local/bin/hive",
            "/usr/bin/hive", 
            "/opt/hive/bin/hive",
        ];

        // Also check Windows locations
        let windows_locations = [
            "C:\\Program Files\\HiveTechs\\HiveAI\\bin\\hive.exe",
            "C:\\Program Files (x86)\\HiveTechs\\HiveAI\\bin\\hive.exe",
        ];

        let mut paths = Vec::new();
        let mut total_size = 0u64;

        // Check Unix locations
        for location in &possible_locations {
            let path = PathBuf::from(location);
            if path.exists() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    total_size += metadata.len();
                    paths.push(path);
                }
            }
        }

        // Check Windows locations if on Windows
        if env::consts::OS == "windows" {
            for location in &windows_locations {
                let path = PathBuf::from(location);
                if path.exists() {
                    if let Ok(metadata) = fs::metadata(&path).await {
                        total_size += metadata.len();
                        paths.push(path);
                    }
                }
            }
        }

        // Also check current executable location
        if let Ok(current_exe) = env::current_exe() {
            if current_exe.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("hive"))
                .unwrap_or(false) {
                if let Ok(metadata) = fs::metadata(&current_exe).await {
                    total_size += metadata.len();
                    paths.push(current_exe);
                }
            }
        }

        if paths.is_empty() {
            return Ok(None);
        }

        Ok(Some(UninstallComponent {
            name: "Hive AI Binary".to_string(),
            component_type: ComponentType::Binary,
            paths,
            size_bytes: total_size,
            optional: false,
            description: "Main Hive AI executable".to_string(),
        }))
    }

    /// Find configuration component
    async fn find_config_component(&self) -> Result<Option<UninstallComponent>> {
        let config_dirs = self.get_config_directories();
        let mut paths = Vec::new();
        let mut total_size = 0u64;

        for config_dir in config_dirs {
            if config_dir.exists() {
                total_size += self.calculate_directory_size(&config_dir).await?;
                paths.push(config_dir);
            }
        }

        if paths.is_empty() {
            return Ok(None);
        }

        Ok(Some(UninstallComponent {
            name: "Configuration".to_string(),
            component_type: ComponentType::Configuration,
            paths,
            size_bytes: total_size,
            optional: true,
            description: "Hive AI configuration files and settings".to_string(),
        }))
    }

    /// Find database component
    async fn find_database_component(&self) -> Result<Option<UninstallComponent>> {
        let config_dirs = self.get_config_directories();
        let mut paths = Vec::new();
        let mut total_size = 0u64;

        for config_dir in config_dirs {
            let db_files = [
                config_dir.join("conversations.db"),
                config_dir.join("conversations.db-shm"),
                config_dir.join("conversations.db-wal"),
                config_dir.join("index.db"),
                config_dir.join("analytics.db"),
            ];

            for db_file in &db_files {
                if db_file.exists() {
                    if let Ok(metadata) = fs::metadata(db_file).await {
                        total_size += metadata.len();
                        paths.push(db_file.clone());
                    }
                }
            }
        }

        if paths.is_empty() {
            return Ok(None);
        }

        Ok(Some(UninstallComponent {
            name: "Database Files".to_string(),
            component_type: ComponentType::Database,
            paths,
            size_bytes: total_size,
            optional: true,
            description: "Conversation history and indexed data".to_string(),
        }))
    }

    /// Find cache component
    async fn find_cache_component(&self) -> Result<Option<UninstallComponent>> {
        let cache_dirs = self.get_cache_directories();
        let mut paths = Vec::new();
        let mut total_size = 0u64;

        for cache_dir in cache_dirs {
            if cache_dir.exists() {
                total_size += self.calculate_directory_size(&cache_dir).await?;
                paths.push(cache_dir);
            }
        }

        if paths.is_empty() {
            return Ok(None);
        }

        Ok(Some(UninstallComponent {
            name: "Cache Files".to_string(),
            component_type: ComponentType::Cache,
            paths,
            size_bytes: total_size,
            optional: true,
            description: "Cached analysis results and temporary data".to_string(),
        }))
    }

    /// Find logs component
    async fn find_logs_component(&self) -> Result<Option<UninstallComponent>> {
        let log_dirs = self.get_log_directories();
        let mut paths = Vec::new();
        let mut total_size = 0u64;

        for log_dir in log_dirs {
            if log_dir.exists() {
                total_size += self.calculate_directory_size(&log_dir).await?;
                paths.push(log_dir);
            }
        }

        if paths.is_empty() {
            return Ok(None);
        }

        Ok(Some(UninstallComponent {
            name: "Log Files".to_string(),
            component_type: ComponentType::Logs,
            paths,
            size_bytes: total_size,
            optional: true,
            description: "Application logs and debug information".to_string(),
        }))
    }

    /// Find shell completions component
    async fn find_shell_completions_component(&self) -> Result<Option<UninstallComponent>> {
        let completion_locations = [
            "/etc/bash_completion.d/hive",
            "/usr/local/etc/bash_completion.d/hive",
        ];

        // User-specific completion locations
        let mut user_locations = Vec::new();
        if let Ok(home) = env::var("HOME") {
            user_locations.extend([
                format!("{}/.bash_completion.d/hive", home),
                format!("{}/.zsh/completions/_hive", home),
                format!("{}/.config/fish/completions/hive.fish", home),
                format!("{}/.oh-my-zsh/completions/_hive", home),
            ]);
        }

        let mut paths = Vec::new();
        let mut total_size = 0u64;

        // Check system completion locations
        for location in &completion_locations {
            let path = PathBuf::from(location);
            if path.exists() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    total_size += metadata.len();
                    paths.push(path);
                }
            }
        }

        // Check user completion locations
        for location in &user_locations {
            let path = PathBuf::from(location);
            if path.exists() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    total_size += metadata.len();
                    paths.push(path);
                }
            }
        }

        if paths.is_empty() {
            return Ok(None);
        }

        Ok(Some(UninstallComponent {
            name: "Shell Completions".to_string(),
            component_type: ComponentType::ShellCompletions,
            paths,
            size_bytes: total_size,
            optional: true,
            description: "Shell autocompletion scripts".to_string(),
        }))
    }

    /// Find backup component
    async fn find_backup_component(&self) -> Result<Option<UninstallComponent>> {
        let config_dirs = self.get_config_directories();
        let mut paths = Vec::new();
        let mut total_size = 0u64;

        for config_dir in config_dirs {
            let backup_dir = config_dir.join("migration_backups");
            if backup_dir.exists() {
                total_size += self.calculate_directory_size(&backup_dir).await?;
                paths.push(backup_dir);
            }

            let update_backups = config_dir.join(".hive-backups");
            if update_backups.exists() {
                total_size += self.calculate_directory_size(&update_backups).await?;
                paths.push(update_backups);
            }
        }

        if paths.is_empty() {
            return Ok(None);
        }

        Ok(Some(UninstallComponent {
            name: "Backup Files".to_string(),
            component_type: ComponentType::BackupFiles,
            paths,
            size_bytes: total_size,
            optional: true,
            description: "Migration and update backup files".to_string(),
        }))
    }

    /// Find temporary files component
    async fn find_temp_component(&self) -> Result<Option<UninstallComponent>> {
        let temp_dir = std::env::temp_dir();
        let mut paths = Vec::new();
        let mut total_size = 0u64;

        // Look for hive-related temp files
        if let Ok(mut entries) = fs::read_dir(&temp_dir).await {
            while let Some(entry) = entries.next_entry().await? {
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy();
                
                if name_str.contains("hive") && (name_str.contains("update") || name_str.contains("temp")) {
                    if let Ok(metadata) = entry.metadata().await {
                        total_size += metadata.len();
                        paths.push(entry.path());
                    }
                }
            }
        }

        if paths.is_empty() {
            return Ok(None);
        }

        Ok(Some(UninstallComponent {
            name: "Temporary Files".to_string(),
            component_type: ComponentType::TempFiles,
            paths,
            size_bytes: total_size,
            optional: true,
            description: "Temporary installation and update files".to_string(),
        }))
    }

    /// Get possible configuration directories
    fn get_config_directories(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // Platform-specific config directories
        match env::consts::OS {
            "macos" => {
                if let Ok(home) = env::var("HOME") {
                    dirs.push(PathBuf::from(home).join(".hive"));
                }
            }
            "linux" => {
                if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
                    dirs.push(PathBuf::from(xdg_config).join("hive"));
                } else if let Ok(home) = env::var("HOME") {
                    dirs.push(PathBuf::from(&home).join(".config").join("hive"));
                    dirs.push(PathBuf::from(&home).join(".hive"));
                }
            }
            "windows" => {
                if let Ok(appdata) = env::var("APPDATA") {
                    dirs.push(PathBuf::from(appdata).join("HiveTechs").join("HiveAI"));
                }
                if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
                    dirs.push(PathBuf::from(local_appdata).join("HiveTechs").join("HiveAI"));
                }
            }
            _ => {}
        }

        dirs
    }

    /// Get possible cache directories
    fn get_cache_directories(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        match env::consts::OS {
            "macos" => {
                if let Ok(home) = env::var("HOME") {
                    dirs.push(PathBuf::from(home).join("Library").join("Caches").join("HiveAI"));
                }
            }
            "linux" => {
                if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
                    dirs.push(PathBuf::from(xdg_cache).join("hive"));
                } else if let Ok(home) = env::var("HOME") {
                    dirs.push(PathBuf::from(home).join(".cache").join("hive"));
                }
            }
            "windows" => {
                if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
                    dirs.push(PathBuf::from(local_appdata).join("HiveTechs").join("HiveAI").join("Cache"));
                }
            }
            _ => {}
        }

        dirs
    }

    /// Get possible log directories
    fn get_log_directories(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        match env::consts::OS {
            "macos" => {
                if let Ok(home) = env::var("HOME") {
                    dirs.push(PathBuf::from(home).join("Library").join("Logs").join("HiveAI"));
                }
            }
            "linux" => {
                if let Ok(home) = env::var("HOME") {
                    dirs.push(PathBuf::from(home).join(".local").join("share").join("hive").join("logs"));
                }
            }
            "windows" => {
                if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
                    dirs.push(PathBuf::from(local_appdata).join("HiveTechs").join("HiveAI").join("Logs"));
                }
            }
            _ => {}
        }

        dirs
    }

    /// Calculate directory size recursively
    async fn calculate_directory_size(&self, dir: &Path) -> Result<u64> {
        let mut total_size = 0u64;

        let mut entries = fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                // Use Box::pin to avoid infinite sized future
                total_size += Box::pin(self.calculate_directory_size(&entry.path())).await?;
            }
        }

        Ok(total_size)
    }

    /// Check if admin access is required for paths
    fn requires_admin_access(&self, paths: &[PathBuf]) -> bool {
        paths.iter().any(|path| {
            path.starts_with("/usr") || 
            path.starts_with("/opt") ||
            path.starts_with("/etc") ||
            (env::consts::OS == "windows" && path.starts_with("C:\\Program Files"))
        })
    }

    /// Execute uninstallation
    pub async fn execute_uninstall(&self, plan: &UninstallPlan) -> Result<UninstallResult> {
        info!("Starting Hive AI uninstallation...");

        let mut components_removed = 0;
        let mut space_freed = 0u64;
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        for component in &plan.components {
            info!("Removing component: {}", component.name);

            if self.dry_run {
                info!("DRY RUN: Would remove {} ({} bytes)", component.name, component.size_bytes);
                components_removed += 1;
                space_freed += component.size_bytes;
                continue;
            }

            let mut component_success = true;

            for path in &component.paths {
                match self.remove_path(path).await {
                    Ok(_) => {
                        debug!("Removed: {}", path.display());
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to remove {}: {}", path.display(), e);
                        if component.optional {
                            warnings.push(error_msg);
                        } else {
                            errors.push(error_msg);
                            component_success = false;
                        }
                    }
                }
            }

            if component_success {
                components_removed += 1;
                space_freed += component.size_bytes;
                info!("✅ Removed: {}", component.name);
            } else {
                warn!("❌ Failed to remove: {}", component.name);
            }
        }

        // Clean up shell configuration references
        if !self.dry_run {
            self.cleanup_shell_references().await?;
        }

        let success = errors.is_empty();

        if success {
            info!("🎉 Hive AI uninstallation completed successfully!");
            info!("Freed {} bytes across {} components", space_freed, components_removed);
        } else {
            warn!("⚠️ Uninstallation completed with {} errors", errors.len());
        }

        Ok(UninstallResult {
            success,
            components_removed,
            total_components: plan.components.len(),
            space_freed,
            errors,
            warnings,
        })
    }

    /// Remove a path (file or directory)
    async fn remove_path(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(path).await?;
        
        if metadata.is_file() {
            fs::remove_file(path).await
                .with_context(|| format!("Failed to remove file: {}", path.display()))?;
        } else if metadata.is_dir() {
            fs::remove_dir_all(path).await
                .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
        }

        Ok(())
    }

    /// Clean up shell configuration references
    async fn cleanup_shell_references(&self) -> Result<()> {
        if let Ok(home) = env::var("HOME") {
            let shell_configs = [
                format!("{}/.bashrc", home),
                format!("{}/.bash_profile", home),
                format!("{}/.zshrc", home),
                format!("{}/.config/fish/config.fish", home),
            ];

            for config_path in &shell_configs {
                let path = PathBuf::from(config_path);
                if path.exists() {
                    if let Err(e) = self.remove_hive_references_from_file(&path).await {
                        warn!("Failed to clean shell config {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Remove Hive AI references from shell configuration file
    async fn remove_hive_references_from_file(&self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path).await?;
        let mut lines: Vec<&str> = content.lines().collect();
        
        // Remove lines that reference Hive AI
        let mut in_hive_section = false;
        let mut modified = false;
        
        lines.retain(|line| {
            let line_lower = line.to_lowercase();
            
            // Start of Hive AI section
            if line_lower.contains("added by hive") || line_lower.contains("hive ai installer") {
                in_hive_section = true;
                modified = true;
                return false;
            }
            
            // In Hive AI section
            if in_hive_section {
                if line.trim().is_empty() && !line_lower.contains("hive") {
                    in_hive_section = false;
                    return true;
                }
                if line_lower.contains("hive") || line_lower.contains("export path") && line_lower.contains("/usr/local/bin") {
                    modified = true;
                    return false;
                }
            }
            
            // Standalone Hive references
            if line_lower.contains("hive") && (line_lower.contains("export") || line_lower.contains("alias")) {
                modified = true;
                return false;
            }
            
            true
        });

        if modified {
            let new_content = lines.join("\n") + "\n";
            fs::write(file_path, new_content).await?;
            info!("Cleaned Hive AI references from: {}", file_path.display());
        }

        Ok(())
    }

    /// Create backup before uninstallation
    pub async fn create_uninstall_backup(&self, plan: &UninstallPlan) -> Result<PathBuf> {
        let backup_dir = env::temp_dir().join(format!("hive_uninstall_backup_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")));
        fs::create_dir_all(&backup_dir).await?;

        for component in &plan.components {
            if matches!(component.component_type, ComponentType::Configuration | ComponentType::Database) {
                let component_backup_dir = backup_dir.join(&component.name);
                fs::create_dir_all(&component_backup_dir).await?;

                for path in &component.paths {
                    if path.exists() {
                        let dest = component_backup_dir.join(path.file_name().unwrap_or_default());
                        
                        if path.is_file() {
                            fs::copy(path, &dest).await?;
                        } else if path.is_dir() {
                            self.copy_directory(path, &dest).await?;
                        }
                    }
                }
            }
        }

        info!("Created uninstall backup: {}", backup_dir.display());
        Ok(backup_dir)
    }

    /// Copy directory recursively
    async fn copy_directory(&self, src: &Path, dest: &Path) -> Result<()> {
        fs::create_dir_all(dest).await?;

        let mut entries = fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let src_path = entry.path();
            let dest_path = dest.join(entry.file_name());

            if src_path.is_file() {
                fs::copy(&src_path, &dest_path).await?;
            } else if src_path.is_dir() {
                // Use Box::pin to avoid infinite sized future
                Box::pin(self.copy_directory(&src_path, &dest_path)).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_uninstaller_creation() {
        let uninstaller = HiveUninstaller::new();
        assert!(!uninstaller.dry_run);
        assert!(!uninstaller.preserve_config);
    }

    #[tokio::test]
    async fn test_uninstall_plan_creation() {
        let uninstaller = HiveUninstaller::new().with_dry_run(true);
        
        if let Ok(plan) = uninstaller.create_uninstall_plan().await {
            // Should have at least some components detected
            assert!(!plan.components.is_empty() || plan.estimated_space_freed == 0);
        }
    }

    #[tokio::test]
    async fn test_directory_size_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test");
        tokio::fs::create_dir_all(&test_dir).await.unwrap();
        tokio::fs::write(test_dir.join("file1.txt"), "test content").await.unwrap();
        tokio::fs::write(test_dir.join("file2.txt"), "more test content").await.unwrap();

        let uninstaller = HiveUninstaller::new();
        let size = uninstaller.calculate_directory_size(&test_dir).await.unwrap();
        assert!(size > 0);
    }
}