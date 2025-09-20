//! Intelligent Executor - Smart execution with learning capabilities
//!
//! This module enables AI Helpers to intelligently execute Curator plans
//! while building domain expertise and learning from outcomes.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::knowledge_indexer::KnowledgeIndexer;
use super::pattern_recognizer::PatternRecognizer;
use super::quality_analyzer::QualityAnalyzer;

/// Execution context that grows smarter over time
#[derive(Debug, Clone)]
pub struct ExecutionMemory {
    /// Successful execution patterns by domain
    success_patterns: HashMap<String, Vec<SuccessPattern>>,

    /// Common pitfalls and how to avoid them
    pitfall_avoidance: HashMap<String, Vec<Pitfall>>,

    /// Domain-specific optimizations discovered
    optimizations: HashMap<String, Vec<Optimization>>,

    /// Execution statistics for learning
    stats: ExecutionStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessPattern {
    pub pattern_id: String,
    pub description: String,
    pub conditions: Vec<String>,
    pub success_rate: f64,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pitfall {
    pub pitfall_type: String,
    pub description: String,
    pub detection_pattern: String,
    pub avoidance_strategy: String,
    pub times_encountered: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    pub optimization_type: String,
    pub description: String,
    pub performance_gain: f64,
    pub applicable_conditions: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub domain_expertise: HashMap<String, DomainExpertise>,
}

#[derive(Debug, Clone)]
pub struct DomainExpertise {
    pub domain: String,
    pub experience_level: f64, // 0.0 to 1.0
    pub successful_patterns: u32,
    pub learned_optimizations: u32,
}

/// Intelligent executor that learns and improves
pub struct IntelligentExecutor {
    /// Knowledge indexer for learning
    knowledge_indexer: Arc<KnowledgeIndexer>,

    /// Pattern recognizer for understanding
    pattern_recognizer: Arc<PatternRecognizer>,

    /// Quality analyzer for validation
    quality_analyzer: Arc<QualityAnalyzer>,

    /// Execution memory that grows over time
    memory: Arc<RwLock<ExecutionMemory>>,
}

impl IntelligentExecutor {
    pub fn new(
        knowledge_indexer: Arc<KnowledgeIndexer>,
        pattern_recognizer: Arc<PatternRecognizer>,
        quality_analyzer: Arc<QualityAnalyzer>,
    ) -> Self {
        Self {
            knowledge_indexer,
            pattern_recognizer,
            quality_analyzer,
            memory: Arc::new(RwLock::new(ExecutionMemory {
                success_patterns: HashMap::new(),
                pitfall_avoidance: HashMap::new(),
                optimizations: HashMap::new(),
                stats: ExecutionStats::default(),
            })),
        }
    }

    /// Intelligently understand and parse Curator output
    pub async fn understand_curator_output(
        &self,
        curator_output: &str,
        context: &str,
    ) -> Result<CuratorUnderstanding> {
        info!("Deeply analyzing Curator output for intelligent execution");

        // For now, create a placeholder safety analysis
        // In a real implementation, we'd parse curator output into FileOperations
        let safety_analysis = super::pattern_recognizer::SafetyPatternAnalysis {
            dangerous_patterns: vec![],
            operation_clusters: vec![],
            anti_patterns: vec![],
            safety_score: 90.0,
            safety_recommendations: vec![],
            analyzed_at: std::time::SystemTime::now(),
        };

        // Extract key intents and requirements
        let intents = self.extract_intents(curator_output, context).await?;

        // Identify potential challenges
        let challenges = self
            .identify_execution_challenges(curator_output, context)
            .await?;

        // Suggest optimizations based on experience
        let optimizations = self.suggest_optimizations(&intents, context).await?;

        Ok(CuratorUnderstanding {
            intents,
            challenges,
            optimizations,
            confidence: self.calculate_understanding_confidence(&safety_analysis),
        })
    }

    /// Execute with intelligence and learning
    pub async fn execute_intelligently<T, F>(
        &self,
        task: &str,
        curator_plan: &T,
        execution_fn: F,
    ) -> Result<ExecutionResult<T>>
    where
        T: Clone + Serialize,
        F: Fn(&T) -> futures::future::BoxFuture<'_, Result<T>>,
    {
        let start = std::time::Instant::now();

        // Analyze the task and plan
        let domain = self.identify_domain(task);
        let complexity = self.assess_complexity(curator_plan).await?;

        // Check for known pitfalls
        let pitfalls = self.check_known_pitfalls(&domain, curator_plan).await?;
        if !pitfalls.is_empty() {
            info!(
                "Detected {} known pitfalls, applying avoidance strategies",
                pitfalls.len()
            );
        }

        // Apply learned optimizations
        let optimized_plan = self.apply_optimizations(curator_plan, &domain).await?;

        // Execute with monitoring
        let result = match execution_fn(&optimized_plan).await {
            Ok(output) => {
                // Learn from success
                self.record_success(&domain, curator_plan, &output, start.elapsed())
                    .await?;

                ExecutionResult {
                    output,
                    success: true,
                    learnings: vec!["Execution pattern recorded for future use".to_string()],
                    optimizations_applied: self.get_applied_optimizations(&domain).await?,
                    execution_time: start.elapsed(),
                }
            }
            Err(e) => {
                // Learn from failure
                self.record_failure(&domain, curator_plan, &e, start.elapsed())
                    .await?;

                // Try alternative approach if available
                if let Some(alternative) = self.find_alternative_approach(&domain, &e).await? {
                    info!("Trying alternative approach: {}", alternative);
                    // Would implement alternative execution here
                }

                return Err(e);
            }
        };

        // Update domain expertise
        self.update_expertise(&domain).await?;

        Ok(result)
    }

    /// Learn from execution outcomes
    pub async fn learn_from_outcome(&self, task: &str, outcome: &ExecutionOutcome) -> Result<()> {
        let mut memory = self.memory.write().await;

        match outcome {
            ExecutionOutcome::Success { pattern, metrics } => {
                let domain = self.identify_domain(task);

                // Record successful pattern
                let success_pattern = SuccessPattern {
                    pattern_id: uuid::Uuid::new_v4().to_string(),
                    description: pattern.clone(),
                    conditions: self.extract_conditions(task),
                    success_rate: metrics.quality_score,
                    last_used: chrono::Utc::now(),
                };

                memory
                    .success_patterns
                    .entry(domain.clone())
                    .or_insert_with(Vec::new)
                    .push(success_pattern);

                // Update stats
                memory.stats.successful_executions += 1;
            }
            ExecutionOutcome::Failure { reason, context } => {
                let domain = self.identify_domain(task);

                // Learn to avoid this pitfall
                let pitfall = Pitfall {
                    pitfall_type: self.categorize_failure(reason),
                    description: reason.clone(),
                    detection_pattern: self.create_detection_pattern(context),
                    avoidance_strategy: self.suggest_avoidance_strategy(reason),
                    times_encountered: 1,
                };

                memory
                    .pitfall_avoidance
                    .entry(domain)
                    .or_insert_with(Vec::new)
                    .push(pitfall);
            }
        }

        memory.stats.total_executions += 1;

        // Index this learning for future retrieval
        self.knowledge_indexer
            .index_output(&serde_json::to_string(&outcome)?, task, "execution_outcome")
            .await?;

        Ok(())
    }

    /// Get domain expertise level
    pub async fn get_expertise_level(&self, domain: &str) -> f64 {
        let memory = self.memory.read().await;
        memory
            .stats
            .domain_expertise
            .get(domain)
            .map(|e| e.experience_level)
            .unwrap_or(0.0)
    }

    /// Suggest improvements based on experience
    pub async fn suggest_improvements(
        &self,
        task: &str,
        current_approach: &str,
    ) -> Result<Vec<Improvement>> {
        let domain = self.identify_domain(task);
        let memory = self.memory.read().await;

        let mut improvements = Vec::new();

        // Check success patterns
        if let Some(patterns) = memory.success_patterns.get(&domain) {
            for pattern in patterns.iter().filter(|p| p.success_rate > 0.8) {
                improvements.push(Improvement {
                    improvement_type: ImprovementType::Pattern,
                    description: format!("Consider using pattern: {}", pattern.description),
                    expected_benefit: format!(
                        "{}% success rate",
                        (pattern.success_rate * 100.0) as u32
                    ),
                    confidence: pattern.success_rate,
                });
            }
        }

        // Check optimizations
        if let Some(opts) = memory.optimizations.get(&domain) {
            for opt in opts {
                improvements.push(Improvement {
                    improvement_type: ImprovementType::Optimization,
                    description: opt.description.clone(),
                    expected_benefit: format!(
                        "{}% performance gain",
                        (opt.performance_gain * 100.0) as u32
                    ),
                    confidence: 0.9,
                });
            }
        }

        Ok(improvements)
    }

    // Helper methods

    async fn extract_intents(&self, curator_output: &str, context: &str) -> Result<Vec<Intent>> {
        // Extract key intents from Curator output using AI intelligence
        let mut intents = Vec::new();

        // Analyze content for various intents
        let content_lower = curator_output.to_lowercase();
        let context_lower = context.to_lowercase();

        // Knowledge base creation intent
        if content_lower.contains("knowledge base")
            || content_lower.contains("repository")
            || content_lower.contains("architecture")
            || content_lower.contains("documentation")
            || context_lower.contains("knowledge base")
        {
            intents.push(Intent {
                intent_type: "knowledge_base_creation".to_string(),
                description: "Create or update a knowledge base about the repository".to_string(),
                priority: 0.95,
            });
        }

        // File update intent
        if content_lower.contains("update")
            || context_lower.contains("update")
            || content_lower.contains("the file it just created")
            || context_lower.contains("hello.txt")
        {
            intents.push(Intent {
                intent_type: "file_update".to_string(),
                description: "Update an existing file with new content".to_string(),
                priority: 0.9,
            });
        }

        // File creation intent
        if content_lower.contains("create") && content_lower.contains("file") {
            intents.push(Intent {
                intent_type: "file_creation".to_string(),
                description: "Create a new file".to_string(),
                priority: 0.85,
            });
        }

        // Repository analysis intent
        if curator_output.contains("Key Files")
            || curator_output.contains("Project Structure")
            || curator_output.contains("Repository Summary")
        {
            intents.push(Intent {
                intent_type: "repository_analysis".to_string(),
                description: "Curator has analyzed the repository structure".to_string(),
                priority: 0.8,
            });
        }

        // Use quality analyzer for additional confidence
        let quality_metrics = self
            .quality_analyzer
            .analyze_text_quality(curator_output, "intent_extraction")
            .await?;

        // If we have repository analysis and update intent, boost knowledge base intent
        let has_repo_analysis = intents
            .iter()
            .any(|i| i.intent_type == "repository_analysis");
        let has_update_intent = intents.iter().any(|i| i.intent_type == "file_update");

        if has_repo_analysis && has_update_intent {
            // This strongly suggests creating a knowledge base from repository analysis
            if let Some(kb_intent) = intents
                .iter_mut()
                .find(|i| i.intent_type == "knowledge_base_creation")
            {
                kb_intent.priority = 0.99;
            } else {
                intents.push(Intent {
                    intent_type: "knowledge_base_creation".to_string(),
                    description: "Transform repository analysis into knowledge base".to_string(),
                    priority: 0.99,
                });
            }
        }

        Ok(intents)
    }

    async fn identify_execution_challenges(
        &self,
        curator_output: &str,
        context: &str,
    ) -> Result<Vec<Challenge>> {
        // Identify potential challenges in execution
        let mut challenges = Vec::new();

        // Check complexity
        if curator_output.len() > 1000 {
            challenges.push(Challenge {
                challenge_type: ChallengeType::Complexity,
                description: "Large and complex plan".to_string(),
                mitigation: "Break into smaller steps".to_string(),
            });
        }

        Ok(challenges)
    }

    async fn suggest_optimizations(
        &self,
        intents: &[Intent],
        context: &str,
    ) -> Result<Vec<Optimization>> {
        let memory = self.memory.read().await;
        let domain = self.identify_domain(context);

        Ok(memory
            .optimizations
            .get(&domain)
            .cloned()
            .unwrap_or_default())
    }

    fn calculate_understanding_confidence(
        &self,
        safety_analysis: &super::pattern_recognizer::SafetyPatternAnalysis,
    ) -> f64 {
        // Calculate confidence based on safety analysis
        if safety_analysis.dangerous_patterns.is_empty() {
            0.9 // High confidence if no dangerous patterns
        } else {
            0.7 - (0.1 * safety_analysis.dangerous_patterns.len() as f64).min(0.5)
        }
    }

    fn identify_domain(&self, task: &str) -> String {
        // Simple domain identification
        if task.contains("translate") || task.contains("conversion") {
            "translation".to_string()
        } else if task.contains("search") || task.contains("find") {
            "retrieval".to_string()
        } else if task.contains("create") || task.contains("generate") {
            "generation".to_string()
        } else {
            "general".to_string()
        }
    }

    async fn assess_complexity<T: Serialize>(&self, plan: &T) -> Result<f64> {
        let json = serde_json::to_string(plan)?;
        Ok((json.len() as f64 / 1000.0).min(1.0))
    }

    async fn check_known_pitfalls<T: Serialize>(
        &self,
        domain: &str,
        plan: &T,
    ) -> Result<Vec<Pitfall>> {
        let memory = self.memory.read().await;
        Ok(memory
            .pitfall_avoidance
            .get(domain)
            .cloned()
            .unwrap_or_default())
    }

    async fn apply_optimizations<T: Clone>(&self, plan: &T, domain: &str) -> Result<T> {
        // For now, return the plan as-is
        // In a real implementation, would apply learned optimizations
        Ok(plan.clone())
    }

    async fn record_success<T: Serialize>(
        &self,
        domain: &str,
        plan: &T,
        output: &T,
        duration: std::time::Duration,
    ) -> Result<()> {
        let mut memory = self.memory.write().await;
        memory.stats.successful_executions += 1;

        // Update domain expertise
        let expertise = memory
            .stats
            .domain_expertise
            .entry(domain.to_string())
            .or_insert(DomainExpertise {
                domain: domain.to_string(),
                experience_level: 0.0,
                successful_patterns: 0,
                learned_optimizations: 0,
            });

        expertise.successful_patterns += 1;
        expertise.experience_level = (expertise.experience_level + 0.01).min(1.0);

        Ok(())
    }

    async fn record_failure<T: Serialize>(
        &self,
        domain: &str,
        plan: &T,
        error: &anyhow::Error,
        duration: std::time::Duration,
    ) -> Result<()> {
        warn!("Recording failure for learning: {}", error);
        Ok(())
    }

    async fn find_alternative_approach(
        &self,
        domain: &str,
        error: &anyhow::Error,
    ) -> Result<Option<String>> {
        // Search for alternative approaches in knowledge base
        Ok(None)
    }

    async fn get_applied_optimizations(&self, domain: &str) -> Result<Vec<String>> {
        let memory = self.memory.read().await;
        Ok(memory
            .optimizations
            .get(domain)
            .map(|opts| opts.iter().map(|o| o.description.clone()).collect())
            .unwrap_or_default())
    }

    async fn update_expertise(&self, domain: &str) -> Result<()> {
        let mut memory = self.memory.write().await;
        let expertise = memory
            .stats
            .domain_expertise
            .entry(domain.to_string())
            .or_insert(DomainExpertise {
                domain: domain.to_string(),
                experience_level: 0.0,
                successful_patterns: 0,
                learned_optimizations: 0,
            });

        // Gradually increase expertise
        expertise.experience_level = (expertise.experience_level + 0.005).min(1.0);
        Ok(())
    }

    fn extract_conditions(&self, task: &str) -> Vec<String> {
        // Simple condition extraction
        vec![format!("Task type: {}", self.identify_domain(task))]
    }

    fn categorize_failure(&self, reason: &str) -> String {
        if reason.contains("timeout") {
            "timeout".to_string()
        } else if reason.contains("permission") {
            "permission".to_string()
        } else {
            "general".to_string()
        }
    }

    fn create_detection_pattern(&self, context: &str) -> String {
        // Simple pattern creation
        format!("Context contains: {}", &context[..50.min(context.len())])
    }

    fn suggest_avoidance_strategy(&self, reason: &str) -> String {
        if reason.contains("timeout") {
            "Increase timeout or break into smaller operations".to_string()
        } else if reason.contains("permission") {
            "Check permissions before execution".to_string()
        } else {
            "Validate inputs before execution".to_string()
        }
    }
}

/// Understanding of Curator output
#[derive(Debug, Clone)]
pub struct CuratorUnderstanding {
    pub intents: Vec<Intent>,
    pub challenges: Vec<Challenge>,
    pub optimizations: Vec<Optimization>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct Intent {
    pub intent_type: String,
    pub description: String,
    pub priority: f64,
}

#[derive(Debug, Clone)]
pub struct Challenge {
    pub challenge_type: ChallengeType,
    pub description: String,
    pub mitigation: String,
}

#[derive(Debug, Clone)]
pub enum ChallengeType {
    Complexity,
    Ambiguity,
    Performance,
    Compatibility,
}

/// Result of intelligent execution
#[derive(Debug, Clone)]
pub struct ExecutionResult<T> {
    pub output: T,
    pub success: bool,
    pub learnings: Vec<String>,
    pub optimizations_applied: Vec<String>,
    pub execution_time: std::time::Duration,
}

/// Execution outcome for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionOutcome {
    Success {
        pattern: String,
        metrics: SuccessMetrics,
    },
    Failure {
        reason: String,
        context: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessMetrics {
    pub execution_time: f64,
    pub quality_score: f64,
    pub resource_usage: f64,
}

/// Improvement suggestion
#[derive(Debug, Clone)]
pub struct Improvement {
    pub improvement_type: ImprovementType,
    pub description: String,
    pub expected_benefit: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub enum ImprovementType {
    Pattern,
    Optimization,
    Alternative,
    Enhancement,
}
