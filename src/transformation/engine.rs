//! Main transformation engine that orchestrates AI-powered code improvements

use anyhow::{anyhow, Result};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::{
    applier::CodeApplier, conflict::ConflictResolver, history::TransformationHistory,
    preview::PreviewSystem, syntax::SyntaxAwareModifier, types::*,
};
use crate::analysis::{SymbolIndexer, TreeSitterParser};
use crate::consensus::ConsensusEngine;
use crate::core::{context::ContextBuilder, security::check_read_access, Language};

/// Main transformation engine that coordinates all code transformation operations
pub struct TransformationEngine {
    context: Arc<ContextBuilder>,
    consensus_engine: Arc<ConsensusEngine>,
    parser: Arc<Mutex<TreeSitterParser>>,
    symbol_index: Arc<Mutex<SymbolIndexer>>,
    syntax_modifier: SyntaxAwareModifier,
    conflict_resolver: ConflictResolver,
    preview_system: PreviewSystem,
    history: Arc<Mutex<TransformationHistory>>,
    applier: CodeApplier,
    config: TransformConfig,
}

impl TransformationEngine {
    /// Create a new transformation engine
    pub async fn new(context: Arc<ContextBuilder>) -> Result<Self> {
        let consensus_engine = Arc::new(ConsensusEngine::new(None).await?);
        let parser = Arc::new(Mutex::new(TreeSitterParser::new(Language::Rust)?));

        // Get database manager from context
        let db_manager = Arc::new(crate::core::database::DatabaseManager::default().await?);
        let symbol_index = Arc::new(Mutex::new(SymbolIndexer::new(db_manager).await?));

        // Get configuration directory from context
        let config_dir = dirs::config_dir()
            .map(|d| d.join("hive"))
            .unwrap_or_else(|| std::path::PathBuf::from(".hive"));

        let history = Arc::new(Mutex::new(TransformationHistory::new(&config_dir)?));

        Ok(Self {
            context: context.clone(),
            consensus_engine,
            parser: parser.clone(),
            symbol_index,
            syntax_modifier: SyntaxAwareModifier::new(parser.clone()),
            conflict_resolver: ConflictResolver::new(),
            preview_system: PreviewSystem::new(),
            history: history.clone(),
            applier: CodeApplier::new(history, parser.clone()),
            config: TransformConfig::default(),
        })
    }

    /// Transform code based on the given request
    pub async fn transform(&self, request: TransformationRequest) -> Result<TransformationPreview> {
        // Check file access permissions
        check_read_access(&request.file_path)?;

        // Read the current file content
        let content = tokio::fs::read_to_string(&request.file_path).await?;

        // Parse the file to understand its structure
        let mut parser = self.parser.lock().await;
        let ast = parser.parse(&content)?;

        // Update symbol index
        let mut symbol_index = self.symbol_index.lock().await;
        symbol_index
            .index_file(&request.file_path, &content)
            .await?;
        drop(symbol_index);

        // Generate improvement suggestions using consensus engine
        let suggestions = self.generate_suggestions(&request, &content).await?;

        // Convert suggestions to concrete changes
        let changes = self.create_changes(&request, &content, suggestions).await?;

        // Check for conflicts
        let conflicts = self.conflict_resolver.check_conflicts(&changes)?;
        if !conflicts.is_empty() {
            return Err(anyhow!("Conflicts detected: {:?}", conflicts));
        }

        // Create transformation object
        let transformation = Transformation {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            request: request.clone(),
            changes,
            description: format!(
                "Improve {} aspect of {}",
                request.aspect,
                request.file_path.display()
            ),
            applied: false,
            transaction_id: None,
            confidence: 0.8,
            impact_score: 0.5,
            tags: vec![request.aspect.clone()],
        };

        // Generate preview
        let preview = self
            .preview_system
            .generate_preview(&transformation)
            .await?;

        Ok(preview)
    }

    /// Apply a transformation that was previously previewed
    pub async fn apply_transformation(&self, transformation_id: &str) -> Result<()> {
        let mut history = self.history.lock().await;
        let transformation = history
            .get_transformation(transformation_id)
            .await
            .ok_or_else(|| anyhow!("Transformation not found"))?;

        if transformation.applied {
            return Err(anyhow!("Transformation already applied"));
        }

        // Apply the transformation
        let transaction_id = self.applier.apply(transformation.clone()).await?;

        // Update transformation status
        history
            .mark_applied(transformation_id, &transaction_id)
            .await?;

        Ok(())
    }

    /// Undo the last applied transformation
    pub async fn undo(&self) -> Result<()> {
        let mut history = self.history.lock().await;
        let last_transaction = history.get_last_transaction().await?;

        self.applier.undo(&last_transaction.id).await?;
        history.mark_undone(&last_transaction.id).await?;

        Ok(())
    }

    /// Redo the last undone transformation
    pub async fn redo(&self) -> Result<()> {
        let mut history = self.history.lock().await;
        let last_undone = history.get_last_undone().await?;

        self.applier.redo(&last_undone.id).await?;
        history.mark_redone(&last_undone.id).await?;

        Ok(())
    }

    /// Generate AI suggestions for code improvement
    async fn generate_suggestions(
        &self,
        request: &TransformationRequest,
        content: &str,
    ) -> Result<Vec<String>> {
        let prompt = self.build_improvement_prompt(request, content)?;

        let response = self.consensus_engine.process(&prompt, None).await?;

        // Parse suggestions from the response
        self.parse_suggestions(&response.result.unwrap_or_default())
    }

    /// Build the prompt for the consensus engine
    fn build_improvement_prompt(
        &self,
        request: &TransformationRequest,
        content: &str,
    ) -> Result<String> {
        let prompt = format!(
            r#"You are an expert code reviewer. Analyze the following code and suggest improvements
focused on the "{}" aspect.

File: {}

Code:
```
{}
```

Please provide specific, actionable improvements. For each suggestion:
1. Identify the exact lines to change
2. Provide the improved code
3. Explain why this change improves the {} aspect
4. Rate your confidence (0.0 to 1.0)

Format your response as a numbered list of improvements."#,
            request.aspect,
            request.file_path.display(),
            content,
            request.aspect
        );

        if let Some(context) = &request.context {
            Ok(format!("{}\n\nAdditional context: {}", prompt, context))
        } else {
            Ok(prompt)
        }
    }

    /// Parse suggestions from the AI response
    fn parse_suggestions(&self, response: &str) -> Result<Vec<String>> {
        // Simple parsing - in production this would be more sophisticated
        let suggestions: Vec<String> = response
            .lines()
            .filter(|line| line.trim().starts_with(|c: char| c.is_numeric()))
            .map(|line| line.to_string())
            .collect();

        if suggestions.is_empty() {
            return Err(anyhow!("No suggestions found in response"));
        }

        Ok(suggestions)
    }

    /// Convert AI suggestions into concrete code changes
    async fn create_changes(
        &self,
        request: &TransformationRequest,
        original_content: &str,
        suggestions: Vec<String>,
    ) -> Result<Vec<CodeChange>> {
        let mut changes = Vec::new();

        for suggestion in suggestions
            .iter()
            .take(self.config.max_changes_per_transform)
        {
            // This is simplified - in production we'd parse the suggestion format more carefully
            if let Some(change) = self
                .parse_single_change(request, original_content, suggestion)
                .await?
            {
                if change.confidence >= self.config.min_confidence {
                    changes.push(change);
                }
            }
        }

        Ok(changes)
    }

    /// Parse a single suggestion into a CodeChange
    async fn parse_single_change(
        &self,
        request: &TransformationRequest,
        original_content: &str,
        suggestion: &str,
    ) -> Result<Option<CodeChange>> {
        // This is a simplified implementation
        // In production, we'd use more sophisticated parsing

        // For now, create a mock change
        Ok(Some(CodeChange {
            file_path: request.file_path.clone(),
            original_content: original_content.to_string(),
            new_content: original_content.to_string(), // Would be modified based on suggestion
            line_range: (1, 10),                       // Would be parsed from suggestion
            description: suggestion.to_string(),
            confidence: 0.85, // Would be parsed from suggestion
        }))
    }
}

/// Public convenience function for transforming code
pub async fn transform_code(
    context: Arc<ContextBuilder>,
    request: TransformationRequest,
) -> Result<TransformationPreview> {
    let engine = TransformationEngine::new(context).await?;
    engine.transform(request).await
}
