// Expert Profile System - 10+ Expert Templates
// Port of TypeScript expert-profile-templates.ts with 100% feature parity

use crate::consensus::types::{ConsensusProfile, ModelCriteria, SemanticModelDescriptor};
use crate::core::database::Database;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: ProfileCategory,
    pub scope: ProfileScope,
    pub expert_level: ExpertLevel,
    pub selection_strategy: SelectionStrategy,
    pub fixed_models: Option<FixedModels>,
    pub selection_criteria: Option<StageSelectionCriteria>,
    pub temperatures: StageTemperatures,
    pub routing_preferences: Option<RoutingPreferences>,
    pub budget_profile: Option<BudgetProfile>,
    pub performance_profile: Option<PerformanceProfile>,
    pub use_cases: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileCategory {
    Speed,
    Quality,
    Cost,
    Specialized,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileScope {
    Minimal,
    Basic,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpertLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionStrategy {
    Fixed,
    Dynamic,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedModels {
    pub generator: ModelReference,
    pub refiner: ModelReference,
    pub validator: ModelReference,
    pub curator: ModelReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelReference {
    InternalId(u32),
    OpenRouterId(String),
    Semantic(SemanticModelDescriptor),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSelectionCriteria {
    pub generator: ModelCriteria,
    pub refiner: ModelCriteria,
    pub validator: ModelCriteria,
    pub curator: ModelCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTemperatures {
    pub generator: f64,
    pub refiner: f64,
    pub validator: f64,
    pub curator: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPreferences {
    pub generator: Option<String>,
    pub refiner: Option<String>,
    pub validator: Option<String>,
    pub curator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetProfile {
    pub priority: BudgetPriority,
    pub max_cost_per_conversation: Option<f64>,
    pub preferred_cost_range: Option<CostRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetPriority {
    Cost,
    Performance,
    Balanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostRange {
    UltraLow,
    Low,
    Medium,
    High,
    Premium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub priority: PerformancePriority,
    pub max_latency_ms: Option<u32>,
    pub quality_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformancePriority {
    Speed,
    Quality,
    Balanced,
}

pub struct ExpertProfileManager {
    templates: Vec<ExpertTemplate>,
}

impl ExpertProfileManager {
    pub fn new() -> Self {
        Self {
            templates: Self::create_expert_templates(),
        }
    }

    /// Create all 10+ expert templates matching TypeScript version
    fn create_expert_templates() -> Vec<ExpertTemplate> {
        vec![
            // Lightning Fast - Ultra-high-speed consensus
            ExpertTemplate {
                id: "lightning-fast".to_string(),
                name: "Lightning Fast".to_string(),
                description: "Ultra-high-speed consensus optimized for rapid prototyping and quick answers".to_string(),
                category: ProfileCategory::Speed,
                scope: ProfileScope::Minimal,
                expert_level: ExpertLevel::Beginner,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        cost_range: Some(CostRange::Low),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "speed-optimized".to_string(),
                            tier: Some("standard".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    refiner: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        cost_range: Some(CostRange::UltraLow),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "cost-efficient".to_string(),
                            tier: Some("budget".to_string()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("cost-efficient".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "cost-efficient".to_string(),
                            tier: Some("budget".to_string()),
                            providers: Some(vec!["google".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        cost_range: Some(CostRange::Low),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "speed-optimized".to_string(),
                            tier: Some("standard".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
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
                    "Debugging small issues".to_string(),
                ],
                tags: vec!["speed".to_string(), "budget-friendly".to_string(), "beginner".to_string(), "rapid".to_string()],
            },

            // Precision Architect - Maximum quality
            ExpertTemplate {
                id: "precision-architect".to_string(),
                name: "Precision Architect".to_string(),
                description: "Maximum quality consensus for complex architectural decisions and critical code".to_string(),
                category: ProfileCategory::Quality,
                scope: ProfileScope::Production,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("flagship".to_string()),
                            context_window: Some("xl".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    refiner: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("large".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("flagship".to_string()),
                            context_window: Some("large".to_string()),
                            providers: Some(vec!["openai".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["reasoning".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            capabilities: Some(vec!["reasoning".to_string()]),
                            providers: Some(vec!["google".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("flagship".to_string()),
                            context_window: Some("xl".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
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
                    max_cost_per_conversation: Some(1.00),
                    preferred_cost_range: Some(CostRange::Premium),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.95),
                }),
                use_cases: vec![
                    "System architecture".to_string(),
                    "Complex algorithms".to_string(),
                    "Production code review".to_string(),
                    "Security analysis".to_string(),
                ],
                tags: vec!["quality".to_string(), "expert".to_string(), "architecture".to_string(), "premium".to_string(), "production".to_string()],
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
                        ranking_position: Some("cost-efficient".to_string()),
                        cost_range: Some(CostRange::Low),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "cost-efficient".to_string(),
                            tier: Some("standard".to_string()),
                            providers: Some(vec!["meta-llama".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    refiner: ModelCriteria {
                        ranking_position: Some("cost-efficient".to_string()),
                        cost_range: Some(CostRange::UltraLow),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "cost-efficient".to_string(),
                            tier: Some("budget".to_string()),
                            providers: Some(vec!["google".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("cost-efficient".to_string()),
                        cost_range: Some(CostRange::UltraLow),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "cost-efficient".to_string(),
                            tier: Some("budget".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("cost-efficient".to_string()),
                        cost_range: Some(CostRange::Low),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "cost-efficient".to_string(),
                            tier: Some("standard".to_string()),
                            providers: Some(vec!["mistral".to_string(), "mistralai".to_string()]),
                            ..Default::default()
                        })),
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
                    curator: Some(":floor".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Cost,
                    max_cost_per_conversation: Some(0.10),
                    preferred_cost_range: Some(CostRange::Low),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Balanced,
                    max_latency_ms: None,
                    quality_threshold: Some(0.8),
                }),
                use_cases: vec![
                    "Learning projects".to_string(),
                    "Personal development".to_string(),
                    "Small team projects".to_string(),
                    "Budget-conscious development".to_string(),
                ],
                tags: vec!["cost-effective".to_string(), "budget".to_string(), "efficient".to_string(), "balanced".to_string()],
            },

            // Research Deep Dive - Comprehensive analysis
            ExpertTemplate {
                id: "research-deep-dive".to_string(),
                name: "Research Deep Dive".to_string(),
                description: "Comprehensive analysis for research, documentation, and knowledge discovery".to_string(),
                category: ProfileCategory::Specialized,
                scope: ProfileScope::Production,
                expert_level: ExpertLevel::Advanced,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        context_window: Some("xl".to_string()),
                        capabilities: Some(vec!["research".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            context_window: Some("xl".to_string()),
                            capabilities: Some(vec!["research".to_string()]),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    refiner: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        context_window: Some("large".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            context_window: Some("large".to_string()),
                            providers: Some(vec!["openai".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-15".to_string()),
                        capabilities: Some(vec!["analysis".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("standard".to_string()),
                            capabilities: Some(vec!["analysis".to_string()]),
                            providers: Some(vec!["google".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        context_window: Some("xl".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("flagship".to_string()),
                            context_window: Some("xl".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.8,
                    refiner: 0.6,
                    validator: 0.4,
                    curator: 0.7,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":online".to_string()),
                    refiner: Some(":default".to_string()),
                    validator: Some(":default".to_string()),
                    curator: Some(":online".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: Some(CostRange::High),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.9),
                }),
                use_cases: vec![
                    "Technical research".to_string(),
                    "Market analysis".to_string(),
                    "Academic writing".to_string(),
                    "Documentation".to_string(),
                ],
                tags: vec!["research".to_string(), "analysis".to_string(), "comprehensive".to_string(), "web-enabled".to_string()],
            },

            // Startup MVP - Balanced for MVP development
            ExpertTemplate {
                id: "startup-mvp".to_string(),
                name: "Startup MVP".to_string(),
                description: "Balanced consensus for MVP development with good quality and reasonable cost".to_string(),
                category: ProfileCategory::Production,
                scope: ProfileScope::Basic,
                expert_level: ExpertLevel::Intermediate,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        cost_range: Some(CostRange::Medium),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "balanced".to_string(),
                            tier: Some("premium".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    refiner: ModelCriteria {
                        ranking_position: Some("top-15".to_string()),
                        cost_range: Some(CostRange::Low),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "balanced".to_string(),
                            tier: Some("standard".to_string()),
                            providers: Some(vec!["openai".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("cost-efficient".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "balanced".to_string(),
                            tier: Some("standard".to_string()),
                            providers: Some(vec!["google".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        cost_range: Some(CostRange::Medium),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "balanced".to_string(),
                            tier: Some("premium".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.6,
                    refiner: 0.4,
                    validator: 0.3,
                    curator: 0.5,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":default".to_string()),
                    refiner: Some(":floor".to_string()),
                    validator: Some(":floor".to_string()),
                    curator: Some(":default".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Balanced,
                    max_cost_per_conversation: Some(0.25),
                    preferred_cost_range: Some(CostRange::Medium),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Balanced,
                    max_latency_ms: None,
                    quality_threshold: Some(0.85),
                }),
                use_cases: vec![
                    "MVP development".to_string(),
                    "Startup projects".to_string(),
                    "Feature prototyping".to_string(),
                    "Small team development".to_string(),
                ],
                tags: vec!["startup".to_string(), "mvp".to_string(), "balanced".to_string(), "practical".to_string()],
            },

            // Enterprise Grade - Production-ready with fixed models
            ExpertTemplate {
                id: "enterprise-grade".to_string(),
                name: "Enterprise Grade".to_string(),
                description: "Production-ready consensus with enterprise security, reliability, and compliance".to_string(),
                category: ProfileCategory::Production,
                scope: ProfileScope::Production,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Hybrid,
                fixed_models: Some(FixedModels {
                    generator: ModelReference::InternalId(1), // anthropic/claude-3.5-sonnet
                    refiner: ModelReference::InternalId(2),   // openai/gpt-4o
                    validator: ModelReference::InternalId(5), // google/gemini-pro-1.5
                    curator: ModelReference::InternalId(6),   // anthropic/claude-3-opus
                }),
                selection_criteria: None,
                temperatures: StageTemperatures {
                    generator: 0.5,
                    refiner: 0.3,
                    validator: 0.2,
                    curator: 0.4,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":nitro".to_string()),
                    refiner: Some(":nitro".to_string()),
                    validator: Some(":default".to_string()),
                    curator: Some(":nitro".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: Some(CostRange::Premium),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.95),
                }),
                use_cases: vec![
                    "Enterprise applications".to_string(),
                    "Mission-critical systems".to_string(),
                    "Financial services".to_string(),
                    "Healthcare systems".to_string(),
                ],
                tags: vec!["enterprise".to_string(), "production".to_string(), "reliable".to_string(), "premium".to_string(), "compliance".to_string()],
            },

            // Creative Innovator - High-creativity
            ExpertTemplate {
                id: "creative-innovator".to_string(),
                name: "Creative Innovator".to_string(),
                description: "High-creativity consensus for innovative solutions and creative problem solving".to_string(),
                category: ProfileCategory::Specialized,
                scope: ProfileScope::Basic,
                expert_level: ExpertLevel::Advanced,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["creative".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            capabilities: Some(vec!["creative".to_string()]),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    refiner: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            providers: Some(vec!["openai".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-15".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("standard".to_string()),
                            providers: Some(vec!["google".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["creative".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("flagship".to_string()),
                            capabilities: Some(vec!["creative".to_string()]),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.9,
                    refiner: 0.7,
                    validator: 0.5,
                    curator: 0.8,
                },
                routing_preferences: None,
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: Some(CostRange::High),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.85),
                }),
                use_cases: vec![
                    "Creative coding".to_string(),
                    "Innovative solutions".to_string(),
                    "Brainstorming".to_string(),
                    "Novel algorithms".to_string(),
                ],
                tags: vec!["creative".to_string(), "innovative".to_string(), "experimental".to_string(), "high-temperature".to_string()],
            },

            // Security Focused - Security-first analysis
            ExpertTemplate {
                id: "security-focused".to_string(),
                name: "Security Focused".to_string(),
                description: "Security-first consensus optimized for secure coding and vulnerability analysis".to_string(),
                category: ProfileCategory::Specialized,
                scope: ProfileScope::Production,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["security".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            capabilities: Some(vec!["security".to_string()]),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    refiner: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["security".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            capabilities: Some(vec!["security".to_string()]),
                            providers: Some(vec!["openai".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["analysis".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("standard".to_string()),
                            capabilities: Some(vec!["analysis".to_string()]),
                            providers: Some(vec!["google".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["security".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("flagship".to_string()),
                            capabilities: Some(vec!["security".to_string()]),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.4,
                    refiner: 0.3,
                    validator: 0.2,
                    curator: 0.3,
                },
                routing_preferences: Some(RoutingPreferences {
                    generator: Some(":nitro".to_string()),
                    refiner: Some(":nitro".to_string()),
                    validator: Some(":default".to_string()),
                    curator: Some(":nitro".to_string()),
                }),
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: Some(CostRange::High),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.95),
                }),
                use_cases: vec![
                    "Security audits".to_string(),
                    "Vulnerability analysis".to_string(),
                    "Secure coding".to_string(),
                    "Compliance reviews".to_string(),
                ],
                tags: vec!["security".to_string(), "analysis".to_string(), "compliance".to_string(), "expert".to_string(), "careful".to_string()],
            },

            // ML/AI Specialist - Machine learning focused
            ExpertTemplate {
                id: "ml-ai-specialist".to_string(),
                name: "ML/AI Specialist".to_string(),
                description: "Specialized consensus for machine learning and AI development projects".to_string(),
                category: ProfileCategory::Specialized,
                scope: ProfileScope::Production,
                expert_level: ExpertLevel::Expert,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["ml".to_string(), "ai".to_string()]),
                        context_window: Some("xl".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            capabilities: Some(vec!["ml".to_string(), "ai".to_string()]),
                            context_window: Some("xl".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    refiner: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["ml".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            capabilities: Some(vec!["ml".to_string()]),
                            providers: Some(vec!["openai".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["analysis".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("standard".to_string()),
                            capabilities: Some(vec!["analysis".to_string()]),
                            providers: Some(vec!["google".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-5".to_string()),
                        capabilities: Some(vec!["ml".to_string(), "ai".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("flagship".to_string()),
                            capabilities: Some(vec!["ml".to_string(), "ai".to_string()]),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.6,
                    refiner: 0.4,
                    validator: 0.3,
                    curator: 0.5,
                },
                routing_preferences: None,
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Performance,
                    max_cost_per_conversation: None,
                    preferred_cost_range: Some(CostRange::High),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.9),
                }),
                use_cases: vec![
                    "ML model development".to_string(),
                    "AI system design".to_string(),
                    "Data science".to_string(),
                    "Neural networks".to_string(),
                ],
                tags: vec!["ml".to_string(), "ai".to_string(), "data-science".to_string(), "specialist".to_string(), "advanced".to_string()],
            },

            // Debugging Detective - Methodical debugging
            ExpertTemplate {
                id: "debugging-detective".to_string(),
                name: "Debugging Detective".to_string(),
                description: "Methodical consensus optimized for debugging, troubleshooting, and error analysis".to_string(),
                category: ProfileCategory::Specialized,
                scope: ProfileScope::Basic,
                expert_level: ExpertLevel::Intermediate,
                selection_strategy: SelectionStrategy::Dynamic,
                fixed_models: None,
                selection_criteria: Some(StageSelectionCriteria {
                    generator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["debugging".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            capabilities: Some(vec!["debugging".to_string()]),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    refiner: ModelCriteria {
                        ranking_position: Some("top-15".to_string()),
                        capabilities: Some(vec!["analysis".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "balanced".to_string(),
                            tier: Some("standard".to_string()),
                            capabilities: Some(vec!["analysis".to_string()]),
                            providers: Some(vec!["openai".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    validator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        capabilities: Some(vec!["reasoning".to_string()]),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("standard".to_string()),
                            capabilities: Some(vec!["reasoning".to_string()]),
                            providers: Some(vec!["google".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                    curator: ModelCriteria {
                        ranking_position: Some("top-10".to_string()),
                        fallback: Some(ModelReference::Semantic(SemanticModelDescriptor {
                            model_type: "premium-quality".to_string(),
                            tier: Some("premium".to_string()),
                            providers: Some(vec!["anthropic".to_string()]),
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                }),
                temperatures: StageTemperatures {
                    generator: 0.3,
                    refiner: 0.2,
                    validator: 0.1,
                    curator: 0.3,
                },
                routing_preferences: None,
                budget_profile: Some(BudgetProfile {
                    priority: BudgetPriority::Balanced,
                    max_cost_per_conversation: None,
                    preferred_cost_range: Some(CostRange::Medium),
                }),
                performance_profile: Some(PerformanceProfile {
                    priority: PerformancePriority::Quality,
                    max_latency_ms: None,
                    quality_threshold: Some(0.9),
                }),
                use_cases: vec![
                    "Bug debugging".to_string(),
                    "Error analysis".to_string(),
                    "Code troubleshooting".to_string(),
                    "System diagnostics".to_string(),
                ],
                tags: vec!["debugging".to_string(), "troubleshooting".to_string(), "methodical".to_string(), "precise".to_string()],
            },
        ]
    }

    /// Get all templates
    pub fn get_templates(&self) -> &[ExpertTemplate] {
        &self.templates
    }

    /// Get templates by filter
    pub fn get_templates_filtered(&self, filter: &TemplateFilter) -> Vec<&ExpertTemplate> {
        self.templates
            .iter()
            .filter(|template| {
                if let Some(category) = &filter.category {
                    if !matches!(
                        (&template.category, category.as_str()),
                        (ProfileCategory::Speed, "speed") |
                        (ProfileCategory::Quality, "quality") |
                        (ProfileCategory::Cost, "cost") |
                        (ProfileCategory::Specialized, "specialized") |
                        (ProfileCategory::Production, "production")
                    ) {
                        return false;
                    }
                }

                if let Some(scope) = &filter.scope {
                    if !matches!(
                        (&template.scope, scope.as_str()),
                        (ProfileScope::Minimal, "minimal") |
                        (ProfileScope::Basic, "basic") |
                        (ProfileScope::Production, "production")
                    ) {
                        return false;
                    }
                }

                if let Some(expert_level) = &filter.expert_level {
                    if !matches!(
                        (&template.expert_level, expert_level.as_str()),
                        (ExpertLevel::Beginner, "beginner") |
                        (ExpertLevel::Intermediate, "intermediate") |
                        (ExpertLevel::Advanced, "advanced") |
                        (ExpertLevel::Expert, "expert")
                    ) {
                        return false;
                    }
                }

                if let Some(use_cases) = &filter.use_cases {
                    if !use_cases.iter().any(|filter_use_case| {
                        template.use_cases.iter().any(|template_use_case| {
                            template_use_case.to_lowercase().contains(&filter_use_case.to_lowercase())
                        })
                    }) {
                        return false;
                    }
                }

                if let Some(tags) = &filter.tags {
                    if !tags.iter().any(|tag| template.tags.contains(tag)) {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Find template by ID
    pub fn get_template(&self, id: &str) -> Option<&ExpertTemplate> {
        self.templates.iter().find(|template| template.id == id)
    }

    /// Find best template for a question
    pub fn find_best_template(&self, question: &str, preferences: Option<&TemplatePreferences>) -> Vec<&ExpertTemplate> {
        let question_lower = question.to_lowercase();
        let mut scores = HashMap::new();

        // Analyze question content for relevant templates
        for template in &self.templates {
            let mut score = 0.0;

            // Use case matching
            for use_case in &template.use_cases {
                if question_lower.contains(&use_case.to_lowercase()) {
                    score += 10.0;
                }
            }

            // Tag matching
            for tag in &template.tags {
                if question_lower.contains(tag) {
                    score += 5.0;
                }
            }

            // Category matching
            if question_lower.contains("debug") && template.id == "debugging-detective" {
                score += 15.0;
            }
            if question_lower.contains("security") && template.id == "security-focused" {
                score += 15.0;
            }
            if (question_lower.contains("ml") || question_lower.contains("ai")) && template.id == "ml-ai-specialist" {
                score += 15.0;
            }
            if (question_lower.contains("fast") || question_lower.contains("quick")) && matches!(template.category, ProfileCategory::Speed) {
                score += 10.0;
            }
            if (question_lower.contains("budget") || question_lower.contains("cheap")) && matches!(template.category, ProfileCategory::Cost) {
                score += 10.0;
            }
            if (question_lower.contains("enterprise") || question_lower.contains("production")) && template.id == "enterprise-grade" {
                score += 15.0;
            }

            // Preference matching
            if let Some(prefs) = preferences {
                if let Some(budget) = &prefs.budget {
                    match budget.as_str() {
                        "low" => {
                            if template.budget_profile.as_ref().map_or(false, |bp| matches!(bp.priority, BudgetPriority::Cost)) {
                                score += 8.0;
                            }
                        }
                        "high" => {
                            if template.budget_profile.as_ref().map_or(false, |bp| matches!(bp.priority, BudgetPriority::Performance)) {
                                score += 8.0;
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(speed) = &prefs.speed {
                    match speed.as_str() {
                        "fast" => {
                            if matches!(template.category, ProfileCategory::Speed) {
                                score += 8.0;
                            }
                        }
                        "quality" => {
                            if matches!(template.category, ProfileCategory::Quality) {
                                score += 8.0;
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(expert_level) = &prefs.expert_level {
                    if matches!(
                        (&template.expert_level, expert_level.as_str()),
                        (ExpertLevel::Beginner, "beginner") |
                        (ExpertLevel::Intermediate, "intermediate") |
                        (ExpertLevel::Advanced, "advanced") |
                        (ExpertLevel::Expert, "expert")
                    ) {
                        score += 5.0;
                    }
                }
            }

            scores.insert(template.id.clone(), score);
        }

        // Return top 3 templates, sorted by score
        let mut result: Vec<&ExpertTemplate> = self.templates
            .iter()
            .filter(|template| scores.get(&template.id).copied().unwrap_or(0.0) > 0.0)
            .collect();

        result.sort_by(|a, b| {
            let score_a = scores.get(&a.id).copied().unwrap_or(0.0);
            let score_b = scores.get(&b.id).copied().unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        result.into_iter().take(3).collect()
    }

    /// Get template recommendations for a user level
    pub fn get_recommendations(&self, user_level: Option<&ExpertLevel>) -> Vec<&ExpertTemplate> {
        let level = user_level.unwrap_or(&ExpertLevel::Intermediate);

        match level {
            ExpertLevel::Beginner => {
                vec![
                    self.get_template("lightning-fast").unwrap(),
                    self.get_template("budget-optimizer").unwrap(),
                    self.get_template("startup-mvp").unwrap(),
                ]
            }
            ExpertLevel::Intermediate => {
                vec![
                    self.get_template("startup-mvp").unwrap(),
                    self.get_template("debugging-detective").unwrap(),
                    self.get_template("research-deep-dive").unwrap(),
                ]
            }
            ExpertLevel::Advanced => {
                vec![
                    self.get_template("precision-architect").unwrap(),
                    self.get_template("creative-innovator").unwrap(),
                    self.get_template("ml-ai-specialist").unwrap(),
                ]
            }
            ExpertLevel::Expert => {
                vec![
                    self.get_template("enterprise-grade").unwrap(),
                    self.get_template("security-focused").unwrap(),
                    self.get_template("precision-architect").unwrap(),
                ]
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemplateFilter {
    pub category: Option<String>,
    pub scope: Option<String>,
    pub expert_level: Option<String>,
    pub use_cases: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct TemplatePreferences {
    pub budget: Option<String>,
    pub speed: Option<String>,
    pub expert_level: Option<String>,
}

impl Default for TemplateFilter {
    fn default() -> Self {
        Self {
            category: None,
            scope: None,
            expert_level: None,
            use_cases: None,
            tags: None,
        }
    }
}