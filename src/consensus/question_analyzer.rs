//! Question Context Analyzer
//!
//! Determines whether a question is repository-specific or general programming/AI question.
//! This helps the consensus pipeline decide when to inject repository context.

use anyhow::Result;
use tracing::{debug, info};
use std::sync::Arc;

/// Categories of questions
#[derive(Debug, Clone, PartialEq)]
pub enum QuestionCategory {
    /// Question is about the current repository/codebase
    RepositorySpecific,
    /// General programming, framework, or concept question
    GeneralProgramming,
    /// AI/ML related questions
    AIRelated,
    /// Other general questions
    General,
}

/// Analyzes questions to determine their context and intent
pub struct QuestionAnalyzer {
    /// Keywords that indicate repository-specific questions
    repo_keywords: Vec<&'static str>,
    /// Keywords that indicate general programming questions
    general_keywords: Vec<&'static str>,
    /// Keywords that indicate AI/ML questions
    ai_keywords: Vec<&'static str>,
}

impl QuestionAnalyzer {
    pub fn new() -> Self {
        Self {
            repo_keywords: vec![
                "this code", "this project", "this repo", "this repository",
                "this file", "this function", "this class", "this method",
                "my code", "my project", "my repo", "my repository",
                "our code", "our project", "our repo", "our repository",
                "current code", "current project", "current repo", "current repository",
                "@codebase", "in this", "here", "this implementation",
                "analyze this", "review this", "check this", "fix this",
                "update this", "modify this", "change this", "improve this",
                "the code", "the project", "the repo", "the repository",
            ],
            general_keywords: vec![
                "difference between", "compare", "versus", "vs", "or",
                "what is", "how does", "explain", "when to use",
                "best practice", "which is better", "pros and cons",
                "advantages", "disadvantages", "tell me about",
                "angular", "vue", "react", "svelte", "ember",
                "python", "rust", "javascript", "typescript", "java",
                "design pattern", "algorithm", "data structure",
                "performance", "optimization", "security",
            ],
            ai_keywords: vec![
                "ai", "ml", "machine learning", "artificial intelligence",
                "llm", "large language model", "neural network",
                "deep learning", "transformer", "gpt", "claude",
                "consensus", "ai assistant", "chatbot", "nlp",
            ],
            ai_helpers,
        }
    }

    /// Analyze a question and determine its category
    pub async fn analyze(&self, question: &str) -> QuestionCategory {
        // First try AI-powered semantic analysis if available
        if let Some(ref ai_helpers) = self.ai_helpers {
            match ai_helpers.intelligent_orchestrator
                .make_intelligent_context_decision(question, true)
                .await {
                Ok(decision) => {
                    info!("🤖 AI semantic analysis: category={:?}, confidence={:.2}, should_use_repo={}", 
                        decision.primary_category, decision.confidence, decision.should_use_repo);
                    
                    // Map AI decision to our categories
                    use crate::ai_helpers::intelligent_context_orchestrator::QuestionCategory as AICategory;
                    match decision.primary_category {
                        AICategory::RepositorySpecific => return QuestionCategory::RepositorySpecific,
                        AICategory::GeneralProgramming => return QuestionCategory::GeneralProgramming,
                        AICategory::ComputerScience => return QuestionCategory::GeneralProgramming,
                        AICategory::AcademicKnowledge => return QuestionCategory::General,
                        AICategory::GeneralKnowledge => return QuestionCategory::General,
                        _ => {
                            // For Hybrid/Ambiguous, check if should use repo
                            if decision.should_use_repo {
                                return QuestionCategory::RepositorySpecific;
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("AI semantic analysis failed: {}, falling back to keywords", e);
                }
            }
        }
        
        // Fallback to keyword-based analysis
        self.analyze_with_keywords(question)
    }
    
    /// Keyword-based analysis (fallback when AI is unavailable)
    fn analyze_with_keywords(&self, question: &str) -> QuestionCategory {
        let question_lower = question.to_lowercase();
        
        // Count keyword matches for each category
        let repo_score = self.calculate_score(&question_lower, &self.repo_keywords);
        let general_score = self.calculate_score(&question_lower, &self.general_keywords);
        let ai_score = self.calculate_score(&question_lower, &self.ai_keywords);
        
        debug!(
            "Question analysis scores - Repo: {}, General: {}, AI: {}",
            repo_score, general_score, ai_score
        );
        
        // Determine category based on scores
        if repo_score > 0 && repo_score >= general_score {
            // Repository-specific takes precedence if mentioned
            QuestionCategory::RepositorySpecific
        } else if ai_score > general_score {
            QuestionCategory::AIRelated
        } else if general_score > 0 {
            QuestionCategory::GeneralProgramming
        } else if repo_score > 0 {
            // Weak repository reference
            QuestionCategory::RepositorySpecific
        } else {
            QuestionCategory::General
        }
    }

    /// Calculate keyword match score
    fn calculate_score(&self, text: &str, keywords: &[&str]) -> u32 {
        keywords.iter()
            .filter(|keyword| text.contains(*keyword))
            .count() as u32
    }

    /// Check if repository context should be used
    pub fn should_use_repository_context(
        &self,
        question: &str,
        has_open_repository: bool,
    ) -> bool {
        // If no repository is open, never use repository context
        if !has_open_repository {
            info!("No repository open, skipping repository context");
            return false;
        }

        let category = self.analyze(question);
        
        match category {
            QuestionCategory::RepositorySpecific => {
                info!("Question is repository-specific, using repository context");
                true
            }
            QuestionCategory::GeneralProgramming => {
                // For general programming questions, check if they might benefit from examples
                let might_benefit = question.to_lowercase().contains("example") ||
                                  question.to_lowercase().contains("implement") ||
                                  question.to_lowercase().contains("how to");
                
                if might_benefit {
                    info!("General programming question might benefit from repository examples");
                    true
                } else {
                    info!("General programming question, skipping repository context");
                    false
                }
            }
            _ => {
                info!("Question category {:?}, skipping repository context", category);
                false
            }
        }
    }

    /// Get a context prefix based on question category
    pub fn get_context_prefix(&self, category: &QuestionCategory) -> &'static str {
        match category {
            QuestionCategory::RepositorySpecific => {
                "REPOSITORY-SPECIFIC CONTEXT: The user is asking about their current codebase."
            }
            QuestionCategory::GeneralProgramming => {
                "GENERAL PROGRAMMING CONTEXT: The user is asking a general programming question, not specific to their repository."
            }
            QuestionCategory::AIRelated => {
                "AI/ML CONTEXT: The user is asking about AI/ML concepts or capabilities."
            }
            QuestionCategory::General => {
                "GENERAL CONTEXT: This is a general question not specific to any codebase."
            }
        }
    }
}

impl Default for QuestionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_repository_specific_detection() {
        let analyzer = QuestionAnalyzer::new();
        
        assert_eq!(
            analyzer.analyze("analyze this code and tell me what it does"),
            QuestionCategory::RepositorySpecific
        );
        
        assert_eq!(
            analyzer.analyze("what's wrong with my implementation?"),
            QuestionCategory::RepositorySpecific
        );
        
        assert_eq!(
            analyzer.analyze("review the current project structure"),
            QuestionCategory::RepositorySpecific
        );
    }

    #[test]
    fn test_general_programming_detection() {
        let analyzer = QuestionAnalyzer::new();
        
        assert_eq!(
            analyzer.analyze("what's the difference between angular and vue?"),
            QuestionCategory::GeneralProgramming
        );
        
        assert_eq!(
            analyzer.analyze("explain async/await in javascript"),
            QuestionCategory::GeneralProgramming
        );
        
        assert_eq!(
            analyzer.analyze("compare react hooks vs class components"),
            QuestionCategory::GeneralProgramming
        );
    }

    #[test]
    fn test_ai_detection() {
        let analyzer = QuestionAnalyzer::new();
        
        assert_eq!(
            analyzer.analyze("how does the consensus AI pipeline work?"),
            QuestionCategory::AIRelated
        );
        
        assert_eq!(
            analyzer.analyze("explain transformer architecture"),
            QuestionCategory::AIRelated
        );
    }

    #[test]
    fn test_repository_context_usage() {
        let analyzer = QuestionAnalyzer::new();
        
        // With repository open
        assert!(analyzer.should_use_repository_context(
            "analyze this code",
            true
        ));
        
        assert!(!analyzer.should_use_repository_context(
            "what's the difference between angular and vue?",
            true
        ));
        
        // Without repository open
        assert!(!analyzer.should_use_repository_context(
            "analyze this code",
            false
        ));
    }
}