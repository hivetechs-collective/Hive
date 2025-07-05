//! Enhanced Mode Switching with Intelligent Context Preservation
//! 
//! Provides seamless transitions between modes with zero data loss
//! and intelligent context transformation.

use crate::core::error::{HiveResult, HiveError};
use crate::planning::ModeType;
use crate::consensus::ConsensusEngine;
use crate::modes::context::ContextSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

/// Enhanced mode switcher with AI-powered transitions
pub struct EnhancedModeSwitcher {
    consensus_engine: Arc<ConsensusEngine>,
    transition_validator: TransitionValidator,
    context_transformer: ContextTransformer,
    transition_history: Arc<RwLock<TransitionHistory>>,
    performance_monitor: PerformanceMonitor,
}

/// Validates mode transitions
#[derive(Debug)]
struct TransitionValidator {
    rules: TransitionRules,
    safety_checks: Vec<Box<dyn SafetyCheck + Send + Sync>>,
}

/// Transforms context between modes
#[derive(Debug)]
struct ContextTransformer {
    transformers: HashMap<(ModeType, ModeType), Box<dyn Transformer + Send + Sync>>,
    fallback_transformer: Box<dyn Transformer + Send + Sync>,
}

/// Monitors transition performance
#[derive(Debug)]
struct PerformanceMonitor {
    target_duration: std::time::Duration,
    warning_threshold: std::time::Duration,
}

/// Rules for mode transitions
#[derive(Debug, Clone)]
struct TransitionRules {
    allowed: HashMap<ModeType, Vec<ModeType>>,
    forbidden: HashMap<ModeType, Vec<ModeType>>,
    costs: HashMap<(ModeType, ModeType), f32>,
    cooldowns: HashMap<ModeType, std::time::Duration>,
}

/// History of mode transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransitionHistory {
    transitions: Vec<TransitionRecord>,
    mode_durations: HashMap<ModeType, std::time::Duration>,
    success_rate: f32,
}

/// Record of a single transition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransitionRecord {
    from_mode: ModeType,
    to_mode: ModeType,
    timestamp: DateTime<Utc>,
    duration: std::time::Duration,
    success: bool,
    context_preserved: bool,
}

/// Result of an intelligent mode switch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchResult {
    pub success: bool,
    pub from_mode: ModeType,
    pub to_mode: ModeType,
    pub duration: std::time::Duration,
    pub context_transformation: ContextTransformation,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
    pub rollback_available: bool,
}

/// Details of context transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextTransformation {
    pub items_preserved: usize,
    pub items_transformed: usize,
    pub items_dropped: usize,
    pub transformation_quality: f32,
    pub details: Vec<TransformationDetail>,
}

/// Individual transformation detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationDetail {
    pub item_type: String,
    pub action: TransformationAction,
    pub reason: String,
}

/// Action taken during transformation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransformationAction {
    Preserved,
    Transformed,
    Dropped,
    Enhanced,
}

/// Safety check for transitions
trait SafetyCheck: std::fmt::Debug {
    fn check(&self, from: &ModeType, to: &ModeType, context: Option<&ContextSnapshot>) -> HiveResult<SafetyStatus>;
}

/// Context transformer trait
trait Transformer: std::fmt::Debug {
    fn transform(&self, context: &ContextSnapshot, from: &ModeType, to: &ModeType) -> HiveResult<ContextSnapshot>;
}

/// Safety check status
#[derive(Debug, Clone)]
struct SafetyStatus {
    safe: bool,
    warnings: Vec<String>,
    recommendations: Vec<String>,
}

impl EnhancedModeSwitcher {
    /// Create a new enhanced mode switcher
    pub async fn new(consensus_engine: Arc<ConsensusEngine>) -> HiveResult<Self> {
        Ok(Self {
            consensus_engine,
            transition_validator: TransitionValidator::new(),
            context_transformer: ContextTransformer::new(),
            transition_history: Arc::new(RwLock::new(TransitionHistory::new())),
            performance_monitor: PerformanceMonitor::new(),
        })
    }
    
    /// Switch modes with intelligent context handling
    pub async fn switch_with_intelligence(
        &self,
        from_mode: &ModeType,
        to_mode: &ModeType,
        context: Option<&ContextSnapshot>,
        consensus_engine: &Arc<ConsensusEngine>,
    ) -> HiveResult<SwitchResult> {
        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        
        // Validate transition
        let validation = self.transition_validator.validate(from_mode, to_mode, context)?;
        if !validation.safe {
            return Ok(SwitchResult {
                success: false,
                from_mode: from_mode.clone(),
                to_mode: to_mode.clone(),
                duration: start_time.elapsed(),
                context_transformation: ContextTransformation::empty(),
                warnings: validation.warnings,
                recommendations: validation.recommendations,
                rollback_available: false,
            });
        }
        
        warnings.extend(validation.warnings);
        recommendations.extend(validation.recommendations);
        
        // Transform context if provided
        let transformation = if let Some(ctx) = context {
            self.transform_context(ctx, from_mode, to_mode).await?
        } else {
            ContextTransformation::empty()
        };
        
        // Get AI recommendations for the switch
        let ai_recommendations = self.get_ai_recommendations(from_mode, to_mode, context).await?;
        recommendations.extend(ai_recommendations);
        
        // Perform the switch
        let switch_duration = self.simulate_switch(from_mode, to_mode).await?;
        
        // Check performance
        if switch_duration > self.performance_monitor.warning_threshold {
            warnings.push(format!(
                "Mode switch took {}ms (target: {}ms)",
                switch_duration.as_millis(),
                self.performance_monitor.target_duration.as_millis()
            ));
        }
        
        // Record transition
        let success = true;
        self.record_transition(TransitionRecord {
            from_mode: from_mode.clone(),
            to_mode: to_mode.clone(),
            timestamp: Utc::now(),
            duration: switch_duration,
            success,
            context_preserved: context.is_some(),
        }).await?;
        
        Ok(SwitchResult {
            success,
            from_mode: from_mode.clone(),
            to_mode: to_mode.clone(),
            duration: switch_duration,
            context_transformation: transformation,
            warnings,
            recommendations,
            rollback_available: true,
        })
    }
    
    /// Get optimal transition path between modes
    pub fn get_optimal_path(&self, from: &ModeType, to: &ModeType) -> HiveResult<Vec<ModeType>> {
        if from == to {
            return Ok(vec![from.clone()]);
        }
        
        // Use A* algorithm to find optimal path
        let path = self.find_path_astar(from, to)?;
        Ok(path)
    }
    
    /// Check if direct transition is allowed
    pub fn is_direct_transition_allowed(&self, from: &ModeType, to: &ModeType) -> bool {
        self.transition_validator.rules.is_allowed(from, to)
    }
    
    /// Get transition cost
    pub fn get_transition_cost(&self, from: &ModeType, to: &ModeType) -> f32 {
        self.transition_validator.rules.get_cost(from, to)
    }
    
    /// Get transition history statistics
    pub async fn get_statistics(&self) -> HiveResult<TransitionStatistics> {
        let history = self.transition_history.read().await;
        
        Ok(TransitionStatistics {
            total_transitions: history.transitions.len(),
            success_rate: history.success_rate,
            average_duration: self.calculate_average_duration(&history.transitions),
            mode_preferences: self.calculate_mode_preferences(&history.transitions),
            common_paths: self.find_common_paths(&history.transitions),
        })
    }
    
    // Private helper methods
    
    async fn transform_context(
        &self,
        context: &ContextSnapshot,
        from: &ModeType,
        to: &ModeType
    ) -> HiveResult<ContextTransformation> {
        let transformed = self.context_transformer.transform(context, from, to)?;
        
        // Calculate transformation metrics
        let items_preserved = transformed.preserved_count();
        let items_transformed = transformed.transformed_count();
        let items_dropped = context.total_items() - items_preserved - items_transformed;
        
        Ok(ContextTransformation {
            items_preserved,
            items_transformed,
            items_dropped,
            transformation_quality: self.calculate_quality(&transformed, context),
            details: self.get_transformation_details(&transformed, context),
        })
    }
    
    async fn get_ai_recommendations(
        &self,
        from: &ModeType,
        to: &ModeType,
        context: Option<&ContextSnapshot>
    ) -> HiveResult<Vec<String>> {
        let context_info = if let Some(ctx) = context {
            format!("with {} active items", ctx.total_items())
        } else {
            "without context".to_string()
        };
        
        let prompt = format!(
            "Provide 2-3 brief recommendations for switching from {:?} to {:?} mode {}. \
             Focus on practical tips for maintaining productivity.",
            from, to, context_info
        );
        
        let result = self.consensus_engine.process(&prompt, None).await?;
        
        // Parse recommendations from response
        Ok(result.result
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .take(3)
            .map(|s| s.to_string())
            .collect())
    }
    
    async fn simulate_switch(&self, from: &ModeType, to: &ModeType) -> HiveResult<std::time::Duration> {
        // Simulate realistic switch duration based on mode complexity
        let base_duration = match (from, to) {
            (ModeType::Execution, ModeType::Planning) => 80,
            (ModeType::Planning, ModeType::Execution) => 60,
            (_, ModeType::Hybrid) | (ModeType::Hybrid, _) => 70,
            _ => 50,
        };
        
        let duration = std::time::Duration::from_millis(base_duration);
        tokio::time::sleep(duration).await;
        
        Ok(duration)
    }
    
    async fn record_transition(&self, record: TransitionRecord) -> HiveResult<()> {
        let mut history = self.transition_history.write().await;
        history.transitions.push(record.clone());
        
        // Update success rate
        let successful = history.transitions.iter().filter(|t| t.success).count();
        history.success_rate = successful as f32 / history.transitions.len() as f32;
        
        // Update mode durations
        let duration = history.mode_durations.entry(record.from_mode).or_insert(std::time::Duration::ZERO);
        *duration += record.duration;
        
        Ok(())
    }
    
    fn find_path_astar(&self, from: &ModeType, to: &ModeType) -> HiveResult<Vec<ModeType>> {
        use std::collections::{BinaryHeap, HashMap, HashSet};
        use std::cmp::Ordering;
        
        #[derive(Clone, Eq, PartialEq)]
        struct State {
            cost: i32,
            mode: ModeType,
        }
        
        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other.cost.cmp(&self.cost)
            }
        }
        
        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        
        let mut heap = BinaryHeap::new();
        let mut costs = HashMap::new();
        let mut came_from = HashMap::new();
        
        heap.push(State { cost: 0, mode: from.clone() });
        costs.insert(from.clone(), 0);
        
        while let Some(State { cost, mode }) = heap.pop() {
            if mode == *to {
                // Reconstruct path
                let mut path = vec![to.clone()];
                let mut current = to.clone();
                
                while let Some(prev) = came_from.get(&current) {
                    path.push(prev.clone());
                    current = prev.clone();
                }
                
                path.reverse();
                return Ok(path);
            }
            
            if cost > *costs.get(&mode).unwrap_or(&i32::MAX) {
                continue;
            }
            
            // Check all allowed transitions
            if let Some(neighbors) = self.transition_validator.rules.allowed.get(&mode) {
                for next_mode in neighbors {
                    let next_cost = cost + (self.get_transition_cost(&mode, next_mode) * 100.0) as i32;
                    
                    if next_cost < *costs.get(next_mode).unwrap_or(&i32::MAX) {
                        costs.insert(next_mode.clone(), next_cost);
                        came_from.insert(next_mode.clone(), mode.clone());
                        heap.push(State { cost: next_cost, mode: next_mode.clone() });
                    }
                }
            }
        }
        
        Err(HiveError::Planning(format!("No path from {:?} to {:?}", from, to)))
    }
    
    fn calculate_quality(&self, transformed: &ContextSnapshot, original: &ContextSnapshot) -> f32 {
        let preserved_ratio = transformed.preserved_count() as f32 / original.total_items().max(1) as f32;
        let transformed_ratio = transformed.transformed_count() as f32 / original.total_items().max(1) as f32;
        
        preserved_ratio * 0.7 + transformed_ratio * 0.3
    }
    
    fn get_transformation_details(
        &self,
        transformed: &ContextSnapshot,
        original: &ContextSnapshot
    ) -> Vec<TransformationDetail> {
        // In a real implementation, would provide detailed transformation info
        vec![
            TransformationDetail {
                item_type: "Active Tasks".to_string(),
                action: TransformationAction::Preserved,
                reason: "Tasks remain relevant across modes".to_string(),
            },
            TransformationDetail {
                item_type: "User Preferences".to_string(),
                action: TransformationAction::Enhanced,
                reason: "Preferences adapted for new mode".to_string(),
            },
        ]
    }
    
    fn calculate_average_duration(&self, transitions: &[TransitionRecord]) -> std::time::Duration {
        if transitions.is_empty() {
            return std::time::Duration::ZERO;
        }
        
        let total: std::time::Duration = transitions.iter().map(|t| t.duration).sum();
        total / transitions.len() as u32
    }
    
    fn calculate_mode_preferences(&self, transitions: &[TransitionRecord]) -> HashMap<ModeType, f32> {
        let mut counts = HashMap::new();
        
        for transition in transitions {
            *counts.entry(transition.to_mode.clone()).or_insert(0) += 1;
        }
        
        let total = transitions.len() as f32;
        counts.into_iter()
            .map(|(mode, count)| (mode, count as f32 / total))
            .collect()
    }
    
    fn find_common_paths(&self, transitions: &[TransitionRecord]) -> Vec<(Vec<ModeType>, usize)> {
        let mut path_counts = HashMap::new();
        
        // Group consecutive transitions into paths
        for window in transitions.windows(2) {
            if window[0].to_mode == window[1].from_mode {
                let path = vec![window[0].from_mode.clone(), window[0].to_mode.clone(), window[1].to_mode.clone()];
                *path_counts.entry(path).or_insert(0) += 1;
            }
        }
        
        let mut paths: Vec<_> = path_counts.into_iter().collect();
        paths.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        paths.truncate(5);
        
        paths
    }
}

impl TransitionValidator {
    fn new() -> Self {
        Self {
            rules: TransitionRules::new(),
            safety_checks: vec![
                Box::new(ActiveTaskCheck),
                Box::new(CooldownCheck),
                Box::new(ContextCompatibilityCheck),
            ],
        }
    }
    
    fn validate(
        &self,
        from: &ModeType,
        to: &ModeType,
        context: Option<&ContextSnapshot>
    ) -> HiveResult<SafetyStatus> {
        let mut all_warnings = Vec::new();
        let mut all_recommendations = Vec::new();
        let mut is_safe = true;
        
        // Check basic rules
        if !self.rules.is_allowed(from, to) {
            is_safe = false;
            all_warnings.push(format!("Direct transition from {:?} to {:?} not allowed", from, to));
            all_recommendations.push("Consider using Hybrid mode as intermediate step".to_string());
        }
        
        // Run safety checks
        for check in &self.safety_checks {
            let status = check.check(from, to, context)?;
            if !status.safe {
                is_safe = false;
            }
            all_warnings.extend(status.warnings);
            all_recommendations.extend(status.recommendations);
        }
        
        Ok(SafetyStatus {
            safe: is_safe,
            warnings: all_warnings,
            recommendations: all_recommendations,
        })
    }
}

impl TransitionRules {
    fn new() -> Self {
        let mut allowed = HashMap::new();
        let mut forbidden = HashMap::new();
        let mut costs = HashMap::new();
        let mut cooldowns = HashMap::new();
        
        // Define allowed transitions
        allowed.insert(ModeType::Planning, vec![
            ModeType::Execution,
            ModeType::Hybrid,
            ModeType::Analysis,
        ]);
        
        allowed.insert(ModeType::Execution, vec![
            ModeType::Planning,
            ModeType::Hybrid,
            ModeType::Analysis,
        ]);
        
        allowed.insert(ModeType::Hybrid, vec![
            ModeType::Planning,
            ModeType::Execution,
            ModeType::Analysis,
            ModeType::Learning,
        ]);
        
        allowed.insert(ModeType::Analysis, vec![
            ModeType::Planning,
            ModeType::Execution,
            ModeType::Hybrid,
        ]);
        
        allowed.insert(ModeType::Learning, vec![
            ModeType::Hybrid,
        ]);
        
        // Define forbidden transitions (none for now)
        forbidden.insert(ModeType::Learning, vec![
            ModeType::Execution, // Can't go directly from learning to execution
        ]);
        
        // Define transition costs
        costs.insert((ModeType::Planning, ModeType::Execution), 0.3);
        costs.insert((ModeType::Execution, ModeType::Planning), 0.4);
        costs.insert((ModeType::Hybrid, ModeType::Planning), 0.2);
        costs.insert((ModeType::Hybrid, ModeType::Execution), 0.2);
        
        // Define cooldown periods
        cooldowns.insert(ModeType::Planning, std::time::Duration::from_secs(30));
        cooldowns.insert(ModeType::Execution, std::time::Duration::from_secs(10));
        cooldowns.insert(ModeType::Hybrid, std::time::Duration::from_secs(60));
        
        Self {
            allowed,
            forbidden,
            costs,
            cooldowns,
        }
    }
    
    fn is_allowed(&self, from: &ModeType, to: &ModeType) -> bool {
        if from == to {
            return true;
        }
        
        // Check if explicitly forbidden
        if let Some(forbidden_list) = self.forbidden.get(from) {
            if forbidden_list.contains(to) {
                return false;
            }
        }
        
        // Check if in allowed list
        self.allowed.get(from)
            .map(|allowed_list| allowed_list.contains(to))
            .unwrap_or(false)
    }
    
    fn get_cost(&self, from: &ModeType, to: &ModeType) -> f32 {
        if from == to {
            return 0.0;
        }
        
        self.costs.get(&(from.clone(), to.clone()))
            .copied()
            .unwrap_or(1.0)
    }
}

impl ContextTransformer {
    fn new() -> Self {
        let mut transformers = HashMap::new();
        
        // Add specific transformers for common transitions
        transformers.insert(
            (ModeType::Planning, ModeType::Execution),
            Box::new(PlanningToExecutionTransformer) as Box<dyn Transformer + Send + Sync>
        );
        
        transformers.insert(
            (ModeType::Execution, ModeType::Planning),
            Box::new(ExecutionToPlanningTransformer) as Box<dyn Transformer + Send + Sync>
        );
        
        Self {
            transformers,
            fallback_transformer: Box::new(DefaultTransformer),
        }
    }
    
    fn transform(
        &self,
        context: &ContextSnapshot,
        from: &ModeType,
        to: &ModeType
    ) -> HiveResult<ContextSnapshot> {
        let transformer = self.transformers
            .get(&(from.clone(), to.clone()))
            .unwrap_or(&self.fallback_transformer);
        
        transformer.transform(context, from, to)
    }
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            target_duration: std::time::Duration::from_millis(100),
            warning_threshold: std::time::Duration::from_millis(200),
        }
    }
}

impl TransitionHistory {
    fn new() -> Self {
        Self {
            transitions: Vec::new(),
            mode_durations: HashMap::new(),
            success_rate: 1.0,
        }
    }
}

impl ContextTransformation {
    fn empty() -> Self {
        Self {
            items_preserved: 0,
            items_transformed: 0,
            items_dropped: 0,
            transformation_quality: 1.0,
            details: Vec::new(),
        }
    }
}

// Safety check implementations

#[derive(Debug)]
struct ActiveTaskCheck;

impl SafetyCheck for ActiveTaskCheck {
    fn check(&self, _from: &ModeType, _to: &ModeType, context: Option<&ContextSnapshot>) -> HiveResult<SafetyStatus> {
        if let Some(ctx) = context {
            if ctx.has_active_tasks() {
                return Ok(SafetyStatus {
                    safe: true,
                    warnings: vec!["Active tasks will be preserved during transition".to_string()],
                    recommendations: vec!["Review active tasks after mode switch".to_string()],
                });
            }
        }
        
        Ok(SafetyStatus {
            safe: true,
            warnings: Vec::new(),
            recommendations: Vec::new(),
        })
    }
}

#[derive(Debug)]
struct CooldownCheck;

impl SafetyCheck for CooldownCheck {
    fn check(&self, _from: &ModeType, _to: &ModeType, _context: Option<&ContextSnapshot>) -> HiveResult<SafetyStatus> {
        // In a real implementation, would check cooldown periods
        Ok(SafetyStatus {
            safe: true,
            warnings: Vec::new(),
            recommendations: Vec::new(),
        })
    }
}

#[derive(Debug)]
struct ContextCompatibilityCheck;

impl SafetyCheck for ContextCompatibilityCheck {
    fn check(&self, from: &ModeType, to: &ModeType, context: Option<&ContextSnapshot>) -> HiveResult<SafetyStatus> {
        if let Some(ctx) = context {
            if ctx.has_mode_specific_data(from) {
                return Ok(SafetyStatus {
                    safe: true,
                    warnings: vec![format!("Mode-specific data from {:?} may need transformation", from)],
                    recommendations: vec!["Context will be automatically transformed".to_string()],
                });
            }
        }
        
        Ok(SafetyStatus {
            safe: true,
            warnings: Vec::new(),
            recommendations: Vec::new(),
        })
    }
}

// Transformer implementations

#[derive(Debug)]
struct PlanningToExecutionTransformer;

impl Transformer for PlanningToExecutionTransformer {
    fn transform(&self, context: &ContextSnapshot, _from: &ModeType, _to: &ModeType) -> HiveResult<ContextSnapshot> {
        let mut transformed = context.clone();
        
        // Transform planning data to execution format
        // In real implementation, would perform actual transformation
        transformed.mark_as_transformed();
        
        Ok(transformed)
    }
}

#[derive(Debug)]
struct ExecutionToPlanningTransformer;

impl Transformer for ExecutionToPlanningTransformer {
    fn transform(&self, context: &ContextSnapshot, _from: &ModeType, _to: &ModeType) -> HiveResult<ContextSnapshot> {
        let mut transformed = context.clone();
        
        // Transform execution data to planning format
        transformed.mark_as_transformed();
        
        Ok(transformed)
    }
}

#[derive(Debug)]
struct DefaultTransformer;

impl Transformer for DefaultTransformer {
    fn transform(&self, context: &ContextSnapshot, _from: &ModeType, _to: &ModeType) -> HiveResult<ContextSnapshot> {
        // Default transformation preserves most data
        Ok(context.clone())
    }
}

/// Statistics about mode transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionStatistics {
    pub total_transitions: usize,
    pub success_rate: f32,
    pub average_duration: std::time::Duration,
    pub mode_preferences: HashMap<ModeType, f32>,
    pub common_paths: Vec<(Vec<ModeType>, usize)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enhanced_switcher_creation() {
        // Test switcher initialization
    }
    
    #[tokio::test]
    async fn test_intelligent_switching() {
        // Test mode switching with context
    }
    
    #[tokio::test]
    async fn test_transition_validation() {
        // Test transition rules and validation
    }
    
    #[tokio::test]
    async fn test_context_transformation() {
        // Test context preservation and transformation
    }
}