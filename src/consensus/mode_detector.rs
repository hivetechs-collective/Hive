//! Smart Mode Detection
//!
//! Automatically determines whether to use direct execution or full consensus
//! based on request complexity, patterns, and context.

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ai_helpers::AIHelperEcosystem;

/// Execution mode for a request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Fast path - use generator only with inline execution
    Direct,
    
    /// Full consensus - use all 4 stages for complex analysis
    Consensus,
    
    /// Hybrid - use consensus but with inline operations
    HybridConsensus,
}

/// Pattern matcher for request classification
#[derive(Debug, Clone)]
pub struct PatternMatcher {
    pattern: regex::Regex,
    mode: ExecutionMode,
    confidence: f32,
    description: String,
}

impl PatternMatcher {
    pub fn new(pattern: &str, mode: ExecutionMode, confidence: f32, description: &str) -> Result<Self> {
        Ok(Self {
            pattern: regex::Regex::new(pattern)?,
            mode,
            confidence,
            description: description.to_string(),
        })
    }

    pub fn matches(&self, text: &str) -> Option<(ExecutionMode, f32)> {
        if self.pattern.is_match(text) {
            Some((self.mode, self.confidence))
        } else {
            None
        }
    }
}

/// Complexity analyzer for requests
pub struct ComplexityAnalyzer {
    word_count_threshold: usize,
    entity_count_threshold: usize,
    nested_clause_threshold: usize,
}

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            word_count_threshold: 50,
            entity_count_threshold: 5,
            nested_clause_threshold: 3,
        }
    }

    /// Analyze request complexity
    pub fn analyze(&self, request: &str) -> f32 {
        let mut complexity = 0.0;
        
        // Word count complexity
        let word_count = request.split_whitespace().count();
        if word_count > self.word_count_threshold {
            complexity += 0.3;
        }
        
        // Check for multiple requirements (and, also, additionally, etc.)
        let multi_requirement_words = ["and", "also", "additionally", "furthermore", "plus"];
        let requirement_count = multi_requirement_words.iter()
            .filter(|&&word| request.to_lowercase().contains(word))
            .count();
        complexity += (requirement_count as f32) * 0.1;
        
        // Check for conditional logic (if, when, unless, etc.)
        let conditional_words = ["if", "when", "unless", "while", "until", "after", "before"];
        let conditional_count = conditional_words.iter()
            .filter(|&&word| request.to_lowercase().split_whitespace().any(|w| w == word))
            .count();
        complexity += (conditional_count as f32) * 0.15;
        
        // Check for analysis keywords - but only for clearly complex analysis requests
        let analysis_words = ["analyze", "debug", "investigate", "review", "audit", "assess"];
        if analysis_words.iter().any(|&word| request.to_lowercase().contains(word)) {
            complexity += 0.4;
        }
        
        // "explain" is often used in simple questions, be less aggressive
        if request.to_lowercase().contains("explain") && request.len() > 50 {
            complexity += 0.2;
        }
        
        // Check for architecture/design keywords
        let design_words = ["architecture", "design", "structure", "pattern", "framework", "system"];
        if design_words.iter().any(|&word| request.to_lowercase().contains(word)) {
            complexity += 0.3;
        }
        
        // Cap complexity at 1.0
        complexity.min(1.0)
    }
}

/// Smart mode detector
pub struct ModeDetector {
    patterns: Vec<PatternMatcher>,
    complexity_analyzer: ComplexityAnalyzer,
    ai_helpers: Option<Arc<AIHelperEcosystem>>,
    complexity_threshold: f32,
    openrouter_client: Option<Arc<crate::consensus::openrouter::OpenRouterClient>>,
    generator_model: Option<String>,
}

impl ModeDetector {
    pub fn new() -> Result<Self> {
        let patterns = vec![
            // Direct mode patterns - simple operations and factual questions
            PatternMatcher::new(
                r"(?i)^(create|make|add|write)\s+(a\s+)?(new\s+)?(simple\s+|empty\s+|basic\s+)?file\s+(called|named)\s+\w+\.(txt|md|json)$",
                ExecutionMode::Direct,
                0.85,
                "Very simple file creation"
            )?,
            PatternMatcher::new(
                r"(?i)^(delete|remove)\s+(the\s+)?file\s+\w+\.(txt|md|json)$",
                ExecutionMode::Direct,
                0.85,
                "Simple file deletion"
            )?,
            PatternMatcher::new(
                r"(?i)^(rename|move)\s+(the\s+)?file\s+\w+\s+to\s+\w+$",
                ExecutionMode::Direct,
                0.85,
                "File rename/move"
            )?,
            
            // Simple factual questions - should get direct answers
            PatternMatcher::new(
                r"(?i)^(what|which)(\s+is)?\s+(the\s+)?(name\s+of\s+the\s+|current\s+)?(repo|repository|project|folder|directory)(\s+name)?(\?)?$",
                ExecutionMode::Direct,
                0.9,
                "Simple repository name question"
            )?,
            PatternMatcher::new(
                r"(?i)^(what|where)(\s+is)?\s+(my\s+|the\s+)?(current\s+)?(directory|folder|location)(\?)?$",
                ExecutionMode::Direct,
                0.9,
                "Simple directory question"
            )?,
            PatternMatcher::new(
                r"(?i)^(list|show|what)\s+(files?|directories|folders?)(\s+(are\s+)?(here|in\s+this\s+(directory|folder)))?(\?)?$",
                ExecutionMode::Direct,
                0.85,
                "Simple file listing question"
            )?,
            PatternMatcher::new(
                r"(?i)^(which|what)\s+(file|folder|directory)\s+(is\s+)?(selected|open|current)(\?)?$",
                ExecutionMode::Direct,
                0.85,
                "Simple file selection question"
            )?,
            PatternMatcher::new(
                r"(?i)(what|what's|whats)\s+(is\s+)?(the\s+)?(name|title)\s+(of\s+)?(this|the\s+current|my\s+current)?\s*(repo|repository|project)",
                ExecutionMode::Direct,
                0.95,
                "Repository name question"
            )?,
            PatternMatcher::new(
                r"(?i)(what|what's|whats)\s+(is\s+)?(this|the\s+current|my\s+current)?\s*(repo|repository|project)(\s+(name|called))?",
                ExecutionMode::Direct,
                0.9,
                "Simple repository question"
            )?,
            PatternMatcher::new(
                r"(?i)^(what|what's|whats)\s+(is\s+)?(\d+)\s*[\+\-\*\/]\s*(\d+)",
                ExecutionMode::Direct,
                0.95,
                "Simple math calculation"
            )?,
            PatternMatcher::new(
                r"(?i)^(calculate|compute|solve)\s+(\d+)\s*[\+\-\*\/]\s*(\d+)",
                ExecutionMode::Direct,
                0.95,
                "Math calculation request"
            )?,
            PatternMatcher::new(
                r"(?i)^(what|what's|whats|who|who's|where|where's|when|how\s+many)\s+.{1,40}$",
                ExecutionMode::Direct,
                0.75,
                "Short factual questions"
            )?,
            PatternMatcher::new(
                r"(?i)^(is|are|can|does|do|did|will|would|should)\s+.{1,40}$",
                ExecutionMode::Direct,
                0.7,
                "Simple yes/no questions"
            )?,
            
            // Consensus mode patterns - favor collaborative analysis for complex/interesting work
            PatternMatcher::new(
                r"(?i)(analyze|explain|investigate|debug|diagnose|understand|explore)",
                ExecutionMode::Consensus,
                0.9,
                "Analysis and exploration requests"
            )?,
            PatternMatcher::new(
                r"(?i)(architecture|design|structure|pattern|best\s+practice|approach|strategy)",
                ExecutionMode::Consensus,
                0.9,
                "Architecture and design requests"
            )?,
            PatternMatcher::new(
                r"(?i)(implement|build|create|write|develop).*(system|framework|library|api|component|feature|function|class|module)",
                ExecutionMode::Consensus,
                0.85,
                "Implementation requests"
            )?,
            PatternMatcher::new(
                r"(?i)(refactor|optimize|improve|enhance|fix|solve|troubleshoot)",
                ExecutionMode::Consensus,
                0.85,
                "Improvement and problem-solving requests"
            )?,
            PatternMatcher::new(
                r"(?i)(compare|evaluate|assess|review|recommend|suggest|advise)",
                ExecutionMode::Consensus,
                0.85,
                "Evaluation and recommendation requests"
            )?,
            PatternMatcher::new(
                r"(?i)(how\s+(to|do|can)\s+.{10,}|why\s+(does|is|should)\s+.{10,}|when\s+(to|should)\s+.{10,})",
                ExecutionMode::Consensus,
                0.8,
                "Complex analytical question patterns"
            )?,
            PatternMatcher::new(
                r"(?i)(help|assist|guide)\s+(me\s+)?(with|to)",
                ExecutionMode::Consensus,
                0.75,
                "Help and guidance requests"
            )?,
            
            // Hybrid patterns - consensus with file operations
            PatternMatcher::new(
                r"(?i)(implement|create|build)\s+.*(feature|component|module)",
                ExecutionMode::HybridConsensus,
                0.8,
                "Feature implementation"
            )?,
            PatternMatcher::new(
                r"(?i)fix\s+.*(bug|issue|problem|error)",
                ExecutionMode::HybridConsensus,
                0.85,
                "Bug fix"
            )?,
            PatternMatcher::new(
                r"(?i)(refactor|improve|clean\s*up)\s+.*(code|function|class|module)",
                ExecutionMode::HybridConsensus,
                0.8,
                "Code refactoring"
            )?,
        ];
        
        Ok(Self {
            patterns,
            complexity_analyzer: ComplexityAnalyzer::new(),
            ai_helpers: None,
            complexity_threshold: 0.4, // Lower threshold = more queries go to consensus for better thinking
            openrouter_client: None,
            generator_model: None,
        })
    }

    /// Set AI helpers for enhanced detection
    pub fn with_ai_helpers(mut self, ai_helpers: Arc<AIHelperEcosystem>) -> Self {
        self.ai_helpers = Some(ai_helpers);
        self
    }
    
    /// Set OpenRouter client and generator model for LLM-based routing
    pub fn with_routing_config(
        mut self, 
        client: Arc<crate::consensus::openrouter::OpenRouterClient>,
        generator_model: String
    ) -> Self {
        self.openrouter_client = Some(client);
        self.generator_model = Some(generator_model);
        self
    }

    /// Detect the appropriate execution mode for a request
    pub async fn detect_mode(&self, request: &str) -> ExecutionMode {
        tracing::debug!("🔍 Mode detector analyzing request: '{}'", request);
        
        // Use the Generator model from the current profile to make routing decision
        if let (Some(client), Some(model)) = (&self.openrouter_client, &self.generator_model) {
            tracing::info!("🤖 Using Generator model {} for routing decision", model);
            
            // Build the routing guidance prompt - let the LLM decide based on complexity
            let routing_prompt = format!(
                r#"Question: "{}"

Analyze the complexity of this question. 

If it has a simple, straightforward answer that can be given immediately, respond: DIRECT

If it requires complex analysis, multiple perspectives, or detailed reasoning, respond: CONSENSUS

YOUR RESPONSE MUST BE EXACTLY ONE WORD: Either "DIRECT" or "CONSENSUS"

Answer:"#,
                request
            );
            
            let messages = vec![
                crate::consensus::openrouter::OpenRouterMessage {
                    role: "system".to_string(),
                    content: "You are a routing assistant that determines whether questions need simple direct answers or complex multi-stage analysis. Respond with ONLY one word: DIRECT or CONSENSUS.".to_string(),
                },
                crate::consensus::openrouter::OpenRouterMessage {
                    role: "user".to_string(),
                    content: routing_prompt,
                },
            ];
            
            let req = crate::consensus::openrouter::OpenRouterRequest {
                model: model.clone(),
                messages,
                temperature: Some(0.1), // Low temperature for consistent routing
                max_tokens: Some(10), // Only need one word
                stream: Some(false),
                top_p: None,
                frequency_penalty: None,
                presence_penalty: None,
                provider: None,
            };
            
            match client.chat_completion(req).await {
                Ok(response) => {
                    let raw_response = response.choices.first()
                        .and_then(|c| c.message.as_ref())
                        .map(|m| m.content.clone())
                        .unwrap_or_else(|| String::new());
                    
                    tracing::info!("🤖 LLM routing response: '{}'", raw_response);
                    
                    // Handle empty or invalid responses
                    if raw_response.trim().is_empty() {
                        tracing::warn!("⚠️ LLM returned empty response for routing - defaulting to Consensus for safety");
                        return ExecutionMode::Consensus;
                    }
                    
                    let decision = raw_response.trim().to_uppercase();
                    
                    if decision.contains("DIRECT") {
                        tracing::info!("🎯 Generator model decided: Direct mode for simple question");
                        return ExecutionMode::Direct;
                    } else {
                        tracing::info!("🎯 Generator model decided: Consensus mode for complex question");
                        return ExecutionMode::Consensus;
                    }
                }
                Err(e) => {
                    tracing::error!("❌ CRITICAL: Generator model routing failed: {}", e);
                    tracing::error!("Cannot proceed without LLM - profile or API key may be broken");
                    // This is a critical failure - the LLM must be available
                    panic!("LLM routing decision failed - cannot continue without working LLM: {}", e);
                }
            }
        }
        
        // If we get here, we don't have proper LLM configuration
        // This is a critical error - we cannot make routing decisions without an LLM
        tracing::error!("❌ CRITICAL: No LLM configuration available for routing decision");
        tracing::error!("Profile may be missing Generator model or OpenRouter client not initialized");
        panic!("Cannot make routing decision without LLM - system configuration error")
    }

    /// Check if this is a simple file operation
    /// NO PATTERN MATCHING - must use AI Helper decision
    pub fn is_simple_file_operation(&self, _request: &str) -> bool {
        // NO PATTERN MATCHING - LLM or nothing
        // This should be determined by AI Helpers, not patterns
        false
    }

    /// Check if this requires deep analysis
    /// NO PATTERN MATCHING - must use AI Helper decision
    pub fn requires_deep_analysis(&self, _request: &str) -> bool {
        // NO PATTERN MATCHING - LLM or nothing
        // This should be determined by AI Helpers, not patterns
        false
    }

    /// Get a confidence score for the mode detection
    pub async fn get_mode_confidence(&self, request: &str, mode: ExecutionMode) -> f32 {
        let detected_mode = self.detect_mode(request).await;
        
        if detected_mode == mode {
            // Check pattern confidence
            for pattern in &self.patterns {
                if let Some((pattern_mode, confidence)) = pattern.matches(request) {
                    if pattern_mode == mode {
                        return confidence;
                    }
                }
            }
            
            // Default confidence based on complexity
            let complexity = self.complexity_analyzer.analyze(request);
            match mode {
                ExecutionMode::Direct => 1.0 - complexity,
                ExecutionMode::Consensus => complexity,
                ExecutionMode::HybridConsensus => 0.7, // Medium confidence for hybrid
            }
        } else {
            0.0 // Low confidence if mode doesn't match detection
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mode_detection_simple_operations() {
        let detector = ModeDetector::new().unwrap();
        
        // Simple file operations should use direct mode
        assert_eq!(
            detector.detect_mode("create a new file called test.rs").await,
            ExecutionMode::Direct
        );
        
        assert_eq!(
            detector.detect_mode("update the file with new content").await,
            ExecutionMode::Direct
        );
        
        assert_eq!(
            detector.detect_mode("delete the file").await,
            ExecutionMode::Direct
        );
    }

    #[tokio::test]
    async fn test_mode_detection_complex_operations() {
        let detector = ModeDetector::new().unwrap();
        
        // Complex operations should use consensus mode
        assert_eq!(
            detector.detect_mode("analyze the architecture of this codebase").await,
            ExecutionMode::Consensus
        );
        
        assert_eq!(
            detector.detect_mode("explain how the authentication system works").await,
            ExecutionMode::Consensus
        );
        
        assert_eq!(
            detector.detect_mode("debug why the tests are failing").await,
            ExecutionMode::Consensus
        );
    }

    #[tokio::test]
    async fn test_mode_detection_hybrid_operations() {
        let detector = ModeDetector::new().unwrap();
        
        // Feature implementations should use hybrid mode
        assert_eq!(
            detector.detect_mode("implement a login feature").await,
            ExecutionMode::HybridConsensus
        );
        
        assert_eq!(
            detector.detect_mode("fix the bug in the authentication module").await,
            ExecutionMode::HybridConsensus
        );
    }

    #[test]
    fn test_complexity_analyzer() {
        let analyzer = ComplexityAnalyzer::new();
        
        // Simple request
        let simple = "create a file";
        assert!(analyzer.analyze(simple) < 0.3);
        
        // Complex request
        let complex = "analyze the current authentication system and explain how it handles user sessions, \
                      then suggest improvements for security and performance, considering scalability";
        assert!(analyzer.analyze(complex) > 0.6);
        
        // Medium complexity
        let medium = "create a login function that validates user credentials";
        let complexity = analyzer.analyze(medium);
        assert!(complexity >= 0.3 && complexity <= 0.6);
    }
}