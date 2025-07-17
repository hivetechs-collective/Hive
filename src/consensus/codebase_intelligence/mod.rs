//! Codebase Intelligence System
//! 
//! Provides deep analysis and indexing of entire codebases triggered by @codebase command.
//! Creates searchable knowledge base accessible to all consensus stages.

pub mod scanner;
pub mod extractor;
pub mod analyzer;
pub mod storage;
pub mod search;

use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;

pub use scanner::CodebaseScanner;
pub use extractor::{ObjectExtractor, ExtractedObject, ObjectKind};
pub use analyzer::{RelationshipAnalyzer, ArchitectureDeriver};
pub use storage::CodebaseStorage;
pub use search::CodebaseSearch;

/// Main interface for codebase intelligence
pub struct CodebaseIntelligence {
    repository_path: Option<PathBuf>,
    scanner: Arc<CodebaseScanner>,
    extractor: Arc<ObjectExtractor>,
    analyzer: Arc<RelationshipAnalyzer>,
    storage: Arc<CodebaseStorage>,
    search: Arc<CodebaseSearch>,
    current_scan_id: Arc<RwLock<Option<String>>>,
}

impl CodebaseIntelligence {
    /// Create a new codebase intelligence system
    pub fn new(database: Arc<crate::core::database::DatabaseManager>) -> Self {
        let storage = Arc::new(CodebaseStorage::new(database.clone()));
        let search = Arc::new(CodebaseSearch::new(database));
        
        Self {
            repository_path: None,
            scanner: Arc::new(CodebaseScanner::new()),
            extractor: Arc::new(ObjectExtractor::new()),
            analyzer: Arc::new(RelationshipAnalyzer::new()),
            storage,
            search,
            current_scan_id: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Set the repository path to analyze
    pub fn set_repository(&mut self, path: PathBuf) {
        self.repository_path = Some(path);
    }
    
    /// Check if a message contains @codebase command
    pub fn is_codebase_command(message: &str) -> bool {
        message.trim().starts_with("@codebase") || message.trim() == "@codebase"
    }
    
    /// Extract command arguments from @codebase message
    pub fn parse_command(message: &str) -> CodebaseCommand {
        let trimmed = message.trim();
        if trimmed == "@codebase" {
            CodebaseCommand::FullScan
        } else if trimmed.starts_with("@codebase refresh") {
            CodebaseCommand::Refresh
        } else if trimmed.starts_with("@codebase search ") {
            let query = trimmed.trim_start_matches("@codebase search ").trim();
            CodebaseCommand::Search(query.to_string())
        } else {
            CodebaseCommand::FullScan
        }
    }
    
    /// Run codebase analysis
    pub async fn analyze_codebase(&self, progress_callback: impl Fn(AnalysisProgress) + Send + Sync + 'static) -> Result<AnalysisResult> {
        let repo_path = self.repository_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No repository path set"))?;
            
        progress_callback(AnalysisProgress::Starting);
        
        // 1. Scan all files
        progress_callback(AnalysisProgress::Scanning { current: 0, total: 0 });
        let scan_result = self.scanner.scan_repository(repo_path).await?;
        
        // 2. Extract objects from code
        progress_callback(AnalysisProgress::Extracting { current: 0, total: scan_result.files.len() });
        let mut all_objects = Vec::new();
        
        for (idx, file) in scan_result.files.iter().enumerate() {
            if let Ok(objects) = self.extractor.extract_from_file(file).await {
                all_objects.extend(objects);
            }
            progress_callback(AnalysisProgress::Extracting { 
                current: idx + 1, 
                total: scan_result.files.len() 
            });
        }
        
        // 3. Analyze relationships
        progress_callback(AnalysisProgress::Analyzing);
        let relationships = self.analyzer.analyze_objects(&all_objects).await?;
        let architecture = self.analyzer.derive_architecture(&all_objects, &relationships).await?;
        
        // 4. Store in database
        progress_callback(AnalysisProgress::Indexing);
        let scan_id = self.storage.store_scan(
            repo_path,
            &all_objects,
            &relationships,
            &architecture
        ).await?;
        
        // Update current scan ID
        *self.current_scan_id.write().await = Some(scan_id.clone());
        
        // 5. Build search index
        self.search.rebuild_index(&scan_id).await?;
        
        progress_callback(AnalysisProgress::Complete);
        
        Ok(AnalysisResult {
            scan_id,
            total_files: scan_result.files.len(),
            total_objects: all_objects.len(),
            architecture: architecture.pattern,
            key_concepts: self.extract_key_concepts(&all_objects),
        })
    }
    
    /// Get context for a specific question using indexed knowledge
    pub async fn get_context_for_question(&self, question: &str) -> Result<String> {
        let scan_id = self.current_scan_id.read().await.clone()
            .ok_or_else(|| anyhow::anyhow!("No codebase analysis available. Run @codebase first."))?;
            
        // Search for relevant objects
        let search_results = self.search.search_objects(&scan_id, question).await?;
        let architecture = self.storage.get_architecture(&scan_id).await?;
        
        // Build context
        let mut context = String::new();
        context.push_str("## 🧠 INTELLIGENT CODEBASE CONTEXT\n\n");
        
        if let Some(arch) = architecture {
            context.push_str(&format!("**Architecture**: {}\n", arch));
        }
        
        if !search_results.is_empty() {
            context.push_str("\n**Relevant Components**:\n");
            for (idx, obj) in search_results.iter().take(10).enumerate() {
                context.push_str(&format!(
                    "{}. `{}` ({}) at `{}`:{}\n",
                    idx + 1,
                    obj.name,
                    obj.kind,
                    obj.file_path.display(),
                    obj.line_start
                ));
                if let Some(doc) = &obj.documentation {
                    context.push_str(&format!("   {}\n", doc));
                }
                if !obj.context.is_empty() {
                    context.push_str(&format!("   **Purpose**: {}\n", obj.context));
                }
                context.push_str("\n");
            }
        }
        
        Ok(context)
    }
    
    /// Check if codebase has been analyzed
    pub async fn has_analysis(&self) -> bool {
        self.current_scan_id.read().await.is_some()
    }
    
    /// Extract key concepts from objects
    fn extract_key_concepts(&self, objects: &[ExtractedObject]) -> Vec<String> {
        use std::collections::HashMap;
        
        let mut concept_freq: HashMap<String, usize> = HashMap::new();
        
        for obj in objects {
            // Extract words from object names
            let words = obj.name.split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 3)
                .map(|w| w.to_lowercase());
                
            for word in words {
                *concept_freq.entry(word).or_insert(0) += 1;
            }
        }
        
        // Sort by frequency and take top concepts
        let mut concepts: Vec<_> = concept_freq.into_iter().collect();
        concepts.sort_by(|a, b| b.1.cmp(&a.1));
        
        concepts.into_iter()
            .take(10)
            .map(|(word, _)| word)
            .collect()
    }
}

/// Commands that can be issued with @codebase
#[derive(Debug, Clone)]
pub enum CodebaseCommand {
    FullScan,
    Refresh,
    Search(String),
}

/// Progress updates during analysis
#[derive(Debug, Clone)]
pub enum AnalysisProgress {
    Starting,
    Scanning { current: usize, total: usize },
    Extracting { current: usize, total: usize },
    Analyzing,
    Indexing,
    Complete,
}

/// Result of codebase analysis
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub scan_id: String,
    pub total_files: usize,
    pub total_objects: usize,
    pub architecture: String,
    pub key_concepts: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_parsing() {
        assert!(CodebaseIntelligence::is_codebase_command("@codebase"));
        assert!(CodebaseIntelligence::is_codebase_command("@codebase refresh"));
        assert!(CodebaseIntelligence::is_codebase_command(" @codebase "));
        assert!(!CodebaseIntelligence::is_codebase_command("codebase"));
        
        match CodebaseIntelligence::parse_command("@codebase search consensus") {
            CodebaseCommand::Search(query) => assert_eq!(query, "consensus"),
            _ => panic!("Expected search command"),
        }
    }
}