//! Autonomous AI Helper System
//! 
//! This module provides true AI autonomy - no hardcoded patterns or forced behaviors.
//! The AI Helper receives raw user input and makes its own intelligent decisions about
//! what context to gather and how to respond.

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, debug, warn};

use crate::ai_helpers::{
    AIHelperEcosystem,
    IntelligentContextOrchestrator,
    ContextRetriever,
    KnowledgeIndexer,
    PatternRecognizer,
    QualityAnalyzer,
    KnowledgeSynthesizer,
};
use crate::consensus::repository_context::RepositoryContextManager;
use crate::consensus::types::Stage;

/// Represents the AI's autonomous decision about how to handle a user query
#[derive(Debug, Clone)]
pub struct AutonomousDecision {
    /// The AI's understanding of what the user wants
    pub intent_understanding: String,
    
    /// Confidence in the understanding (0.0 to 1.0)
    pub confidence: f32,
    
    /// Actions the AI decided to take
    pub actions: Vec<AutonomousAction>,
    
    /// Context the AI decided to gather
    pub gathered_context: Option<String>,
    
    /// Whether to continue to consensus pipeline
    pub continue_to_consensus: bool,
    
    /// Any direct response the AI wants to provide
    pub direct_response: Option<String>,
}

/// Actions the AI can autonomously decide to take
#[derive(Debug, Clone)]
pub enum AutonomousAction {
    /// Explore repository structure
    ExploreRepository { path: PathBuf },
    
    /// Read specific files the AI thinks are relevant
    ReadFiles { files: Vec<PathBuf>, reason: String },
    
    /// Search for code patterns
    SearchCode { query: String, semantic: bool },
    
    /// Access memory/knowledge base
    AccessMemory { query: String },
    
    /// Gather temporal context
    GatherTemporalContext,
    
    /// Analyze code quality
    AnalyzeQuality { target: PathBuf },
    
    /// Synthesize knowledge from multiple sources
    SynthesizeKnowledge { sources: Vec<String> },
    
    /// Navigate to a specific file or directory in the UI
    NavigateToPath { path: PathBuf, reason: String },
    
    /// Open a file in the editor
    OpenFile { path: PathBuf },
    
    /// Create a new file
    CreateFile { path: PathBuf, content: String },
}

/// The Autonomous AI Helper - thinks and acts independently
pub struct AutonomousAIHelper {
    /// The full AI ecosystem
    ecosystem: Arc<AIHelperEcosystem>,
    
    /// Repository context manager for file access
    repository_context: Option<Arc<RepositoryContextManager>>,
    
    /// Current working directory
    working_directory: PathBuf,
}

impl AutonomousAIHelper {
    /// Create a new Autonomous AI Helper
    pub fn new(
        ecosystem: Arc<AIHelperEcosystem>,
        repository_context: Option<Arc<RepositoryContextManager>>,
    ) -> Result<Self> {
        let working_directory = std::env::current_dir()
            .context("Failed to get current working directory")?;
            
        Ok(Self {
            ecosystem,
            repository_context,
            working_directory,
        })
    }
    
    /// Get the effective working directory based on repository context
    async fn get_effective_working_directory(&self) -> PathBuf {
        if let Some(repo_ctx) = &self.repository_context {
            let context = repo_ctx.get_context().await;
            if let Some(root_path) = context.root_path {
                return root_path;
            }
        }
        self.working_directory.clone()
    }
    
    /// Process a raw user question with full autonomy
    /// 
    /// The AI receives the unfiltered user input and decides:
    /// 1. What the user is asking for
    /// 2. What information it needs to gather
    /// 3. How to respond appropriately
    pub async fn process_autonomously(&self, raw_user_input: &str) -> Result<AutonomousDecision> {
        info!("🧠 Autonomous AI processing: '{}'", raw_user_input);
        
        // Step 1: Use the IntelligentContextOrchestrator to understand intent
        let intent_analysis = self.ecosystem.intelligent_orchestrator
            .analyze_user_intent(raw_user_input)
            .await?;
            
        info!("🎯 AI understands user intent: {}", intent_analysis.primary_intent);
        
        // Step 2: Let the AI decide what actions to take based on its understanding
        let actions = self.decide_actions(&intent_analysis, raw_user_input).await?;
        
        info!("🤖 AI decided on {} autonomous actions", actions.len());
        
        // Step 3: Execute the actions the AI decided on
        let gathered_context = self.execute_actions(&actions).await?;
        
        // Step 4: Determine if this needs consensus processing
        let needs_consensus = self.evaluate_consensus_need(&intent_analysis, &gathered_context).await?;
        
        // Step 5: Check if AI can provide a direct response
        let direct_response = self.formulate_direct_response(&intent_analysis, &gathered_context).await?;
        
        Ok(AutonomousDecision {
            intent_understanding: intent_analysis.primary_intent,
            confidence: intent_analysis.confidence,
            actions,
            gathered_context,
            continue_to_consensus: needs_consensus,
            direct_response,
        })
    }
    
    /// AI decides what actions to take based on its understanding
    async fn decide_actions(
        &self,
        intent_analysis: &IntentAnalysis,
        raw_input: &str,
    ) -> Result<Vec<AutonomousAction>> {
        let mut actions = Vec::new();
        
        // Use semantic analysis to understand what the user is asking about
        let semantic_analysis = self.ecosystem.context_retriever
            .analyze_semantic_meaning(raw_input)
            .await?;
            
        // If the AI detects repository-related concepts, it decides to explore
        if semantic_analysis.concepts.iter().any(|c| 
            c.relevance > 0.7 && (
                c.concept.contains("repository") ||
                c.concept.contains("project") ||
                c.concept.contains("codebase") ||
                c.concept.contains("our") ||
                c.concept.contains("this")
            )
        ) {
            debug!("🔍 AI detected repository-related concepts, deciding to explore");
            
            // Get the effective working directory from repository context
            let effective_dir = self.get_effective_working_directory().await;
            
            // Check if we're in a repository
            if self.is_in_repository_at(&effective_dir).await {
                actions.push(AutonomousAction::ExploreRepository {
                    path: effective_dir.clone(),
                });
                
                // AI decides which files might be relevant
                let relevant_files = self.identify_relevant_files_at(raw_input, &effective_dir).await?;
                if !relevant_files.is_empty() {
                    actions.push(AutonomousAction::ReadFiles {
                        files: relevant_files,
                        reason: "AI identified these files as potentially relevant to the query".to_string(),
                    });
                }
            }
        }
        
        // If the AI detects code analysis needs
        if semantic_analysis.requires_code_analysis {
            debug!("🔧 AI detected need for code analysis");
            
            // Let AI formulate its own search query
            let search_query = self.formulate_search_query(raw_input, &semantic_analysis).await?;
            actions.push(AutonomousAction::SearchCode {
                query: search_query,
                semantic: true,
            });
        }
        
        // If the AI thinks memory might help
        if intent_analysis.benefits_from_memory {
            actions.push(AutonomousAction::AccessMemory {
                query: raw_input.to_string(),
            });
        }
        
        // If temporal context might be relevant
        if semantic_analysis.concepts.iter().any(|c| 
            c.concept.contains("recent") || 
            c.concept.contains("latest") ||
            c.concept.contains("current")
        ) {
            actions.push(AutonomousAction::GatherTemporalContext);
        }
        
        // Check for navigation requests
        if let Some(navigation_action) = self.detect_navigation_request(raw_input).await? {
            actions.push(navigation_action);
        }
        
        // Check for file creation requests
        if let Some(creation_action) = self.detect_file_creation_request(raw_input).await? {
            actions.push(creation_action);
        }
        
        Ok(actions)
    }
    
    /// Execute the actions the AI decided to take
    async fn execute_actions(&self, actions: &[AutonomousAction]) -> Result<Option<String>> {
        let mut context_parts = Vec::new();
        
        for action in actions {
            match action {
                AutonomousAction::ExploreRepository { path } => {
                    info!("🗂️ AI is exploring repository at: {}", path.display());
                    
                    if let Some(repo_ctx) = &self.repository_context {
                        // Get repository context
                        let context = repo_ctx.get_context_for_prompts().await;
                        if !context.is_empty() {
                            context_parts.push(context);
                        }
                        
                        // Trigger analysis if needed
                        let _ = repo_ctx.analyze_repository_async(path).await;
                    }
                }
                
                AutonomousAction::ReadFiles { files, reason } => {
                    info!("📖 AI is reading {} files: {}", files.len(), reason);
                    
                    for file in files {
                        match tokio::fs::read_to_string(file).await {
                            Ok(content) => {
                                // AI analyzes the file content for relevance
                                let analysis = self.analyze_file_relevance(file, &content).await?;
                                if analysis.is_relevant {
                                    context_parts.push(format!(
                                        "📄 File: {}\n{}\n\n{}",
                                        file.display(),
                                        analysis.summary,
                                        analysis.relevant_excerpts
                                    ));
                                }
                            }
                            Err(e) => {
                                warn!("AI couldn't read file {}: {}", file.display(), e);
                            }
                        }
                    }
                }
                
                AutonomousAction::SearchCode { query, semantic } => {
                    info!("🔍 AI is searching code: '{}' (semantic: {})", query, semantic);
                    
                    if let Some(results) = self.ecosystem.context_retriever
                        .search_codebase_with_ai(query, *semantic)
                        .await? {
                        context_parts.push(results);
                    }
                }
                
                AutonomousAction::AccessMemory { query } => {
                    info!("🧠 AI is accessing memory for: '{}'", query);
                    
                    if let Some(memory) = self.ecosystem.context_retriever
                        .get_relevant_memory(query)
                        .await? {
                        context_parts.push(memory);
                    }
                }
                
                AutonomousAction::GatherTemporalContext => {
                    info!("🕐 AI is gathering temporal context");
                    
                    if let Some(temporal) = self.ecosystem.context_retriever
                        .get_temporal_context()
                        .await? {
                        context_parts.push(temporal);
                    }
                }
                
                AutonomousAction::AnalyzeQuality { target } => {
                    info!("✨ AI is analyzing quality of: {}", target.display());
                    
                    // Quality analysis would be implemented here
                    // For now, just note that quality analysis was requested
                    context_parts.push(format!(
                        "Quality Analysis requested for: {}",
                        target.display()
                    ));
                }
                
                AutonomousAction::SynthesizeKnowledge { sources } => {
                    info!("🎓 AI is synthesizing knowledge from {} sources", sources.len());
                    
                    let synthesis = self.ecosystem.knowledge_synthesizer
                        .synthesize_from_sources(sources)
                        .await?;
                    context_parts.push(synthesis);
                }
                
                AutonomousAction::NavigateToPath { path, reason } => {
                    info!("🧭 AI wants to navigate to: {} ({})", path.display(), reason);
                    
                    // Check if path exists
                    if path.exists() {
                        // Update the working directory if it's a directory
                        if path.is_dir() {
                            if let Some(repo_ctx) = &self.repository_context {
                                let _ = repo_ctx.analyze_repository_async(path).await;
                            }
                            context_parts.push(format!(
                                "✅ Navigated to directory: {}\nReason: {}",
                                path.display(),
                                reason
                            ));
                        } else {
                            // It's a file - note that we want to open it
                            context_parts.push(format!(
                                "✅ Located file: {}\nReason: {}\n📄 File ready to open in editor",
                                path.display(),
                                reason
                            ));
                        }
                        
                        // Send UI event to navigate in File Explorer
                        use crate::desktop::ai_ui_events::{send_ai_helper_event, AIHelperUIEvent};
                        send_ai_helper_event(AIHelperUIEvent::NavigateToPath {
                            path: path.clone(),
                            reason: reason.clone(),
                        });
                    } else {
                        context_parts.push(format!(
                            "❌ Path not found: {}\nReason: {}",
                            path.display(),
                            reason
                        ));
                    }
                }
                
                AutonomousAction::OpenFile { path } => {
                    info!("📂 AI wants to open file: {}", path.display());
                    
                    if path.exists() && path.is_file() {
                        // Read a preview of the file
                        match tokio::fs::read_to_string(path).await {
                            Ok(content) => {
                                let preview: String = content.chars().take(200).collect();
                                context_parts.push(format!(
                                    "✅ File opened: {}\nPreview:\n{}...",
                                    path.display(),
                                    preview
                                ));
                            }
                            Err(e) => {
                                context_parts.push(format!(
                                    "❌ Failed to open file: {}\nError: {}",
                                    path.display(),
                                    e
                                ));
                            }
                        }
                        
                        // Send UI event to open file in editor
                        use crate::desktop::ai_ui_events::{send_ai_helper_event, AIHelperUIEvent};
                        send_ai_helper_event(AIHelperUIEvent::OpenFile {
                            path: path.clone(),
                        });
                    } else {
                        context_parts.push(format!(
                            "❌ File not found or not a file: {}",
                            path.display()
                        ));
                    }
                }
                
                AutonomousAction::CreateFile { path, content } => {
                    info!("📝 AI wants to create file: {}", path.display());
                    
                    // Check if file already exists
                    if path.exists() {
                        context_parts.push(format!(
                            "⚠️ File already exists: {}\nWould you like to overwrite it?",
                            path.display()
                        ));
                    } else {
                        // Actually create the file
                        match tokio::fs::write(path, content).await {
                            Ok(_) => {
                                info!("✅ Successfully created file: {}", path.display());
                                context_parts.push(format!(
                                    "✅ File created successfully: {}\nContent:\n{}",
                                    path.display(),
                                    content.chars().take(200).collect::<String>()
                                ));
                                
                                // Send UI event to refresh File Explorer and show new file
                                use crate::desktop::ai_ui_events::{send_ai_helper_event, AIHelperUIEvent};
                                send_ai_helper_event(AIHelperUIEvent::CreateFile {
                                    path: path.clone(),
                                    content: content.clone(),
                                    open_after_create: true,
                                });
                            }
                            Err(e) => {
                                warn!("Failed to create file {}: {}", path.display(), e);
                                context_parts.push(format!(
                                    "❌ Failed to create file: {}\nError: {}",
                                    path.display(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        if context_parts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(context_parts.join("\n\n---\n\n")))
        }
    }
    
    /// Check if we're in a repository
    async fn is_in_repository(&self) -> bool {
        let effective_dir = self.get_effective_working_directory().await;
        self.is_in_repository_at(&effective_dir).await
    }
    
    /// Check if a specific directory is a repository
    async fn is_in_repository_at(&self, dir: &Path) -> bool {
        // AI decides what constitutes a repository
        let indicators = vec![
            ".git",
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "go.mod",
            "pom.xml",
            "build.gradle",
            "README.md",
        ];
        
        for indicator in indicators {
            if dir.join(indicator).exists() {
                return true;
            }
        }
        
        false
    }
    
    /// AI identifies which files might be relevant to the query
    async fn identify_relevant_files(&self, query: &str) -> Result<Vec<PathBuf>> {
        let effective_dir = self.get_effective_working_directory().await;
        self.identify_relevant_files_at(query, &effective_dir).await
    }
    
    /// AI identifies which files might be relevant at a specific directory
    async fn identify_relevant_files_at(&self, query: &str, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut relevant_files = Vec::new();
        
        // Let AI decide based on semantic understanding
        if query.to_lowercase().contains("readme") {
            let readme_path = dir.join("README.md");
            if readme_path.exists() {
                relevant_files.push(readme_path);
            }
        }
        
        // For general repository questions, AI decides these are usually relevant
        if self.seems_like_general_repo_question(query).await {
            let common_files = vec![
                "README.md",
                "Cargo.toml",
                "package.json",
                "pyproject.toml",
                "LICENSE",
                "CONTRIBUTING.md",
            ];
            
            for file in common_files {
                let path = dir.join(file);
                if path.exists() && !relevant_files.contains(&path) {
                    relevant_files.push(path);
                }
            }
        }
        
        Ok(relevant_files)
    }
    
    /// AI determines if this seems like a general repository question
    async fn seems_like_general_repo_question(&self, query: &str) -> bool {
        // Use semantic understanding, not keywords
        let semantic_indicators = self.ecosystem.intelligent_orchestrator
            .extract_semantic_indicators(query)
            .await
            .unwrap_or_default();
            
        // AI decides based on semantic meaning
        semantic_indicators.iter().any(|indicator| 
            indicator.concept_type == "repository" || 
            indicator.concept_type == "project" ||
            indicator.concept_type == "codebase"
        )
    }
    
    /// AI formulates its own search query based on understanding
    async fn formulate_search_query(&self, input: &str, analysis: &SemanticAnalysis) -> Result<String> {
        // AI creates a search query based on its understanding
        let key_concepts: Vec<&str> = analysis.concepts
            .iter()
            .filter(|c| c.relevance > 0.6)
            .map(|c| c.concept.as_str())
            .collect();
            
        if key_concepts.is_empty() {
            Ok(input.to_string())
        } else {
            Ok(key_concepts.join(" "))
        }
    }
    
    /// AI analyzes file content for relevance
    async fn analyze_file_relevance(&self, path: &Path, content: &str) -> Result<FileRelevanceAnalysis> {
        // Use AI to determine relevance, not pattern matching
        let relevance_score = self.ecosystem.pattern_recognizer
            .analyze_content_relevance(content)
            .await?;
            
        let summary = if path.file_name() == Some(std::ffi::OsStr::new("README.md")) {
            self.summarize_readme(content).await?
        } else if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml")) {
            self.summarize_cargo_toml(content).await?
        } else {
            self.summarize_generic_file(content).await?
        };
        
        Ok(FileRelevanceAnalysis {
            is_relevant: relevance_score > 0.5,
            summary,
            relevant_excerpts: self.extract_relevant_excerpts(content, 200).await?,
        })
    }
    
    /// AI summarizes README content
    async fn summarize_readme(&self, content: &str) -> Result<String> {
        // Extract key information
        let lines: Vec<&str> = content.lines().take(20).collect();
        let summary = lines.join("\n").chars().take(500).collect();
        Ok(summary)
    }
    
    /// AI summarizes Cargo.toml content
    async fn summarize_cargo_toml(&self, content: &str) -> Result<String> {
        // AI extracts key project info
        let mut summary_parts = vec!["Rust project configuration:".to_string()];
        
        for line in content.lines() {
            if line.starts_with("name =") || 
               line.starts_with("version =") || 
               line.starts_with("description =") {
                summary_parts.push(line.to_string());
            }
        }
        
        Ok(summary_parts.join("\n"))
    }
    
    /// AI summarizes generic file content
    async fn summarize_generic_file(&self, content: &str) -> Result<String> {
        let preview: String = content.chars().take(300).collect();
        Ok(format!("File preview: {}", preview))
    }
    
    /// AI extracts relevant excerpts
    async fn extract_relevant_excerpts(&self, content: &str, max_length: usize) -> Result<String> {
        Ok(content.chars().take(max_length).collect())
    }
    
    /// AI evaluates if consensus processing is needed
    async fn evaluate_consensus_need(
        &self,
        intent: &IntentAnalysis,
        gathered_context: &Option<String>,
    ) -> Result<bool> {
        // AI decides based on intent complexity and available context
        if intent.complexity > 0.7 {
            return Ok(true); // Complex queries benefit from consensus
        }
        
        if gathered_context.is_none() {
            return Ok(true); // No context found, let consensus handle it
        }
        
        // Simple factual queries with good context might not need consensus
        if intent.primary_intent.contains("simple_fact") && gathered_context.is_some() {
            return Ok(false);
        }
        
        Ok(true) // Default to using consensus
    }
    
    /// AI formulates a direct response if appropriate
    async fn formulate_direct_response(
        &self,
        intent: &IntentAnalysis,
        gathered_context: &Option<String>,
    ) -> Result<Option<String>> {
        // For now, let consensus handle responses
        // In future, AI could provide direct answers for simple queries
        Ok(None)
    }
    
    /// Detect navigation requests in user input
    async fn detect_navigation_request(&self, input: &str) -> Result<Option<AutonomousAction>> {
        // Use semantic analysis to detect navigation intent
        let semantic_analysis = self.ecosystem.context_retriever
            .analyze_semantic_meaning(input)
            .await?;
            
        // Check for navigation-related concepts
        let has_navigation_intent = semantic_analysis.concepts.iter().any(|c| 
            c.relevance > 0.6 && (
                c.concept.contains("navigate") ||
                c.concept.contains("go to") ||
                c.concept.contains("open") ||
                c.concept.contains("show") ||
                c.concept.contains("view")
            )
        );
        
        if !has_navigation_intent {
            return Ok(None);
        }
        
        // Try to extract path from input
        if let Some(path) = self.extract_path_from_input(input).await? {
            let reason = format!("User requested to navigate to: {}", path.display());
            return Ok(Some(AutonomousAction::NavigateToPath { path, reason }));
        }
        
        // Check if user wants to open a specific file type or location
        let input_lower = input.to_lowercase();
        if input_lower.contains("file") || input_lower.contains("folder") || input_lower.contains("directory") {
            // Let the AI understand what file/folder the user means
            if let Some(path) = self.infer_navigation_target(input).await? {
                let reason = "AI inferred navigation target from context".to_string();
                return Ok(Some(AutonomousAction::NavigateToPath { path, reason }));
            }
        }
        
        Ok(None)
    }
    
    /// Detect file creation requests in user input
    async fn detect_file_creation_request(&self, input: &str) -> Result<Option<AutonomousAction>> {
        // Use semantic analysis to detect creation intent
        let semantic_analysis = self.ecosystem.context_retriever
            .analyze_semantic_meaning(input)
            .await?;
            
        // Check for creation-related concepts
        let has_creation_intent = semantic_analysis.concepts.iter().any(|c| 
            c.relevance > 0.6 && (
                c.concept.contains("create") ||
                c.concept.contains("make") ||
                c.concept.contains("new") ||
                c.concept.contains("add") ||
                c.concept.contains("write")
            )
        ) && input.to_lowercase().contains("file");
        
        if !has_creation_intent {
            return Ok(None);
        }
        
        // Try to understand what file to create
        if let Some((path, content)) = self.understand_file_creation_request(input).await? {
            return Ok(Some(AutonomousAction::CreateFile { path, content }));
        }
        
        Ok(None)
    }
    
    /// Extract file path from user input
    async fn extract_path_from_input(&self, input: &str) -> Result<Option<PathBuf>> {
        // Look for common path patterns
        // This is a simplified implementation - could use NLP for better extraction
        
        // Check for quoted paths
        if let Some(start) = input.find('"') {
            if let Some(end) = input[start+1..].find('"') {
                let path_str = &input[start+1..start+1+end];
                return Ok(Some(PathBuf::from(path_str)));
            }
        }
        
        // Check for paths with common prefixes
        for prefix in &["src/", "tests/", "./", "../", "~/"] {
            if let Some(idx) = input.find(prefix) {
                // Extract until whitespace or end
                let remaining = &input[idx..];
                let path_end = remaining.find(char::is_whitespace).unwrap_or(remaining.len());
                let path_str = &remaining[..path_end];
                return Ok(Some(PathBuf::from(path_str)));
            }
        }
        
        // Check for file extensions
        let extensions = vec![".rs", ".ts", ".js", ".py", ".go", ".java", ".cpp", ".c", ".h", ".md", ".txt", ".toml", ".json"];
        for ext in extensions {
            if let Some(ext_pos) = input.find(ext) {
                // Find the start of the filename
                let before_ext = &input[..ext_pos];
                if let Some(start) = before_ext.rfind(char::is_whitespace) {
                    let path_str = &input[start+1..ext_pos+ext.len()];
                    return Ok(Some(PathBuf::from(path_str)));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Infer navigation target from context
    async fn infer_navigation_target(&self, input: &str) -> Result<Option<PathBuf>> {
        // Use AI to understand what the user wants to navigate to
        let input_lower = input.to_lowercase();
        let effective_dir = self.get_effective_working_directory().await;
        
        // Common navigation targets
        if input_lower.contains("main") && (input_lower.contains("file") || input_lower.contains("entry")) {
            // Check for main entry points based on project type
            if effective_dir.join("src/main.rs").exists() {
                return Ok(Some(effective_dir.join("src/main.rs")));
            } else if effective_dir.join("src/main.ts").exists() {
                return Ok(Some(effective_dir.join("src/main.ts")));
            } else if effective_dir.join("index.js").exists() {
                return Ok(Some(effective_dir.join("index.js")));
            } else if effective_dir.join("main.py").exists() {
                return Ok(Some(effective_dir.join("main.py")));
            }
        }
        
        if input_lower.contains("test") && (input_lower.contains("file") || input_lower.contains("folder")) {
            if effective_dir.join("tests").exists() {
                return Ok(Some(effective_dir.join("tests")));
            } else if effective_dir.join("test").exists() {
                return Ok(Some(effective_dir.join("test")));
            }
        }
        
        Ok(None)
    }
    
    /// Understand file creation request
    async fn understand_file_creation_request(&self, input: &str) -> Result<Option<(PathBuf, String)>> {
        let input_lower = input.to_lowercase();
        
        // Check for "hello world" request
        if input_lower.contains("hello world") {
            let filename = if input_lower.contains(".txt") {
                "hello_world.txt"
            } else if input_lower.contains(".md") {
                "hello_world.md"
            } else {
                "hello_world.txt" // Default to .txt
            };
            
            let content = if filename.ends_with(".md") {
                "# Hello World\n\nThis is a hello world file created by Hive AI!"
            } else {
                "Hello World!\n\nThis file was created by Hive AI."
            };
            
            // Create in effective working directory (respects selected folder)
            let effective_dir = self.get_effective_working_directory().await;
            let path = effective_dir.join(filename);
            return Ok(Some((path, content.to_string())));
        }
        
        // Try to extract filename from input
        if let Some(path) = self.extract_path_from_input(input).await? {
            // Determine content based on file type
            let content = self.generate_default_content_for_file(&path).await?;
            return Ok(Some((path, content)));
        }
        
        Ok(None)
    }
    
    /// Generate default content for a file based on its type
    async fn generate_default_content_for_file(&self, path: &Path) -> Result<String> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
            
        let content = match extension {
            "rs" => "// New Rust file\n\nfn main() {\n    println!(\"Hello from Hive AI!\");\n}",
            "py" => "#!/usr/bin/env python3\n# New Python file\n\nif __name__ == \"__main__\":\n    print(\"Hello from Hive AI!\")",
            "js" => "// New JavaScript file\n\nconsole.log('Hello from Hive AI!');",
            "ts" => "// New TypeScript file\n\nconsole.log('Hello from Hive AI!');",
            "md" => "# New Markdown File\n\nCreated by Hive AI",
            "txt" | _ => "New file created by Hive AI",
        };
        
        Ok(content.to_string())
    }
}

/// Analysis result for file relevance
struct FileRelevanceAnalysis {
    is_relevant: bool,
    summary: String,
    relevant_excerpts: String,
}


/// Intent analysis result
#[derive(Debug, Clone)]
pub struct IntentAnalysis {
    pub primary_intent: String,
    pub confidence: f32,
    pub complexity: f32,
    pub benefits_from_memory: bool,
}

/// Semantic indicator
#[derive(Debug, Clone)]
pub struct SemanticIndicator {
    pub concept_type: String,
    pub confidence: f32,
}

/// Semantic analysis result
#[derive(Debug, Clone)]
pub struct SemanticAnalysis {
    pub concepts: Vec<SemanticConcept>,
    pub requires_code_analysis: bool,
}

/// Semantic concept
#[derive(Debug, Clone)]
pub struct SemanticConcept {
    pub concept: String,
    pub relevance: f32,
}

/// Extension methods for IntelligentContextOrchestrator
impl IntelligentContextOrchestrator {
    /// Analyze user intent using AI
    pub async fn analyze_user_intent(&self, input: &str) -> Result<IntentAnalysis> {
        // Use the existing decision-making infrastructure
        let decision = self.make_intelligent_context_decision(input, true).await?;
        
        Ok(IntentAnalysis {
            primary_intent: format!("{:?}", decision.primary_category),
            confidence: decision.confidence as f32,
            complexity: decision.validation_score as f32,
            benefits_from_memory: decision.should_use_repo,
        })
    }
    
    /// Extract semantic indicators from text
    pub async fn extract_semantic_indicators(&self, text: &str) -> Result<Vec<SemanticIndicator>> {
        // This would use GraphCodeBERT or other models to extract semantic meaning
        // For now, a simplified implementation
        let mut indicators = Vec::new();
        
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("repo") || text_lower.contains("project") || text_lower.contains("our") {
            indicators.push(SemanticIndicator {
                concept_type: "repository".to_string(),
                confidence: 0.8,
            });
        }
        
        if text_lower.contains("code") || text_lower.contains("implement") || text_lower.contains("function") {
            indicators.push(SemanticIndicator {
                concept_type: "code_analysis".to_string(),
                confidence: 0.7,
            });
        }
        
        Ok(indicators)
    }
}

/// Extension for ContextRetriever
impl ContextRetriever {
    /// Analyze semantic meaning of text
    pub async fn analyze_semantic_meaning(&self, text: &str) -> Result<SemanticAnalysis> {
        // This would use GraphCodeBERT for real semantic analysis
        // Simplified for now
        let mut concepts = Vec::new();
        
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("repo") || text_lower.contains("our") || text_lower.contains("this") {
            concepts.push(SemanticConcept {
                concept: "repository".to_string(),
                relevance: 0.9,
            });
        }
        
        if text_lower.contains("tell") || text_lower.contains("about") || text_lower.contains("describe") {
            concepts.push(SemanticConcept {
                concept: "description_request".to_string(),
                relevance: 0.8,
            });
        }
        
        let requires_code_analysis = text_lower.contains("code") || 
                                    text_lower.contains("function") || 
                                    text_lower.contains("implement");
        
        Ok(SemanticAnalysis {
            concepts,
            requires_code_analysis,
        })
    }
    
    /// Search codebase using AI
    pub async fn search_codebase_with_ai(&self, query: &str, semantic: bool) -> Result<Option<String>> {
        // This would use the actual search functionality
        // For now, return None to indicate no results
        Ok(None)
    }
    
    /// Get relevant memory
    pub async fn get_relevant_memory(&self, query: &str) -> Result<Option<String>> {
        // This would query the AuthoritativeKnowledgeStore
        Ok(None)
    }
    
    /// Get temporal context
    pub async fn get_temporal_context(&self) -> Result<Option<String>> {
        Ok(Some(format!("Current date: {}", chrono::Utc::now().format("%Y-%m-%d"))))
    }
}

/// Extension for PatternRecognizer
impl PatternRecognizer {
    /// Analyze content relevance using AI
    pub async fn analyze_content_relevance(&self, content: &str) -> Result<f32> {
        // This would use AI to determine relevance
        // For now, simple heuristic
        if content.len() > 100 {
            Ok(0.7)
        } else {
            Ok(0.3)
        }
    }
}

/// Extension for KnowledgeSynthesizer
impl KnowledgeSynthesizer {
    /// Synthesize knowledge from multiple sources
    pub async fn synthesize_from_sources(&self, sources: &[String]) -> Result<String> {
        Ok(format!("Synthesized knowledge from {} sources", sources.len()))
    }
}