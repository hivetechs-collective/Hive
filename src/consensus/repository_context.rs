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
#[cfg(feature = "desktop-legacy")]
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

    /// Project type
    pub project_type: ProjectType,

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
        use crate::analysis::{dependency::DependencyAnalyzer, symbol_index::SymbolIndexer};
        use crate::core::database::{get_database, DatabaseManager};

        // Get database manager
        let db = get_database().await?;

        let symbol_indexer = Arc::new(SymbolIndexer::new(db.clone()).await?);
        let dependency_analyzer = Arc::new(DependencyAnalyzer::new().await?);
        let analyzer =
            Arc::new(RepositoryAnalyzer::new(symbol_indexer, dependency_analyzer).await?);

        let context = Arc::new(RwLock::new(RepositoryContext {
            root_path: None,
            active_file: None,
            open_files: Vec::new(),
            analysis: None,
            git_info: None,
            project_type: ProjectType::Unknown,
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
    #[cfg(feature = "desktop-legacy")]
    pub async fn update_from_ide_state(&self, app_state: &AppState) -> HiveResult<()> {
        let mut context = self.context.write().await;

        // First check if a directory is selected in the file explorer
        // This takes precedence over current_project to reflect what the user has selected
        let effective_root = if let Some(selected_dir) = &app_state.file_explorer.selected_directory
        {
            info!(
                "Using selected directory from File Explorer: {}",
                selected_dir.display()
            );
            selected_dir.clone()
        } else if let Some(project_info) = &app_state.current_project {
            project_info.root_path.clone()
        } else {
            // Fall back to current working directory
            std::env::current_dir().unwrap_or_default()
        };

        // Update root path if it changed
        if context.root_path.as_ref() != Some(&effective_root) {
            info!("Repository root changed to: {}", effective_root.display());
            context.root_path = Some(effective_root.clone());

            // Trigger repository analysis for new root
            self.analyze_repository_async(&effective_root).await?;
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
                debug!(
                    "Repository analysis found in cache for {}",
                    root_path.display()
                );

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
                    info!(
                        "Repository analysis completed for: {}",
                        root_path_owned.display()
                    );

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
                        ctx.git_info = analyze_git_info(&root_path_owned).await;
                        ctx.last_updated = std::time::SystemTime::now();
                    }

                    debug!("Repository context updated with analysis results");
                }
                Err(e) => {
                    warn!(
                        "Repository analysis failed for {}: {}",
                        root_path_owned.display(),
                        e
                    );
                }
            }
        });

        Ok(())
    }

    /// Get repository analysis for use in consensus pipeline
    pub async fn get_analysis_for_consensus(&self) -> Option<RepositoryAnalysis> {
        self.context.read().await.analysis.clone()
    }

    /// Detect key files in the repository
    async fn detect_key_files(&self, root_path: &Path) -> Vec<String> {
        let mut key_files = Vec::new();

        // Check for various project files
        let files_to_check = [
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "go.mod",
            "pom.xml",
            "build.gradle",
            "CMakeLists.txt",
            "Makefile",
            ".gitignore",
            "README.md",
            "LICENSE",
            "Dockerfile",
            "requirements.txt",
            "setup.py",
            "tsconfig.json",
            "webpack.config.js",
        ];

        for file in &files_to_check {
            if root_path.join(file).exists() {
                key_files.push(file.to_string());
            }
        }

        // Check for source directories
        let dirs_to_check = ["src", "lib", "test", "tests", "spec", "docs"];
        for dir in &dirs_to_check {
            if root_path.join(dir).is_dir() {
                key_files.push(format!("{}/", dir));
            }
        }

        key_files
    }

    /// Detect the build system based on project files
    async fn detect_build_system(&self, root_path: &Path) -> Option<String> {
        if root_path.join("Cargo.toml").exists() {
            Some("Cargo (Rust)".to_string())
        } else if root_path.join("package.json").exists() {
            Some("npm/yarn (JavaScript/TypeScript)".to_string())
        } else if root_path.join("requirements.txt").exists()
            || root_path.join("pyproject.toml").exists()
        {
            Some("pip/poetry (Python)".to_string())
        } else if root_path.join("go.mod").exists() {
            Some("go mod (Go)".to_string())
        } else if root_path.join("pom.xml").exists() {
            Some("Maven (Java)".to_string())
        } else if root_path.join("build.gradle").exists() {
            Some("Gradle (Java/Kotlin)".to_string())
        } else if root_path.join("CMakeLists.txt").exists() {
            Some("CMake (C/C++)".to_string())
        } else if root_path.join("Makefile").exists() {
            Some("Make".to_string())
        } else {
            None
        }
    }

    /// Try to get repository description from README or package files
    async fn get_repository_description(&self, root_path: &Path) -> String {
        // Try README files
        for readme_name in &["README.md", "README.txt", "README", "readme.md"] {
            let readme_path = root_path.join(readme_name);
            if readme_path.exists() {
                if let Ok(content) = tokio::fs::read_to_string(&readme_path).await {
                    // Extract first few meaningful lines
                    let lines: Vec<&str> = content
                        .lines()
                        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                        .take(3)
                        .collect();
                    if !lines.is_empty() {
                        return lines.join(" ").chars().take(300).collect();
                    }
                }
            }
        }

        // Try Cargo.toml for Rust projects
        let cargo_path = root_path.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&cargo_path).await {
                // Simple extraction of description field
                for line in content.lines() {
                    if line.starts_with("description") && line.contains('=') {
                        if let Some(desc) = line.split('=').nth(1) {
                            return desc.trim().trim_matches('"').to_string();
                        }
                    }
                }
            }
        }

        // Try package.json for Node.js projects
        let package_path = root_path.join("package.json");
        if package_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&package_path).await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(desc) = json.get("description").and_then(|d| d.as_str()) {
                        return desc.to_string();
                    }
                }
            }
        }

        String::new()
    }

    /// Get context information for prompts
    pub async fn get_context_for_prompts(&self) -> String {
        let context = self.context.read().await;

        let mut prompt_context = String::new();

        // Add strong instructions that will persist through all stages
        prompt_context.push_str("## 🎯 CRITICAL REPOSITORY CONTEXT - MAINTAIN FOCUS\n\n");
        prompt_context.push_str("⚠️ IMPORTANT: You are analyzing a SPECIFIC repository. ALL responses must relate to THIS repository.\n");
        prompt_context.push_str("DO NOT make assumptions about other projects. Base your analysis ONLY on the following context:\n\n");

        if let Some(root_path) = &context.root_path {
            prompt_context.push_str(&format!("📁 Repository Path: {}\n", root_path.display()));

            // Extract repository name from path
            if let Some(repo_name) = root_path.file_name() {
                prompt_context.push_str(&format!(
                    "📋 Repository Name: {}\n",
                    repo_name.to_string_lossy()
                ));
            }
        }

        // Always include project type, even without full analysis
        prompt_context.push_str(&format!("🔧 Project Type: {:?}\n", context.project_type));

        // Check for key Rust project indicators
        if context
            .root_path
            .as_ref()
            .map(|p| p.join("Cargo.toml").exists())
            .unwrap_or(false)
        {
            prompt_context.push_str("\n⚡ CRITICAL: This is a RUST project with Cargo.toml!\n");
            prompt_context
                .push_str("DO NOT describe it as Node.js, JavaScript, or any other language.\n");
            prompt_context.push_str("Key files: Cargo.toml, src/main.rs, src/lib.rs\n");

            // Check if this is the Hive AI project specifically
            if context
                .root_path
                .as_ref()
                .map(|p| p.ends_with("hive"))
                .unwrap_or(false)
            {
                prompt_context.push_str("\n🐝 PROJECT DESCRIPTION:\n");
                prompt_context.push_str("This is Hive AI - a complete Rust reimplementation of the original TypeScript Hive AI.\n");
                prompt_context.push_str("It provides AI-powered development assistance with a revolutionary 4-stage consensus engine.\n");
                prompt_context.push_str("\nKEY FEATURES:\n");
                prompt_context.push_str(
                    "- 4-Stage Consensus Pipeline: Generator → Refiner → Validator → Curator\n",
                );
                prompt_context.push_str("- 323+ AI models via OpenRouter integration\n");
                prompt_context.push_str("- Repository Intelligence for codebase understanding\n");
                prompt_context.push_str("- VS Code-like TUI interface\n");
                prompt_context
                    .push_str("- 10-40x performance improvement over TypeScript version\n");
                prompt_context.push_str("- Enterprise features and hooks system\n");
                prompt_context
                    .push_str("- Authoritative Knowledge Store (hive mind consciousness)\n");
                prompt_context.push_str("\nMAIN COMPONENTS:\n");
                prompt_context
                    .push_str("- src/consensus/: 4-stage consensus engine implementation\n");
                prompt_context
                    .push_str("- src/ai_helpers/: AI-powered code analysis and context\n");
                prompt_context.push_str("- src/tui/: Terminal user interface (VS Code-like)\n");
                prompt_context.push_str("- src/analysis/: Repository and code analysis\n");
                prompt_context.push_str("- src/bin/hive-consensus.rs: GUI application\n");
            }
        }

        // Try to read README or similar files for project description
        if let Some(root_path) = &context.root_path {
            let description = self.get_repository_description(root_path).await;
            if !description.is_empty() {
                prompt_context.push_str(&format!("\n📄 PROJECT DESCRIPTION:\n{}\n", description));
            }
        }

        if let Some(analysis) = &context.analysis {
            prompt_context.push_str(&format!(
                "🏗️ Architecture: {:?}\n",
                analysis.architecture.primary_pattern
            ));

            // Add quality metrics
            prompt_context.push_str(&format!(
                "\n📊 Quality Score: {:.1}/10\n",
                analysis.quality.overall_score
            ));

            // Add architecture info
            if !analysis.architecture.detected_patterns.is_empty() {
                prompt_context.push_str("\n🏗️ Architecture Patterns:\n");
                for pattern in &analysis.architecture.detected_patterns {
                    prompt_context.push_str(&format!("  - {:?}\n", pattern));
                }
            }

            // Add project-specific details based on detected files
            if let Some(root_path) = &context.root_path {
                prompt_context.push_str("\n📝 PROJECT DETAILS:\n");

                // List actual files found in the project
                let key_files = self.detect_key_files(root_path).await;
                if !key_files.is_empty() {
                    prompt_context.push_str("Key files detected:\n");
                    for file in &key_files {
                        prompt_context.push_str(&format!("  - {}\n", file));
                    }
                }

                // Add build system info
                if let Some(build_system) = self.detect_build_system(root_path).await {
                    prompt_context.push_str(&format!("Build System: {}\n", build_system));
                }

                prompt_context.push_str("\nIMPORTANT: Base your analysis ONLY on the actual files and structure found in THIS repository.\n");
            }
        }

        if let Some(active_file) = &context.active_file {
            prompt_context.push_str(&format!(
                "\n✏️ Currently Editing: {}\n",
                active_file.display()
            ));
        }

        if !context.open_files.is_empty() {
            prompt_context.push_str("\n📂 Open Files:\n");
            for file in &context.open_files {
                prompt_context.push_str(&format!("  - {}\n", file.path.display()));
            }
        }

        prompt_context.push_str("\n⚡ CONSENSUS INSTRUCTION: When analyzing or discussing code, ALWAYS refer to THIS specific repository.\n");
        prompt_context.push_str(
            "Do NOT invent or assume files/features that don't exist in this repository.\n",
        );

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
            last_updated: std::time::SystemTime::now(),
        }
    }
}
