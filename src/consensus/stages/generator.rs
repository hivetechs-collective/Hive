// Generator Stage - Initial response generation with context injection
// First stage of the consensus pipeline

use crate::consensus::stages::ConsensusStage;
use crate::consensus::types::{Message, Stage, StagePrompts};
use crate::core::context::QueryContext;
use anyhow::Result;

pub struct GeneratorStage;

impl ConsensusStage for GeneratorStage {
    fn stage(&self) -> Stage {
        Stage::Generator
    }

    fn system_prompt(&self) -> &'static str {
        StagePrompts::generator_system()
    }

    fn build_messages(
        &self,
        question: &str,
        _previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>> {
        let mut messages = vec![Message {
            role: "system".to_string(),
            content: self
                .enhance_system_prompt_with_context(question)
                .to_string(),
        }];

        // Add rich context if provided (semantic, temporal, or repository)
        if let Some(ctx) = context {
            let enhanced_context = self.structure_context(ctx, question);
            messages.push(Message {
                role: "system".to_string(),
                content: enhanced_context,
            });
        }
        
        // Add the user's question with enhanced formatting
        messages.push(Message {
            role: "user".to_string(),
            content: self.format_user_question(question),
        });

        Ok(messages)
    }
}

impl GeneratorStage {
    pub fn new() -> Self {
        Self
    }

    /// Detect the scope of the question
    pub fn detect_question_scope(&self, question: &str) -> &'static str {
        let question_lower = question.to_lowercase();
        let word_count = question.split_whitespace().count();

        // Basic questions (minimal scope)
        if word_count <= 5
            || question_lower.contains("what is")
            || question_lower.contains("how to")
        {
            "minimal"
        }
        // Regular questions (basic scope)
        else if word_count <= 15 && !self.needs_repository_context(question) {
            "basic"
        }
        // Complex questions (production scope)
        else {
            "production"
        }
    }

    /// Enhance system prompt with question-specific context
    pub fn enhance_system_prompt_with_context(&self, question: &str) -> String {
        let base_prompt = StagePrompts::generator_system();
        let scope = self.detect_question_scope(question);

        let mut enhanced = format!("{}", base_prompt);

        // Add scope-specific instructions
        match scope {
            "minimal" => {
                enhanced.push_str("\n\nQUICK ANSWER MODE: Provide a concise, direct answer.")
            }
            "basic" => {
                enhanced.push_str("\n\nSTANDARD MODE: Provide a clear, well-structured answer.")
            }
            "production" => enhanced.push_str(
                "\n\nCOMPREHENSIVE MODE: Provide an in-depth analysis with multiple perspectives.",
            ),
            _ => {}
        }

        if self.needs_repository_context(question) {
            enhanced.push_str("\n\nCODE ANALYSIS MODE: This question appears to be about code or repository content. Use any provided repository context (symbols, dependencies, file structure) to give accurate, specific answers. Reference actual code patterns and project structure when available.");
        }

        if self.needs_temporal_context(question) {
            enhanced.push_str("\n\nTEMPORAL AWARENESS MODE: This question requires current information. Use any provided temporal context (current date, market hours, recent events) to give up-to-date answers. Prioritize recent information and clearly indicate currency of data.");
        }

        enhanced
    }

    /// Structure context based on type and question
    pub fn structure_context(&self, context: &str, question: &str) -> String {
        let mut structured = String::new();

        // Parse context type based on content
        if context.contains("## Memory Context") || context.contains("## Recent Context") {
            structured.push_str("🧠 AUTHORITATIVE MEMORY CONTEXT:\n");
            structured.push_str(context);
            structured.push_str("\n\n⚡ IMPORTANT: The above memory context contains AUTHORITATIVE ANSWERS from previous conversations. ");
            structured.push_str("These are curated, validated responses that should be treated as the primary source of truth. ");
            structured.push_str("Use this information as the foundation for your response, building upon it rather than contradicting it.\n");
        } else if context.contains("IMPORTANT: Today's date is") {
            structured.push_str("🕒 TEMPORAL CONTEXT:\n");
            structured.push_str(context);
        } else if context.contains("symbols:") || context.contains("dependencies:") || context.contains("Repository Path:") {
            structured.push_str("🧠 REPOSITORY CONTEXT:\n");
            structured.push_str(&self.format_repository_context(context));
        } else if context.contains("File-Based Repository Analysis") {
            structured.push_str("📁 FILE-BASED REPOSITORY ANALYSIS:\n");
            structured.push_str(context);
            structured.push_str("\n\n⚠️ CRITICAL: The above contains ACTUAL FILE CONTENTS from the repository. ");
            structured.push_str("You MUST use these real code snippets in your response. ");
            structured.push_str("DO NOT make up generic examples - use the provided code!\n");
        } else {
            structured.push_str("📋 ADDITIONAL CONTEXT:\n");
            structured.push_str(context);
        }

        // Add context usage instructions
        structured.push_str("\n\n📌 CONTEXT USAGE:\n");
        if context.contains("## Memory Context") || context.contains("## Recent Context") {
            structured.push_str(
                "- ALWAYS prioritize the authoritative memory context over general knowledge\n",
            );
            structured
                .push_str("- Build upon previous answers rather than starting from scratch\n");
            structured.push_str(
                "- Reference specific details from the memory context in your response\n",
            );
        }
        if self.needs_repository_context(question) {
            structured.push_str(
                "- Reference specific symbols, files, and patterns from the repository context\n",
            );
            structured.push_str(
                "- Use architecture and dependency information to provide accurate suggestions\n",
            );
        }
        if self.needs_temporal_context(question) {
            structured
                .push_str("- Use current date information to prioritize recent developments\n");
            structured.push_str(
                "- Include temporal awareness in web searches and information retrieval\n",
            );
        }

        structured
    }

    /// Format repository context for better readability
    fn format_repository_context(&self, context: &str) -> String {
        let mut formatted = String::new();

        for line in context.lines() {
            if line.starts_with("File:") {
                formatted.push_str(&format!("📄 {}\n", line));
            } else if line.starts_with("Symbol:") {
                formatted.push_str(&format!("🔧 {}\n", line));
            } else if line.starts_with("Dependency:") {
                formatted.push_str(&format!("🔗 {}\n", line));
            } else {
                formatted.push_str(&format!("{}\n", line));
            }
        }

        formatted
    }

    /// Format user question with better structure
    pub fn format_user_question(&self, question: &str) -> String {
        let scope = self.detect_question_scope(question);
        format!("USER QUESTION (Scope: {}):\n{}", scope, question)
    }

    /// Enhance prompt with semantic context
    pub fn enhance_with_context(&self, question: &str, context: &str) -> String {
        format!(
            "Context Information:\n{}\n\nUser Question: {}",
            context, question
        )
    }

    /// Check if question benefits from repository context
    pub fn needs_repository_context(&self, question: &str) -> bool {
        let code_indicators = [
            "this code",
            "this function",
            "this file",
            "this class",
            "this method",
            "this module",
            "this repo",
            "this project",
            "the code",
            "the function",
            "analyze",
            "explain",
            "review",
            "debug",
            "optimize",
            "refactor",
            "implementation",
            "architecture",
            "dependency",
            "import",
            "export",
            "structure",
        ];

        let question_lower = question.to_lowercase();
        code_indicators
            .iter()
            .any(|&indicator| question_lower.contains(indicator))
    }

    /// Check if question needs temporal context
    pub fn needs_temporal_context(&self, question: &str) -> bool {
        let temporal_indicators = [
            "latest",
            "recent",
            "current",
            "today",
            "now",
            "this week",
            "this month",
            "2024",
            "2025",
            "what's new",
            "recent updates",
            "current version",
            "search",
            "lookup",
            "find online",
            "web search",
            "news",
            "trends",
            "latest release",
            "just released",
            "recently published",
            "stock price",
            "market",
            "cryptocurrency",
            "bitcoin",
        ];

        let question_lower = question.to_lowercase();
        temporal_indicators
            .iter()
            .any(|&indicator| question_lower.contains(indicator))
    }

    /// Build context-aware prompt based on query context
    pub fn build_contextual_prompt(&self, query_context: Option<&QueryContext>) -> String {
        if let Some(ctx) = query_context {
            let mut prompt = String::new();

            if !ctx.code_snippets.is_empty() {
                prompt.push_str("RELEVANT CODE:\n");
                for (i, snippet) in ctx.code_snippets.iter().take(3).enumerate() {
                    prompt.push_str(&format!(
                        "{}. {} (lines {}-{}):\n```{}\n{}\n```\n\n",
                        i + 1,
                        snippet.file.display(),
                        snippet.start_line,
                        snippet.end_line,
                        snippet.language.as_str(),
                        snippet.content
                    ));
                }
            }

            if !ctx.symbols.is_empty() {
                prompt.push_str("RELEVANT SYMBOLS:\n");
                for symbol in ctx.symbols.iter().take(5) {
                    prompt.push_str(&format!("- {} ({})\n", symbol.name, symbol.kind.as_str()));
                }
                prompt.push('\n');
            }

            if !ctx.file_summaries.is_empty() {
                prompt.push_str("PROJECT STRUCTURE:\n");
                for summary in ctx.file_summaries.iter().take(5) {
                    prompt.push_str(&format!(
                        "- {}: {}\n",
                        summary.path.display(),
                        summary.description
                    ));
                }
                prompt.push('\n');
            }

            prompt
        } else {
            String::new()
        }
    }
}
