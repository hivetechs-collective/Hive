//! Expert profile system for consensus engine
//! 
//! Provides TypeScript-compatible expert templates with enhanced capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expert profile manager
#[derive(Debug, Clone)]
pub struct ExpertProfileManager {
    templates: Vec<ExpertTemplate>,
}

/// Expert template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: ProfileCategory,
    pub scope: ProfileScope,
    pub expert_level: ExpertLevel,
    pub selection_strategy: SelectionStrategy,
    pub fixed_models: Option<StageModels>,
    pub selection_criteria: Option<StageSelectionCriteria>,
    pub temperatures: StageTemperatures,
    pub routing_preferences: Option<RoutingPreferences>,
    pub budget_profile: Option<BudgetProfile>,
    pub performance_profile: Option<PerformanceProfile>,
    pub use_cases: Vec<String>,
    pub tags: Vec<String>,
}

/// Profile categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileCategory {
    Speed,
    Quality,
    Cost,
    Research,
    Troubleshooting,
}

/// Profile scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileScope {
    Minimal,
    Basic,
    Standard,
    Production,
    Research,
}

/// Expert level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpertLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Research,
}

/// Selection strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionStrategy {
    Fixed,
    Dynamic,
    Adaptive,
}

/// Fixed models for each stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageModels {
    pub generator: String,
    pub refiner: String,
    pub validator: String,
    pub curator: String,
}

/// Selection criteria for each stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSelectionCriteria {
    pub generator: ModelCriteria,
    pub refiner: ModelCriteria,
    pub validator: ModelCriteria,
    pub curator: ModelCriteria,
}

/// Model selection criteria
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCriteria {
    pub ranking_position: Option<String>,
    pub cost_range: Option<CostRange>,
    pub context_window: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub fallback: Option<ModelReference>,
}

/// Cost ranges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostRange {
    UltraLow,
    Low,
    Medium,
    High,
    Premium,
}

/// Model reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelReference {
    Fixed(String),
    Semantic(SemanticModelDescriptor),
}

/// Semantic model descriptor
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemanticModelDescriptor {
    pub model_type: String,
    pub tier: Option<String>,
    pub context_window: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub providers: Option<Vec<String>>,
}

/// Stage temperatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTemperatures {
    pub generator: f32,
    pub refiner: f32,
    pub validator: f32,
    pub curator: f32,
}

/// Routing preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPreferences {
    pub generator: Option<String>,
    pub refiner: Option<String>,
    pub validator: Option<String>,
    pub curator: Option<String>,
}

/// Budget profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetProfile {
    pub priority: BudgetPriority,
    pub max_cost_per_conversation: Option<f64>,
    pub preferred_cost_range: Option<CostRange>,
}

/// Budget priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetPriority {
    Cost,
    Performance,
    Balanced,
}

/// Performance profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub priority: PerformancePriority,
    pub max_latency_ms: Option<u32>,
    pub quality_threshold: Option<f32>,
}

/// Performance priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformancePriority {
    Speed,
    Quality,
    Balanced,
}

impl ExpertProfileManager {
    pub fn new() -> Self {
        Self {
            templates: Self::create_expert_templates(),
        }
    }

    /// Create simplified expert templates
    fn create_expert_templates() -> Vec<ExpertTemplate> {
        vec![
            // Lightning Fast - Ultra-high-speed consensus
            ExpertTemplate {
                id: "lightning-fast".to_string(),
                name: "Lightning Fast".to_string(),
                description: "Ultra-high-speed consensus optimized for rapid prototyping".to_string(),
                category: ProfileCategory::Speed,
                scope: ProfileScope::Minimal,
                expert_level: ExpertLevel::Beginner,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        cost_range: Some(CostRange::Low),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        cost_range: Some(CostRange::UltraLow),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("cost-efficient".to_string()),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        cost_range: Some(CostRange::Low),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.3,
                    refiner: 0.2,
                    validator: 0.1,
                    curator: 0.3,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":nitro".to_string()),
                    refiner: Some(":floor".to_string()),
                    validator: Some(":floor".to_string()),
                    curator: Some(":nitro".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Cost,
                    max_cost_per_conversation: Some(0.05),
                    preferred_cost_range: Some(CostRange::UltraLow),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Speed,
                    max_latency_ms: Some(2000),
                    quality_threshold: Some(0.7),
                }),
                use_cases: vec![
                    "Quick prototyping".to_string(),
                    "Simple questions".to_string(),
                    "Learning".to_string(),
                ],
                tags: vec!["speed".to_string(), "budget-friendly".to_string()],
            },

            // Precision Architect - Maximum quality
            ExpertTemplate {
                id: "precision-architect".to_string(),
                name: "Precision Architect".to_string(),
                description: "Maximum quality consensus for complex architectural decisions".to_string(),
                category: ProfileCategory::Quality,
                scope: ProfileScope::Production,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("large".to_string()),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["reasoning".to_string()]),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.7,
                    refiner: 0.5,
                    validator: 0.3,
                    curator: 0.6,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":nitro".to_string()),
                    refiner: Some(":nitro".to_string()),
                    validator: Some(":default".to_string()),
                    curator: Some(":nitro".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: Some(2.0),
                    preferred_cost_range: Some(CostRange::Premium),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: Some(30000),
                    quality_threshold: Some(0.95),
                }),
                use_cases: vec![
                    "Architecture decisions".to_string(),
                    "Complex algorithms".to_string(),
                    "Production code review".to_string(),
                ],
                tags: vec!["quality".to_string(), "expert".to_string(), "architecture".to_string()],
            },

            // Budget Optimizer - Cost-efficient
            ExpertTemplate {
                id: "budget-optimizer".to_string(),
                name: "Budget Optimizer".to_string(),
                description: "Cost-efficient consensus that maximizes value while minimizing expenses".to_string(),
                category: ProfileCategory::Cost,
                scope: ProfileScope::Basic,
                expert_level: ExpertLevel::Intermediate,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        cost_range: Some(CostRange::UltraLow),
                        ranking_position: Some("cost-efficient".to_string()),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        cost_range: Some(CostRange::UltraLow),
                        ranking_position: Some("cost-efficient".to_string()),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        cost_range: Some(CostRange::UltraLow),
                        ranking_position: Some("cost-efficient".to_string()),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        cost_range: Some(CostRange::Low),
                        ranking_position: Some("top-20".to_string()),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.4,
                    refiner: 0.3,
                    validator: 0.2,
                    curator: 0.4,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":floor".to_string()),
                    refiner: Some(":floor".to_string()),
                    validator: Some(":floor".to_string()),
                    curator: Some(":default".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Cost,
                    max_cost_per_conversation: Some(0.02),
                    preferred_cost_range: Some(CostRange::UltraLow),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Balanced,
                    max_latency_ms: Some(10000),
                    quality_threshold: Some(0.8),
                }),
                use_cases: vec![
                    "Cost-conscious development".to_string(),
                    "High-volume processing".to_string(),
                    "Experimentation".to_string(),
                ],
                tags: vec!["cost".to_string(), "budget".to_string(), "efficient".to_string()],
            },

            // Research Specialist - Deep exploration
            ExpertTemplate {
                id: "research-specialist".to_string(),
                name: "Research Specialist".to_string(),
                description: "Deep exploration consensus for research and complex problem-solving".to_string(),
                category: ProfileCategory::Research,
                scope: ProfileScope::Research,
                expert_level: ExpertLevel::Research,
                selection_strategy: SelectionStrategy::Adaptive,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-3".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: Some(vec!["reasoning".to_string(), "analysis".to_string()]),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: Some(vec!["synthesis".to_string()]),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["reasoning".to_string(), "verification".to_string()]),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-3".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: Some(vec!["summarization".to_string()]),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.8,
                    refiner: 0.6,
                    validator: 0.3,
                    curator: 0.5,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":nitro".to_string()),
                    refiner: Some(":nitro".to_string()),
                    validator: Some(":nitro".to_string()),
                    curator: Some(":nitro".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: Some(5.0),
                    preferred_cost_range: Some(CostRange::Premium),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: Some(60000),
                    quality_threshold: Some(0.98),
                }),
                use_cases: vec![
                    "Research papers".to_string(),
                    "Complex analysis".to_string(),
                    "Deep problem-solving".to_string(),
                    "Scientific exploration".to_string(),
                ],
                tags: vec!["research".to_string(), "deep".to_string(), "exploration".to_string()],
            },

            // Debug Specialist - Troubleshooting focus
            ExpertTemplate {
                id: "debug-specialist".to_string(),
                name: "Debug Specialist".to_string(),
                description: "Specialized consensus for debugging and troubleshooting complex issues".to_string(),
                category: ProfileCategory::Troubleshooting,
                scope: ProfileScope::Standard,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["code".to_string(), "debugging".to_string()]),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["analysis".to_string()]),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["verification".to_string()]),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["code".to_string()]),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.3,
                    refiner: 0.2,
                    validator: 0.1,
                    curator: 0.3,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":default".to_string()),
                    refiner: Some(":default".to_string()),
                    validator: Some(":default".to_string()),
                    curator: Some(":default".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Balanced,
                    max_cost_per_conversation: Some(0.5),
                    preferred_cost_range: Some(CostRange::Medium),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Balanced,
                    max_latency_ms: Some(15000),
                    quality_threshold: Some(0.9),
                }),
                use_cases: vec![
                    "Bug fixing".to_string(),
                    "Error analysis".to_string(),
                    "Performance debugging".to_string(),
                    "Code review".to_string(),
                ],
                tags: vec!["debug".to_string(), "troubleshooting".to_string(), "code".to_string()],
            },

            // Balanced Generalist - All-purpose
            ExpertTemplate {
                id: "balanced-generalist".to_string(),
                name: "Balanced Generalist".to_string(),
                description: "Well-rounded consensus for general-purpose development tasks".to_string(),
                category: ProfileCategory::Quality,
                scope: ProfileScope::Standard,
                expert_level: ExpertLevel::Intermediate,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        cost_range: Some(CostRange::Medium),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        cost_range: Some(CostRange::Low),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-15".to_string()),
                        cost_range: Some(CostRange::Low),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        cost_range: Some(CostRange::Medium),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.6,
                    refiner: 0.4,
                    validator: 0.2,
                    curator: 0.5,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":default".to_string()),
                    refiner: Some(":default".to_string()),
                    validator: Some(":default".to_string()),
                    curator: Some(":default".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Balanced,
                    max_cost_per_conversation: Some(0.25),
                    preferred_cost_range: Some(CostRange::Medium),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Balanced,
                    max_latency_ms: Some(10000),
                    quality_threshold: Some(0.85),
                }),
                use_cases: vec![
                    "General development".to_string(),
                    "Documentation".to_string(),
                    "Code generation".to_string(),
                    "Explanations".to_string(),
                ],
                tags: vec!["balanced".to_string(), "general".to_string(), "versatile".to_string()],
            },

            // Enterprise Architect - Production systems
            ExpertTemplate {
                id: "enterprise-architect".to_string(),
                name: "Enterprise Architect".to_string(),
                description: "Enterprise-grade consensus for production systems and architecture".to_string(),
                category: ProfileCategory::Quality,
                scope: ProfileScope::Production,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Adaptive,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: Some(vec!["architecture".to_string(), "systems".to_string()]),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["best-practices".to_string()]),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["security".to_string(), "compliance".to_string()]),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["documentation".to_string()]),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.5,
                    refiner: 0.3,
                    validator: 0.2,
                    curator: 0.4,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":nitro".to_string()),
                    refiner: Some(":nitro".to_string()),
                    validator: Some(":nitro".to_string()),
                    curator: Some(":nitro".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: Some(3.0),
                    preferred_cost_range: Some(CostRange::High),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: Some(30000),
                    quality_threshold: Some(0.97),
                }),
                use_cases: vec![
                    "System architecture".to_string(),
                    "Enterprise planning".to_string(),
                    "Security design".to_string(),
                    "Compliance review".to_string(),
                ],
                tags: vec!["enterprise".to_string(), "architecture".to_string(), "production".to_string()],
            },

            // Creative Innovator - Innovative solutions
            ExpertTemplate {
                id: "creative-innovator".to_string(),
                name: "Creative Innovator".to_string(),
                description: "Creative consensus for innovative solutions and out-of-the-box thinking".to_string(),
                category: ProfileCategory::Research,
                scope: ProfileScope::Standard,
                expert_level: ExpertLevel::Advanced,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["creative".to_string()]),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["synthesis".to_string()]),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-15".to_string()),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["creative".to_string()]),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.9,
                    refiner: 0.7,
                    validator: 0.4,
                    curator: 0.8,
                },
                routing_preferences: None,
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Balanced,
                    max_cost_per_conversation: Some(0.75),
                    preferred_cost_range: Some(CostRange::Medium),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Balanced,
                    max_latency_ms: Some(20000),
                    quality_threshold: Some(0.88),
                }),
                use_cases: vec![
                    "Brainstorming".to_string(),
                    "Innovation".to_string(),
                    "Creative solutions".to_string(),
                    "Prototyping".to_string(),
                ],
                tags: vec!["creative".to_string(), "innovative".to_string(), "exploration".to_string()],
            },

            // Teaching Assistant - Educational focus
            ExpertTemplate {
                id: "teaching-assistant".to_string(),
                name: "Teaching Assistant".to_string(),
                description: "Educational consensus optimized for clear explanations and learning".to_string(),
                category: ProfileCategory::Quality,
                scope: ProfileScope::Basic,
                expert_level: ExpertLevel::Intermediate,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-15".to_string()),
                        capabilities: Some(vec!["explanation".to_string()]),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        ranking_position: Some("top-15".to_string()),
                        capabilities: Some(vec!["clarity".to_string()]),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-20".to_string()),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-15".to_string()),
                        capabilities: Some(vec!["educational".to_string()]),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.5,
                    refiner: 0.4,
                    validator: 0.2,
                    curator: 0.5,
                },
                routing_preferences: None,
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Balanced,
                    max_cost_per_conversation: Some(0.15),
                    preferred_cost_range: Some(CostRange::Low),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Balanced,
                    max_latency_ms: Some(8000),
                    quality_threshold: Some(0.85),
                }),
                use_cases: vec![
                    "Teaching".to_string(),
                    "Explanations".to_string(),
                    "Documentation".to_string(),
                    "Learning resources".to_string(),
                ],
                tags: vec!["education".to_string(), "teaching".to_string(), "clear".to_string()],
            },
        ]
    }

    /// Get all templates
    pub fn get_templates(&self) -> &[ExpertTemplate] {
        &self.templates
    }

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&ExpertTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// Get templates by category
    pub fn get_templates_by_category(&self, category: &ProfileCategory) -> Vec<&ExpertTemplate> {
        self.templates
            .iter()
            .filter(|t| std::mem::discriminant(&t.category) == std::mem::discriminant(category))
            .collect()
    }

    /// Get templates by expert level
    pub fn get_templates_by_level(&self, level: &ExpertLevel) -> Vec<&ExpertTemplate> {
        self.templates
            .iter()
            .filter(|t| std::mem::discriminant(&t.expert_level) == std::mem::discriminant(level))
            .collect()
    }

    /// Get recommended templates for use case
    pub fn get_recommended_for_use_case(&self, use_case: &str) -> Vec<&ExpertTemplate> {
        self.templates
            .iter()
            .filter(|t| t.use_cases.iter().any(|uc| uc.to_lowercase().contains(&use_case.to_lowercase())))
            .collect()
    }
}

impl Default for ExpertProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Template filter for querying
#[derive(Debug, Clone, Default)]
pub struct TemplateFilter {
    pub category: Option<ProfileCategory>,
    pub scope: Option<ProfileScope>,
    pub expert_level: Option<ExpertLevel>,
    pub use_cases: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Template preferences for automatic selection
#[derive(Debug, Clone)]
pub struct TemplatePreferences {
    pub budget: Option<String>,
    pub speed: Option<String>,
    pub expert_level: Option<String>,
}