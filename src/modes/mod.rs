//! Dual-Mode Operation System for HiveTechs Consensus
//! 
//! Provides intelligent mode detection, seamless switching, and context preservation
//! for planning and execution modes with user preference learning.

pub mod detector;
pub mod switcher;
pub mod hybrid;
pub mod context;
pub mod preferences;
pub mod visualization;

use crate::core::error::HiveResult;
use crate::planning::{ModeType, PlanningContext};
use crate::consensus::ConsensusEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub use self::detector::{EnhancedModeDetector, DetectionResult};
pub use self::switcher::{EnhancedModeSwitcher, SwitchResult};
pub use self::hybrid::{HybridModeEngine, HybridTask};
pub use self::context::{ContextManager, ModeContext, ContextSnapshot};
pub use self::preferences::{PreferenceManager, UserPreference, LearningData};
pub use self::visualization::{ModeVisualizer, ModeStatus};

/// Dual-Mode Operation Manager
/// 
/// Orchestrates intelligent mode switching between planning and execution
/// with context preservation and user preference learning.
pub struct ModeManager {
    detector: Arc<RwLock<EnhancedModeDetector>>,
    switcher: Arc<RwLock<EnhancedModeSwitcher>>,
    hybrid_engine: Arc<RwLock<HybridModeEngine>>,
    context_manager: Arc<RwLock<ContextManager>>,
    preference_manager: Arc<RwLock<PreferenceManager>>,
    visualizer: Arc<RwLock<ModeVisualizer>>,
    consensus_engine: Arc<ConsensusEngine>,
    current_mode: Arc<RwLock<ModeType>>,
}

impl ModeManager {
    /// Create a new mode manager
    pub async fn new(consensus_engine: Arc<ConsensusEngine>) -> HiveResult<Self> {
        let detector = Arc::new(RwLock::new(
            EnhancedModeDetector::new(consensus_engine.clone()).await?
        ));
        
        let switcher = Arc::new(RwLock::new(
            EnhancedModeSwitcher::new(consensus_engine.clone()).await?
        ));
        
        let hybrid_engine = Arc::new(RwLock::new(
            HybridModeEngine::new(consensus_engine.clone()).await?
        ));
        
        let context_manager = Arc::new(RwLock::new(
            ContextManager::new().await?
        ));
        
        let preference_manager = Arc::new(RwLock::new(
            PreferenceManager::new().await?
        ));
        
        let visualizer = Arc::new(RwLock::new(
            ModeVisualizer::new()
        ));
        
        Ok(Self {
            detector,
            switcher,
            hybrid_engine,
            context_manager,
            preference_manager,
            visualizer,
            consensus_engine,
            current_mode: Arc::new(RwLock::new(ModeType::Hybrid)),
        })
    }
    
    /// Detect the appropriate mode for a query
    pub async fn detect_mode(&self, query: &str, context: &PlanningContext) -> HiveResult<DetectionResult> {
        // Apply user preferences to context
        let enhanced_context = {
            let pref_manager = self.preference_manager.read().await;
            pref_manager.enhance_context(context.clone()).await?
        };
        
        // Detect mode using enhanced detector with consensus
        let detector = self.detector.read().await;
        let result = detector.detect_with_consensus(query, &enhanced_context).await?;
        
        // Learn from detection
        {
            let mut pref_manager = self.preference_manager.write().await;
            pref_manager.learn_from_detection(query, &result).await?;
        }
        
        // Update visualization
        {
            let mut visualizer = self.visualizer.write().await;
            visualizer.update_detection_result(&result);
        }
        
        Ok(result)
    }
    
    /// Switch to a different mode with context preservation
    pub async fn switch_mode(&self, target_mode: ModeType, preserve_context: bool) -> HiveResult<SwitchResult> {
        let current_mode = *self.current_mode.read().await;
        
        // Preserve context if requested
        let context_snapshot = if preserve_context {
            let ctx_manager = self.context_manager.read().await;
            Some(ctx_manager.capture_snapshot(&current_mode).await?)
        } else {
            None
        };
        
        // Execute switch
        let switcher = self.switcher.read().await;
        let result = switcher.switch_with_intelligence(
            &current_mode,
            &target_mode,
            context_snapshot.as_ref(),
            &self.consensus_engine
        ).await?;
        
        // Update current mode if successful
        if result.success {
            *self.current_mode.write().await = result.to_mode.clone();
            
            // Restore context if preserved
            if let Some(snapshot) = context_snapshot {
                let mut ctx_manager = self.context_manager.write().await;
                ctx_manager.restore_snapshot(&result.to_mode, &snapshot).await?;
            }
            
            // Learn from switch
            let mut pref_manager = self.preference_manager.write().await;
            pref_manager.learn_from_switch(&result).await?;
        }
        
        // Update visualization
        {
            let mut visualizer = self.visualizer.write().await;
            visualizer.update_switch_result(&result);
        }
        
        Ok(result)
    }
    
    /// Execute a task in hybrid mode
    pub async fn execute_hybrid(&self, query: &str, context: &PlanningContext) -> HiveResult<HybridTask> {
        let hybrid_engine = self.hybrid_engine.read().await;
        let task = hybrid_engine.create_hybrid_task(query, context).await?;
        
        // Execute with mode transitions
        let result = hybrid_engine.execute_with_transitions(
            &task,
            &self.switcher,
            &self.context_manager
        ).await?;
        
        // Learn from execution
        {
            let mut pref_manager = self.preference_manager.write().await;
            pref_manager.learn_from_hybrid_execution(&result).await?;
        }
        
        Ok(result)
    }
    
    /// Get current mode status
    pub async fn get_status(&self) -> HiveResult<ModeStatus> {
        let current_mode = *self.current_mode.read().await;
        let visualizer = self.visualizer.read().await;
        let status = visualizer.get_current_status(&current_mode);
        Ok(status)
    }
    
    /// Get mode recommendation with confidence
    pub async fn get_recommendation(&self, query: &str, context: &PlanningContext) -> HiveResult<ModeRecommendation> {
        let result = self.detect_mode(query, context).await?;
        
        Ok(ModeRecommendation {
            recommended_mode: result.primary_mode,
            confidence: result.confidence,
            reasoning: result.reasoning.join("; "),
            alternatives: result.alternatives,
            user_preference_weight: result.preference_influence,
        })
    }
    
    /// Update user preferences
    pub async fn update_preferences(&self, preferences: UserPreference) -> HiveResult<()> {
        let mut pref_manager = self.preference_manager.write().await;
        pref_manager.update_preferences(preferences).await
    }
    
    /// Get learning statistics
    pub async fn get_learning_stats(&self) -> HiveResult<LearningStats> {
        let pref_manager = self.preference_manager.read().await;
        let stats = pref_manager.get_learning_stats().await?;
        Ok(stats)
    }
    
    /// Reset to default mode
    pub async fn reset_mode(&self) -> HiveResult<()> {
        *self.current_mode.write().await = ModeType::Hybrid;
        
        let mut ctx_manager = self.context_manager.write().await;
        ctx_manager.clear_context().await?;
        
        let mut visualizer = self.visualizer.write().await;
        visualizer.reset();
        
        Ok(())
    }
}

/// Mode recommendation with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeRecommendation {
    pub recommended_mode: ModeType,
    pub confidence: f32,
    pub reasoning: String,
    pub alternatives: Vec<(ModeType, f32)>,
    pub user_preference_weight: f32,
}

/// Learning statistics for mode operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStats {
    pub total_detections: usize,
    pub total_switches: usize,
    pub mode_accuracy: f32,
    pub preference_influence: f32,
    pub top_patterns: Vec<(String, usize)>,
    pub mode_distribution: std::collections::HashMap<ModeType, usize>,
}

/// Mode operation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub auto_switch: bool,
    pub learning_enabled: bool,
    pub visualization_enabled: bool,
    pub context_preservation: ContextPreservationLevel,
    pub confidence_threshold: f32,
}

/// Level of context preservation during mode switches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextPreservationLevel {
    Minimal,
    Standard,
    Full,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            auto_switch: true,
            learning_enabled: true,
            visualization_enabled: true,
            context_preservation: ContextPreservationLevel::Standard,
            confidence_threshold: 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mode_manager_creation() {
        // Test mode manager initialization
    }
    
    #[tokio::test]
    async fn test_mode_detection() {
        // Test intelligent mode detection
    }
    
    #[tokio::test]
    async fn test_mode_switching() {
        // Test seamless mode switching
    }
    
    #[tokio::test]
    async fn test_hybrid_execution() {
        // Test hybrid mode execution
    }
}