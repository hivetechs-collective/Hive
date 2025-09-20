//! Context Injector - Intelligent context preparation for consensus stages
//!
//! This module analyzes questions and injects relevant historical knowledge
//! from the AuthoritativeKnowledgeStore to enhance consensus responses.

use anyhow::Result;
use chrono::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::authoritative_store::{AuthoritativeKnowledgeStore, CuratedFact};
use crate::ai_helpers::AIHelperEcosystem;
use crate::consensus::types::Stage;

/// Injected context with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectedContext {
    /// Target consensus stage
    pub stage: Stage,

    /// Relevant facts retrieved
    pub facts: Vec<RankedFact>,

    /// Formatted context string ready for model
    pub formatted_context: String,

    /// Metadata about the injection
    pub metadata: InjectionMetadata,
}

/// A fact with relevance ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedFact {
    pub fact: CuratedFact,
    pub relevance_score: f32,
    pub relevance_reason: String,
}

/// Metadata about context injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionMetadata {
    pub total_facts_considered: usize,
    pub facts_injected: usize,
    pub temporal_window_days: i64,
    pub relevance_threshold: f32,
    pub injection_timestamp: DateTime<Utc>,
    pub processing_time_ms: u64,
}

/// Question analysis results
#[derive(Debug, Clone)]
pub struct QuestionAnalysis {
    pub key_concepts: Vec<String>,
    pub entities: Vec<String>,
    pub question_type: QuestionType,
    pub temporal_focus: TemporalFocus,
    pub complexity_score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuestionType {
    Factual,
    Explanatory,
    Procedural,
    Comparative,
    Creative,
    Analytical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemporalFocus {
    Historical,
    Current,
    Future,
    Timeless,
}

/// Context injector for intelligent context preparation
pub struct ContextInjector {
    /// Knowledge store containing facts
    knowledge_store: Arc<AuthoritativeKnowledgeStore>,

    /// AI helpers for intelligent analysis
    ai_helpers: Arc<AIHelperEcosystem>,

    /// Time window for temporal relevance
    temporal_window: Duration,

    /// Minimum relevance score for inclusion
    relevance_threshold: f32,

    /// Maximum facts to inject per stage
    max_facts_per_stage: usize,
}

impl ContextInjector {
    /// Create a new context injector
    pub async fn new(
        knowledge_store: Arc<AuthoritativeKnowledgeStore>,
        ai_helpers: Arc<AIHelperEcosystem>,
    ) -> Result<Self> {
        Ok(Self {
            knowledge_store,
            ai_helpers,
            temporal_window: Duration::days(7), // 7 days
            relevance_threshold: 0.7,
            max_facts_per_stage: 10,
        })
    }

    /// Configure temporal window
    pub fn with_temporal_window(mut self, window: Duration) -> Self {
        self.temporal_window = window;
        self
    }

    /// Configure relevance threshold
    pub fn with_relevance_threshold(mut self, threshold: f32) -> Self {
        self.relevance_threshold = threshold;
        self
    }

    /// Inject relevant context for a specific stage
    pub async fn inject_context(
        &self,
        question: &str,
        stage: Stage,
        existing_context: Option<String>,
    ) -> Result<InjectedContext> {
        let start_time = std::time::Instant::now();
        info!("🧠 Injecting intelligent context for {:?} stage", stage);

        // 1. Analyze the question using AI helpers
        let question_analysis = self.analyze_question(question).await?;
        debug!(
            "Question analysis: {} concepts, {} entities, {:?} type",
            question_analysis.key_concepts.len(),
            question_analysis.entities.len(),
            question_analysis.question_type
        );

        // 2. Retrieve relevant facts from different dimensions
        let temporal_facts = self.get_temporal_facts(&question_analysis).await?;
        let thematic_facts = self.get_thematic_facts(&question_analysis).await?;
        let entity_facts = self.get_entity_facts(&question_analysis).await?;

        let total_facts = temporal_facts.len() + thematic_facts.len() + entity_facts.len();
        debug!(
            "Retrieved {} temporal, {} thematic, {} entity facts",
            temporal_facts.len(),
            thematic_facts.len(),
            entity_facts.len()
        );

        // 3. Rank and filter by relevance
        let ranked_facts = self
            .rank_facts_by_relevance(
                temporal_facts,
                thematic_facts,
                entity_facts,
                &question_analysis,
                stage,
            )
            .await?;

        // 4. Format context for the specific stage
        let formatted_context = self
            .format_for_stage(&ranked_facts, stage, existing_context)
            .await?;

        // 5. Create metadata
        let metadata = InjectionMetadata {
            total_facts_considered: total_facts,
            facts_injected: ranked_facts.len(),
            temporal_window_days: self.temporal_window.num_days(),
            relevance_threshold: self.relevance_threshold,
            injection_timestamp: Utc::now(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
        };

        info!(
            "✅ Injected {} relevant facts for {:?} stage in {}ms",
            ranked_facts.len(),
            stage,
            metadata.processing_time_ms
        );

        Ok(InjectedContext {
            stage,
            facts: ranked_facts,
            formatted_context,
            metadata,
        })
    }

    /// Analyze question using AI helpers
    async fn analyze_question(&self, question: &str) -> Result<QuestionAnalysis> {
        // Create indexed knowledge from the question for analysis
        let question_indexed = crate::ai_helpers::IndexedKnowledge {
            id: uuid::Uuid::new_v4().to_string(),
            content: question.to_string(),
            embedding: vec![],
            metadata: serde_json::json!({
                "type": "question",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
        };

        // Use pattern recognizer to analyze question patterns
        let patterns = self
            .ai_helpers
            .pattern_recognizer
            .analyze_patterns(&question_indexed)
            .await
            .unwrap_or_else(|_| vec![]);

        // Extract key concepts from patterns and question
        let concepts = self
            .extract_key_concepts_from_patterns(question, &patterns)
            .await?;

        // Extract entities
        let entities = self.extract_entities(question).await?;

        // Determine question type
        let question_type = self.classify_question_type(question);

        // Determine temporal focus
        let temporal_focus = self.determine_temporal_focus(question);

        // Calculate complexity
        let complexity_score = self.calculate_complexity(question, &concepts, &entities);

        Ok(QuestionAnalysis {
            key_concepts: concepts,
            entities,
            question_type,
            temporal_focus,
            complexity_score,
        })
    }

    /// Get temporally relevant facts
    async fn get_temporal_facts(&self, analysis: &QuestionAnalysis) -> Result<Vec<CuratedFact>> {
        match analysis.temporal_focus {
            TemporalFocus::Current => {
                // Get recent facts within temporal window
                self.knowledge_store
                    .get_recent_facts(self.temporal_window)
                    .await
            }
            TemporalFocus::Historical => {
                // Get all facts, sorted by date
                self.knowledge_store.get_all_facts_sorted_by_date().await
            }
            _ => {
                // For future or timeless, use concept-based retrieval
                Ok(Vec::new())
            }
        }
    }

    /// Get thematically relevant facts
    async fn get_thematic_facts(&self, analysis: &QuestionAnalysis) -> Result<Vec<CuratedFact>> {
        let mut all_facts = Vec::new();

        // For each key concept, find similar facts
        for concept in &analysis.key_concepts {
            let similar = self.knowledge_store.find_similar(concept, 5).await?;
            all_facts.extend(similar);
        }

        // Deduplicate by fact ID
        let mut seen_ids = std::collections::HashSet::new();
        all_facts.retain(|fact| seen_ids.insert(fact.id.clone()));

        Ok(all_facts)
    }

    /// Get entity-specific facts
    async fn get_entity_facts(&self, analysis: &QuestionAnalysis) -> Result<Vec<CuratedFact>> {
        if analysis.entities.is_empty() {
            return Ok(Vec::new());
        }

        self.knowledge_store
            .get_facts_about_entities(&analysis.entities)
            .await
    }

    /// Rank facts by relevance to question and stage
    async fn rank_facts_by_relevance(
        &self,
        temporal_facts: Vec<CuratedFact>,
        thematic_facts: Vec<CuratedFact>,
        entity_facts: Vec<CuratedFact>,
        analysis: &QuestionAnalysis,
        stage: Stage,
    ) -> Result<Vec<RankedFact>> {
        let mut ranked_facts = Vec::new();

        // Score temporal facts
        for fact in temporal_facts {
            let score = self.calculate_temporal_relevance(&fact, analysis);
            if score >= self.relevance_threshold {
                ranked_facts.push(RankedFact {
                    fact,
                    relevance_score: score,
                    relevance_reason: "Recent and temporally relevant".to_string(),
                });
            }
        }

        // Score thematic facts
        for fact in thematic_facts {
            let score = self.calculate_thematic_relevance(&fact, analysis).await?;
            if score >= self.relevance_threshold {
                ranked_facts.push(RankedFact {
                    fact,
                    relevance_score: score,
                    relevance_reason: "Thematically similar to question concepts".to_string(),
                });
            }
        }

        // Score entity facts
        for fact in entity_facts {
            let score = self.calculate_entity_relevance(&fact, analysis);
            if score >= self.relevance_threshold {
                ranked_facts.push(RankedFact {
                    fact,
                    relevance_score: score,
                    relevance_reason: "Contains relevant entities".to_string(),
                });
            }
        }

        // Sort by relevance score
        ranked_facts.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Apply stage-specific filtering
        let stage_limit = self.get_stage_limit(stage);
        ranked_facts.truncate(stage_limit);

        Ok(ranked_facts)
    }

    /// Format facts for specific stage
    async fn format_for_stage(
        &self,
        ranked_facts: &[RankedFact],
        stage: Stage,
        existing_context: Option<String>,
    ) -> Result<String> {
        let mut formatted = String::new();

        // Add stage-specific header
        formatted.push_str(&format!(
            "## 🧠 AI-Injected Context for {:?} Stage\n\n",
            stage
        ));

        // Add existing context if provided
        if let Some(existing) = existing_context {
            formatted.push_str(&existing);
            formatted.push_str("\n\n");
        }

        // Add facts based on stage requirements
        match stage {
            Stage::Generator => {
                formatted.push_str("### Relevant Historical Knowledge:\n\n");
                for (i, ranked_fact) in ranked_facts.iter().enumerate() {
                    formatted.push_str(&format!(
                        "{}. **{}** (relevance: {:.0}%)\n   {}\n   *Source: {} ({})*\n\n",
                        i + 1,
                        ranked_fact.relevance_reason,
                        ranked_fact.relevance_score * 100.0,
                        ranked_fact.fact.content,
                        ranked_fact.fact.source_question,
                        ranked_fact.fact.created_at.format("%Y-%m-%d")
                    ));
                }
            }
            Stage::Refiner => {
                formatted.push_str("### Facts to Consider for Refinement:\n\n");
                for ranked_fact in ranked_facts {
                    formatted.push_str(&format!(
                        "- {}\n  *(Confidence: {:.0}%, Theme: {})*\n\n",
                        ranked_fact.fact.content,
                        ranked_fact.fact.curator_confidence * 100.0,
                        ranked_fact
                            .fact
                            .topics
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "general".to_string())
                    ));
                }
            }
            Stage::Validator => {
                formatted.push_str("### Facts for Validation:\n\n");
                for ranked_fact in ranked_facts {
                    formatted.push_str(&format!(
                        "⚠️ Validate against: {}\n   Source: {} | Confidence: {:.0}%\n\n",
                        ranked_fact.fact.content,
                        ranked_fact.fact.source_question,
                        ranked_fact.fact.curator_confidence * 100.0
                    ));
                }
            }
            Stage::Curator => {
                formatted.push_str("### Authoritative Facts for Synthesis:\n\n");
                for ranked_fact in ranked_facts {
                    formatted.push_str(&format!(
                        "📚 {}\n   *Validated on {} with {:.0}% confidence*\n\n",
                        ranked_fact.fact.content,
                        ranked_fact.fact.created_at.format("%Y-%m-%d"),
                        ranked_fact.fact.curator_confidence * 100.0
                    ));
                }
            }
        }

        Ok(formatted)
    }

    // Helper methods

    async fn extract_key_concepts_from_patterns(
        &self,
        question: &str,
        patterns: &[crate::ai_helpers::Pattern],
    ) -> Result<Vec<String>> {
        // Extract concepts from patterns first
        let mut concepts: Vec<String> = patterns.iter().flat_map(|p| p.examples.clone()).collect();

        // Add concepts from simple word analysis
        let additional_concepts = self.extract_key_concepts(question).await?;
        concepts.extend(additional_concepts);

        // Deduplicate
        concepts.sort();
        concepts.dedup();

        Ok(concepts)
    }

    async fn extract_key_concepts(&self, question: &str) -> Result<Vec<String>> {
        // Use AI helpers to extract concepts
        // For now, use simple keyword extraction
        let words: Vec<&str> = question.split_whitespace().collect();
        let concepts = words
            .into_iter()
            .filter(|w| w.len() > 4 && !is_common_word(w))
            .map(|w| w.to_string())
            .collect();
        Ok(concepts)
    }

    async fn extract_entities(&self, question: &str) -> Result<Vec<String>> {
        // Extract proper nouns and specific terms
        let mut entities = Vec::new();

        // Simple heuristic: capitalized words
        for word in question.split_whitespace() {
            if word.chars().next().map_or(false, |c| c.is_uppercase()) && word.len() > 2 {
                entities.push(word.to_string());
            }
        }

        Ok(entities)
    }

    fn classify_question_type(&self, question: &str) -> QuestionType {
        let q_lower = question.to_lowercase();

        if q_lower.starts_with("what is") || q_lower.starts_with("who is") {
            QuestionType::Factual
        } else if q_lower.starts_with("how") || q_lower.starts_with("why") {
            QuestionType::Explanatory
        } else if q_lower.contains("compare") || q_lower.contains("difference") {
            QuestionType::Comparative
        } else if q_lower.contains("create") || q_lower.contains("generate") {
            QuestionType::Creative
        } else if q_lower.contains("analyze") || q_lower.contains("evaluate") {
            QuestionType::Analytical
        } else {
            QuestionType::Procedural
        }
    }

    fn determine_temporal_focus(&self, question: &str) -> TemporalFocus {
        let q_lower = question.to_lowercase();

        if q_lower.contains("latest") || q_lower.contains("recent") || q_lower.contains("current") {
            TemporalFocus::Current
        } else if q_lower.contains("history") || q_lower.contains("past") || q_lower.contains("was")
        {
            TemporalFocus::Historical
        } else if q_lower.contains("future")
            || q_lower.contains("will")
            || q_lower.contains("upcoming")
        {
            TemporalFocus::Future
        } else {
            TemporalFocus::Timeless
        }
    }

    fn calculate_complexity(
        &self,
        question: &str,
        concepts: &[String],
        entities: &[String],
    ) -> f32 {
        let word_count = question.split_whitespace().count() as f32;
        let concept_count = concepts.len() as f32;
        let entity_count = entities.len() as f32;

        // Simple complexity heuristic
        (word_count / 10.0 + concept_count / 3.0 + entity_count / 2.0).min(1.0)
    }

    fn calculate_temporal_relevance(&self, fact: &CuratedFact, analysis: &QuestionAnalysis) -> f32 {
        match analysis.temporal_focus {
            TemporalFocus::Current => {
                // More recent = more relevant
                let age = Utc::now() - fact.created_at;
                let age_days = age.num_days() as f32;
                let window_days = self.temporal_window.num_days() as f32;
                (1.0 - (age_days / window_days)).max(0.0)
            }
            _ => 0.5, // Neutral relevance for non-temporal queries
        }
    }

    async fn calculate_thematic_relevance(
        &self,
        fact: &CuratedFact,
        analysis: &QuestionAnalysis,
    ) -> Result<f32> {
        // Use AI helpers to calculate semantic similarity
        let mut total_score = 0.0;
        let mut count = 0;

        for concept in &analysis.key_concepts {
            // Simple keyword matching for now
            if fact
                .content
                .to_lowercase()
                .contains(&concept.to_lowercase())
            {
                total_score += 1.0;
            }
            count += 1;
        }

        if count > 0 {
            Ok((total_score / count as f32).min(1.0))
        } else {
            Ok(0.0)
        }
    }

    fn calculate_entity_relevance(&self, fact: &CuratedFact, analysis: &QuestionAnalysis) -> f32 {
        let mut matches = 0;

        for entity in &analysis.entities {
            if fact.content.contains(entity) {
                matches += 1;
            }
        }

        if analysis.entities.is_empty() {
            0.0
        } else {
            (matches as f32 / analysis.entities.len() as f32).min(1.0)
        }
    }

    fn get_stage_limit(&self, stage: Stage) -> usize {
        match stage {
            Stage::Generator => self.max_facts_per_stage,
            Stage::Refiner => self.max_facts_per_stage / 2,
            Stage::Validator => self.max_facts_per_stage / 2,
            Stage::Curator => self.max_facts_per_stage,
        }
    }
}

fn is_common_word(word: &str) -> bool {
    const COMMON_WORDS: &[&str] = &[
        "the", "is", "at", "which", "on", "and", "a", "an", "as", "are", "was", "were", "been",
        "be", "being", "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "shall", "can", "this", "that", "these", "those", "what",
        "where", "when", "how", "why", "who", "whom", "whose", "which",
    ];
    COMMON_WORDS.contains(&word.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_question_analysis() {
        // Test implementation will be added
    }

    #[tokio::test]
    async fn test_context_injection() {
        // Test implementation will be added
    }
}
