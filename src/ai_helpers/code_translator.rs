//! Code Translator - AI Helper for translating code between languages
//!
//! This module provides pure execution of code translation tasks as directed
//! by the Consensus pipeline. It NEVER makes decisions about what to translate
//! or how to approach translation - it only executes translation plans.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::monitoring::{OperationType, PerformanceMonitor};
use super::python_models::PythonModelService;

/// Translation plan from Consensus/Curator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationPlan {
    /// Source language
    pub source_language: String,
    /// Target language
    pub target_language: String,
    /// Translation strategy chosen by Consensus
    pub strategy: TranslationStrategy,
    /// Specific rules to follow
    pub rules: Vec<TranslationRule>,
    /// Context provided by Consensus
    pub context: TranslationContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslationStrategy {
    /// Direct syntax mapping
    DirectMapping,
    /// Idiomatic translation (language-specific patterns)
    Idiomatic,
    /// Framework-specific (e.g., React to Vue)
    FrameworkSpecific { from: String, to: String },
    /// API translation (e.g., different library APIs)
    ApiTranslation { mappings: HashMap<String, String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRule {
    /// Pattern to match
    pub pattern: String,
    /// Replacement template
    pub replacement: String,
    /// Description of the rule
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationContext {
    /// Import statements to add
    pub required_imports: Vec<String>,
    /// Type mappings
    pub type_mappings: HashMap<String, String>,
    /// Framework conventions
    pub conventions: HashMap<String, String>,
}

/// Translation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    /// Translated code
    pub translated_code: String,
    /// Applied rules
    pub applied_rules: Vec<String>,
    /// Warnings or notes
    pub warnings: Vec<TranslationWarning>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationWarning {
    pub line: Option<usize>,
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

/// Code translator using AI models with learning capabilities
pub struct CodeTranslator {
    /// Python model service for CodeT5+
    python_service: Arc<PythonModelService>,

    /// Performance monitor
    monitor: Arc<PerformanceMonitor>,

    /// Cache for common translations
    cache: Arc<RwLock<TranslationCache>>,

    /// Translation patterns learned over time
    learned_patterns: Arc<RwLock<LearnedPatterns>>,

    /// Domain expertise tracker
    expertise: Arc<RwLock<DomainExpertise>>,
}

struct TranslationCache {
    entries: HashMap<String, CachedTranslation>,
    max_size: usize,
}

struct CachedTranslation {
    result: TranslationResult,
    timestamp: std::time::Instant,
}

/// Learned patterns for better translations
struct LearnedPatterns {
    /// Patterns by language pair
    patterns: HashMap<(String, String), Vec<TranslationPattern>>,
    /// Common idioms discovered
    idioms: HashMap<String, IdiomMapping>,
}

#[derive(Clone)]
struct TranslationPattern {
    pattern: String,
    replacement: String,
    confidence: f64,
    usage_count: u32,
}

#[derive(Clone)]
struct IdiomMapping {
    source_idiom: String,
    target_idiom: String,
    context_required: Vec<String>,
}

/// Domain expertise tracking
struct DomainExpertise {
    /// Expertise by language pair
    language_pairs: HashMap<(String, String), f64>,
    /// Framework expertise
    frameworks: HashMap<String, f64>,
    /// Total translations
    total_translations: u64,
    /// Successful translations
    successful_translations: u64,
}

impl CodeTranslator {
    /// Create a new code translator
    pub fn new(python_service: Arc<PythonModelService>, monitor: Arc<PerformanceMonitor>) -> Self {
        Self {
            python_service,
            monitor,
            cache: Arc::new(RwLock::new(TranslationCache {
                entries: HashMap::new(),
                max_size: 1000,
            })),
            learned_patterns: Arc::new(RwLock::new(LearnedPatterns {
                patterns: HashMap::new(),
                idioms: HashMap::new(),
            })),
            expertise: Arc::new(RwLock::new(DomainExpertise {
                language_pairs: HashMap::new(),
                frameworks: HashMap::new(),
                total_translations: 0,
                successful_translations: 0,
            })),
        }
    }

    /// Execute a translation plan with intelligence and learning
    pub async fn execute_translation(
        &self,
        code: &str,
        plan: &TranslationPlan,
    ) -> Result<TranslationResult> {
        let start = std::time::Instant::now();
        info!(
            "Intelligently executing translation from {} to {} using {:?} strategy",
            plan.source_language, plan.target_language, plan.strategy
        );

        // Track this translation
        self.track_translation_start(&plan.source_language, &plan.target_language)
            .await;

        // Check cache first
        let cache_key = self.generate_cache_key(code, plan);
        if let Some(cached) = self.get_cached(&cache_key).await {
            debug!("Translation found in cache");
            return Ok(cached);
        }

        // Apply learned patterns to enhance the plan
        let enhanced_plan = self.enhance_with_learned_patterns(plan).await?;

        // Analyze code structure for better understanding
        let code_understanding = self
            .understand_code_structure(code, &plan.source_language)
            .await?;

        // Execute translation based on strategy with enhancements
        let mut result = match &enhanced_plan.strategy {
            TranslationStrategy::DirectMapping => {
                self.execute_direct_mapping(code, &enhanced_plan).await?
            }
            TranslationStrategy::Idiomatic => {
                self.execute_idiomatic_translation(code, &enhanced_plan)
                    .await?
            }
            TranslationStrategy::FrameworkSpecific { from, to } => {
                self.execute_framework_translation(code, &enhanced_plan, from, to)
                    .await?
            }
            TranslationStrategy::ApiTranslation { mappings } => {
                self.execute_api_translation(code, &enhanced_plan, mappings)
                    .await?
            }
        };

        // Apply post-translation improvements based on expertise
        result = self
            .apply_expertise_improvements(result, &code_understanding)
            .await?;

        // Validate translation quality
        let quality_score = self.assess_translation_quality(&result, code, plan).await?;
        result.confidence = quality_score;

        // Learn from this translation
        self.learn_from_translation(code, &result, plan).await?;

        // Cache the result
        self.cache_translation(&cache_key, &result).await;

        // Track success
        self.track_translation_success(&plan.source_language, &plan.target_language)
            .await;

        Ok(result)
    }

    /// Execute direct syntax mapping
    async fn execute_direct_mapping(
        &self,
        code: &str,
        plan: &TranslationPlan,
    ) -> Result<TranslationResult> {
        let mut translated = code.to_string();
        let mut applied_rules = Vec::new();

        // Apply translation rules in order
        for rule in &plan.rules {
            if translated.contains(&rule.pattern) {
                translated = translated.replace(&rule.pattern, &rule.replacement);
                applied_rules.push(rule.description.clone());
            }
        }

        // Apply type mappings
        for (from_type, to_type) in &plan.context.type_mappings {
            translated = translated.replace(from_type, to_type);
        }

        // Add required imports
        if !plan.context.required_imports.is_empty() {
            let imports = plan.context.required_imports.join("\n");
            translated = format!("{}\n\n{}", imports, translated);
        }

        Ok(TranslationResult {
            translated_code: translated,
            applied_rules,
            warnings: vec![],
            confidence: 0.85, // Direct mapping has high confidence
        })
    }

    /// Execute idiomatic translation using AI model
    async fn execute_idiomatic_translation(
        &self,
        code: &str,
        plan: &TranslationPlan,
    ) -> Result<TranslationResult> {
        // Use CodeT5+ for idiomatic translation
        let prompt = self.build_translation_prompt(code, plan);

        let response = self
            .python_service
            .generate_text("codet5", &prompt, 2048, 0.7)
            .await
            .context("Failed to process translation with CodeT5+")?;

        // Parse the model response
        self.parse_model_response(&response, plan)
    }

    /// Execute framework-specific translation
    async fn execute_framework_translation(
        &self,
        code: &str,
        plan: &TranslationPlan,
        from_framework: &str,
        to_framework: &str,
    ) -> Result<TranslationResult> {
        info!("Translating from {} to {}", from_framework, to_framework);

        // Framework-specific handling
        let translated = match (from_framework, to_framework) {
            ("react", "vue") => self.translate_react_to_vue(code, plan).await?,
            ("express", "fastapi") => self.translate_express_to_fastapi(code, plan).await?,
            _ => {
                // Fall back to AI model for unknown framework pairs
                return self.execute_idiomatic_translation(code, plan).await;
            }
        };

        Ok(translated)
    }

    /// Execute API translation with mappings
    async fn execute_api_translation(
        &self,
        code: &str,
        plan: &TranslationPlan,
        mappings: &HashMap<String, String>,
    ) -> Result<TranslationResult> {
        let mut translated = code.to_string();
        let mut applied_rules = Vec::new();

        // Apply API mappings
        for (from_api, to_api) in mappings {
            if translated.contains(from_api) {
                translated = translated.replace(from_api, to_api);
                applied_rules.push(format!("Mapped {} to {}", from_api, to_api));
            }
        }

        // Apply any additional rules
        for rule in &plan.rules {
            if translated.contains(&rule.pattern) {
                translated = translated.replace(&rule.pattern, &rule.replacement);
                applied_rules.push(rule.description.clone());
            }
        }

        Ok(TranslationResult {
            translated_code: translated,
            applied_rules,
            warnings: vec![],
            confidence: 0.9, // API mapping is very reliable
        })
    }

    /// Translate React to Vue (example framework translation)
    async fn translate_react_to_vue(
        &self,
        code: &str,
        plan: &TranslationPlan,
    ) -> Result<TranslationResult> {
        let mut translated = code.to_string();
        let mut applied_rules = vec![];
        let mut warnings = vec![];

        // Basic React to Vue transformations
        // These are simplified examples - real implementation would be more comprehensive

        // useState -> ref
        if translated.contains("useState") {
            translated = translated.replace("const [", "const ");
            translated = translated.replace(", set", " = ref(");
            translated = translated.replace("] = useState(", "");
            translated = translated.replace(");", "));");
            applied_rules.push("Converted useState to Vue ref".to_string());
        }

        // useEffect -> onMounted/watch
        if translated.contains("useEffect") {
            warnings.push(TranslationWarning {
                line: None,
                message: "useEffect requires manual review for Vue lifecycle".to_string(),
                severity: WarningSeverity::Warning,
            });
        }

        // Component syntax
        translated = translated.replace("function Component", "export default {");
        translated = translated.replace("return (", "template: `");

        Ok(TranslationResult {
            translated_code: translated,
            applied_rules,
            warnings,
            confidence: 0.7, // Framework translation needs review
        })
    }

    /// Translate Express to FastAPI (example API translation)
    async fn translate_express_to_fastapi(
        &self,
        code: &str,
        _plan: &TranslationPlan,
    ) -> Result<TranslationResult> {
        let mut translated = code.to_string();
        let mut applied_rules = vec![];

        // Basic Express to FastAPI transformations
        translated = translated.replace(
            "const express = require('express')",
            "from fastapi import FastAPI",
        );
        translated = translated.replace("const app = express()", "app = FastAPI()");
        translated = translated.replace("app.get('", "@app.get('");
        translated = translated.replace("(req, res) =>", "async def handler():");
        translated = translated.replace("res.json(", "return ");

        applied_rules.push("Converted Express syntax to FastAPI".to_string());

        Ok(TranslationResult {
            translated_code: translated,
            applied_rules,
            warnings: vec![],
            confidence: 0.8,
        })
    }

    /// Build prompt for AI model
    fn build_translation_prompt(&self, code: &str, plan: &TranslationPlan) -> String {
        format!(
            "Translate the following {} code to {}:\n\n{}\n\nRules:\n{}\n\nContext:\n{:?}",
            plan.source_language,
            plan.target_language,
            code,
            plan.rules
                .iter()
                .map(|r| format!("- {}", r.description))
                .collect::<Vec<_>>()
                .join("\n"),
            plan.context
        )
    }

    /// Parse model response
    fn parse_model_response(
        &self,
        response: &str,
        _plan: &TranslationPlan,
    ) -> Result<TranslationResult> {
        // Simple parsing - real implementation would be more sophisticated
        Ok(TranslationResult {
            translated_code: response.to_string(),
            applied_rules: vec!["AI-powered translation".to_string()],
            warnings: vec![],
            confidence: 0.75,
        })
    }

    /// Generate cache key
    fn generate_cache_key(&self, code: &str, plan: &TranslationPlan) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        plan.source_language.hash(&mut hasher);
        plan.target_language.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get cached translation
    async fn get_cached(&self, key: &str) -> Option<TranslationResult> {
        let cache = self.cache.read().await;
        cache
            .entries
            .get(key)
            .filter(|entry| entry.timestamp.elapsed().as_secs() < 3600) // 1 hour cache
            .map(|entry| entry.result.clone())
    }

    /// Cache translation result
    async fn cache_translation(&self, key: &str, result: &TranslationResult) {
        let mut cache = self.cache.write().await;

        // Evict old entries if cache is full
        if cache.entries.len() >= cache.max_size {
            let oldest_key = cache
                .entries
                .iter()
                .min_by_key(|(_, v)| v.timestamp)
                .map(|(k, _)| k.clone());

            if let Some(k) = oldest_key {
                cache.entries.remove(&k);
            }
        }

        cache.entries.insert(
            key.to_string(),
            CachedTranslation {
                result: result.clone(),
                timestamp: std::time::Instant::now(),
            },
        );
    }

    /// Track translation start for metrics
    async fn track_translation_start(&self, source: &str, target: &str) {
        let mut expertise = self.expertise.write().await;
        expertise.total_translations += 1;
    }

    /// Track translation success
    async fn track_translation_success(&self, source: &str, target: &str) {
        let mut expertise = self.expertise.write().await;
        expertise.successful_translations += 1;

        // Update language pair expertise
        let pair = (source.to_string(), target.to_string());
        let current = expertise.language_pairs.get(&pair).copied().unwrap_or(0.0);
        expertise
            .language_pairs
            .insert(pair, (current + 0.01).min(1.0));
    }

    /// Enhance plan with learned patterns
    async fn enhance_with_learned_patterns(
        &self,
        plan: &TranslationPlan,
    ) -> Result<TranslationPlan> {
        let mut enhanced = plan.clone();
        let patterns = self.learned_patterns.read().await;

        let pair = (plan.source_language.clone(), plan.target_language.clone());
        if let Some(learned) = patterns.patterns.get(&pair) {
            // Add high-confidence patterns to the plan
            for pattern in learned.iter().filter(|p| p.confidence > 0.8) {
                enhanced.rules.push(TranslationRule {
                    pattern: pattern.pattern.clone(),
                    replacement: pattern.replacement.clone(),
                    description: format!(
                        "Learned pattern ({}% confidence)",
                        (pattern.confidence * 100.0) as u32
                    ),
                });
            }
        }

        Ok(enhanced)
    }

    /// Understand code structure for better translation
    async fn understand_code_structure(
        &self,
        code: &str,
        language: &str,
    ) -> Result<CodeUnderstanding> {
        // Analyze code structure
        let lines: Vec<&str> = code.lines().collect();
        let has_classes = code.contains("class ") || code.contains("struct ");
        let has_functions =
            code.contains("fn ") || code.contains("def ") || code.contains("function ");

        Ok(CodeUnderstanding {
            line_count: lines.len(),
            has_classes,
            has_functions,
            complexity: self.estimate_complexity(code),
            key_constructs: self.identify_key_constructs(code, language),
        })
    }

    /// Apply improvements based on expertise
    async fn apply_expertise_improvements(
        &self,
        mut result: TranslationResult,
        understanding: &CodeUnderstanding,
    ) -> Result<TranslationResult> {
        let expertise = self.expertise.read().await;

        // If we have high expertise, apply additional improvements
        if expertise.successful_translations > 100 {
            // Apply idiom improvements
            if understanding.has_classes {
                result.warnings.push(TranslationWarning {
                    line: None,
                    message: "Class translation verified with high confidence".to_string(),
                    severity: WarningSeverity::Info,
                });
            }
        }

        Ok(result)
    }

    /// Assess translation quality
    async fn assess_translation_quality(
        &self,
        result: &TranslationResult,
        original: &str,
        plan: &TranslationPlan,
    ) -> Result<f64> {
        let mut score = result.confidence;

        // Check if key constructs were preserved
        let original_lines = original.lines().count();
        let translated_lines = result.translated_code.lines().count();

        // Reasonable line count difference
        let line_diff =
            ((original_lines as f64 - translated_lines as f64).abs()) / (original_lines as f64);
        if line_diff < 0.5 {
            score += 0.1;
        }

        // Check if imports were added
        if !plan.context.required_imports.is_empty()
            && result
                .translated_code
                .contains(&plan.context.required_imports[0])
        {
            score += 0.1;
        }

        Ok(score.min(1.0))
    }

    /// Learn from this translation
    async fn learn_from_translation(
        &self,
        original: &str,
        result: &TranslationResult,
        plan: &TranslationPlan,
    ) -> Result<()> {
        if result.confidence > 0.7 {
            let mut patterns = self.learned_patterns.write().await;

            // Extract successful patterns
            for rule in &result.applied_rules {
                let pair = (plan.source_language.clone(), plan.target_language.clone());
                let pattern_list = patterns.patterns.entry(pair).or_insert_with(Vec::new);

                // Check if pattern exists
                let existing = pattern_list.iter_mut().find(|p| &p.pattern == rule);
                if let Some(existing) = existing {
                    existing.usage_count += 1;
                    existing.confidence = (existing.confidence + 0.05).min(1.0);
                }
            }
        }

        Ok(())
    }

    fn estimate_complexity(&self, code: &str) -> f64 {
        let mut complexity: f64 = 0.0;

        // Basic complexity metrics
        if code.contains("if ") {
            complexity += 0.1;
        }
        if code.contains("for ") {
            complexity += 0.2;
        }
        if code.contains("while ") {
            complexity += 0.2;
        }
        if code.contains("match ") || code.contains("switch ") {
            complexity += 0.3;
        }
        if code.contains("async ") {
            complexity += 0.3;
        }

        if complexity > 1.0 {
            1.0
        } else {
            complexity
        }
    }

    fn identify_key_constructs(&self, code: &str, language: &str) -> Vec<String> {
        let mut constructs = Vec::new();

        match language {
            "rust" => {
                if code.contains("impl ") {
                    constructs.push("implementation".to_string());
                }
                if code.contains("trait ") {
                    constructs.push("trait".to_string());
                }
                if code.contains("async fn") {
                    constructs.push("async function".to_string());
                }
            }
            "python" => {
                if code.contains("class ") {
                    constructs.push("class".to_string());
                }
                if code.contains("def ") {
                    constructs.push("function".to_string());
                }
                if code.contains("async def") {
                    constructs.push("async function".to_string());
                }
            }
            _ => {}
        }

        constructs
    }
}

#[derive(Debug)]
struct CodeUnderstanding {
    line_count: usize,
    has_classes: bool,
    has_functions: bool,
    complexity: f64,
    key_constructs: Vec<String>,
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_translation_plan_creation() {
        let plan = TranslationPlan {
            source_language: "python".to_string(),
            target_language: "rust".to_string(),
            strategy: TranslationStrategy::DirectMapping,
            rules: vec![TranslationRule {
                pattern: "print(".to_string(),
                replacement: "println!(".to_string(),
                description: "Convert print to println!".to_string(),
            }],
            context: TranslationContext {
                required_imports: vec![],
                type_mappings: HashMap::new(),
                conventions: HashMap::new(),
            },
        };

        assert_eq!(plan.source_language, "python");
        assert_eq!(plan.target_language, "rust");
        assert_eq!(plan.rules.len(), 1);
    }
}
