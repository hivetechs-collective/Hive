//! Pattern Recognizer - Uses UniXcoder for identifying patterns across knowledge base
//!
//! This module identifies recurring themes, detects knowledge evolution, suggests
//! connections between different pieces of knowledge, and provides comprehensive
//! safety pattern detection for file operations.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::python_models::{ModelRequest, ModelResponse, PythonModelService};
use crate::ai_helpers::{IndexedKnowledge, Pattern, PatternType};
use crate::consensus::operation_intelligence::{
    AntiPattern, ClusterType, DangerousPattern, DangerousPatternType, DangerousSeverity,
    OperationCluster, OperationContext,
};
use crate::consensus::stages::file_aware_curator::FileOperation;

/// Configuration for Pattern Recognizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Model for cross-language pattern recognition
    pub pattern_model: String,

    /// Minimum confidence for pattern detection
    pub min_confidence: f64,

    /// Maximum patterns to track
    pub max_patterns: usize,

    /// Pattern decay rate (how quickly patterns become less relevant)
    pub decay_rate: f64,

    /// Safety pattern detection sensitivity (0.0-1.0, higher = more sensitive)
    pub safety_sensitivity: f64,

    /// Maximum operations to analyze for clustering
    pub max_cluster_operations: usize,

    /// Anti-pattern detection threshold
    pub anti_pattern_threshold: f64,
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            pattern_model: "microsoft/unixcoder-base".to_string(),
            min_confidence: 0.75,
            max_patterns: 1000,
            decay_rate: 0.95,
            safety_sensitivity: 0.8, // High sensitivity for safety
            max_cluster_operations: 50,
            anti_pattern_threshold: 0.7,
        }
    }
}

/// Safety pattern analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyPatternAnalysis {
    /// Detected dangerous patterns
    pub dangerous_patterns: Vec<DangerousPattern>,

    /// Operation clustering analysis
    pub operation_clusters: Vec<OperationCluster>,

    /// Detected anti-patterns
    pub anti_patterns: Vec<AntiPattern>,

    /// Overall safety score (0-100, higher = safer)
    pub safety_score: f32,

    /// Safety recommendations
    pub safety_recommendations: Vec<String>,

    /// Analysis timestamp
    pub analyzed_at: SystemTime,
}

/// File operation safety analysis
#[derive(Debug, Clone)]
pub struct OperationSafetyPattern {
    /// Pattern type detected
    pub pattern_type: SafetyPatternType,

    /// Description of the safety concern
    pub description: String,

    /// Severity level
    pub severity: DangerousSeverity,

    /// Confidence in detection
    pub confidence: f32,

    /// Affected file patterns
    pub affected_files: Vec<String>,

    /// Suggested mitigation
    pub mitigation: Option<String>,
}

/// Types of safety patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyPatternType {
    MassDeletion,
    OverwriteWithoutBackup,
    CircularDependency,
    BreakingChange,
    DataLoss,
    SecurityVulnerability,
    ConcurrentAccess,
    ResourceExhaustion,
    PermissionEscalation,
    ConfigurationDrift,
}

/// Repository safety metrics
#[derive(Debug, Clone)]
pub struct RepositorySafetyMetrics {
    /// Number of risky operations detected
    pub risky_operations: usize,

    /// Safety score trend (improving/declining)
    pub safety_trend: SafetyTrend,

    /// Most common dangerous patterns
    pub common_dangers: Vec<DangerousPatternType>,

    /// Files with highest risk
    pub high_risk_files: Vec<PathBuf>,

    /// Recommended safety improvements
    pub improvements: Vec<String>,
}

/// Safety trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyTrend {
    Improving { rate: f32 },
    Declining { rate: f32 },
    Stable,
    InsufficientData,
}

/// Pattern Recognizer using UniXcoder with safety analysis
pub struct PatternRecognizer {
    config: PatternConfig,

    /// Python model service
    python_service: Arc<PythonModelService>,

    /// Tracked patterns with their metadata
    pattern_store: Arc<RwLock<PatternStore>>,

    /// Pattern detection cache
    detection_cache: Arc<RwLock<lru::LruCache<String, Vec<Pattern>>>>,

    /// Safety pattern analysis cache
    safety_cache: Arc<RwLock<lru::LruCache<String, SafetyPatternAnalysis>>>,

    /// Operation safety history for trend analysis
    safety_history: Arc<RwLock<Vec<OperationSafetyRecord>>>,

    /// Known dangerous file patterns
    dangerous_file_patterns: Arc<RwLock<HashSet<String>>>,

    /// Repository safety metrics cache
    safety_metrics_cache: Arc<RwLock<Option<(RepositorySafetyMetrics, SystemTime)>>>,
}

impl std::fmt::Debug for PatternRecognizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PatternRecognizer")
            .field("config", &self.config)
            .field("python_service", &"<PythonModelService>")
            .field("pattern_store", &"<PatternStore>")
            .field("detection_cache", &"<LruCache>")
            .field("safety_cache", &"<LruCache>")
            .field("safety_history", &"<Vec<OperationSafetyRecord>>")
            .field("dangerous_file_patterns", &self.dangerous_file_patterns)
            .field("safety_metrics_cache", &"<Option<RepositorySafetyMetrics>>")
            .finish()
    }
}

/// Historical safety record for trend analysis
#[derive(Debug, Clone)]
struct OperationSafetyRecord {
    operation_hash: String,
    safety_score: f32,
    dangerous_patterns_count: usize,
    timestamp: SystemTime,
    operation_type: String,
}

/// Store for tracking patterns over time
#[derive(Default)]
struct PatternStore {
    /// All tracked patterns
    patterns: HashMap<String, TrackedPattern>,

    /// Pattern relationships
    relationships: HashMap<String, Vec<String>>,

    /// Pattern evolution history
    evolution_history: Vec<PatternEvolution>,
}

/// A pattern being tracked over time
#[derive(Debug, Clone)]
struct TrackedPattern {
    pattern: Pattern,
    first_seen: chrono::DateTime<chrono::Utc>,
    last_seen: chrono::DateTime<chrono::Utc>,
    occurrence_count: usize,
    strength: f64,
    related_facts: Vec<String>,
}

/// Evolution of a pattern over time
#[derive(Debug, Clone)]
struct PatternEvolution {
    pattern_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    change_type: EvolutionType,
    description: String,
}

#[derive(Debug, Clone)]
enum EvolutionType {
    Emerged,
    Strengthened,
    Weakened,
    Merged,
    Split,
}

impl PatternRecognizer {
    /// Create a new Pattern Recognizer with safety analysis capabilities
    pub async fn new(python_service: Arc<PythonModelService>) -> Result<Self> {
        let config = PatternConfig::default();
        let pattern_store = Arc::new(RwLock::new(PatternStore::default()));
        let detection_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(500).unwrap(),
        )));
        let safety_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(200).unwrap(),
        )));

        // Initialize dangerous file patterns with common risky patterns
        let dangerous_patterns = Self::initialize_dangerous_patterns();

        Ok(Self {
            config,
            python_service,
            pattern_store,
            detection_cache,
            safety_cache,
            safety_history: Arc::new(RwLock::new(Vec::new())),
            dangerous_file_patterns: Arc::new(RwLock::new(dangerous_patterns)),
            safety_metrics_cache: Arc::new(RwLock::new(None)),
        })
    }

    /// Initialize known dangerous file patterns
    fn initialize_dangerous_patterns() -> HashSet<String> {
        let mut patterns = HashSet::new();

        // System and sensitive files
        patterns.insert("/etc/passwd".to_string());
        patterns.insert("/etc/shadow".to_string());
        patterns.insert("/etc/hosts".to_string());
        patterns.insert("/etc/sudoers".to_string());
        patterns.insert("/.env".to_string());
        patterns.insert("/.env.production".to_string());
        patterns.insert("/config/secrets".to_string());
        patterns.insert("/config/database".to_string());

        // Build and dependency directories
        patterns.insert("/node_modules/".to_string());
        patterns.insert("/target/".to_string());
        patterns.insert("/.git/".to_string());
        patterns.insert("/dist/".to_string());
        patterns.insert("/build/".to_string());

        // Configuration files
        patterns.insert("package-lock.json".to_string());
        patterns.insert("Cargo.lock".to_string());
        patterns.insert("yarn.lock".to_string());
        patterns.insert("poetry.lock".to_string());

        // Database files
        patterns.insert("*.db".to_string());
        patterns.insert("*.sqlite".to_string());
        patterns.insert("*.sql".to_string());

        patterns
    }

    /// Analyze patterns in indexed knowledge
    pub async fn analyze_patterns(&self, indexed: &IndexedKnowledge) -> Result<Vec<Pattern>> {
        // Check cache
        if let Some(cached) = self.detection_cache.read().await.peek(&indexed.id) {
            return Ok(cached.clone());
        }

        // Detect various pattern types
        let mut patterns = Vec::new();

        // 1. Check for recurring patterns
        if let Some(recurring) = self.detect_recurring_pattern(indexed).await? {
            patterns.push(recurring);
        }

        // 2. Check for evolution patterns
        if let Some(evolution) = self.detect_evolution_pattern(indexed).await? {
            patterns.push(evolution);
        }

        // 3. Check for relationship patterns
        if let Some(relationship) = self.detect_relationship_pattern(indexed).await? {
            patterns.push(relationship);
        }

        // 4. Check for contradiction patterns
        if let Some(contradiction) = self.detect_contradiction_pattern(indexed).await? {
            patterns.push(contradiction);
        }

        // 5. Generate insight patterns
        if let Some(insight) = self.generate_insight_pattern(indexed).await? {
            patterns.push(insight);
        }

        // Update pattern store
        self.update_pattern_store(&patterns, indexed).await?;

        // Cache results
        self.detection_cache
            .write()
            .await
            .put(indexed.id.clone(), patterns.clone());

        Ok(patterns)
    }

    /// Detect recurring patterns
    async fn detect_recurring_pattern(
        &self,
        indexed: &IndexedKnowledge,
    ) -> Result<Option<Pattern>> {
        let store = self.pattern_store.read().await;

        // Get embeddings for comparison
        let embeddings = self
            .python_service
            .generate_embeddings(&self.config.pattern_model, vec![indexed.content.clone()])
            .await?;

        let current_embedding = embeddings
            .into_iter()
            .next()
            .context("No embedding returned")?;

        // Compare with existing patterns using cosine similarity
        for tracked in store.patterns.values() {
            // Generate embedding for the pattern
            let pattern_embeddings = self
                .python_service
                .generate_embeddings(
                    &self.config.pattern_model,
                    vec![tracked.pattern.description.clone()],
                )
                .await?;

            if let Some(pattern_embedding) = pattern_embeddings.into_iter().next() {
                let similarity = self.cosine_similarity(&current_embedding, &pattern_embedding);

                if similarity > self.config.min_confidence {
                    return Ok(Some(Pattern {
                        pattern_type: PatternType::Recurring,
                        description: format!("Similar to: {}", tracked.pattern.description),
                        confidence: similarity,
                        examples: vec![indexed.content.clone()],
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Detect evolution patterns
    async fn detect_evolution_pattern(
        &self,
        indexed: &IndexedKnowledge,
    ) -> Result<Option<Pattern>> {
        // Look for patterns that show knowledge evolution
        // e.g., "This updates our understanding of X"

        if indexed.content.contains("update") || indexed.content.contains("evolve") {
            return Ok(Some(Pattern {
                pattern_type: PatternType::Evolution,
                description: "Knowledge evolution detected".to_string(),
                confidence: 0.8,
                examples: vec![indexed.content.clone()],
            }));
        }

        Ok(None)
    }

    /// Detect relationship patterns
    async fn detect_relationship_pattern(
        &self,
        indexed: &IndexedKnowledge,
    ) -> Result<Option<Pattern>> {
        // Look for connections between concepts
        // e.g., "X is related to Y", "X depends on Y"

        if indexed.content.contains("related to") || indexed.content.contains("depends on") {
            return Ok(Some(Pattern {
                pattern_type: PatternType::Relationship,
                description: "Concept relationship detected".to_string(),
                confidence: 0.85,
                examples: vec![indexed.content.clone()],
            }));
        }

        Ok(None)
    }

    /// Detect contradiction patterns
    async fn detect_contradiction_pattern(
        &self,
        indexed: &IndexedKnowledge,
    ) -> Result<Option<Pattern>> {
        // Look for contradictions with existing knowledge
        // e.g., "However", "In contrast", "Actually"

        if indexed.content.contains("however") || indexed.content.contains("actually") {
            return Ok(Some(Pattern {
                pattern_type: PatternType::Contradiction,
                description: "Potential contradiction detected".to_string(),
                confidence: 0.7,
                examples: vec![indexed.content.clone()],
            }));
        }

        Ok(None)
    }

    /// Generate insight patterns
    async fn generate_insight_pattern(
        &self,
        indexed: &IndexedKnowledge,
    ) -> Result<Option<Pattern>> {
        // Generate meta-insights from accumulated patterns

        let store = self.pattern_store.read().await;

        // If we have enough patterns, try to generate an insight
        if store.patterns.len() > 10 {
            // TODO: Use model to generate actual insights
            return Ok(Some(Pattern {
                pattern_type: PatternType::Insight,
                description: "Meta-insight from pattern analysis".to_string(),
                confidence: 0.9,
                examples: vec!["Accumulated wisdom suggests...".to_string()],
            }));
        }

        Ok(None)
    }

    /// Update the pattern store with new patterns
    async fn update_pattern_store(
        &self,
        patterns: &[Pattern],
        indexed: &IndexedKnowledge,
    ) -> Result<()> {
        let mut store = self.pattern_store.write().await;
        let now = chrono::Utc::now();

        for pattern in patterns {
            let pattern_id = format!(
                "{:?}_{}",
                pattern.pattern_type,
                blake3::hash(pattern.description.as_bytes()).to_hex()
            );

            if let Some(tracked) = store.patterns.get_mut(&pattern_id) {
                // Update existing pattern
                tracked.last_seen = now;
                tracked.occurrence_count += 1;
                tracked.strength = (tracked.strength * self.config.decay_rate) + pattern.confidence;
                tracked.related_facts.push(indexed.id.clone());
            } else {
                // Create new tracked pattern
                let tracked = TrackedPattern {
                    pattern: pattern.clone(),
                    first_seen: now,
                    last_seen: now,
                    occurrence_count: 1,
                    strength: pattern.confidence,
                    related_facts: vec![indexed.id.clone()],
                };
                store.patterns.insert(pattern_id.clone(), tracked);

                // Record emergence
                store.evolution_history.push(PatternEvolution {
                    pattern_id,
                    timestamp: now,
                    change_type: EvolutionType::Emerged,
                    description: format!("New pattern emerged: {}", pattern.description),
                });
            }
        }

        // Prune weak patterns if over limit
        if store.patterns.len() > self.config.max_patterns {
            self.prune_weak_patterns(&mut store);
        }

        Ok(())
    }

    /// Remove patterns that have become too weak
    fn prune_weak_patterns(&self, store: &mut PatternStore) {
        let threshold = self.config.min_confidence * 0.5;

        store.patterns.retain(|id, tracked| {
            if tracked.strength < threshold {
                store.evolution_history.push(PatternEvolution {
                    pattern_id: id.clone(),
                    timestamp: chrono::Utc::now(),
                    change_type: EvolutionType::Weakened,
                    description: "Pattern pruned due to low strength".to_string(),
                });
                false
            } else {
                true
            }
        });
    }

    /// Analyze code patterns in text for question classification
    pub async fn analyze_code_patterns(&self, text: &str, task: &str) -> Result<Vec<Pattern>> {
        // Create a simple indexed knowledge for pattern analysis
        let indexed = IndexedKnowledge {
            id: format!("temp_{}", chrono::Utc::now().timestamp()),
            content: text.to_string(),
            embedding: vec![], // Empty embedding for temporary analysis
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };

        // Use existing pattern analysis but filter for code-specific patterns
        let all_patterns = self.analyze_patterns(&indexed).await?;

        // Filter patterns relevant to code analysis
        let code_patterns = all_patterns
            .into_iter()
            .filter(|p| {
                matches!(
                    p.pattern_type,
                    PatternType::Recurring | PatternType::Evolution | PatternType::Relationship
                )
            })
            .collect();

        Ok(code_patterns)
    }

    /// Get pattern statistics
    pub async fn get_stats(&self) -> PatternStats {
        let store = self.pattern_store.read().await;

        let mut type_counts = HashMap::new();
        for tracked in store.patterns.values() {
            *type_counts
                .entry(format!("{:?}", tracked.pattern.pattern_type))
                .or_insert(0) += 1;
        }

        PatternStats {
            total_patterns: store.patterns.len(),
            pattern_types: type_counts,
            evolution_events: store.evolution_history.len(),
            strongest_patterns: self.get_strongest_patterns(&store, 5),
        }
    }

    /// Get the strongest patterns
    fn get_strongest_patterns(&self, store: &PatternStore, limit: usize) -> Vec<String> {
        let mut patterns: Vec<_> = store.patterns.values().collect();
        patterns.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());

        patterns
            .into_iter()
            .take(limit)
            .map(|p| p.pattern.description.clone())
            .collect()
    }

    // === Safety Pattern Analysis Methods ===

    /// Analyze file operations for dangerous patterns and safety concerns
    pub async fn analyze_operation_safety(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<SafetyPatternAnalysis> {
        info!(
            "🔍 Analyzing {} operations for safety patterns",
            operations.len()
        );

        // Generate cache key for this analysis
        let cache_key = self.generate_safety_cache_key(operations, context);

        // Check cache first
        {
            let cache = self.safety_cache.read().await;
            if let Some(cached) = cache.peek(&cache_key) {
                debug!("📋 Using cached safety analysis");
                return Ok(cached.clone());
            }
        }

        // Analyze dangerous patterns
        let dangerous_patterns = self.detect_dangerous_patterns(operations, context).await?;

        // Analyze operation clustering
        let operation_clusters = self
            .analyze_operation_clustering(operations, context)
            .await?;

        // Detect anti-patterns
        let anti_patterns = self.detect_anti_patterns(operations, context).await?;

        // Calculate overall safety score
        let safety_score =
            self.calculate_safety_score(&dangerous_patterns, &operation_clusters, &anti_patterns);

        // Generate safety recommendations
        let safety_recommendations = self
            .generate_safety_recommendations(
                &dangerous_patterns,
                &operation_clusters,
                &anti_patterns,
                context,
            )
            .await?;

        let analysis = SafetyPatternAnalysis {
            dangerous_patterns,
            operation_clusters,
            anti_patterns,
            safety_score,
            safety_recommendations,
            analyzed_at: SystemTime::now(),
        };

        // Cache the result
        {
            let mut cache = self.safety_cache.write().await;
            cache.put(cache_key, analysis.clone());
        }

        // Record for trend analysis
        self.record_safety_analysis(&analysis, operations).await?;

        info!(
            "📊 Safety analysis complete: {:.1}% safety score",
            analysis.safety_score
        );
        Ok(analysis)
    }

    /// Detect dangerous patterns in file operations
    async fn detect_dangerous_patterns(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<Vec<DangerousPattern>> {
        let mut dangerous_patterns = Vec::new();

        // Check for mass deletion
        if let Some(mass_deletion) = self.detect_mass_deletion(operations).await? {
            dangerous_patterns.push(mass_deletion);
        }

        // Check for overwrite without backup
        if let Some(overwrite) = self.detect_overwrite_without_backup(operations).await? {
            dangerous_patterns.push(overwrite);
        }

        // Check for breaking changes
        if let Some(breaking_change) = self.detect_breaking_changes(operations, context).await? {
            dangerous_patterns.push(breaking_change);
        }

        // Check for data loss patterns
        if let Some(data_loss) = self.detect_data_loss_patterns(operations).await? {
            dangerous_patterns.push(data_loss);
        }

        // Check for security vulnerabilities
        if let Some(security) = self
            .detect_security_vulnerabilities(operations, context)
            .await?
        {
            dangerous_patterns.push(security);
        }

        debug!(
            "🚨 Detected {} dangerous patterns",
            dangerous_patterns.len()
        );
        Ok(dangerous_patterns)
    }

    /// Detect mass deletion patterns
    async fn detect_mass_deletion(
        &self,
        operations: &[FileOperation],
    ) -> Result<Option<DangerousPattern>> {
        let delete_count = operations
            .iter()
            .filter(|op| matches!(op, FileOperation::Delete { .. }))
            .count();

        // Consider it mass deletion if >5 deletions or >30% of operations are deletions
        let is_mass_deletion = delete_count > 5
            || (operations.len() > 1 && delete_count as f32 / operations.len() as f32 > 0.3);

        if is_mass_deletion {
            return Ok(Some(DangerousPattern {
                pattern_type: DangerousPatternType::MassDeletion,
                description: format!(
                    "Mass deletion detected: {} files being deleted",
                    delete_count
                ),
                severity: if delete_count > 10 {
                    DangerousSeverity::Critical
                } else {
                    DangerousSeverity::High
                },
                mitigation: Some(
                    "Create backups before deletion, consider soft deletion instead".to_string(),
                ),
            }));
        }

        Ok(None)
    }

    /// Detect overwrite without backup patterns
    async fn detect_overwrite_without_backup(
        &self,
        operations: &[FileOperation],
    ) -> Result<Option<DangerousPattern>> {
        let update_count = operations
            .iter()
            .filter(|op| matches!(op, FileOperation::Update { .. }))
            .count();

        // Check if there are many updates without explicit backup operations
        if update_count > 3 {
            return Ok(Some(DangerousPattern {
                pattern_type: DangerousPatternType::OverwriteWithoutBackup,
                description: format!(
                    "Multiple file overwrites detected: {} updates without backup",
                    update_count
                ),
                severity: DangerousSeverity::Medium,
                mitigation: Some(
                    "Enable automatic backups or create explicit backup operations".to_string(),
                ),
            }));
        }

        Ok(None)
    }

    /// Detect breaking changes in operations
    async fn detect_breaking_changes(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<Option<DangerousPattern>> {
        let breaking_indicators = [
            "breaking",
            "deprecated",
            "removed",
            "deleted",
            "incompatible",
            "major version",
            "api change",
            "interface change",
        ];

        let has_breaking_change = operations.iter().any(|op| {
            let content = match op {
                FileOperation::Create { content, .. } => content,
                FileOperation::Update { content, .. } => content,
                FileOperation::Append { content, .. } => content,
                _ => return false,
            };

            breaking_indicators.iter().any(|indicator| {
                content.to_lowercase().contains(indicator)
                    || context.source_question.to_lowercase().contains(indicator)
            })
        });

        if has_breaking_change {
            return Ok(Some(DangerousPattern {
                pattern_type: DangerousPatternType::BreakingChange,
                description: "Potential breaking change detected in operation content".to_string(),
                severity: DangerousSeverity::High,
                mitigation: Some(
                    "Review API compatibility, add deprecation notices, update documentation"
                        .to_string(),
                ),
            }));
        }

        Ok(None)
    }

    /// Detect data loss patterns
    async fn detect_data_loss_patterns(
        &self,
        operations: &[FileOperation],
    ) -> Result<Option<DangerousPattern>> {
        let dangerous_patterns = self.dangerous_file_patterns.read().await;

        let data_loss_risk = operations.iter().any(|op| {
            let path = match op {
                FileOperation::Create { path, .. } => path,
                FileOperation::Update { path, .. } => path,
                FileOperation::Append { path, .. } => path,
                FileOperation::Delete { path } => path,
                FileOperation::Rename { from, .. } => from,
            };

            let path_str = path.to_string_lossy().to_lowercase();
            dangerous_patterns.iter().any(|pattern| {
                if pattern.contains('*') {
                    // Wildcard pattern matching
                    let pattern_prefix = pattern.replace("*", "");
                    path_str.contains(&pattern_prefix)
                } else {
                    path_str.contains(pattern)
                }
            })
        });

        if data_loss_risk {
            return Ok(Some(DangerousPattern {
                pattern_type: DangerousPatternType::DataLoss,
                description: "Operations target sensitive files that could result in data loss"
                    .to_string(),
                severity: DangerousSeverity::Critical,
                mitigation: Some(
                    "Create full backups, verify file importance, use read-only mode first"
                        .to_string(),
                ),
            }));
        }

        Ok(None)
    }

    /// Detect security vulnerabilities in operations
    async fn detect_security_vulnerabilities(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<Option<DangerousPattern>> {
        let security_risks = [
            "password",
            "secret",
            "key",
            "token",
            "credential",
            "auth",
            "private",
            "confidential",
            "api_key",
            "access_token",
        ];

        let has_security_risk = operations.iter().any(|op| {
            let (path, content) = match op {
                FileOperation::Create { path, content } => (path, Some(content)),
                FileOperation::Update { path, content } => (path, Some(content)),
                FileOperation::Append { path, content } => (path, Some(content)),
                FileOperation::Delete { path } => (path, None),
                FileOperation::Rename { from, .. } => (from, None),
            };

            let path_str = path.to_string_lossy().to_lowercase();
            let content_risk = content.map_or(false, |c| {
                security_risks
                    .iter()
                    .any(|risk| c.to_lowercase().contains(risk))
            });
            let path_risk = security_risks.iter().any(|risk| path_str.contains(risk));

            content_risk || path_risk
        });

        if has_security_risk {
            return Ok(Some(DangerousPattern {
                pattern_type: DangerousPatternType::SecurityVulnerability,
                description: "Operations involve sensitive security-related files or content".to_string(),
                severity: DangerousSeverity::Critical,
                mitigation: Some("Review for secrets exposure, use environment variables, encrypt sensitive data".to_string()),
            }));
        }

        Ok(None)
    }

    /// Analyze operation clustering for intelligent grouping
    async fn analyze_operation_clustering(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<Vec<OperationCluster>> {
        if operations.len() < 2 {
            return Ok(Vec::new());
        }

        let mut clusters = Vec::new();

        // Group operations by type and analyze patterns
        let mut operation_groups: HashMap<String, Vec<&FileOperation>> = HashMap::new();

        for operation in operations {
            let group_key = self.classify_operation_group(operation, context);
            operation_groups
                .entry(group_key)
                .or_insert_with(Vec::new)
                .push(operation);
        }

        // Create clusters for each significant group
        for (group_type, group_operations) in operation_groups {
            if group_operations.len() >= 2 {
                let cluster_type =
                    self.determine_cluster_type(&group_type, &group_operations, context);
                let execution_order = self.determine_execution_order(&group_operations);

                clusters.push(OperationCluster {
                    name: format!("{} Operations", group_type),
                    operations: group_operations.into_iter().cloned().collect(),
                    cluster_type,
                    execution_order,
                });
            }
        }

        debug!("🔗 Created {} operation clusters", clusters.len());
        Ok(clusters)
    }

    /// Classify operation into logical groups
    fn classify_operation_group(
        &self,
        operation: &FileOperation,
        context: &OperationContext,
    ) -> String {
        match operation {
            FileOperation::Create { path, .. } => {
                if self.is_test_file(path) {
                    "Test Creation".to_string()
                } else if self.is_config_file(path) {
                    "Configuration".to_string()
                } else if self.is_documentation_file(path) {
                    "Documentation".to_string()
                } else {
                    "File Creation".to_string()
                }
            }
            FileOperation::Update { path, .. } => {
                if self.is_test_file(path) {
                    "Test Update".to_string()
                } else if self.is_config_file(path) {
                    "Configuration Update".to_string()
                } else {
                    "File Update".to_string()
                }
            }
            FileOperation::Delete { path } => {
                if self.is_temporary_file(path) {
                    "Cleanup".to_string()
                } else {
                    "File Deletion".to_string()
                }
            }
            FileOperation::Rename { .. } => "File Reorganization".to_string(),
            FileOperation::Append { .. } => "File Append".to_string(),
        }
    }

    /// Determine cluster type based on operations
    fn determine_cluster_type(
        &self,
        group_type: &str,
        operations: &[&FileOperation],
        context: &OperationContext,
    ) -> ClusterType {
        match group_type {
            s if s.contains("Test") => ClusterType::Testing,
            s if s.contains("Configuration") => ClusterType::Migration,
            s if s.contains("Documentation") => ClusterType::Cleanup,
            s if s.contains("Cleanup") => ClusterType::Cleanup,
            s if s.contains("Reorganization") => ClusterType::Refactoring,
            _ => {
                // Analyze context for more specific classification
                let question_lower = context.source_question.to_lowercase();
                if question_lower.contains("refactor") || question_lower.contains("restructure") {
                    ClusterType::Refactoring
                } else if question_lower.contains("fix") || question_lower.contains("bug") {
                    ClusterType::BugFix
                } else if question_lower.contains("feature") || question_lower.contains("add") {
                    ClusterType::FeatureAddition
                } else {
                    ClusterType::Refactoring
                }
            }
        }
    }

    /// Determine optimal execution order for operations
    fn determine_execution_order(&self, operations: &[&FileOperation]) -> Vec<usize> {
        // Simple ordering: Creates first, then updates/appends, then renames, finally deletes
        let mut ordered_indices = Vec::new();

        // Phase 1: Creates
        for (i, op) in operations.iter().enumerate() {
            if matches!(op, FileOperation::Create { .. }) {
                ordered_indices.push(i);
            }
        }

        // Phase 2: Updates and Appends
        for (i, op) in operations.iter().enumerate() {
            if matches!(
                op,
                FileOperation::Update { .. } | FileOperation::Append { .. }
            ) {
                ordered_indices.push(i);
            }
        }

        // Phase 3: Renames
        for (i, op) in operations.iter().enumerate() {
            if matches!(op, FileOperation::Rename { .. }) {
                ordered_indices.push(i);
            }
        }

        // Phase 4: Deletes
        for (i, op) in operations.iter().enumerate() {
            if matches!(op, FileOperation::Delete { .. }) {
                ordered_indices.push(i);
            }
        }

        ordered_indices
    }

    /// Detect anti-patterns in operations
    async fn detect_anti_patterns(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Result<Vec<AntiPattern>> {
        let mut anti_patterns = Vec::new();

        // Check for duplicate file operations
        if let Some(duplicate) = self.detect_duplicate_operations(operations) {
            anti_patterns.push(duplicate);
        }

        // Check for inefficient operation sequences
        if let Some(inefficient) = self.detect_inefficient_sequences(operations) {
            anti_patterns.push(inefficient);
        }

        // Check for risky operation combinations
        if let Some(risky_combo) = self.detect_risky_combinations(operations, context) {
            anti_patterns.push(risky_combo);
        }

        // Check for poor separation of concerns
        if let Some(poor_separation) = self.detect_poor_separation(operations, context) {
            anti_patterns.push(poor_separation);
        }

        debug!("⚠️ Detected {} anti-patterns", anti_patterns.len());
        Ok(anti_patterns)
    }

    /// Detect duplicate or redundant operations
    fn detect_duplicate_operations(&self, operations: &[FileOperation]) -> Option<AntiPattern> {
        let mut file_operations: HashMap<PathBuf, Vec<&FileOperation>> = HashMap::new();

        for operation in operations {
            let path = match operation {
                FileOperation::Create { path, .. } => path,
                FileOperation::Update { path, .. } => path,
                FileOperation::Append { path, .. } => path,
                FileOperation::Delete { path } => path,
                FileOperation::Rename { from, .. } => from,
            };

            file_operations
                .entry(path.clone())
                .or_insert_with(Vec::new)
                .push(operation);
        }

        // Check for files with multiple operations
        let duplicates: Vec<_> = file_operations
            .iter()
            .filter(|(_, ops)| ops.len() > 1)
            .collect();

        if !duplicates.is_empty() {
            return Some(AntiPattern {
                pattern_type: "Duplicate Operations".to_string(),
                description: format!(
                    "Multiple operations on same files: {} files affected",
                    duplicates.len()
                ),
                alternative: Some("Combine operations or review for necessity".to_string()),
            });
        }

        None
    }

    /// Detect inefficient operation sequences
    fn detect_inefficient_sequences(&self, operations: &[FileOperation]) -> Option<AntiPattern> {
        // Look for create -> delete sequences
        let mut creates = HashMap::new();
        let mut deletes = HashSet::new();

        for (i, operation) in operations.iter().enumerate() {
            match operation {
                FileOperation::Create { path, .. } => {
                    creates.insert(path, i);
                }
                FileOperation::Delete { path } => {
                    if creates.contains_key(path) {
                        deletes.insert(path);
                    }
                }
                _ => {}
            }
        }

        if !deletes.is_empty() {
            return Some(AntiPattern {
                pattern_type: "Create-Delete Inefficiency".to_string(),
                description: format!(
                    "Files created and then deleted in same operation set: {} files",
                    deletes.len()
                ),
                alternative: Some(
                    "Remove unnecessary create operations or review workflow".to_string(),
                ),
            });
        }

        None
    }

    /// Detect risky operation combinations
    fn detect_risky_combinations(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Option<AntiPattern> {
        let has_deletes = operations
            .iter()
            .any(|op| matches!(op, FileOperation::Delete { .. }));
        let has_creates = operations
            .iter()
            .any(|op| matches!(op, FileOperation::Create { .. }));
        let has_renames = operations
            .iter()
            .any(|op| matches!(op, FileOperation::Rename { .. }));

        // Risky: Deletes combined with creates/renames
        if has_deletes && (has_creates || has_renames) {
            return Some(AntiPattern {
                pattern_type: "Risky Operation Mix".to_string(),
                description: "Combining deletions with creates/renames increases error risk"
                    .to_string(),
                alternative: Some(
                    "Separate destructive operations from constructive ones".to_string(),
                ),
            });
        }

        None
    }

    /// Detect poor separation of concerns
    fn detect_poor_separation(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> Option<AntiPattern> {
        let mut concerns = HashSet::new();

        for operation in operations {
            if let Some(path) = self.get_operation_path(operation) {
                if self.is_test_file(&path) {
                    concerns.insert("testing");
                }
                if self.is_config_file(&path) {
                    concerns.insert("configuration");
                }
                if self.is_documentation_file(&path) {
                    concerns.insert("documentation");
                }
                if self.is_source_file(&path) {
                    concerns.insert("source_code");
                }
            }
        }

        // If operations span multiple concerns, suggest separation
        if concerns.len() > 2 {
            return Some(AntiPattern {
                pattern_type: "Mixed Concerns".to_string(),
                description: format!(
                    "Operations span {} different concerns in single batch",
                    concerns.len()
                ),
                alternative: Some(
                    "Separate operations by concern (tests, docs, config, code)".to_string(),
                ),
            });
        }

        None
    }

    /// Calculate overall safety score
    fn calculate_safety_score(
        &self,
        dangerous_patterns: &[DangerousPattern],
        operation_clusters: &[OperationCluster],
        anti_patterns: &[AntiPattern],
    ) -> f32 {
        let mut safety_score = 100.0;

        // Deduct points for dangerous patterns
        for pattern in dangerous_patterns {
            let deduction = match pattern.severity {
                DangerousSeverity::Critical => 30.0,
                DangerousSeverity::High => 20.0,
                DangerousSeverity::Medium => 10.0,
                DangerousSeverity::Low => 5.0,
            };
            safety_score -= deduction;
        }

        // Deduct points for anti-patterns
        safety_score -= anti_patterns.len() as f32 * 5.0;

        // Small boost for well-organized clusters
        if operation_clusters.len() > 0 {
            safety_score += 2.0;
        }

        safety_score.clamp(0.0, 100.0)
    }

    /// Generate safety recommendations
    async fn generate_safety_recommendations(
        &self,
        dangerous_patterns: &[DangerousPattern],
        operation_clusters: &[OperationCluster],
        anti_patterns: &[AntiPattern],
        context: &OperationContext,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Recommendations based on dangerous patterns
        for pattern in dangerous_patterns {
            if let Some(mitigation) = &pattern.mitigation {
                recommendations.push(format!("🚨 {}: {}", pattern.description, mitigation));
            }
        }

        // Recommendations based on anti-patterns
        for anti_pattern in anti_patterns {
            if let Some(alternative) = &anti_pattern.alternative {
                recommendations.push(format!("⚠️ {}: {}", anti_pattern.pattern_type, alternative));
            }
        }

        // General safety recommendations
        if dangerous_patterns.len() > 2 {
            recommendations.push(
                "💡 Consider breaking this operation into smaller, safer batches".to_string(),
            );
        }

        if operation_clusters
            .iter()
            .any(|c| matches!(c.cluster_type, ClusterType::Migration))
        {
            recommendations
                .push("🔄 Test migration operations in a safe environment first".to_string());
        }

        // Repository-specific recommendations
        if context.related_files.len() > 10 {
            recommendations
                .push("📁 Large number of related files - ensure proper validation".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("✅ Operations appear safe based on current analysis".to_string());
        }

        Ok(recommendations)
    }

    /// Record safety analysis for trend tracking
    async fn record_safety_analysis(
        &self,
        analysis: &SafetyPatternAnalysis,
        operations: &[FileOperation],
    ) -> Result<()> {
        let mut history = self.safety_history.write().await;

        for operation in operations {
            let operation_hash = self.generate_operation_hash(operation);
            let operation_type = self.get_operation_type_name(operation);

            history.push(OperationSafetyRecord {
                operation_hash,
                safety_score: analysis.safety_score,
                dangerous_patterns_count: analysis.dangerous_patterns.len(),
                timestamp: SystemTime::now(),
                operation_type,
            });
        }

        // Keep only recent history (last 1000 records)
        let history_len = history.len();
        if history_len > 1000 {
            history.drain(0..history_len - 1000);
        }

        // Clear safety metrics cache since data changed
        *self.safety_metrics_cache.write().await = None;

        Ok(())
    }

    /// Get repository safety metrics
    pub async fn get_repository_safety_metrics(
        &self,
        context: &OperationContext,
    ) -> Result<RepositorySafetyMetrics> {
        // Check cache first (1 hour expiry)
        {
            let cache = self.safety_metrics_cache.read().await;
            if let Some((metrics, timestamp)) = &*cache {
                if timestamp.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(3600) {
                    return Ok(metrics.clone());
                }
            }
        }

        let history = self.safety_history.read().await;

        // Calculate metrics
        let risky_operations = history
            .iter()
            .filter(|record| record.safety_score < 70.0)
            .count();

        let safety_trend = self.calculate_safety_trend(&history);

        let common_dangers = self.analyze_common_danger_patterns(&history).await?;

        let high_risk_files = self.identify_high_risk_files(&history);

        let improvements = self
            .suggest_safety_improvements(&history, risky_operations)
            .await?;

        let metrics = RepositorySafetyMetrics {
            risky_operations,
            safety_trend,
            common_dangers,
            high_risk_files,
            improvements,
        };

        // Cache the result
        {
            let mut cache = self.safety_metrics_cache.write().await;
            *cache = Some((metrics.clone(), SystemTime::now()));
        }

        Ok(metrics)
    }

    /// Calculate safety trend over time
    fn calculate_safety_trend(&self, history: &[OperationSafetyRecord]) -> SafetyTrend {
        if history.len() < 10 {
            return SafetyTrend::InsufficientData;
        }

        // Split history into two halves
        let mid = history.len() / 2;
        let first_half = &history[..mid];
        let second_half = &history[mid..];

        let first_avg =
            first_half.iter().map(|r| r.safety_score).sum::<f32>() / first_half.len() as f32;
        let second_avg =
            second_half.iter().map(|r| r.safety_score).sum::<f32>() / second_half.len() as f32;

        let improvement_rate = (second_avg - first_avg) / first_avg;

        if improvement_rate > 0.05 {
            SafetyTrend::Improving {
                rate: improvement_rate,
            }
        } else if improvement_rate < -0.05 {
            SafetyTrend::Declining {
                rate: improvement_rate.abs(),
            }
        } else {
            SafetyTrend::Stable
        }
    }

    /// Analyze common danger patterns from history
    async fn analyze_common_danger_patterns(
        &self,
        history: &[OperationSafetyRecord],
    ) -> Result<Vec<DangerousPatternType>> {
        // This would typically analyze the actual danger patterns from history
        // For now, return common patterns based on operation types
        let mut pattern_counts = HashMap::new();

        for record in history {
            if record.dangerous_patterns_count > 0 {
                // Infer pattern types from operation types and safety scores
                if record.operation_type == "delete" && record.safety_score < 50.0 {
                    *pattern_counts
                        .entry(DangerousPatternType::MassDeletion)
                        .or_insert(0) += 1;
                } else if record.operation_type == "update" && record.safety_score < 60.0 {
                    *pattern_counts
                        .entry(DangerousPatternType::OverwriteWithoutBackup)
                        .or_insert(0) += 1;
                }
            }
        }

        let mut common_patterns: Vec<_> = pattern_counts
            .into_iter()
            .filter(|(_, count)| *count >= 3)
            .map(|(pattern, _)| pattern)
            .collect();

        // Sort by frequency (in a real implementation)
        common_patterns.truncate(5);

        Ok(common_patterns)
    }

    /// Identify files with highest risk based on history
    fn identify_high_risk_files(&self, history: &[OperationSafetyRecord]) -> Vec<PathBuf> {
        // This would analyze actual file paths from operation history
        // For now, return common high-risk file patterns
        vec![
            PathBuf::from(".env"),
            PathBuf::from("config/database.yml"),
            PathBuf::from("package-lock.json"),
            PathBuf::from("Cargo.lock"),
        ]
    }

    /// Suggest safety improvements
    async fn suggest_safety_improvements(
        &self,
        history: &[OperationSafetyRecord],
        risky_count: usize,
    ) -> Result<Vec<String>> {
        let mut improvements = Vec::new();

        if risky_count > history.len() / 4 {
            improvements.push("🔧 Consider implementing automated backup system".to_string());
            improvements.push("📋 Add pre-operation validation checks".to_string());
        }

        if history
            .iter()
            .any(|r| r.operation_type == "delete" && r.safety_score < 40.0)
        {
            improvements.push("🗑️ Implement soft deletion for critical operations".to_string());
        }

        if history.len() > 50 {
            improvements.push(
                "📊 Regular safety audit recommended due to high operation volume".to_string(),
            );
        }

        improvements.push("🛡️ Enable operation confirmation for high-risk patterns".to_string());
        improvements.push("📝 Consider maintaining operation logs for audit trail".to_string());

        Ok(improvements)
    }

    // === Helper Methods ===

    /// Generate cache key for safety analysis
    fn generate_safety_cache_key(
        &self,
        operations: &[FileOperation],
        context: &OperationContext,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for operation in operations {
            format!("{:?}", operation).hash(&mut hasher);
        }
        context.source_question.hash(&mut hasher);
        format!("safety_{:x}", hasher.finish())
    }

    /// Generate hash for operation
    fn generate_operation_hash(&self, operation: &FileOperation) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}", operation).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get operation type name
    fn get_operation_type_name(&self, operation: &FileOperation) -> String {
        match operation {
            FileOperation::Create { .. } => "create".to_string(),
            FileOperation::Update { .. } => "update".to_string(),
            FileOperation::Append { .. } => "append".to_string(),
            FileOperation::Delete { .. } => "delete".to_string(),
            FileOperation::Rename { .. } => "rename".to_string(),
        }
    }

    /// Get path from operation
    fn get_operation_path(&self, operation: &FileOperation) -> Option<PathBuf> {
        match operation {
            FileOperation::Create { path, .. } => Some(path.clone()),
            FileOperation::Update { path, .. } => Some(path.clone()),
            FileOperation::Append { path, .. } => Some(path.clone()),
            FileOperation::Delete { path } => Some(path.clone()),
            FileOperation::Rename { from, .. } => Some(from.clone()),
        }
    }

    /// Check if file is a test file
    fn is_test_file(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.contains("test")
            || path_str.contains("spec")
            || path_str.ends_with("_test.rs")
            || path_str.ends_with(".test.js")
            || path_str.ends_with(".spec.js")
    }

    /// Check if file is a configuration file
    fn is_config_file(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.contains("config")
            || path_str.ends_with(".toml")
            || path_str.ends_with(".yml")
            || path_str.ends_with(".yaml")
            || path_str.ends_with(".json")
            || path_str.contains(".env")
    }

    /// Check if file is documentation
    fn is_documentation_file(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.ends_with(".md")
            || path_str.ends_with(".txt")
            || path_str.contains("readme")
            || path_str.contains("doc")
            || path_str.contains("manual")
    }

    /// Check if file is source code
    fn is_source_file(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.ends_with(".rs")
            || path_str.ends_with(".js")
            || path_str.ends_with(".ts")
            || path_str.ends_with(".py")
            || path_str.ends_with(".java")
            || path_str.ends_with(".cpp")
            || path_str.ends_with(".c")
            || path_str.ends_with(".go")
    }

    /// Check if file is temporary
    fn is_temporary_file(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.starts_with("/tmp/")
            || path_str.contains("temp")
            || path_str.ends_with(".tmp")
            || path_str.ends_with(".bak")
            || path_str.ends_with("~")
    }

    /// Calculate cosine similarity between two embeddings
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            (dot_product / (norm_a * norm_b)) as f64
        }
    }
}

/// Statistics about patterns
#[derive(Debug, Clone)]
pub struct PatternStats {
    pub total_patterns: usize,
    pub pattern_types: HashMap<String, usize>,
    pub evolution_events: usize,
    pub strongest_patterns: Vec<String>,
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pattern_detection() {
        // Test pattern detection logic
    }
}
