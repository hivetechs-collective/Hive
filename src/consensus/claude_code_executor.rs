//! Claude Code Executor
//!
//! The revolutionary hybrid executor that bridges Claude Code's unified intelligence
//! with the consensus pipeline's multi-model validation capabilities.
//! 
//! This replaces the DirectExecutionHandler with a more sophisticated system that can:
//! - Execute directly with Claude's capabilities
//! - Intelligently invoke consensus for validation
//! - Support three execution modes (Direct, ConsensusAssisted, ConsensusRequired)
//! - Accumulate knowledge from curator outputs

use std::sync::Arc;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, RwLock, Mutex};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use rusqlite::params;

use crate::consensus::streaming::{StreamingCallbacks, ConsensusEvent};
use crate::consensus::types::{Stage, StageResult, ConsensusProfile, ConsensusResult};
use crate::consensus::pipeline::ConsensusPipeline;
use crate::core::database::DatabaseManager;
use crate::consensus::memory::ConsensusMemory;
use crate::ai_helpers::AIHelperEcosystem;

/// Execution modes for the Claude-Consensus bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaudeExecutionMode {
    /// Claude executes directly with full autonomy
    Direct,
    
    /// Claude must validate major decisions through consensus
    ConsensusAssisted,
    
    /// All Claude plans go through consensus validation (plan mode)
    ConsensusRequired,
}

impl Default for ClaudeExecutionMode {
    fn default() -> Self {
        ClaudeExecutionMode::ConsensusAssisted
    }
}

/// Criteria for deciding when to invoke consensus
#[derive(Debug, Clone)]
pub struct ConsensusInvocationCriteria {
    /// Architectural decisions affecting system design
    pub architectural_change: bool,
    
    /// High-stakes operations (data deletion, major refactoring)
    pub high_risk_operation: bool,
    
    /// Claude's confidence level (0.0 - 1.0)
    pub confidence_level: f64,
    
    /// Multiple valid approaches detected
    pub multiple_approaches: bool,
    
    /// User explicitly requested thoughtful analysis
    pub user_requested_analysis: bool,
    
    /// Complex multi-step operations
    pub complexity_score: f64,
}

/// Analysis of a request to determine execution strategy
#[derive(Debug)]
struct RequestAnalysis {
    pub uncertainty: f64,
    pub impact: Impact,
    pub involves_architecture_decision: bool,
    pub planned_approach: String,
}

#[derive(Debug, PartialEq)]
enum Impact {
    Low,
    Medium,
    High,
}

/// Claude Code process manager
struct ClaudeCodeProcess {
    process: Option<tokio::process::Child>,
    stdin_tx: Option<mpsc::Sender<String>>,
    anthropic_key: String,
}

impl ClaudeCodeProcess {
    /// Start the Claude Code process
    async fn start(anthropic_key: String) -> Result<Self> {
        info!("🚀 Starting Claude Code process...");
        
        let mut cmd = Command::new("claude-code");
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("ANTHROPIC_API_KEY", &anthropic_key);
            
        let mut process = cmd.spawn()
            .context("Failed to start Claude Code process")?;
            
        // Create channel for sending input to Claude
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(100);
        
        // Get stdin handle
        let stdin = process.stdin.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin handle"))?;
            
        // Spawn task to forward messages to Claude's stdin
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let mut stdin = stdin;
            
            while let Some(msg) = stdin_rx.recv().await {
                if let Err(e) = stdin.write_all(msg.as_bytes()).await {
                    error!("Failed to write to Claude stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.write_all(b"\n").await {
                    error!("Failed to write newline to Claude stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    error!("Failed to flush Claude stdin: {}", e);
                    break;
                }
            }
        });
        
        Ok(Self {
            process: Some(process),
            stdin_tx: Some(stdin_tx),
            anthropic_key,
        })
    }
    
    /// Send a message to Claude
    async fn send_message(&self, message: &str) -> Result<()> {
        if let Some(tx) = &self.stdin_tx {
            tx.send(message.to_string()).await
                .context("Failed to send message to Claude")?;
        }
        Ok(())
    }
    
    /// Stop the Claude Code process
    async fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill().await.context("Failed to kill Claude process")?;
        }
        Ok(())
    }
}

/// The main Claude Code executor
pub struct ClaudeCodeExecutor {
    /// Current execution mode
    mode: Arc<RwLock<ClaudeExecutionMode>>,
    
    /// Claude Code process handle
    claude_process: Arc<Mutex<Option<ClaudeCodeProcess>>>,
    
    /// Consensus pipeline for validation
    consensus_pipeline: Option<Arc<ConsensusPipeline>>,
    
    /// Knowledge base for storing curator outputs
    knowledge_base: Option<Arc<ConsensusMemory>>,
    
    /// Database for persistence
    database: Arc<DatabaseManager>,
    
    /// AI helpers for intelligent operations
    ai_helpers: Arc<AIHelperEcosystem>,
    
    /// Current consensus profile
    profile: ConsensusProfile,
    
    /// Anthropic API key
    anthropic_key: Option<String>,
}

impl ClaudeCodeExecutor {
    /// Create a new Claude Code executor
    pub fn new(
        profile: ConsensusProfile,
        database: Arc<DatabaseManager>,
        ai_helpers: Arc<AIHelperEcosystem>,
        knowledge_base: Option<Arc<ConsensusMemory>>,
        anthropic_key: Option<String>,
    ) -> Self {
        Self {
            mode: Arc::new(RwLock::new(ClaudeExecutionMode::default())),
            claude_process: Arc::new(Mutex::new(None)),
            consensus_pipeline: None,
            knowledge_base,
            database,
            ai_helpers,
            profile,
            anthropic_key,
        }
    }
    
    /// Set the consensus pipeline
    pub fn with_consensus_pipeline(mut self, pipeline: Arc<ConsensusPipeline>) -> Self {
        self.consensus_pipeline = Some(pipeline);
        self
    }
    
    /// Set the execution mode
    pub async fn set_mode(&self, mode: ClaudeExecutionMode) {
        *self.mode.write().await = mode;
        info!("🔄 Execution mode changed to: {:?}", mode);
    }
    
    /// Get the current execution mode
    pub async fn get_mode(&self) -> ClaudeExecutionMode {
        *self.mode.read().await
    }
    
    /// Initialize Claude Code process
    pub async fn initialize(&self) -> Result<()> {
        let anthropic_key = self.anthropic_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Anthropic API key not configured"))?;
            
        let mut process_guard = self.claude_process.lock().await;
        if process_guard.is_none() {
            let process = ClaudeCodeProcess::start(anthropic_key.clone()).await?;
            *process_guard = Some(process);
            info!("✅ Claude Code process initialized");
        }
        
        Ok(())
    }
    
    /// Execute a request using the appropriate strategy
    pub async fn execute(
        &self,
        request: &str,
        context: Option<&str>,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<ConsensusResult> {
        // Ensure Claude process is running
        self.initialize().await?;
        
        let mode = self.get_mode().await;
        info!("🤖 Executing in {:?} mode", mode);
        
        match mode {
            ClaudeExecutionMode::Direct => {
                self.execute_with_optional_consensus(request, context, callbacks).await
            }
            ClaudeExecutionMode::ConsensusAssisted => {
                self.execute_with_smart_consensus(request, context, callbacks).await
            }
            ClaudeExecutionMode::ConsensusRequired => {
                self.execute_with_mandatory_consensus(request, context, callbacks).await
            }
        }
    }
    
    /// Execute with optional consensus consultation
    async fn execute_with_optional_consensus(
        &self,
        request: &str,
        context: Option<&str>,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<ConsensusResult> {
        // Analyze the request
        let analysis = self.analyze_request(request).await?;
        
        // Check if we should invoke consensus
        let criteria = ConsensusInvocationCriteria {
            architectural_change: analysis.involves_architecture_decision,
            high_risk_operation: analysis.impact == Impact::High,
            confidence_level: 1.0 - analysis.uncertainty,
            multiple_approaches: false, // Would need deeper analysis
            user_requested_analysis: request.contains("analyze") || request.contains("explain"),
            complexity_score: if analysis.impact == Impact::High { 0.9 } else { 0.3 },
        };
        
        if self.should_invoke_consensus(&criteria) {
            info!("🔍 Invoking consensus for validation...");
            
            // Get Claude's initial plan
            let claude_plan = self.get_claude_plan(request, context).await?;
            
            // Get consensus opinion
            if let Some(pipeline) = &self.consensus_pipeline {
                let consensus_result = pipeline.run(
                    &format!("Validate this approach: {}", claude_plan),
                    None, // context
                    None, // user_id
                ).await?;
                
                // Execute with consensus insights
                self.execute_with_insights(request, context, consensus_result, callbacks).await
            } else {
                // No consensus pipeline, execute directly
                self.execute_directly(request, context, callbacks).await
            }
        } else {
            // Execute directly with confidence
            self.execute_directly(request, context, callbacks).await
        }
    }
    
    /// Execute with smart consensus invocation
    async fn execute_with_smart_consensus(
        &self,
        request: &str,
        context: Option<&str>,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<ConsensusResult> {
        // Similar to optional but with lower thresholds
        self.execute_with_optional_consensus(request, context, callbacks).await
    }
    
    /// Execute with mandatory consensus validation
    async fn execute_with_mandatory_consensus(
        &self,
        request: &str,
        context: Option<&str>,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<ConsensusResult> {
        info!("📋 Generating comprehensive plan for consensus validation...");
        
        // Step 1: Get Claude's comprehensive plan
        let claude_plan = self.generate_comprehensive_plan(request, context).await?;
        
        // Step 2: Send to consensus for evaluation
        if let Some(pipeline) = &self.consensus_pipeline {
            let consensus_result = pipeline.run(
                &format!("Request: {}\n\nProposed Plan:\n{}", request, claude_plan.clone()),
                context.map(|c| c.to_string()), // context as Option<String>
                None, // user_id
            ).await?;
            
            // Step 3: Extract curator guidance from the result
            let curator_guidance = consensus_result.result.clone().unwrap_or_default();
            
            // Step 4: Execute with curator-validated approach
            let execution_result = self.execute_with_guidance(
                request,
                context,
                claude_plan.clone(),
                curator_guidance,
                callbacks,
            ).await?;
            
            // Step 5: Store as authoritative knowledge
            self.store_authoritative_knowledge(
                request,
                &claude_plan,
                &consensus_result,
                &execution_result,
            ).await?;
            
            Ok(execution_result)
        } else {
            // Fallback to direct execution if no consensus pipeline
            warn!("No consensus pipeline available, falling back to direct execution");
            self.execute_directly(request, context, callbacks).await
        }
    }
    
    /// Analyze a request to determine execution strategy
    async fn analyze_request(&self, request: &str) -> Result<RequestAnalysis> {
        let request_lower = request.to_lowercase();
        
        // Simple heuristics for now - can be enhanced with ML
        let uncertainty = if request_lower.contains("?") { 0.3 } else { 0.1 };
        
        let impact = if request_lower.contains("delete") || 
                       request_lower.contains("remove") ||
                       request_lower.contains("refactor") ||
                       request_lower.contains("architecture") {
            Impact::High
        } else if request_lower.contains("update") ||
                  request_lower.contains("modify") {
            Impact::Medium
        } else {
            Impact::Low
        };
        
        let involves_architecture_decision = 
            request_lower.contains("architecture") ||
            request_lower.contains("design") ||
            request_lower.contains("structure") ||
            request_lower.contains("system");
        
        Ok(RequestAnalysis {
            uncertainty,
            impact,
            involves_architecture_decision,
            planned_approach: format!("Direct execution for: {}", request),
        })
    }
    
    /// Check if consensus should be invoked
    fn should_invoke_consensus(&self, criteria: &ConsensusInvocationCriteria) -> bool {
        // Always invoke if user requested or high risk
        if criteria.user_requested_analysis || criteria.high_risk_operation {
            return true;
        }
        
        // Invoke for architectural decisions
        if criteria.architectural_change {
            return true;
        }
        
        // Invoke when uncertain or complex
        if criteria.confidence_level < 0.6 || criteria.complexity_score > 0.8 {
            return true;
        }
        
        // Invoke when multiple valid approaches exist
        criteria.multiple_approaches
    }
    
    /// Get Claude's initial plan for a request
    async fn get_claude_plan(&self, request: &str, context: Option<&str>) -> Result<String> {
        // Send request to Claude for planning
        let process_guard = self.claude_process.lock().await;
        if let Some(process) = process_guard.as_ref() {
            let prompt = format!(
                "Provide a brief plan for this request: {}\nContext: {}",
                request,
                context.unwrap_or("No additional context")
            );
            process.send_message(&prompt).await?;
            
            // TODO: Implement response collection from Claude stdout
            // For now, return a placeholder
            Ok(format!("Claude's plan for: {}", request))
        } else {
            Err(anyhow::anyhow!("Claude process not initialized"))
        }
    }
    
    /// Generate a comprehensive plan (for consensus mode)
    async fn generate_comprehensive_plan(&self, request: &str, context: Option<&str>) -> Result<String> {
        let process_guard = self.claude_process.lock().await;
        if let Some(process) = process_guard.as_ref() {
            let prompt = format!(
                "Generate a detailed, step-by-step plan for this request. Include all technical details, potential risks, and implementation steps:\n\nRequest: {}\nContext: {}",
                request,
                context.unwrap_or("No additional context")
            );
            process.send_message(&prompt).await?;
            
            // TODO: Implement response collection
            Ok(format!("Comprehensive plan for: {}", request))
        } else {
            Err(anyhow::anyhow!("Claude process not initialized"))
        }
    }
    
    /// Execute directly without consensus
    async fn execute_directly(
        &self,
        request: &str,
        context: Option<&str>,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<ConsensusResult> {
        info!("⚡ Executing directly with Claude Code...");
        
        callbacks.on_stage_start(Stage::Generator, "claude-direct")?;
        
        let process_guard = self.claude_process.lock().await;
        if let Some(process) = process_guard.as_ref() {
            let prompt = format!(
                "Execute this request: {}\nContext: {}",
                request,
                context.unwrap_or("No additional context")
            );
            process.send_message(&prompt).await?;
            
            // TODO: Implement streaming response collection
            let response = format!("Claude executed: {}", request);
            
            callbacks.on_stage_chunk(Stage::Generator, &response, &response)?;
            
            let stage_result = StageResult {
                stage_id: "claude-direct".to_string(),
                stage_name: "Claude Direct Execution".to_string(),
                question: request.to_string(),
                answer: response.clone(),
                model: "claude-3.5-sonnet".to_string(),
                conversation_id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                usage: None,
                analytics: None,
            };
            
            callbacks.on_stage_complete(Stage::Generator, &stage_result)?;
            
            Ok(ConsensusResult {
                success: true,
                result: Some(response),
                error: None,
                stages: vec![stage_result],
                conversation_id: Uuid::new_v4().to_string(),
                total_duration: 1.0,
                total_cost: 0.0,
            })
        } else {
            Err(anyhow::anyhow!("Claude process not initialized"))
        }
    }
    
    /// Execute with consensus insights
    async fn execute_with_insights(
        &self,
        request: &str,
        context: Option<&str>,
        consensus_result: ConsensusResult,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<ConsensusResult> {
        info!("🔄 Executing with consensus insights...");
        
        // Extract insights from consensus
        let default_insights = String::new();
        let insights = consensus_result.result.as_ref().unwrap_or(&default_insights);
        
        // Execute with enhanced context
        let enhanced_context = format!(
            "{}\n\nConsensus Insights:\n{}",
            context.unwrap_or(""),
            insights
        );
        
        self.execute_directly(request, Some(&enhanced_context), callbacks).await
    }
    
    /// Execute with curator guidance
    async fn execute_with_guidance(
        &self,
        request: &str,
        context: Option<&str>,
        claude_plan: String,
        curator_guidance: String,
        callbacks: Arc<dyn StreamingCallbacks>,
    ) -> Result<ConsensusResult> {
        info!("📖 Executing with curator guidance...");
        
        let guided_context = format!(
            "{}\n\nOriginal Plan:\n{}\n\nCurator Guidance:\n{}",
            context.unwrap_or(""),
            claude_plan,
            curator_guidance
        );
        
        self.execute_directly(request, Some(&guided_context), callbacks).await
    }
    
    /// Store curator output as authoritative knowledge
    async fn store_authoritative_knowledge(
        &self,
        request: &str,
        claude_plan: &str,
        consensus_result: &ConsensusResult,
        execution_result: &ConsensusResult,
    ) -> Result<()> {
        info!("💾 Storing authoritative knowledge...");
        
        // Store in knowledge base if available
        if let Some(knowledge_base) = &self.knowledge_base {
            knowledge_base.store_curator_output(
                consensus_result.result.as_ref().unwrap_or(&String::new()),
                request,
                consensus_result.stages.clone(),
            ).await?;
        }
        
        // Store execution details in database
        let conn = self.database.get_connection()?;
        conn.execute(
            "INSERT INTO knowledge_conversations (
                id, query, claude_plan, consensus_evaluation, 
                curator_output, execution_result, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                Uuid::new_v4().to_string(),
                request,
                claude_plan,
                serde_json::to_string(&consensus_result)?,
                consensus_result.result.as_ref().unwrap_or(&String::new()),
                serde_json::to_string(&execution_result)?,
                chrono::Utc::now().to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }
}

/// Builder for ClaudeCodeExecutor
pub struct ClaudeCodeExecutorBuilder {
    profile: Option<ConsensusProfile>,
    database: Option<Arc<DatabaseManager>>,
    ai_helpers: Option<Arc<AIHelperEcosystem>>,
    knowledge_base: Option<Arc<ConsensusMemory>>,
    consensus_pipeline: Option<Arc<ConsensusPipeline>>,
    anthropic_key: Option<String>,
}

impl ClaudeCodeExecutorBuilder {
    pub fn new() -> Self {
        Self {
            profile: None,
            database: None,
            ai_helpers: None,
            knowledge_base: None,
            consensus_pipeline: None,
            anthropic_key: None,
        }
    }
    
    pub fn with_profile(mut self, profile: ConsensusProfile) -> Self {
        self.profile = Some(profile);
        self
    }
    
    pub fn with_database(mut self, database: Arc<DatabaseManager>) -> Self {
        self.database = Some(database);
        self
    }
    
    pub fn with_ai_helpers(mut self, helpers: Arc<AIHelperEcosystem>) -> Self {
        self.ai_helpers = Some(helpers);
        self
    }
    
    pub fn with_knowledge_base(mut self, kb: Arc<ConsensusMemory>) -> Self {
        self.knowledge_base = Some(kb);
        self
    }
    
    pub fn with_consensus_pipeline(mut self, pipeline: Arc<ConsensusPipeline>) -> Self {
        self.consensus_pipeline = Some(pipeline);
        self
    }
    
    pub fn with_anthropic_key(mut self, key: String) -> Self {
        self.anthropic_key = Some(key);
        self
    }
    
    pub fn build(self) -> Result<ClaudeCodeExecutor> {
        let mut executor = ClaudeCodeExecutor::new(
            self.profile.ok_or_else(|| anyhow::anyhow!("Profile required"))?,
            self.database.ok_or_else(|| anyhow::anyhow!("Database required"))?,
            self.ai_helpers.ok_or_else(|| anyhow::anyhow!("AI helpers required"))?,
            self.knowledge_base,
            self.anthropic_key,
        );
        
        if let Some(pipeline) = self.consensus_pipeline {
            executor = executor.with_consensus_pipeline(pipeline);
        }
        
        Ok(executor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consensus_invocation_criteria() {
        let executor = ClaudeCodeExecutor::new(
            ConsensusProfile::default(),
            Arc::new(DatabaseManager::new(None).unwrap()),
            Arc::new(AIHelperEcosystem::new()),
            Arc::new(ConsensusMemory::new(Arc::new(DatabaseManager::new(None).unwrap()))),
            None,
        );
        
        // High risk should always invoke consensus
        let criteria = ConsensusInvocationCriteria {
            architectural_change: false,
            high_risk_operation: true,
            confidence_level: 0.9,
            multiple_approaches: false,
            user_requested_analysis: false,
            complexity_score: 0.2,
        };
        assert!(executor.should_invoke_consensus(&criteria));
        
        // Low confidence should invoke consensus
        let criteria = ConsensusInvocationCriteria {
            architectural_change: false,
            high_risk_operation: false,
            confidence_level: 0.4,
            multiple_approaches: false,
            user_requested_analysis: false,
            complexity_score: 0.3,
        };
        assert!(executor.should_invoke_consensus(&criteria));
        
        // Simple operations should not invoke consensus
        let criteria = ConsensusInvocationCriteria {
            architectural_change: false,
            high_risk_operation: false,
            confidence_level: 0.9,
            multiple_approaches: false,
            user_requested_analysis: false,
            complexity_score: 0.2,
        };
        assert!(!executor.should_invoke_consensus(&criteria));
    }
    
    #[tokio::test]
    async fn test_execution_mode_switching() {
        let executor = ClaudeCodeExecutor::new(
            ConsensusProfile::default(),
            Arc::new(DatabaseManager::new(None).unwrap()),
            Arc::new(AIHelperEcosystem::new()),
            Arc::new(ConsensusMemory::new(Arc::new(DatabaseManager::new(None).unwrap()))),
            None,
        );
        
        // Default should be ConsensusAssisted
        assert_eq!(executor.get_mode().await, ClaudeExecutionMode::ConsensusAssisted);
        
        // Test mode switching
        executor.set_mode(ClaudeExecutionMode::Direct).await;
        assert_eq!(executor.get_mode().await, ClaudeExecutionMode::Direct);
        
        executor.set_mode(ClaudeExecutionMode::ConsensusRequired).await;
        assert_eq!(executor.get_mode().await, ClaudeExecutionMode::ConsensusRequired);
    }
}