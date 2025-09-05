//! Hook Registry - Central storage and management of hooks

use super::conditions::HookCondition;
use super::security::SecurityPolicy;
use super::{EventType, HookPriority};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

/// Unique identifier for a hook
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HookId(pub String);

impl HookId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_name(name: &str) -> Self {
        Self(format!(
            "{}-{}",
            name.to_lowercase().replace(' ', "-"),
            Uuid::new_v4().simple()
        ))
    }
}

impl fmt::Display for HookId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A registered hook with all its configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub id: HookId,
    pub name: String,
    pub description: Option<String>,
    pub events: Vec<EventType>,
    pub conditions: Vec<HookCondition>,
    pub actions: Vec<HookAction>,
    pub priority: HookPriority,
    pub enabled: bool,
    pub security: SecurityPolicy,
    pub metadata: HookMetadata,
}

/// Actions that a hook can perform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookAction {
    Command {
        command: String,
        args: Vec<String>,
        #[serde(default)]
        environment: HashMap<String, String>,
    },
    Script {
        language: String,
        content: String,
    },
    HttpRequest {
        url: String,
        method: String,
        #[serde(default)]
        headers: HashMap<String, String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,
    },
    Notification {
        channel: NotificationChannel,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        template: Option<String>,
    },
    ApprovalRequest {
        approvers: Vec<String>,
        message: String,
        timeout_minutes: u32,
    },
    ModifyContext {
        operation: ContextOperation,
        key: String,
        value: serde_json::Value,
    },
}

/// Notification channels for hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Slack,
    Webhook,
    Terminal,
}

/// Operations that can modify the execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextOperation {
    Set,
    Append,
    Remove,
    Merge,
}

/// Metadata about a hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMetadata {
    pub author: Option<String>,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: HashSet<String>,
    pub source: Option<String>,
}

impl Default for HookMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            author: None,
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
            tags: HashSet::new(),
            source: None,
        }
    }
}

/// Registry that stores and manages all hooks
pub struct HookRegistry {
    hooks: HashMap<HookId, Arc<Hook>>,
    event_index: HashMap<EventType, HashSet<HookId>>,
    tag_index: HashMap<String, HashSet<HookId>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            event_index: HashMap::new(),
            tag_index: HashMap::new(),
        }
    }

    /// Register a new hook
    pub fn register(&mut self, hook: Hook) -> Result<()> {
        if self.hooks.contains_key(&hook.id) {
            return Err(anyhow!("Hook with ID {} already exists", hook.id.0));
        }

        // Update event index
        for event in &hook.events {
            self.event_index
                .entry(event.clone())
                .or_insert_with(HashSet::new)
                .insert(hook.id.clone());
        }

        // Update tag index
        for tag in &hook.metadata.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(hook.id.clone());
        }

        let hook_id = hook.id.clone();
        self.hooks.insert(hook_id, Arc::new(hook));

        Ok(())
    }

    /// Unregister a hook
    pub fn unregister(&mut self, hook_id: &HookId) -> Result<()> {
        let hook = self
            .hooks
            .remove(hook_id)
            .ok_or_else(|| anyhow!("Hook with ID {} not found", hook_id.0))?;

        // Remove from event index
        for event in &hook.events {
            if let Some(hooks) = self.event_index.get_mut(event) {
                hooks.remove(hook_id);
                if hooks.is_empty() {
                    self.event_index.remove(event);
                }
            }
        }

        // Remove from tag index
        for tag in &hook.metadata.tags {
            if let Some(hooks) = self.tag_index.get_mut(tag) {
                hooks.remove(hook_id);
                if hooks.is_empty() {
                    self.tag_index.remove(tag);
                }
            }
        }

        Ok(())
    }

    /// Get a hook by ID
    pub fn get(&self, hook_id: &HookId) -> Option<Arc<Hook>> {
        self.hooks.get(hook_id).cloned()
    }

    /// Find hooks that listen to a specific event
    pub fn find_by_event(&self, event: &EventType) -> Vec<Arc<Hook>> {
        self.event_index
            .get(event)
            .map(|hook_ids| {
                hook_ids
                    .iter()
                    .filter_map(|id| self.hooks.get(id))
                    .filter(|hook| hook.enabled)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find hooks by tag
    pub fn find_by_tag(&self, tag: &str) -> Vec<Arc<Hook>> {
        self.tag_index
            .get(tag)
            .map(|hook_ids| {
                hook_ids
                    .iter()
                    .filter_map(|id| self.hooks.get(id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List all hooks
    pub fn list_all(&self) -> Vec<Hook> {
        self.hooks.values().map(|hook| (**hook).clone()).collect()
    }

    /// Enable or disable a hook
    pub fn set_enabled(&mut self, hook_id: &HookId, enabled: bool) -> Result<()> {
        let hook = self
            .hooks
            .get_mut(hook_id)
            .ok_or_else(|| anyhow!("Hook with ID {} not found", hook_id.0))?;

        // Clone the hook and update enabled status
        let mut updated_hook = (**hook).clone();
        updated_hook.enabled = enabled;
        updated_hook.metadata.updated_at = chrono::Utc::now();

        *hook = Arc::new(updated_hook);

        Ok(())
    }

    /// Clear all hooks
    pub fn clear_all(&mut self) -> Result<()> {
        self.hooks.clear();
        self.event_index.clear();
        self.tag_index.clear();
        Ok(())
    }

    /// Get hook statistics
    pub fn get_stats(&self) -> HookRegistryStats {
        HookRegistryStats {
            total_hooks: self.hooks.len(),
            enabled_hooks: self.hooks.values().filter(|h| h.enabled).count(),
            events_monitored: self.event_index.len(),
            unique_tags: self.tag_index.len(),
        }
    }
}

/// Statistics about the hook registry
#[derive(Debug, Serialize)]
pub struct HookRegistryStats {
    pub total_hooks: usize,
    pub enabled_hooks: usize,
    pub events_monitored: usize,
    pub unique_tags: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_registration() {
        let mut registry = HookRegistry::new();

        let hook = Hook {
            id: HookId::new(),
            name: "Test Hook".to_string(),
            description: Some("A test hook".to_string()),
            events: vec![EventType::BeforeCodeModification],
            conditions: vec![],
            actions: vec![],
            priority: HookPriority::Normal,
            enabled: true,
            security: SecurityPolicy::default(),
            metadata: HookMetadata::default(),
        };

        let hook_id = hook.id.clone();

        // Register hook
        assert!(registry.register(hook).is_ok());

        // Verify hook exists
        assert!(registry.get(&hook_id).is_some());

        // Verify event index
        let hooks = registry.find_by_event(&EventType::BeforeCodeModification);
        assert_eq!(hooks.len(), 1);

        // Unregister hook
        assert!(registry.unregister(&hook_id).is_ok());

        // Verify hook removed
        assert!(registry.get(&hook_id).is_none());
    }
}
