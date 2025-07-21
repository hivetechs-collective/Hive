//! Smart Mode Detection
//!
//! Automatically determines whether to use direct execution or full consensus
//! based on request complexity, patterns, and context.

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::consensus::ai_helpers::AIHelperEcosystem;

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
        
        // Check for analysis keywords
        let analysis_words = ["analyze", "explain", "debug", "investigate", "review", "audit", "assess"];
        if analysis_words.iter().any(|&word| request.to_lowercase().contains(word)) {
            complexity += 0.4;
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
}

impl ModeDetector {
    pub fn new() -> Result<Self> {
        let patterns = vec![
            // Direct mode patterns - simple file operations
            PatternMatcher::new(
                r"(?i)^(create|make|add|write)\s+(a\s+)?(new\s+)?(simple\s+)?file",
                ExecutionMode::Direct,
                0.9,
                "Simple file creation"
            )?,
            PatternMatcher::new(
                r"(?i)^(create|add|write)\s+(a\s+)?test\s+for",
                ExecutionMode::Direct,
                0.85,
                "Test creation"
            )?,
            PatternMatcher::new(
                r"(?i)^(update|modify|change|edit)\s+(the\s+)?file",
                ExecutionMode::Direct,
                0.85,
                "Simple file update"
            )?,
            PatternMatcher::new(
                r"(?i)^(delete|remove)\s+(the\s+)?file",
                ExecutionMode::Direct,
                0.9,
                "File deletion"
            )?,
            PatternMatcher::new(
                r"(?i)^(rename|move)\s+(the\s+)?file",
                ExecutionMode::Direct,
                0.9,
                "File rename/move"
            )?,
            PatternMatcher::new(
                r"(?i)^(add|create|implement)\s+a\s+(simple|basic)\s+\w+",
                ExecutionMode::Direct,
                0.8,
                "Simple feature addition"
            )?,
            
            // Consensus mode patterns - complex analysis
            PatternMatcher::new(
                r"(?i)(analyze|explain|investigate|debug|diagnose)",
                ExecutionMode::Consensus,
                0.9,
                "Analysis request"
            )?,
            PatternMatcher::new(
                r"(?i)(architecture|design|structure|pattern|best\s+practice)",
                ExecutionMode::Consensus,
                0.85,
                "Architecture/design request"
            )?,
            PatternMatcher::new(
                r"(?i)(refactor|optimize|improve|enhance)\s+.*(system|architecture|codebase)",
                ExecutionMode::Consensus,
                0.9,
                "Complex refactoring"
            )?,
            PatternMatcher::new(
                r"(?i)(implement|build|create)\s+.*(system|framework|library|api)",
                ExecutionMode::Consensus,
                0.85,
                "System implementation"
            )?,
            PatternMatcher::new(
                r"(?i)(compare|evaluate|assess|review)",
                ExecutionMode::Consensus,
                0.8,
                "Evaluation request"
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
            complexity_threshold: 0.6,
        })
    }

    /// Set AI helpers for enhanced detection
    pub fn with_ai_helpers(mut self, ai_helpers: Arc<AIHelperEcosystem>) -> Self {
        self.ai_helpers = Some(ai_helpers);
        self
    }

    /// Detect the appropriate execution mode for a request
    pub async fn detect_mode(&self, request: &str) -> ExecutionMode {
        // First, check pattern matches
        let mut best_match: Option<(ExecutionMode, f32)> = None;
        
        for pattern in &self.patterns {
            if let Some((mode, confidence)) = pattern.matches(request) {
                if best_match.is_none() || best_match.as_ref().unwrap().1 < confidence {
                    best_match = Some((mode, confidence));
                }
            }
        }
        
        // If we have a high-confidence pattern match, use it
        if let Some((mode, confidence)) = best_match {
            if confidence >= 0.85 {
                return mode;
            }
        }
        
        // Analyze complexity
        let complexity = self.complexity_analyzer.analyze(request);
        
        // If we have AI helpers, get their opinion
        if let Some(ai_helpers) = &self.ai_helpers {
            // This would use AI to analyze the request
            // For now, we'll use the simple heuristics
        }
        
        // Make decision based on complexity and pattern match
        match (best_match, complexity) {
            // High confidence pattern match
            (Some((mode, confidence)), _) if confidence >= 0.8 => mode,
            
            // Low complexity, prefer direct
            (_, complexity) if complexity < 0.3 => ExecutionMode::Direct,
            
            // High complexity, prefer consensus
            (_, complexity) if complexity > self.complexity_threshold => ExecutionMode::Consensus,
            
            // Medium complexity with pattern hint
            (Some((mode, _)), _) => mode,
            
            // Medium complexity, default to hybrid
            _ => ExecutionMode::HybridConsensus,
        }
    }

    /// Check if this is a simple file operation
    pub fn is_simple_file_operation(&self, request: &str) -> bool {
        let simple_patterns = [
            r"(?i)^(create|make|add|write)\s+(a\s+)?(new\s+)?file",
            r"(?i)^(update|modify|change|edit)\s+(the\s+)?file",
            r"(?i)^(delete|remove)\s+(the\s+)?file",
            r"(?i)^(rename|move)\s+(the\s+)?file",
        ];
        
        simple_patterns.iter().any(|pattern| {
            regex::Regex::new(pattern).unwrap().is_match(request)
        })
    }

    /// Check if this requires deep analysis
    pub fn requires_deep_analysis(&self, request: &str) -> bool {
        let analysis_patterns = [
            r"(?i)(analyze|explain|investigate|debug|diagnose)",
            r"(?i)(architecture|design|structure|pattern)",
            r"(?i)(compare|evaluate|assess|review)",
            r"(?i)how\s+(does|do|can|should)",
            r"(?i)why\s+(does|do|is|are)",
        ];
        
        analysis_patterns.iter().any(|pattern| {
            regex::Regex::new(pattern).unwrap().is_match(request)
        })
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