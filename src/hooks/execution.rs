//! Hook Execution Engine - Secure execution of hook actions

use std::sync::Arc;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::{Hook, registry::HookAction, HookId};
use super::security::HookSecurityValidator;
use super::audit::HookAuditLogger;
use super::approval::ApprovalWorkflow;
use super::conditions::ConditionEvaluator;
use super::events::HookEvent;

/// Context for hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub hook_id: HookId,
    pub event: HookEvent,
    pub variables: HashMap<String, Value>,
    pub execution_id: String,
    pub dry_run: bool,
}

impl ExecutionContext {
    pub fn from_event(event: &HookEvent) -> Result<Self> {
        let mut variables = HashMap::new();
        
        // Copy event context into variables
        for (key, value) in &event.context {
            variables.insert(key.clone(), value.clone());
        }
        
        // Add standard variables
        variables.insert("event_type".to_string(), serde_json::to_value(&event.event_type)?);
        variables.insert("timestamp".to_string(), serde_json::to_value(&event.timestamp)?);
        
        Ok(Self {
            hook_id: HookId::new(), // Will be set by executor
            event: event.clone(),
            variables,
            execution_id: uuid::Uuid::new_v4().to_string(),
            dry_run: false,
        })
    }
    
    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }
    
    pub fn set_variable(&mut self, key: String, value: Value) {
        self.variables.insert(key, value);
    }
}

/// Result of hook execution
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub hook_id: HookId,
    pub execution_id: String,
    pub success: bool,
    pub actions_executed: Vec<ActionResult>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Result of a single action execution
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_type: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Executes hooks in a secure environment
pub struct HookExecutor {
    security_validator: Arc<HookSecurityValidator>,
    audit_logger: Arc<HookAuditLogger>,
    approval_workflow: Arc<ApprovalWorkflow>,
    condition_evaluator: ConditionEvaluator,
}

impl HookExecutor {
    pub fn new(
        security_validator: Arc<HookSecurityValidator>,
        audit_logger: Arc<HookAuditLogger>,
        approval_workflow: Arc<ApprovalWorkflow>,
    ) -> Self {
        Self {
            security_validator,
            audit_logger,
            approval_workflow,
            condition_evaluator: ConditionEvaluator::new(),
        }
    }
    
    /// Execute a hook with the given context
    pub async fn execute_hook(&self, hook: &Hook, mut context: ExecutionContext) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        context.hook_id = hook.id.clone();
        
        // Log execution start
        self.audit_logger.log_execution_start(&hook.id, &context).await?;
        
        // Validate hook security
        self.security_validator.validate_hook(hook)?;
        
        // Check conditions
        if !self.condition_evaluator.evaluate(&hook.conditions, &context).await? {
            let result = ExecutionResult {
                hook_id: hook.id.clone(),
                execution_id: context.execution_id.clone(),
                success: true,
                actions_executed: vec![],
                duration_ms: start_time.elapsed().as_millis() as u64,
                error: Some("Conditions not met".to_string()),
            };
            
            self.audit_logger.log_execution_skipped(&hook.id, &context, "Conditions not met").await?;
            return Ok(result);
        }
        
        // Check if approval is needed
        if hook.security.require_approval {
            let approved = self.approval_workflow
                .request_approval(&hook.id, &context)
                .await?;
            
            if !approved {
                let result = ExecutionResult {
                    hook_id: hook.id.clone(),
                    execution_id: context.execution_id.clone(),
                    success: false,
                    actions_executed: vec![],
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error: Some("Approval denied".to_string()),
                };
                
                self.audit_logger.log_execution_denied(&hook.id, &context).await?;
                return Ok(result);
            }
        }
        
        // Execute actions
        let mut action_results = Vec::new();
        let mut overall_success = true;
        
        for action in &hook.actions {
            let action_start = std::time::Instant::now();
            
            let result = match self.execute_action(action, &mut context).await {
                Ok(output) => ActionResult {
                    action_type: self.get_action_type(action),
                    success: true,
                    output,
                    error: None,
                    duration_ms: action_start.elapsed().as_millis() as u64,
                },
                Err(e) => {
                    overall_success = false;
                    ActionResult {
                        action_type: self.get_action_type(action),
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        duration_ms: action_start.elapsed().as_millis() as u64,
                    }
                }
            };
            
            action_results.push(result);
            
            // Stop on first error if configured
            if !overall_success && hook.security.stop_on_error {
                break;
            }
        }
        
        let execution_result = ExecutionResult {
            hook_id: hook.id.clone(),
            execution_id: context.execution_id.clone(),
            success: overall_success,
            actions_executed: action_results,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error: if overall_success { None } else { Some("One or more actions failed".to_string()) },
        };
        
        // Log execution complete
        self.audit_logger.log_execution_complete(&hook.id, &execution_result).await?;
        
        Ok(execution_result)
    }
    
    /// Execute a single action
    async fn execute_action(&self, action: &HookAction, context: &mut ExecutionContext) -> Result<Option<String>> {
        if context.dry_run {
            return Ok(Some("Dry run - action not executed".to_string()));
        }
        
        match action {
            HookAction::Command { command, args, environment } => {
                self.execute_command(command, args, environment, context).await
            }
            HookAction::Script { language, content } => {
                self.execute_script(language, content, context).await
            }
            HookAction::HttpRequest { url, method, headers, body } => {
                self.execute_http_request(url, method, headers, body.as_deref(), context).await
            }
            HookAction::Notification { channel, message, template } => {
                self.execute_notification(channel, message, template.as_deref(), context).await
            }
            HookAction::ApprovalRequest { approvers, message, timeout_minutes } => {
                self.execute_approval_request(approvers, message, *timeout_minutes, context).await
            }
            HookAction::ModifyContext { operation, key, value } => {
                self.execute_context_modification(operation, key, value, context).await
            }
        }
    }
    
    /// Execute a command action
    async fn execute_command(
        &self,
        command: &str,
        args: &[String],
        environment: &HashMap<String, String>,
        context: &ExecutionContext,
    ) -> Result<Option<String>> {
        // Validate command is allowed
        self.security_validator.validate_command(command)?;
        
        // Expand variables in command and args
        let expanded_command = self.expand_variables(command, &context.variables)?;
        let expanded_args: Vec<String> = args.iter()
            .map(|arg| self.expand_variables(arg, &context.variables))
            .collect::<Result<Vec<_>>>()?;
        
        // Build command
        let mut cmd = Command::new(&expanded_command);
        cmd.args(&expanded_args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // Set environment variables
        for (key, value) in environment {
            let expanded_value = self.expand_variables(value, &context.variables)?;
            cmd.env(key, expanded_value);
        }
        
        // Execute with timeout
        let output = timeout(Duration::from_secs(300), cmd.output()).await
            .map_err(|_| anyhow!("Command timed out after 5 minutes"))?
            .map_err(|e| anyhow!("Failed to execute command: {}", e))?;
        
        if output.status.success() {
            Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()))
        } else {
            Err(anyhow!(
                "Command failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
    
    /// Execute a script action
    async fn execute_script(
        &self,
        language: &str,
        content: &str,
        context: &ExecutionContext,
    ) -> Result<Option<String>> {
        // Validate script language is allowed
        self.security_validator.validate_script_language(language)?;
        
        // Expand variables in script content
        let expanded_content = self.expand_variables(content, &context.variables)?;
        
        // Write script to temporary file
        let temp_dir = tempfile::tempdir()?;
        let script_extension = match language {
            "bash" | "sh" => "sh",
            "python" => "py",
            "javascript" | "js" => "js",
            "ruby" => "rb",
            _ => return Err(anyhow!("Unsupported script language: {}", language)),
        };
        
        let script_path = temp_dir.path().join(format!("hook_script.{}", script_extension));
        tokio::fs::write(&script_path, expanded_content).await?;
        
        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&script_path).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&script_path, perms).await?;
        }
        
        // Execute script
        let interpreter = match language {
            "bash" => "bash",
            "sh" => "sh",
            "python" => "python3",
            "javascript" | "js" => "node",
            "ruby" => "ruby",
            _ => language,
        };
        
        let output = timeout(
            Duration::from_secs(300),
            Command::new(interpreter)
                .arg(&script_path)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        ).await
            .map_err(|_| anyhow!("Script timed out after 5 minutes"))?
            .map_err(|e| anyhow!("Failed to execute script: {}", e))?;
        
        if output.status.success() {
            Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()))
        } else {
            Err(anyhow!(
                "Script failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
    
    /// Execute an HTTP request action
    async fn execute_http_request(
        &self,
        url: &str,
        method: &str,
        headers: &HashMap<String, String>,
        body: Option<&str>,
        context: &ExecutionContext,
    ) -> Result<Option<String>> {
        // Validate URL is allowed
        self.security_validator.validate_url(url)?;
        
        // Expand variables
        let expanded_url = self.expand_variables(url, &context.variables)?;
        
        // Build request
        let client = reqwest::Client::new();
        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(&expanded_url),
            "POST" => client.post(&expanded_url),
            "PUT" => client.put(&expanded_url),
            "DELETE" => client.delete(&expanded_url),
            "PATCH" => client.patch(&expanded_url),
            _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
        };
        
        // Add headers
        for (key, value) in headers {
            let expanded_value = self.expand_variables(value, &context.variables)?;
            request = request.header(key, expanded_value);
        }
        
        // Add body if present
        if let Some(body_content) = body {
            let expanded_body = self.expand_variables(body_content, &context.variables)?;
            request = request.body(expanded_body);
        }
        
        // Execute request with timeout
        let response = timeout(Duration::from_secs(30), request.send()).await
            .map_err(|_| anyhow!("HTTP request timed out after 30 seconds"))?
            .map_err(|e| anyhow!("Failed to execute HTTP request: {}", e))?;
        
        if response.status().is_success() {
            let body = response.text().await?;
            Ok(Some(body))
        } else {
            Err(anyhow!("HTTP request failed with status: {}", response.status()))
        }
    }
    
    /// Execute a notification action
    async fn execute_notification(
        &self,
        channel: &super::registry::NotificationChannel,
        message: &str,
        template: Option<&str>,
        context: &ExecutionContext,
    ) -> Result<Option<String>> {
        let expanded_message = self.expand_variables(message, &context.variables)?;
        
        // In a real implementation, this would send to actual notification channels
        // For now, we'll just log it
        tracing::info!("Hook notification [{}]: {}", 
            serde_json::to_string(channel)?, 
            expanded_message
        );
        
        Ok(Some(format!("Notification sent via {:?}", channel)))
    }
    
    /// Execute an approval request action
    async fn execute_approval_request(
        &self,
        approvers: &[String],
        message: &str,
        timeout_minutes: u32,
        context: &ExecutionContext,
    ) -> Result<Option<String>> {
        let expanded_message = self.expand_variables(message, &context.variables)?;
        
        let approval_request = super::approval::ApprovalRequest {
            id: uuid::Uuid::new_v4().to_string(),
            hook_id: context.hook_id.clone(),
            execution_id: context.execution_id.clone(),
            approvers: approvers.to_vec(),
            message: expanded_message,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(timeout_minutes as i64),
            status: super::approval::ApprovalStatus::Pending,
        };
        
        let approved = self.approval_workflow.request_approval_with_details(approval_request).await?;
        
        Ok(Some(format!("Approval request: {}", if approved { "Approved" } else { "Denied" })))
    }
    
    /// Execute a context modification action
    async fn execute_context_modification(
        &self,
        operation: &super::registry::ContextOperation,
        key: &str,
        value: &Value,
        context: &mut ExecutionContext,
    ) -> Result<Option<String>> {
        match operation {
            super::registry::ContextOperation::Set => {
                context.set_variable(key.to_string(), value.clone());
            }
            super::registry::ContextOperation::Append => {
                if let Some(existing) = context.get_variable(key) {
                    if let (Some(arr), Some(new_arr)) = (existing.as_array(), value.as_array()) {
                        let mut combined = arr.clone();
                        combined.extend(new_arr.clone());
                        context.set_variable(key.to_string(), Value::Array(combined));
                    } else {
                        return Err(anyhow!("Cannot append non-array values"));
                    }
                } else {
                    context.set_variable(key.to_string(), value.clone());
                }
            }
            super::registry::ContextOperation::Remove => {
                context.variables.remove(key);
            }
            super::registry::ContextOperation::Merge => {
                if let Some(existing) = context.get_variable(key) {
                    if let (Some(obj), Some(new_obj)) = (existing.as_object(), value.as_object()) {
                        let mut merged = obj.clone();
                        for (k, v) in new_obj {
                            merged.insert(k.clone(), v.clone());
                        }
                        context.set_variable(key.to_string(), Value::Object(merged));
                    } else {
                        return Err(anyhow!("Cannot merge non-object values"));
                    }
                } else {
                    context.set_variable(key.to_string(), value.clone());
                }
            }
        }
        
        Ok(Some(format!("Context modified: {} {:?}", key, operation)))
    }
    
    /// Get action type for logging
    fn get_action_type(&self, action: &HookAction) -> String {
        match action {
            HookAction::Command { .. } => "command".to_string(),
            HookAction::Script { .. } => "script".to_string(),
            HookAction::HttpRequest { .. } => "http_request".to_string(),
            HookAction::Notification { .. } => "notification".to_string(),
            HookAction::ApprovalRequest { .. } => "approval_request".to_string(),
            HookAction::ModifyContext { .. } => "modify_context".to_string(),
        }
    }
    
    /// Expand variables in a string
    fn expand_variables(&self, text: &str, variables: &HashMap<String, Value>) -> Result<String> {
        let mut result = text.to_string();
        
        // Simple variable expansion: ${var_name}
        for (key, value) in variables {
            let pattern = format!("${{{}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => serde_json::to_string(value)?,
            };
            result = result.replace(&pattern, &replacement);
        }
        
        Ok(result)
    }
}