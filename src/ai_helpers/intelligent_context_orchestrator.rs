//! Intelligent Context Orchestrator
//!
//! A sophisticated AI coordination system that uses multiple AI helpers
//! to make intelligent context decisions, preventing context contamination
//! and ensuring appropriate knowledge domain routing.

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};
use crate::consensus::ExecutionMode;

use crate::ai_helpers::{
    ContextRetriever, PatternRecognizer, QualityAnalyzer, KnowledgeSynthesizer, QualityMetrics
};

/// Enhanced context decision with multi-AI validation
#[derive(Debug, Clone)]
pub struct IntelligentContextDecision {
    pub should_use_repo: bool,
    pub confidence: f64,
    pub primary_category: QuestionCategory,
    pub secondary_categories: Vec<QuestionCategory>,
    pub reasoning: String,
    pub validation_score: f64,
    pub pattern_analysis: PatternAnalysis,
    pub quality_assessment: QualityAssessment,
}

/// Comprehensive question categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestionCategory {
    /// Repository-specific questions about current codebase
    RepositorySpecific,
    /// General programming concepts and techniques  
    GeneralProgramming,
    /// Computer science theory and algorithms
    ComputerScience,
    /// Non-programming academic subjects (physics, math, etc.)
    AcademicKnowledge,
    /// General knowledge and trivia
    GeneralKnowledge,
    /// Mixed questions combining multiple domains
    Hybrid,
    /// Unclear or ambiguous questions
    Ambiguous,
}

/// Pattern analysis from PatternRecognizer
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    pub detected_patterns: Vec<String>,
    pub code_indicators: f64,
    pub repo_indicators: f64,
    pub general_indicators: f64,
    pub academic_indicators: f64,
}

/// Quality assessment from QualityAnalyzer
#[derive(Debug, Clone)]
pub struct QualityAssessment {
    pub context_appropriateness: f64,
    pub contamination_risk: f64,
    pub clarity_score: f64,
    pub complexity_level: f64,
}

/// Intelligent Context Orchestrator coordinating all AI helpers
pub struct IntelligentContextOrchestrator {
    context_retriever: Arc<ContextRetriever>,
    pattern_recognizer: Arc<PatternRecognizer>,
    quality_analyzer: Arc<QualityAnalyzer>,
    knowledge_synthesizer: Arc<KnowledgeSynthesizer>,
    
    /// Separate caches for different question types to prevent contamination
    repo_cache: Arc<tokio::sync::RwLock<lru::LruCache<String, IntelligentContextDecision>>>,
    general_cache: Arc<tokio::sync::RwLock<lru::LruCache<String, IntelligentContextDecision>>>,
    academic_cache: Arc<tokio::sync::RwLock<lru::LruCache<String, IntelligentContextDecision>>>,
}

impl IntelligentContextOrchestrator {
    /// Create a new Intelligent Context Orchestrator
    pub fn new(
        context_retriever: Arc<ContextRetriever>,
        pattern_recognizer: Arc<PatternRecognizer>,
        quality_analyzer: Arc<QualityAnalyzer>,
        knowledge_synthesizer: Arc<KnowledgeSynthesizer>,
    ) -> Self {
        let cache_size = std::num::NonZeroUsize::new(50).unwrap();
        
        Self {
            context_retriever,
            pattern_recognizer,
            quality_analyzer,
            knowledge_synthesizer,
            repo_cache: Arc::new(tokio::sync::RwLock::new(lru::LruCache::new(cache_size))),
            general_cache: Arc::new(tokio::sync::RwLock::new(lru::LruCache::new(cache_size))),
            academic_cache: Arc::new(tokio::sync::RwLock::new(lru::LruCache::new(cache_size))),
        }
    }
    
    /// Make an intelligent context decision using all AI helpers
    pub async fn make_intelligent_context_decision(
        &self,
        question: &str,
        has_open_repository: bool,
    ) -> Result<IntelligentContextDecision> {
        info!("🧠 Starting intelligent context analysis for: '{}'", 
            &question[..question.len().min(100)]);
        info!("📂 Repository open: {}", has_open_repository);
        
        // Stage 1: Multi-AI Question Classification
        let classification = self.classify_question_multi_ai(question).await?;
        debug!("📊 Question classification: {:?}", classification.primary_category);
        
        // Stage 2: Pattern Analysis
        let pattern_analysis = self.analyze_question_patterns(question).await?;
        debug!("🔍 Pattern analysis complete: {} patterns detected", 
            pattern_analysis.detected_patterns.len());
        
        // Stage 3: Quality Assessment  
        let quality_assessment = self.assess_context_quality(question, &classification).await?;
        debug!("⭐ Quality assessment: appropriateness={:.2}, contamination_risk={:.2}", 
            quality_assessment.context_appropriateness, quality_assessment.contamination_risk);
        
        // Stage 4: Ensemble Decision Making
        let decision = self.make_ensemble_decision(
            question,
            has_open_repository,
            classification,
            pattern_analysis,
            quality_assessment,
        ).await?;
        
        // Stage 5: Cache with Domain Isolation
        self.cache_decision_by_domain(question, &decision).await;
        
        info!("✅ Intelligent context decision: use_repo={}, confidence={:.2}, category={:?}",
            decision.should_use_repo, decision.confidence, decision.primary_category);
        
        Ok(decision)
    }
    
    /// Classify question using multiple AI helpers for robust analysis
    async fn classify_question_multi_ai(&self, question: &str) -> Result<QuestionClassification> {
        // Use ContextRetriever's semantic analysis
        let context_analysis = self.context_retriever
            .analyze_question_context(question)
            .await?;
        
        // Enhance with additional AI analysis
        let enhanced_categories = self.detect_additional_categories(question);
        
        // Combine insights for comprehensive classification
        let primary_category = self.determine_primary_category(
            &context_analysis.category,
            &enhanced_categories
        );
        
        Ok(QuestionClassification {
            primary_category,
            secondary_categories: enhanced_categories,
            confidence: context_analysis.confidence,
            reasoning: context_analysis.reasoning,
        })
    }
    
    /// Analyze question patterns using PatternRecognizer
    async fn analyze_question_patterns(&self, question: &str) -> Result<PatternAnalysis> {
        // Use PatternRecognizer to identify code vs general patterns
        let patterns = self.pattern_recognizer
            .analyze_code_patterns(question, "question_analysis")
            .await
            .unwrap_or_else(|e| {
                warn!("Pattern analysis failed: {}, using heuristics", e);
                Vec::new()
            });
        
        // Calculate indicator scores
        let code_indicators = self.calculate_code_indicators(question, &patterns);
        let repo_indicators = self.calculate_repo_indicators(question, &patterns);
        let general_indicators = self.calculate_general_indicators(question, &patterns);
        let academic_indicators = self.calculate_academic_indicators(question, &patterns);
        
        Ok(PatternAnalysis {
            detected_patterns: patterns.iter().map(|p| format!("{:?}", p.pattern_type)).collect(),
            code_indicators,
            repo_indicators, 
            general_indicators,
            academic_indicators,
        })
    }
    
    /// Assess context quality using QualityAnalyzer  
    async fn assess_context_quality(
        &self,
        question: &str,
        classification: &QuestionClassification,
    ) -> Result<QualityAssessment> {
        // Use QualityAnalyzer to validate context appropriateness
        let quality_metrics = self.quality_analyzer
            .analyze_text_quality(question, "context_validation")
            .await
            .unwrap_or_else(|e| {
                warn!("Quality analysis failed: {}, using defaults", e);
                Default::default()
            });
        
        // Calculate contamination risk based on category
        let contamination_risk = match classification.primary_category {
            QuestionCategory::RepositorySpecific => 0.1,
            QuestionCategory::GeneralProgramming => 0.3,
            QuestionCategory::ComputerScience => 0.2,
            QuestionCategory::AcademicKnowledge => 0.8, // High risk if mixed with repo
            QuestionCategory::GeneralKnowledge => 0.9,  // Very high risk
            QuestionCategory::Hybrid => 0.6,
            QuestionCategory::Ambiguous => 0.7,
        };
        
        Ok(QualityAssessment {
            context_appropriateness: quality_metrics.overall_score,
            contamination_risk,
            clarity_score: quality_metrics.clarity,
            complexity_level: quality_metrics.complexity,
        })
    }
    
    /// Make ensemble decision combining all AI insights
    async fn make_ensemble_decision(
        &self,
        question: &str,
        has_open_repository: bool,
        classification: QuestionClassification,
        pattern_analysis: PatternAnalysis,
        quality_assessment: QualityAssessment,
    ) -> Result<IntelligentContextDecision> {
        // The ContextRetriever now handles intelligent auto-discovery
        // We just need to make the decision based on the classification
        
        // Calculate repository context suitability score
        let repo_suitability = self.calculate_repo_suitability(
            &classification,
            &pattern_analysis,
            &quality_assessment,
        );
        
        // Apply stricter contamination prevention rules
        let should_use_repo = repo_suitability > 0.7  // Increased threshold from 0.6
            && quality_assessment.contamination_risk < 0.3  // Reduced from 0.5
            && matches!(
                classification.primary_category,
                QuestionCategory::RepositorySpecific  // Only for repository-specific questions
            );
        
        let confidence = if should_use_repo {
            repo_suitability * (1.0 - quality_assessment.contamination_risk)
        } else {
            1.0 - repo_suitability
        };
        
        let reasoning = self.generate_decision_reasoning(
            &classification,
            &pattern_analysis,
            &quality_assessment,
            repo_suitability,
            should_use_repo,
        );
        
        let decision = IntelligentContextDecision {
            should_use_repo,
            confidence,
            primary_category: classification.primary_category.clone(),
            secondary_categories: classification.secondary_categories,
            reasoning,
            validation_score: quality_assessment.context_appropriateness,
            pattern_analysis,
            quality_assessment,
        };
        
        info!("🎯 Final decision: category={:?}, should_use_repo={}, confidence={:.2}, repo_suitability={:.2}",
            decision.primary_category, decision.should_use_repo, decision.confidence, repo_suitability);
        
        Ok(decision)
    }
    
    /// Cache decision in appropriate domain-specific cache
    async fn cache_decision_by_domain(
        &self,
        question: &str,
        decision: &IntelligentContextDecision,
    ) {
        let cache_key = question.to_string();
        
        match decision.primary_category {
            QuestionCategory::RepositorySpecific | QuestionCategory::GeneralProgramming => {
                self.repo_cache.write().await.put(cache_key, decision.clone());
            }
            QuestionCategory::AcademicKnowledge | QuestionCategory::GeneralKnowledge => {
                self.academic_cache.write().await.put(cache_key, decision.clone());
            }
            _ => {
                self.general_cache.write().await.put(cache_key, decision.clone());
            }
        }
    }
    
    // Helper methods for calculations and analysis
    
    fn detect_additional_categories(&self, question: &str) -> Vec<QuestionCategory> {
        let question_lower = question.to_lowercase();
        let mut categories = Vec::new();
        
        // Physics/Science indicators
        if question_lower.contains("quantum") || question_lower.contains("physics") 
            || question_lower.contains("chemistry") || question_lower.contains("biology") {
            categories.push(QuestionCategory::AcademicKnowledge);
        }
        
        // Programming indicators
        if question_lower.contains("code") || question_lower.contains("function")
            || question_lower.contains("algorithm") || question_lower.contains("programming") {
            categories.push(QuestionCategory::GeneralProgramming);
        }
        
        // Repository indicators  
        if question_lower.contains("this") || question_lower.contains("here")
            || question_lower.contains("current") || question_lower.contains("my") {
            categories.push(QuestionCategory::RepositorySpecific);
        }
        
        categories
    }
    
    fn determine_primary_category(
        &self,
        context_category: &str,
        additional_categories: &[QuestionCategory],
    ) -> QuestionCategory {
        // Logic to determine primary category from multiple inputs
        if context_category == "repository_specific" {
            QuestionCategory::RepositorySpecific
        } else if additional_categories.contains(&QuestionCategory::AcademicKnowledge) {
            QuestionCategory::AcademicKnowledge
        } else if additional_categories.contains(&QuestionCategory::GeneralProgramming) {
            QuestionCategory::GeneralProgramming
        } else {
            QuestionCategory::GeneralKnowledge
        }
    }
    
    fn calculate_code_indicators(&self, _question: &str, patterns: &[crate::ai_helpers::Pattern]) -> f64 {
        patterns.iter()
            .filter(|p| matches!(p.pattern_type, crate::ai_helpers::PatternType::Recurring | crate::ai_helpers::PatternType::Evolution))
            .count() as f64 / patterns.len().max(1) as f64
    }
    
    fn calculate_repo_indicators(&self, question: &str, _patterns: &[crate::ai_helpers::Pattern]) -> f64 {
        let repo_keywords = ["this", "here", "current", "my", "our", "the code", "this project"];
        let matches = repo_keywords.iter()
            .filter(|&&keyword| question.to_lowercase().contains(keyword))
            .count();
        matches as f64 / repo_keywords.len() as f64
    }
    
    fn calculate_general_indicators(&self, question: &str, _patterns: &[crate::ai_helpers::Pattern]) -> f64 {
        let general_keywords = ["what is", "how does", "explain", "difference between"];
        let matches = general_keywords.iter()
            .filter(|&&keyword| question.to_lowercase().contains(keyword))
            .count();
        matches as f64 / general_keywords.len() as f64
    }
    
    fn calculate_academic_indicators(&self, question: &str, _patterns: &[crate::ai_helpers::Pattern]) -> f64 {
        let academic_keywords = ["quantum", "physics", "chemistry", "theory", "mathematics"];
        let matches = academic_keywords.iter()
            .filter(|&&keyword| question.to_lowercase().contains(keyword))
            .count();
        matches as f64 / academic_keywords.len() as f64
    }
    
    fn calculate_repo_suitability(
        &self,
        classification: &QuestionClassification,
        pattern_analysis: &PatternAnalysis,
        _quality_assessment: &QualityAssessment,
    ) -> f64 {
        let category_weight = match classification.primary_category {
            QuestionCategory::RepositorySpecific => 1.0,
            QuestionCategory::GeneralProgramming => 0.3,  // Reduced from 0.4
            QuestionCategory::ComputerScience => 0.1,      // Reduced from 0.2
            QuestionCategory::GeneralKnowledge => 0.0,     // Explicit zero for general knowledge
            QuestionCategory::AcademicKnowledge => 0.0,    // Explicit zero for academic knowledge
            _ => 0.0,
        };
        
        // Only consider pattern weight for repository-specific questions
        let pattern_weight = match classification.primary_category {
            QuestionCategory::RepositorySpecific => {
                (pattern_analysis.code_indicators + pattern_analysis.repo_indicators) / 2.0
            },
            _ => 0.0  // No pattern weight for non-repository questions
        };
        
        // More conservative calculation
        (category_weight * 0.8) + (pattern_weight * 0.2)
    }
    
    fn generate_decision_reasoning(
        &self,
        classification: &QuestionClassification,
        pattern_analysis: &PatternAnalysis,
        quality_assessment: &QualityAssessment,
        repo_suitability: f64,
        should_use_repo: bool,
    ) -> String {
        format!(
            "Multi-AI analysis: category={:?}, patterns={}, repo_suitability={:.2}, contamination_risk={:.2}, decision={}",
            classification.primary_category,
            pattern_analysis.detected_patterns.len(),
            repo_suitability,
            quality_assessment.contamination_risk,
            if should_use_repo { "use_repo" } else { "general_knowledge" }
        )
    }
    
    /// Make an intelligent execution mode decision (Direct vs Consensus)
    pub async fn make_execution_mode_decision(&self, question: &str) -> Result<ExecutionMode> {
        info!("🤖 AI Helper analyzing execution mode for: '{}'", 
            &question[..question.len().min(100)]);
        
        // First classify the question
        let classification = self.classify_question_multi_ai(question).await?;
        
        // Analyze complexity
        let pattern_analysis = self.analyze_question_patterns(question).await?;
        let quality_assessment = self.assess_context_quality(question, &classification).await?;
        
        // Decision logic based on AI analysis
        let mode = match classification.primary_category {
            // Simple factual questions should use Direct mode
            QuestionCategory::GeneralKnowledge => {
                info!("📚 General knowledge question - using Direct mode");
                ExecutionMode::Direct
            }
            
            // Academic questions that don't need codebase context
            QuestionCategory::AcademicKnowledge => {
                info!("🎓 Academic question - using Direct mode");
                ExecutionMode::Direct
            }
            
            // Repository-specific questions
            QuestionCategory::RepositorySpecific => {
                // Even repo questions can be simple
                if quality_assessment.complexity_level < 0.3 {
                    info!("📂 Simple repository question - using Direct mode");
                    ExecutionMode::Direct
                } else {
                    info!("📂 Complex repository question - using Consensus mode");
                    ExecutionMode::Consensus
                }
            }
            
            // General programming questions
            QuestionCategory::GeneralProgramming => {
                if quality_assessment.complexity_level < 0.4 {
                    info!("💻 Simple programming question - using Direct mode");
                    ExecutionMode::Direct
                } else {
                    info!("💻 Complex programming question - using Consensus mode");
                    ExecutionMode::Consensus
                }
            }
            
            // Complex CS theory needs consensus
            QuestionCategory::ComputerScience => {
                info!("🔬 Computer science theory - using Consensus mode");
                ExecutionMode::Consensus
            }
            
            // Hybrid/complex questions
            QuestionCategory::Hybrid | QuestionCategory::Ambiguous => {
                if quality_assessment.complexity_level > 0.6 {
                    info!("🔀 Complex/hybrid question - using Consensus mode");
                    ExecutionMode::Consensus
                } else {
                    info!("🔀 Moderate complexity - using Direct mode");
                    ExecutionMode::Direct
                }
            }
        };
        
        info!("🎯 AI Helper decision: {:?} mode (category: {:?}, complexity: {:.2})",
            mode, classification.primary_category, quality_assessment.complexity_level);
        
        Ok(mode)
    }
}

/// Question classification result
#[derive(Debug, Clone)]
struct QuestionClassification {
    primary_category: QuestionCategory,
    secondary_categories: Vec<QuestionCategory>,
    confidence: f64,
    reasoning: String,
}

