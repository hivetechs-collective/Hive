//! Mode Switching System
//!
//! Handles seamless transitions between different operating modes

use crate::core::error::{HiveError, HiveResult};
use crate::planning::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mode switching engine
pub struct ModeSwitcher {
    transition_rules: TransitionRules,
    context_preservers: HashMap<ModeType, Box<dyn ContextPreserver + Send + Sync>>,
}

/// Rules for mode transitions
#[derive(Debug)]
struct TransitionRules {
    allowed_transitions: HashMap<ModeType, Vec<ModeType>>,
    transition_costs: HashMap<(ModeType, ModeType), f32>,
    cooldown_periods: HashMap<ModeType, std::time::Duration>,
}

/// Preserves context during mode transitions
trait ContextPreserver {
    fn preserve(&self, context: &ModeContext) -> HiveResult<PreservedContext>;
    fn restore(&self, preserved: &PreservedContext) -> HiveResult<ModeContext>;
}

/// Context for a specific mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeContext {
    pub mode: ModeType,
    pub active_tasks: Vec<String>,
    pub cached_data: HashMap<String, serde_json::Value>,
    pub user_state: UserState,
    pub session_history: Vec<ModeAction>,
}

/// User state within a mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    pub focus_area: String,
    pub completion_percentage: f32,
    pub last_action: Option<ModeAction>,
    pub preferences: ModePreferences,
}

/// Preferences specific to a mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModePreferences {
    pub auto_save_interval: std::time::Duration,
    pub prompt_style: PromptStyle,
    pub output_verbosity: OutputVerbosity,
}

/// Style of prompts in different modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromptStyle {
    Concise,
    Standard,
    Detailed,
    Interactive,
    Guided,
}

/// Output verbosity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputVerbosity {
    Minimal,
    Standard,
    Verbose,
    Debug,
}

/// Action taken in a mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeAction {
    pub action_type: ActionType,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Type of action taken
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    Query,
    TaskCreation,
    TaskExecution,
    Analysis,
    Planning,
    ModeSwitch,
}

/// Preserved context during transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservedContext {
    pub essential_data: HashMap<String, serde_json::Value>,
    pub task_states: HashMap<String, TaskState>,
    pub user_progress: f32,
    pub breadcrumbs: Vec<String>,
}

/// State of a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub task_id: String,
    pub progress: f32,
    pub current_step: String,
    pub notes: Vec<String>,
}

/// Result of a mode switch operation
#[derive(Debug, Clone)]
pub struct SwitchResult {
    pub success: bool,
    pub from_mode: ModeType,
    pub to_mode: ModeType,
    pub preserved_context: Option<PreservedContext>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

impl ModeSwitcher {
    pub fn new() -> Self {
        Self {
            transition_rules: Self::init_transition_rules(),
            context_preservers: Self::init_context_preservers(),
        }
    }

    /// Switch from current mode to target mode
    pub async fn switch(
        &self,
        current_mode: &ModeType,
        target_mode: ModeType,
        context: &PlanningContext,
    ) -> HiveResult<ModeType> {
        let switch_result = self
            .execute_switch(current_mode, &target_mode, context)
            .await?;

        if switch_result.success {
            Ok(switch_result.to_mode)
        } else {
            Err(HiveError::Planning(format!(
                "Failed to switch from {:?} to {:?}",
                current_mode, target_mode
            )))
        }
    }

    /// Execute a mode switch with full context preservation
    pub async fn execute_switch(
        &self,
        from_mode: &ModeType,
        to_mode: &ModeType,
        context: &PlanningContext,
    ) -> HiveResult<SwitchResult> {
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Check if transition is allowed
        if !self.is_transition_allowed(from_mode, to_mode) {
            return Ok(SwitchResult {
                success: false,
                from_mode: from_mode.clone(),
                to_mode: to_mode.clone(),
                preserved_context: None,
                warnings: vec![format!(
                    "Direct transition from {:?} to {:?} not allowed",
                    from_mode, to_mode
                )],
                recommendations: self.suggest_transition_path(from_mode, to_mode),
            });
        }

        // Check transition cost
        let cost = self.calculate_transition_cost(from_mode, to_mode);
        if cost > 0.8 {
            warnings.push(format!("High transition cost: {:.2}", cost));
            recommendations.push("Consider using Hybrid mode as intermediate step".to_string());
        }

        // Preserve current context
        let preserved_context = self.preserve_context(from_mode, context).await?;

        // Prepare target mode
        self.prepare_target_mode(to_mode, &preserved_context, context)
            .await?;

        // Execute transition
        let success = self
            .perform_transition(from_mode, to_mode, &preserved_context)
            .await?;

        // Post-transition validation
        if success {
            self.validate_transition(to_mode, &preserved_context)
                .await?;
            recommendations.extend(self.get_mode_recommendations(to_mode, context));
        }

        Ok(SwitchResult {
            success,
            from_mode: from_mode.clone(),
            to_mode: to_mode.clone(),
            preserved_context: Some(preserved_context),
            warnings,
            recommendations,
        })
    }

    /// Check if transition between modes is allowed
    pub fn is_transition_allowed(&self, from: &ModeType, to: &ModeType) -> bool {
        if from == to {
            return true; // Same mode is always allowed
        }

        self.transition_rules
            .allowed_transitions
            .get(from)
            .map(|allowed| allowed.contains(to))
            .unwrap_or(false)
    }

    /// Get optimal path for mode transition
    pub fn get_transition_path(&self, from: &ModeType, to: &ModeType) -> HiveResult<Vec<ModeType>> {
        if from == to {
            return Ok(vec![from.clone()]);
        }

        if self.is_transition_allowed(from, to) {
            return Ok(vec![from.clone(), to.clone()]);
        }

        // Find path through intermediate modes (simple BFS)
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        let mut parent: std::collections::HashMap<ModeType, ModeType> =
            std::collections::HashMap::new();

        queue.push_back(from.clone());
        visited.insert(from.clone());

        while let Some(current) = queue.pop_front() {
            if current == *to {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = to.clone();

                while let Some(prev) = parent.get(&node) {
                    path.push(node.clone());
                    node = prev.clone();
                }
                path.push(from.clone());
                path.reverse();

                return Ok(path);
            }

            if let Some(neighbors) = self.transition_rules.allowed_transitions.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        parent.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        Err(HiveError::Planning(format!(
            "No transition path from {:?} to {:?}",
            from, to
        )))
    }

    /// Calculate cost of transitioning between modes
    pub fn calculate_transition_cost(&self, from: &ModeType, to: &ModeType) -> f32 {
        if from == to {
            return 0.0;
        }

        self.transition_rules
            .transition_costs
            .get(&(from.clone(), to.clone()))
            .copied()
            .unwrap_or(1.0) // Default high cost for undefined transitions
    }

    /// Get recommendations for effective mode usage
    pub fn get_mode_recommendations(
        &self,
        mode: &ModeType,
        context: &PlanningContext,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        match mode {
            ModeType::Planning => {
                recommendations
                    .push("Focus on breaking down complex tasks into manageable steps".to_string());
                recommendations
                    .push("Use risk analysis to identify potential issues early".to_string());
                if context.team_size > 1 {
                    recommendations.push(
                        "Consider collaborative planning features for team coordination"
                            .to_string(),
                    );
                }
            }
            ModeType::Execution => {
                recommendations.push("Focus on implementing one task at a time".to_string());
                recommendations.push("Use quick feedback loops for rapid iteration".to_string());
                if context.existing_codebase {
                    recommendations.push(
                        "Run tests frequently to ensure changes don't break existing functionality"
                            .to_string(),
                    );
                }
            }
            ModeType::Hybrid => {
                recommendations
                    .push("Balance planning and execution based on task complexity".to_string());
                recommendations
                    .push("Switch to focused modes when needed for deep work".to_string());
                recommendations.push(
                    "Use this mode for complex projects requiring multiple perspectives"
                        .to_string(),
                );
            }
            ModeType::Analysis => {
                recommendations.push(
                    "Use repository intelligence for comprehensive code understanding".to_string(),
                );
                recommendations
                    .push("Focus on understanding patterns and relationships".to_string());
                recommendations.push("Document findings for future reference".to_string());
            }
            ModeType::Learning => {
                recommendations.push("Adapt to user preferences and working patterns".to_string());
                recommendations.push("Provide suggestions based on historical data".to_string());
            }
        }

        recommendations
    }

    // Private helper methods

    async fn preserve_context(
        &self,
        mode: &ModeType,
        context: &PlanningContext,
    ) -> HiveResult<PreservedContext> {
        // Create a minimal preserved context
        // In a real implementation, this would save current state
        Ok(PreservedContext {
            essential_data: HashMap::new(),
            task_states: HashMap::new(),
            user_progress: 0.0,
            breadcrumbs: vec!["mode_switch".to_string()],
        })
    }

    async fn prepare_target_mode(
        &self,
        mode: &ModeType,
        preserved: &PreservedContext,
        context: &PlanningContext,
    ) -> HiveResult<()> {
        // Prepare the target mode with preserved context
        // In a real implementation, this would set up mode-specific resources
        Ok(())
    }

    async fn perform_transition(
        &self,
        from: &ModeType,
        to: &ModeType,
        preserved: &PreservedContext,
    ) -> HiveResult<bool> {
        // Simulate transition delay based on mode complexity
        let delay = match (from, to) {
            (ModeType::Execution, ModeType::Planning) => std::time::Duration::from_millis(100),
            (ModeType::Planning, ModeType::Execution) => std::time::Duration::from_millis(50),
            (_, ModeType::Hybrid) => std::time::Duration::from_millis(75),
            (ModeType::Hybrid, _) => std::time::Duration::from_millis(25),
            _ => std::time::Duration::from_millis(50),
        };

        tokio::time::sleep(delay).await;
        Ok(true)
    }

    async fn validate_transition(
        &self,
        mode: &ModeType,
        preserved: &PreservedContext,
    ) -> HiveResult<()> {
        // Validate that the transition was successful
        // In a real implementation, this would check mode state
        Ok(())
    }

    fn suggest_transition_path(&self, from: &ModeType, to: &ModeType) -> Vec<String> {
        // Try to find a path through Hybrid mode
        if self.is_transition_allowed(from, &ModeType::Hybrid)
            && self.is_transition_allowed(&ModeType::Hybrid, to)
        {
            vec![format!(
                "Try switching to Hybrid mode first, then to {:?}",
                to
            )]
        } else {
            vec!["Direct transition not available".to_string()]
        }
    }

    // Initialization methods

    fn init_transition_rules() -> TransitionRules {
        let mut allowed_transitions = HashMap::new();

        // Planning mode transitions
        allowed_transitions.insert(
            ModeType::Planning,
            vec![ModeType::Execution, ModeType::Hybrid, ModeType::Analysis],
        );

        // Execution mode transitions
        allowed_transitions.insert(
            ModeType::Execution,
            vec![ModeType::Planning, ModeType::Hybrid, ModeType::Analysis],
        );

        // Hybrid mode can transition to any mode
        allowed_transitions.insert(
            ModeType::Hybrid,
            vec![
                ModeType::Planning,
                ModeType::Execution,
                ModeType::Analysis,
                ModeType::Learning,
            ],
        );

        // Analysis mode transitions
        allowed_transitions.insert(
            ModeType::Analysis,
            vec![ModeType::Planning, ModeType::Execution, ModeType::Hybrid],
        );

        // Learning mode transitions
        allowed_transitions.insert(ModeType::Learning, vec![ModeType::Hybrid]);

        let mut transition_costs = HashMap::new();

        // Low cost transitions (similar contexts)
        transition_costs.insert((ModeType::Planning, ModeType::Execution), 0.3);
        transition_costs.insert((ModeType::Execution, ModeType::Planning), 0.4);
        transition_costs.insert((ModeType::Hybrid, ModeType::Planning), 0.2);
        transition_costs.insert((ModeType::Hybrid, ModeType::Execution), 0.2);
        transition_costs.insert((ModeType::Planning, ModeType::Hybrid), 0.2);
        transition_costs.insert((ModeType::Execution, ModeType::Hybrid), 0.2);

        // Medium cost transitions
        transition_costs.insert((ModeType::Planning, ModeType::Analysis), 0.5);
        transition_costs.insert((ModeType::Execution, ModeType::Analysis), 0.4);
        transition_costs.insert((ModeType::Analysis, ModeType::Planning), 0.4);
        transition_costs.insert((ModeType::Analysis, ModeType::Execution), 0.3);

        // High cost transitions
        transition_costs.insert((ModeType::Learning, ModeType::Hybrid), 0.6);
        transition_costs.insert((ModeType::Hybrid, ModeType::Learning), 0.7);

        let mut cooldown_periods = HashMap::new();
        cooldown_periods.insert(ModeType::Planning, std::time::Duration::from_secs(30));
        cooldown_periods.insert(ModeType::Execution, std::time::Duration::from_secs(10));
        cooldown_periods.insert(ModeType::Hybrid, std::time::Duration::from_secs(60));
        cooldown_periods.insert(ModeType::Analysis, std::time::Duration::from_secs(45));
        cooldown_periods.insert(ModeType::Learning, std::time::Duration::from_secs(120));

        TransitionRules {
            allowed_transitions,
            transition_costs,
            cooldown_periods,
        }
    }

    fn init_context_preservers() -> HashMap<ModeType, Box<dyn ContextPreserver + Send + Sync>> {
        // In a real implementation, would initialize mode-specific preservers
        HashMap::new()
    }
}

// Default context preserver implementation
struct DefaultContextPreserver;

impl ContextPreserver for DefaultContextPreserver {
    fn preserve(&self, context: &ModeContext) -> HiveResult<PreservedContext> {
        Ok(PreservedContext {
            essential_data: context.cached_data.clone(),
            task_states: HashMap::new(),
            user_progress: context.user_state.completion_percentage,
            breadcrumbs: context
                .session_history
                .iter()
                .map(|action| action.description.clone())
                .collect(),
        })
    }

    fn restore(&self, preserved: &PreservedContext) -> HiveResult<ModeContext> {
        Ok(ModeContext {
            mode: ModeType::Hybrid,
            active_tasks: Vec::new(),
            cached_data: preserved.essential_data.clone(),
            user_state: UserState {
                focus_area: "restored".to_string(),
                completion_percentage: preserved.user_progress,
                last_action: None,
                preferences: ModePreferences {
                    auto_save_interval: std::time::Duration::from_secs(60),
                    prompt_style: PromptStyle::Standard,
                    output_verbosity: OutputVerbosity::Standard,
                },
            },
            session_history: Vec::new(),
        })
    }
}

impl Default for ModePreferences {
    fn default() -> Self {
        Self {
            auto_save_interval: std::time::Duration::from_secs(60),
            prompt_style: PromptStyle::Standard,
            output_verbosity: OutputVerbosity::Standard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_switcher_creation() {
        let switcher = ModeSwitcher::new();
        assert!(!switcher.transition_rules.allowed_transitions.is_empty());
    }

    #[test]
    fn test_allowed_transitions() {
        let switcher = ModeSwitcher::new();

        // Test direct transitions
        assert!(switcher.is_transition_allowed(&ModeType::Planning, &ModeType::Execution));
        assert!(switcher.is_transition_allowed(&ModeType::Execution, &ModeType::Planning));
        assert!(switcher.is_transition_allowed(&ModeType::Hybrid, &ModeType::Planning));

        // Test same mode
        assert!(switcher.is_transition_allowed(&ModeType::Planning, &ModeType::Planning));
    }

    #[test]
    fn test_transition_costs() {
        let switcher = ModeSwitcher::new();

        let cost_low =
            switcher.calculate_transition_cost(&ModeType::Planning, &ModeType::Execution);
        let cost_same =
            switcher.calculate_transition_cost(&ModeType::Planning, &ModeType::Planning);

        assert!(cost_same == 0.0);
        assert!(cost_low > 0.0 && cost_low < 1.0);
    }

    #[test]
    fn test_transition_path() {
        let switcher = ModeSwitcher::new();

        // Direct path
        let path = switcher
            .get_transition_path(&ModeType::Planning, &ModeType::Execution)
            .unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], ModeType::Planning);
        assert_eq!(path[1], ModeType::Execution);

        // Same mode
        let path = switcher
            .get_transition_path(&ModeType::Planning, &ModeType::Planning)
            .unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(path[0], ModeType::Planning);
    }

    #[test]
    fn test_mode_recommendations() {
        let switcher = ModeSwitcher::new();
        let context = PlanningContext::default();

        let recommendations = switcher.get_mode_recommendations(&ModeType::Planning, &context);
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("break down")));

        let recommendations = switcher.get_mode_recommendations(&ModeType::Execution, &context);
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("implementing")));
    }

    #[tokio::test]
    async fn test_mode_switch() {
        let switcher = ModeSwitcher::new();
        let context = PlanningContext::default();

        let result = switcher
            .switch(&ModeType::Planning, ModeType::Execution, &context)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ModeType::Execution);
    }
}
