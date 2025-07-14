//! Hook Configuration - Loading and validation of hook configurations

use super::security::{HookSecurityValidator, SecurityPolicy};
use super::{registry::HookMetadata, Hook, HookId, HookPriority};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

/// Hook configuration as loaded from JSON/YAML files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub events: Vec<String>,
    #[serde(default)]
    pub conditions: serde_json::Value,
    pub actions: Vec<serde_json::Value>,
    #[serde(default)]
    pub priority: HookPriority,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub security: SecurityPolicy,
    #[serde(default)]
    pub metadata: HookConfigMetadata,
}

fn default_enabled() -> bool {
    true
}

/// Metadata in hook configuration files
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookConfigMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Loads and validates hook configurations
pub struct HookLoader {
    security_validator: Arc<HookSecurityValidator>,
}

impl HookLoader {
    pub fn new(security_validator: Arc<HookSecurityValidator>) -> Self {
        Self { security_validator }
    }

    /// Load a hook from a configuration file
    pub async fn load_from_file(&self, path: PathBuf) -> Result<Hook> {
        // Read file contents
        let contents = fs::read_to_string(&path)
            .await
            .map_err(|e| anyhow!("Failed to read hook configuration: {}", e))?;

        // Parse based on file extension
        let config: HookConfig = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&contents)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&contents)?,
            _ => {
                return Err(anyhow!(
                    "Unsupported file format. Use .json, .yaml, or .yml"
                ))
            }
        };

        // Convert to Hook
        let hook = self.config_to_hook(config, Some(path))?;

        // Validate security
        self.security_validator.validate_hook(&hook)?;

        Ok(hook)
    }

    /// Load all hooks from a directory
    pub async fn load_from_directory(&self, dir: PathBuf) -> Result<Vec<Hook>> {
        let mut hooks = Vec::new();

        let mut entries = fs::read_dir(&dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip non-hook files
            if !path.is_file() {
                continue;
            }

            match path.extension().and_then(|s| s.to_str()) {
                Some("json") | Some("yaml") | Some("yml") => {
                    match self.load_from_file(path.clone()).await {
                        Ok(hook) => hooks.push(hook),
                        Err(e) => {
                            tracing::warn!("Failed to load hook from {}: {}", path.display(), e);
                        }
                    }
                }
                _ => continue,
            }
        }

        Ok(hooks)
    }

    /// Convert HookConfig to Hook
    fn config_to_hook(&self, config: HookConfig, source_path: Option<PathBuf>) -> Result<Hook> {
        // Parse events
        let events = config
            .events
            .into_iter()
            .map(|e| self.parse_event_type(&e))
            .collect::<Result<Vec<_>>>()?;

        // Parse conditions
        let conditions = if config.conditions.is_null() {
            Vec::new()
        } else if let Some(arr) = config.conditions.as_array() {
            arr.iter()
                .map(|v| serde_json::from_value(v.clone()))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![serde_json::from_value(config.conditions)?]
        };

        // Parse actions
        let actions = config
            .actions
            .into_iter()
            .map(|v| serde_json::from_value(v))
            .collect::<Result<Vec<_>, _>>()?;

        // Create metadata
        let metadata = HookMetadata {
            author: config.metadata.author,
            version: config.metadata.version,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: config.metadata.tags.into_iter().collect(),
            source: source_path.map(|p| p.to_string_lossy().to_string()),
        };

        Ok(Hook {
            id: HookId::from_name(&config.name),
            name: config.name,
            description: config.description,
            events,
            conditions,
            actions,
            priority: config.priority,
            enabled: config.enabled,
            security: config.security,
            metadata,
        })
    }

    /// Parse event type string to EventType enum
    fn parse_event_type(&self, event_str: &str) -> Result<super::EventType> {
        use super::EventType;

        let event = match event_str {
            "before_consensus" => EventType::BeforeConsensus,
            "after_consensus" => EventType::AfterConsensus,
            "before_generator_stage" => EventType::BeforeGeneratorStage,
            "after_generator_stage" => EventType::AfterGeneratorStage,
            "before_refiner_stage" => EventType::BeforeRefinerStage,
            "after_refiner_stage" => EventType::AfterRefinerStage,
            "before_validator_stage" => EventType::BeforeValidatorStage,
            "after_validator_stage" => EventType::AfterValidatorStage,
            "before_curator_stage" => EventType::BeforeCuratorStage,
            "after_curator_stage" => EventType::AfterCuratorStage,
            "consensus_error" => EventType::ConsensusError,
            "before_code_modification" => EventType::BeforeCodeModification,
            "after_code_modification" => EventType::AfterCodeModification,
            "before_file_write" => EventType::BeforeFileWrite,
            "after_file_write" => EventType::AfterFileWrite,
            "before_file_delete" => EventType::BeforeFileDelete,
            "after_file_delete" => EventType::AfterFileDelete,
            "before_analysis" => EventType::BeforeAnalysis,
            "after_analysis" => EventType::AfterAnalysis,
            "analysis_complete" => EventType::AnalysisComplete,
            "quality_gate_check" => EventType::QualityGateCheck,
            "cost_threshold_reached" => EventType::CostThresholdReached,
            "budget_exceeded" => EventType::BudgetExceeded,
            "cost_estimate_available" => EventType::CostEstimateAvailable,
            "before_indexing" => EventType::BeforeIndexing,
            "after_indexing" => EventType::AfterIndexing,
            "repository_changed" => EventType::RepositoryChanged,
            "dependency_changed" => EventType::DependencyChanged,
            "security_check_failed" => EventType::SecurityCheckFailed,
            "untrusted_path_access" => EventType::UntrustedPathAccess,
            "permission_denied" => EventType::PermissionDenied,
            custom if custom.starts_with("custom:") => {
                EventType::Custom(custom.strip_prefix("custom:").unwrap().to_string())
            }
            _ => return Err(anyhow!("Unknown event type: {}", event_str)),
        };

        Ok(event)
    }
}

/// Hook configuration examples
pub fn example_configs() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "auto-format.json",
            r#"{
  "name": "auto-format",
  "description": "Automatically format code before modifications",
  "events": ["before_code_modification"],
  "conditions": {
    "type": "file_pattern",
    "pattern": "*.rs"
  },
  "actions": [
    {
      "type": "command",
      "command": "rustfmt",
      "args": ["--edition", "2021", "${file_path}"]
    }
  ],
  "priority": "high",
  "security": {
    "require_approval": false,
    "allowed_commands": ["rustfmt"],
    "stop_on_error": true
  }
}"#,
        ),
        (
            "cost-control.json",
            r#"{
  "name": "cost-control",
  "description": "Require approval for expensive operations",
  "events": ["cost_threshold_reached"],
  "conditions": {
    "type": "context_variable",
    "key": "estimated_cost",
    "operator": "greater_than",
    "value": 0.10
  },
  "actions": [
    {
      "type": "approval_request",
      "approvers": ["finance-team", "project-lead"],
      "message": "Operation will cost $${estimated_cost}. Approval required.",
      "timeout_minutes": 30
    }
  ],
  "metadata": {
    "tags": ["cost", "approval", "finance"]
  }
}"#,
        ),
        (
            "quality-gate.json",
            r#"{
  "name": "quality-gate",
  "description": "Enforce code quality standards",
  "events": ["quality_gate_check"],
  "conditions": {
    "type": "and",
    "conditions": [
      {
        "type": "context_variable",
        "key": "complexity",
        "operator": "less_than",
        "value": 10
      },
      {
        "type": "context_variable",
        "key": "test_coverage",
        "operator": "greater_or_equal",
        "value": 80
      }
    ]
  },
  "actions": [
    {
      "type": "modify_context",
      "operation": "set",
      "key": "quality_passed",
      "value": true
    },
    {
      "type": "notification",
      "channel": "terminal",
      "message": "✅ Quality gate passed: complexity=${complexity}, coverage=${test_coverage}%"
    }
  ]
}"#,
        ),
        (
            "security-hook.json",
            r#"{
  "name": "security-scan",
  "description": "Run security checks on code changes",
  "events": ["before_code_modification"],
  "conditions": {
    "type": "file_pattern",
    "pattern": "*.rs"
  },
  "actions": [
    {
      "type": "script",
      "language": "bash",
      "content": "echo 'Running security scan'; exit 0"
    }
  ],
  "security": {
    "require_approval": true,
    "approval_message": "Security scan will analyze file",
    "allowed_commands": ["cargo", "npm", "pip"],
    "max_execution_time": 300
  }
}"#,
        ),
        (
            "dangerous-hook.json",
            r#"{
  "name": "dangerous-example",
  "description": "Example of a hook that should be rejected",
  "events": ["before_consensus"],
  "actions": [
    {
      "type": "command",
      "command": "rm",
      "args": ["-rf", "/"]
    }
  ],
  "security": {
    "require_approval": false
  }
}"#,
        ),
    ]
}

/// Generate example hook configuration files
pub async fn generate_examples(output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir).await?;

    for (filename, content) in example_configs() {
        let path = output_dir.join(filename);
        fs::write(path, content).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_json_config() {
        let config_json = r#"{
            "name": "test-hook",
            "events": ["before_consensus"],
            "actions": [
                {
                    "type": "command",
                    "command": "echo",
                    "args": ["Hello"]
                }
            ]
        }"#;

        let config: HookConfig = serde_json::from_str(config_json).unwrap();
        assert_eq!(config.name, "test-hook");
        assert_eq!(config.events.len(), 1);
        assert_eq!(config.actions.len(), 1);
    }
}
