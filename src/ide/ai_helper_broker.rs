#![cfg(feature = "desktop-legacy")]
//! IDE-level AI Helper Broker Service
//!
//! This service runs in the IDE (hive-consensus GUI) and acts as an intelligent
//! broker between the File Explorer state and the consensus engine. It uses
//! local AI models to understand context and prepare information for consensus.

use anyhow::Result;
use dioxus::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// Simplified approach - don't use full AI Helper ecosystem initially
// use crate::ai_helpers::{
//     AIHelperEcosystem,
//     autonomous_ai_helper::AutonomousAIHelper,
// };

/// IDE-level AI Helper Broker that maintains awareness of IDE state
pub struct IDEAIHelperBroker {
    /// Current repository context from File Explorer
    current_repository: Arc<RwLock<Option<RepositoryInfo>>>,

    /// Reference to current_dir signal from GUI
    current_dir: Signal<Option<PathBuf>>,

    /// Reference to file_tree signal from GUI
    file_tree: Signal<Vec<FileItem>>,

    /// Reference to selected_file signal from GUI
    selected_file: Signal<Option<String>>,
}

#[derive(Clone, Debug)]
pub struct RepositoryInfo {
    pub path: PathBuf,
    pub name: String,
    pub files: Vec<PathBuf>,
    pub selected_file: Option<PathBuf>,
}

impl IDEAIHelperBroker {
    /// Create a new IDE AI Helper Broker
    pub async fn new(
        current_dir: Signal<Option<PathBuf>>,
        file_tree: Signal<Vec<FileItem>>,
        selected_file: Signal<Option<String>>,
    ) -> Result<Self> {
        info!("🤖 Creating simplified IDE AI Helper Broker");

        Ok(Self {
            current_repository: Arc::new(RwLock::new(None)),
            current_dir,
            file_tree,
            selected_file,
        })
    }

    /// Update repository context from current IDE state
    pub async fn update_repository_context(&self) -> Result<()> {
        info!("🤖 IDE AI Helper Broker updating repository context");

        // Read current directory from File Explorer
        let current_dir = self.current_dir.read().clone();

        if let Some(dir_path) = current_dir {
            info!(
                "📁 Current directory in File Explorer: {}",
                dir_path.display()
            );

            // Extract repository info from file tree
            let files: Vec<PathBuf> = self
                .file_tree
                .read()
                .iter()
                .map(|item| item.path.clone())
                .collect();

            let selected = self.selected_file.read().clone().map(|s| PathBuf::from(s));

            let repo_info = RepositoryInfo {
                path: dir_path.clone(),
                name: dir_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                files,
                selected_file: selected,
            };

            // Update our repository context
            *self.current_repository.write().await = Some(repo_info.clone());

            info!(
                "✅ Repository context updated: {} with {} files",
                repo_info.name,
                repo_info.files.len()
            );
        } else {
            info!("📂 No directory open in File Explorer");
            *self.current_repository.write().await = None;
        }

        Ok(())
    }

    /// Process user query with repository awareness
    pub async fn process_query_with_context(&self, query: &str) -> Result<EnhancedQuery> {
        info!("🤖 IDE AI Helper processing query with repository context");

        // Update repository context first
        self.update_repository_context().await?;

        // Get current repository info
        let repo_info = self.current_repository.read().await.clone();

        // Build enhanced query with context (simplified approach)
        let mut enhanced_query = EnhancedQuery {
            original_query: query.to_string(),
            repository_context: None,
            selected_file_context: None,
            ai_understanding: "Simple repository awareness".to_string(),
            confidence: 0.8,
            suggested_files: Vec::new(),
        };

        if let Some(repo) = repo_info {
            info!("📁 Adding repository context: {}", repo.name);

            enhanced_query.repository_context = Some(format!(
                "Current repository: {} at {} with {} files loaded in File Explorer",
                repo.name,
                repo.path.display(),
                repo.files.len()
            ));

            if let Some(selected) = repo.selected_file {
                enhanced_query.selected_file_context =
                    Some(format!("Selected file: {}", selected.display()));
            }

            // Simple file suggestion based on common patterns
            if query.contains("rust") || query.contains(".rs") {
                enhanced_query.suggested_files = repo
                    .files
                    .iter()
                    .filter(|f| f.extension().map_or(false, |ext| ext == "rs"))
                    .take(5)
                    .map(|f| f.to_string_lossy().to_string())
                    .collect();
            }
        }

        Ok(enhanced_query)
    }

    /// Get repository description for consensus
    pub async fn get_repository_description(&self) -> Result<String> {
        let repo_info = self.current_repository.read().await.clone();

        if let Some(repo) = repo_info {
            Ok(format!(
                "Repository: {} at {}\nFiles: {} files loaded in File Explorer",
                repo.name,
                repo.path.display(),
                repo.files.len()
            ))
        } else {
            Ok("No repository currently open in the IDE.".to_string())
        }
    }

    /// Handle file navigation requests from consensus
    pub async fn navigate_to_file(&self, file_path: &str) -> Result<()> {
        info!("🤖 IDE AI Helper navigating to file: {}", file_path);

        // Convert to PathBuf
        let path = PathBuf::from(file_path);

        // Note: We can't directly modify Dioxus signals from here since they're not mutable
        // This would need to be done through a message system or UI events
        info!("📁 Navigation request registered for: {}", path.display());

        Ok(())
    }
}

/// Enhanced query with repository context
#[derive(Debug, Clone)]
pub struct EnhancedQuery {
    pub original_query: String,
    pub repository_context: Option<String>,
    pub selected_file_context: Option<String>,
    pub ai_understanding: String,
    pub confidence: f32,
    pub suggested_files: Vec<String>,
}

impl EnhancedQuery {
    /// Convert to consensus-ready format
    pub fn to_consensus_query(&self) -> String {
        let mut parts = vec![self.original_query.clone()];

        if let Some(repo) = &self.repository_context {
            parts.push(format!("\n\n[Repository Context]\n{}", repo));
        }

        if let Some(file) = &self.selected_file_context {
            parts.push(format!("\n[Selected File]\n{}", file));
        }

        if !self.suggested_files.is_empty() {
            parts.push(format!(
                "\n[AI Suggested Files]\n{}",
                self.suggested_files.join("\n")
            ));
        }

        parts.join("\n")
    }
}

// Import FileItem from desktop state
use crate::desktop::state::FileItem;
