//! Repository Context Integration for Consensus Pipeline
//!
//! This module provides Claude Code-style repository awareness to the consensus system,
//! enabling it to understand the current project structure, active files, and codebase context.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::analysis::repository_intelligence::{RepositoryAnalysis, RepositoryAnalyzer};
use crate::core::error::HiveResult;
use crate::desktop::state::{AppState, FileExplorerState};

/// Repository context for consensus pipeline
/// Provides Claude Code-style repository awareness
#[derive(Debug, Clone)]
pub struct RepositoryContext {
    /// Current repository root path
    pub root_path: Option<PathBuf>,
    
    /// Currently active file being edited
    pub active_file: Option<PathBuf>,
    
    /// List of open files in the IDE
    pub open_files: Vec<OpenFileInfo>,
    
    /// Repository analysis results
    pub analysis: Option<RepositoryAnalysis>,
    
    /// Git repository information
    pub git_info: Option<GitRepoInfo>,
    
    /// Project type and language breakdown
    pub project_type: ProjectType,
    pub language_breakdown: HashMap<String, f64>,
    
    /// Key files identified in the project
    pub important_files: Vec<ImportantFile>,
    
    /// Last update timestamp
    pub last_updated: std::time::SystemTime,
}

/// Information about an open file in the IDE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFileInfo {
    pub path: PathBuf,
    pub language: Option<String>,
    pub is_dirty: bool,
    pub last_modified: std::time::SystemTime,
    pub size_bytes: u64,
}

/// Git repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepoInfo {
    pub current_branch: String,
    pub has_uncommitted_changes: bool,
    pub remote_url: Option<String>,
    pub last_commit_hash: Option<String>,
    pub total_commits: u32,
}

/// Project type identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    CSharp,
    Cpp,
    Unknown,
    Mixed,
}

/// Important files in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportantFile {
    pub path: PathBuf,
    pub file_type: ImportantFileType,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportantFileType {
    ConfigFile,    // Cargo.toml, package.json, etc.
    EntryPoint,    // main.rs, index.ts, etc.
    Documentation, // README.md, CHANGELOG.md, etc.
    BuildScript,   // build.rs, webpack.config.js, etc.
    Schema,        // database schemas, API specs, etc.
}

/// Repository Context Manager
/// Integrates with IDE state to provide repository awareness to consensus
pub struct RepositoryContextManager {
    /// Repository analyzer for deep codebase understanding
    analyzer: Arc<RepositoryAnalyzer>,
    
    /// Current repository context
    context: Arc<RwLock<RepositoryContext>>,
    
    /// Cache of analyzed repositories
    analysis_cache: Arc<RwLock<HashMap<PathBuf, RepositoryAnalysis>>>,
}

impl RepositoryContextManager {
    /// Create a new repository context manager
    pub async fn new() -> Result<Self> {
        // Create dependencies for RepositoryAnalyzer
        use crate::analysis::{symbol_index::SymbolIndexer, dependency::DependencyAnalyzer};
        use crate::core::database::{get_database, DatabaseManager};
        
        // Get database manager
        let db = get_database().await?;
        
        let symbol_indexer = Arc::new(SymbolIndexer::new(db.clone()).await?);
        let dependency_analyzer = Arc::new(DependencyAnalyzer::new().await?);
        let analyzer = Arc::new(RepositoryAnalyzer::new(symbol_indexer, dependency_analyzer).await?);
        
        let context = Arc::new(RwLock::new(RepositoryContext {
            root_path: None,
            active_file: None,
            open_files: Vec::new(),
            analysis: None,
            git_info: None,
            project_type: ProjectType::Unknown,
            language_breakdown: HashMap::new(),
            important_files: Vec::new(),
            last_updated: std::time::SystemTime::now(),
        }));
        
        let analysis_cache = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            analyzer,
            context,
            analysis_cache,
        })
    }
    
    /// Update repository context from IDE state
    /// This is called whenever the IDE state changes (file selection, project change, etc.)
    #[instrument(skip(self, app_state))]
    pub async fn update_from_ide_state(&self, app_state: &AppState) -> HiveResult<()> {
        let mut context = self.context.write().await;
        
        // Update root path from file explorer
        if let Some(project_info) = &app_state.current_project {
            if context.root_path.as_ref() != Some(&project_info.root_path) {
                info!("Repository root changed to: {}", project_info.root_path.display());
                context.root_path = Some(project_info.root_path.clone());
                
                // Trigger repository analysis for new root
                self.analyze_repository_async(&project_info.root_path).await?;
            }
        }
        
        // Update active file from file explorer
        if let Some(selected_file) = &app_state.file_explorer.selected_file {
            if context.active_file.as_ref() != Some(selected_file) {
                debug!("Active file changed to: {}", selected_file.display());
                context.active_file = Some(selected_file.clone());
            }
        }
        
        // Update open files list (this would need to be added to AppState)
        // For now, we'll use the selected file as the only "open" file
        context.open_files.clear();
        if let Some(active_file) = context.active_file.clone() {
            context.open_files.push(OpenFileInfo {
                path: active_file.clone(),
                language: detect_language(&active_file),
                is_dirty: false, // TODO: Track dirty state in IDE
                last_modified: std::time::SystemTime::now(),
                size_bytes: 0, // TODO: Get actual file size
            });
        }
        
        context.last_updated = std::time::SystemTime::now();
        
        Ok(())
    }
    
    /// Get current repository context
    pub async fn get_context(&self) -> RepositoryContext {
        self.context.read().await.clone()
    }
    
    /// Analyze the current repository if not already cached
    #[instrument(skip(self))]
    pub async fn analyze_repository_async(&self, root_path: &Path) -> HiveResult<()> {
        // Check if we already have analysis cached
        {
            let cache = self.analysis_cache.read().await;
            if cache.contains_key(root_path) {
                debug!("Repository analysis found in cache for {}", root_path.display());
                
                // Update context with cached analysis
                let mut context = self.context.write().await;
                context.analysis = cache.get(root_path).cloned();
                return Ok(());
            }
        }
        
        info!("Starting repository analysis for: {}", root_path.display());
        
        // Spawn analysis task to avoid blocking
        let analyzer = self.analyzer.clone();
        let analysis_cache = self.analysis_cache.clone();
        let context = self.context.clone();
        let root_path_owned = root_path.to_path_buf();
        
        tokio::spawn(async move {
            match analyzer.analyze_repository(&root_path_owned).await {
                Ok(analysis) => {
                    info!("Repository analysis completed for: {}", root_path_owned.display());
                    
                    // Cache the analysis
                    {
                        let mut cache = analysis_cache.write().await;
                        cache.insert(root_path_owned.clone(), analysis.clone());
                    }
                    
                    // Update context
                    {
                        let mut ctx = context.write().await;
                        ctx.analysis = Some(analysis);
                        ctx.project_type = detect_project_type(&root_path_owned);
                        ctx.language_breakdown = analyze_language_breakdown(&root_path_owned).await;
                        ctx.important_files = identify_important_files(&root_path_owned).await;
                        ctx.git_info = analyze_git_info(&root_path_owned).await;
                        ctx.last_updated = std::time::SystemTime::now();
                    }
                    
                    debug!("Repository context updated with analysis results");
                }
                Err(e) => {
                    warn!("Repository analysis failed for {}: {}", root_path_owned.display(), e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Get repository analysis for use in consensus pipeline
    pub async fn get_analysis_for_consensus(&self) -> Option<RepositoryAnalysis> {
        self.context.read().await.analysis.clone()
    }
    
    /// Get context information for prompts
    pub async fn get_context_for_prompts(&self) -> String {
        let context = self.context.read().await;
        
        let mut prompt_context = String::new();
        
        if let Some(root_path) = &context.root_path {
            prompt_context.push_str(&format!("Repository: {}\n", root_path.display()));
        }
        
        if let Some(active_file) = &context.active_file {
            prompt_context.push_str(&format!("Active file: {}\n", active_file.display()));
        }
        
        if !context.open_files.is_empty() {
            prompt_context.push_str("Open files:\n");
            for file in &context.open_files {
                prompt_context.push_str(&format!("  - {}\n", file.path.display()));
            }
        }
        
        if let Some(analysis) = &context.analysis {
            prompt_context.push_str(&format!("Project type: {:?}\n", context.project_type));
            prompt_context.push_str(&format!("Architecture: {:?}\n", analysis.architecture.primary_pattern));
        }
        
        prompt_context
    }
}

/// Detect the programming language of a file
fn detect_language(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "ts" => "typescript",
            "js" => "javascript",
            "py" => "python",
            "go" => "go",
            "java" => "java",
            "cs" => "csharp",
            "cpp" | "cc" | "cxx" => "cpp",
            "c" => "c",
            "h" | "hpp" => "c-header",
            "md" => "markdown",
            "toml" => "toml",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            _ => "text",
        })
        .map(|s| s.to_string())
}

/// Detect the primary project type
fn detect_project_type(root_path: &Path) -> ProjectType {
    if root_path.join("Cargo.toml").exists() {
        return ProjectType::Rust;
    }
    
    if root_path.join("package.json").exists() {
        if root_path.join("tsconfig.json").exists() {
            return ProjectType::TypeScript;
        }
        return ProjectType::JavaScript;
    }
    
    if root_path.join("requirements.txt").exists() || root_path.join("pyproject.toml").exists() {
        return ProjectType::Python;
    }
    
    if root_path.join("go.mod").exists() {
        return ProjectType::Go;
    }
    
    if root_path.join("pom.xml").exists() || root_path.join("build.gradle").exists() {
        return ProjectType::Java;
    }
    
    if root_path.join("*.csproj").exists() || root_path.join("*.sln").exists() {
        return ProjectType::CSharp;
    }
    
    if root_path.join("CMakeLists.txt").exists() || root_path.join("Makefile").exists() {
        return ProjectType::Cpp;
    }
    
    ProjectType::Unknown
}

/// Analyze language breakdown in the repository
async fn analyze_language_breakdown(root_path: &Path) -> HashMap<String, f64> {
    // TODO: Implement actual language analysis
    // For now, return a simple breakdown based on project type
    let mut breakdown = HashMap::new();
    
    let project_type = detect_project_type(root_path);
    match project_type {
        ProjectType::Rust => {
            breakdown.insert("rust".to_string(), 0.85);
            breakdown.insert("toml".to_string(), 0.15);
        }
        ProjectType::TypeScript => {
            breakdown.insert("typescript".to_string(), 0.80);
            breakdown.insert("json".to_string(), 0.20);
        }
        _ => {
            breakdown.insert("unknown".to_string(), 1.0);
        }
    }
    
    breakdown
}

/// Identify important files in the project
async fn identify_important_files(root_path: &Path) -> Vec<ImportantFile> {
    let mut important_files = Vec::new();
    
    // Config files
    for (filename, description) in [
        ("Cargo.toml", "Rust package manifest"),
        ("package.json", "Node.js package manifest"),
        ("pyproject.toml", "Python project configuration"),
        ("go.mod", "Go module definition"),
        ("pom.xml", "Maven project configuration"),
    ] {
        let path = root_path.join(filename);
        if path.exists() {
            important_files.push(ImportantFile {
                path,
                file_type: ImportantFileType::ConfigFile,
                description: description.to_string(),
            });
        }
    }
    
    // Entry points
    for (filename, description) in [
        ("src/main.rs", "Rust main entry point"),
        ("src/lib.rs", "Rust library entry point"),
        ("index.ts", "TypeScript entry point"),
        ("main.py", "Python main module"),
        ("main.go", "Go main package"),
    ] {
        let path = root_path.join(filename);
        if path.exists() {
            important_files.push(ImportantFile {
                path,
                file_type: ImportantFileType::EntryPoint,
                description: description.to_string(),
            });
        }
    }
    
    // Documentation
    for (filename, description) in [
        ("README.md", "Project documentation"),
        ("CHANGELOG.md", "Change log"),
        ("LICENSE", "License file"),
        ("docs", "Documentation directory"),
    ] {
        let path = root_path.join(filename);
        if path.exists() {
            important_files.push(ImportantFile {
                path,
                file_type: ImportantFileType::Documentation,
                description: description.to_string(),
            });
        }
    }
    
    important_files
}

/// Analyze Git repository information
async fn analyze_git_info(root_path: &Path) -> Option<GitRepoInfo> {
    let git_dir = root_path.join(".git");
    if !git_dir.exists() {
        return None;
    }
    
    // TODO: Implement actual Git analysis using git2 crate
    // For now, return basic info
    Some(GitRepoInfo {
        current_branch: "main".to_string(),
        has_uncommitted_changes: false,
        remote_url: None,
        last_commit_hash: None,
        total_commits: 0,
    })
}

impl Default for RepositoryContext {
    fn default() -> Self {
        Self {
            root_path: None,
            active_file: None,
            open_files: Vec::new(),
            analysis: None,
            git_info: None,
            project_type: ProjectType::Unknown,
            language_breakdown: HashMap::new(),
            important_files: Vec::new(),
            last_updated: std::time::SystemTime::now(),
        }
    }
}