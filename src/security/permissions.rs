//! Enterprise Permission Management System
//! 
//! Provides fine-grained permission control including:
//! - Resource-based permissions
//! - Permission inheritance
//! - Permission templates
//! - Dynamic permission evaluation

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Permission manager for enterprise-grade access control
pub struct PermissionManager {
    permissions: Arc<RwLock<HashMap<String, ResourcePermission>>>,
    templates: Arc<RwLock<HashMap<String, PermissionTemplate>>>,
    inheritance_rules: Arc<RwLock<Vec<PermissionInheritance>>>,
}

/// Resource-specific permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePermission {
    pub id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub permission_name: String,
    pub scope: PermissionScope,
    pub conditions: Vec<PermissionCondition>,
    pub granted_to: PermissionSubject,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
    pub metadata: HashMap<String, String>,
}

/// Permission scope definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionScope {
    /// Permission applies to the specific resource only
    Resource,
    /// Permission applies to the resource and its children
    ResourceAndChildren,
    /// Permission applies to all resources of the same type
    ResourceType,
    /// Permission applies to all resources in the same namespace
    Namespace,
    /// Global permission across all resources
    Global,
}

/// Permission condition for dynamic evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCondition {
    pub condition_type: ConditionType,
    pub operator: ConditionOperator,
    pub value: String,
    pub context_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    TimeRange,
    IpAddress,
    UserAgent,
    ResourceState,
    CustomAttribute,
    ApiKey,
    SessionAge,
    RiskScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    In,
    NotIn,
    Matches, // Regex
}

/// Permission subject (who the permission is granted to)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionSubject {
    User(String),
    Role(String),
    Team(String),
    ApiKey(String),
    ServiceAccount(String),
}

/// Permission template for common patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub permissions: Vec<TemplatePermission>,
    pub variables: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePermission {
    pub permission_name: String,
    pub resource_type: String,
    pub resource_pattern: String, // Can contain variables like ${user_id}
    pub scope: PermissionScope,
    pub conditions: Vec<PermissionCondition>,
}

/// Permission inheritance rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionInheritance {
    pub id: String,
    pub parent_resource_type: String,
    pub child_resource_type: String,
    pub inherited_permissions: Vec<String>,
    pub inheritance_type: InheritanceType,
    pub conditions: Vec<PermissionCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InheritanceType {
    /// Child inherits all permissions from parent
    Full,
    /// Child inherits only specified permissions
    Selective,
    /// Child inherits permissions with reduced scope
    Reduced,
    /// Child inherits permissions with additional conditions
    Conditional,
}

/// Context for permission evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_time: DateTime<Utc>,
    pub resource_state: HashMap<String, String>,
    pub custom_attributes: HashMap<String, String>,
    pub risk_score: Option<u32>,
    pub api_key: Option<String>,
}

impl Default for PermissionContext {
    fn default() -> Self {
        Self {
            user_id: None,
            session_id: None,
            ip_address: None,
            user_agent: None,
            request_time: Utc::now(),
            resource_state: HashMap::new(),
            custom_attributes: HashMap::new(),
            risk_score: None,
            api_key: None,
        }
    }
}

/// Permission evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEvaluationResult {
    pub granted: bool,
    pub reason: String,
    pub matched_permissions: Vec<String>,
    pub failed_conditions: Vec<String>,
    pub evaluation_time_ms: u64,
    pub requires_approval: bool,
    pub approval_reason: Option<String>,
}

impl PermissionManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
            inheritance_rules: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Initialize with default templates and inheritance rules
    pub async fn initialize_defaults(&self) -> Result<()> {
        self.create_default_templates().await?;
        self.create_default_inheritance_rules().await?;
        Ok(())
    }

    async fn create_default_templates(&self) -> Result<()> {
        let templates = vec![
            PermissionTemplate {
                id: "admin_full_access".to_string(),
                name: "Administrator Full Access".to_string(),
                description: "Full administrative access to all resources".to_string(),
                category: "Administrative".to_string(),
                permissions: vec![
                    TemplatePermission {
                        permission_name: "admin".to_string(),
                        resource_type: "*".to_string(),
                        resource_pattern: "*".to_string(),
                        scope: PermissionScope::Global,
                        conditions: vec![],
                    }
                ],
                variables: HashMap::new(),
                created_at: Utc::now(),
                version: "1.0".to_string(),
            },
            PermissionTemplate {
                id: "developer_project_access".to_string(),
                name: "Developer Project Access".to_string(),
                description: "Standard developer access to assigned projects".to_string(),
                category: "Development".to_string(),
                permissions: vec![
                    TemplatePermission {
                        permission_name: "read".to_string(),
                        resource_type: "project".to_string(),
                        resource_pattern: "project:${project_id}".to_string(),
                        scope: PermissionScope::ResourceAndChildren,
                        conditions: vec![],
                    },
                    TemplatePermission {
                        permission_name: "write".to_string(),
                        resource_type: "file".to_string(),
                        resource_pattern: "project:${project_id}:file:*".to_string(),
                        scope: PermissionScope::Resource,
                        conditions: vec![
                            PermissionCondition {
                                condition_type: ConditionType::TimeRange,
                                operator: ConditionOperator::In,
                                value: "09:00-18:00".to_string(),
                                context_key: None,
                            }
                        ],
                    }
                ],
                variables: [("project_id".to_string(), "".to_string())].into_iter().collect(),
                created_at: Utc::now(),
                version: "1.0".to_string(),
            },
            PermissionTemplate {
                id: "readonly_access".to_string(),
                name: "Read-Only Access".to_string(),
                description: "Read-only access to specified resources".to_string(),
                category: "Viewer".to_string(),
                permissions: vec![
                    TemplatePermission {
                        permission_name: "read".to_string(),
                        resource_type: "${resource_type}".to_string(),
                        resource_pattern: "${resource_pattern}".to_string(),
                        scope: PermissionScope::Resource,
                        conditions: vec![],
                    }
                ],
                variables: [
                    ("resource_type".to_string(), "".to_string()),
                    ("resource_pattern".to_string(), "".to_string()),
                ].into_iter().collect(),
                created_at: Utc::now(),
                version: "1.0".to_string(),
            },
        ];

        let mut templates_map = self.templates.write().await;
        for template in templates {
            templates_map.insert(template.id.clone(), template);
        }

        Ok(())
    }

    async fn create_default_inheritance_rules(&self) -> Result<()> {
        let rules = vec![
            PermissionInheritance {
                id: "project_to_file".to_string(),
                parent_resource_type: "project".to_string(),
                child_resource_type: "file".to_string(),
                inherited_permissions: vec!["read".to_string(), "write".to_string()],
                inheritance_type: InheritanceType::Selective,
                conditions: vec![],
            },
            PermissionInheritance {
                id: "team_to_project".to_string(),
                parent_resource_type: "team".to_string(),
                child_resource_type: "project".to_string(),
                inherited_permissions: vec!["read".to_string()],
                inheritance_type: InheritanceType::Reduced,
                conditions: vec![],
            },
        ];

        let mut inheritance_rules = self.inheritance_rules.write().await;
        inheritance_rules.extend(rules);

        Ok(())
    }

    /// Grant permission to a subject
    pub async fn grant_permission(
        &self,
        permission_name: &str,
        resource_type: &str,
        resource_id: &str,
        subject: PermissionSubject,
        scope: PermissionScope,
        granted_by: &str,
        conditions: Vec<PermissionCondition>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<String> {
        let permission_id = uuid::Uuid::new_v4().to_string();
        
        let permission = ResourcePermission {
            id: permission_id.clone(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            permission_name: permission_name.to_string(),
            scope,
            conditions,
            granted_to: subject,
            granted_by: granted_by.to_string(),
            granted_at: Utc::now(),
            expires_at,
            active: true,
            metadata: HashMap::new(),
        };

        let mut permissions = self.permissions.write().await;
        permissions.insert(permission_id.clone(), permission);

        Ok(permission_id)
    }

    /// Revoke permission
    pub async fn revoke_permission(&self, permission_id: &str) -> Result<()> {
        let mut permissions = self.permissions.write().await;
        if let Some(permission) = permissions.get_mut(permission_id) {
            permission.active = false;
            Ok(())
        } else {
            Err(anyhow!("Permission not found: {}", permission_id))
        }
    }

    /// Check if a subject has permission for a resource
    pub async fn check_permission(
        &self,
        subject: &PermissionSubject,
        permission_name: &str,
        resource_type: &str,
        resource_id: &str,
        context: &PermissionContext,
    ) -> Result<PermissionEvaluationResult> {
        let start_time = std::time::Instant::now();
        
        let permissions = self.permissions.read().await;
        let mut matched_permissions = Vec::new();
        let mut failed_conditions = Vec::new();
        let mut requires_approval = false;
        let mut approval_reason = None;

        // Find matching permissions
        for permission in permissions.values() {
            if !permission.active {
                continue;
            }

            // Check if permission has expired
            if let Some(expires_at) = permission.expires_at {
                if expires_at < Utc::now() {
                    continue;
                }
            }

            // Check if subject matches
            if !self.subject_matches(&permission.granted_to, subject) {
                continue;
            }

            // Check if permission name matches
            if permission.permission_name != permission_name && permission.permission_name != "*" {
                continue;
            }

            // Check if resource matches based on scope
            if !self.resource_matches(permission, resource_type, resource_id)? {
                continue;
            }

            // Evaluate conditions
            let (conditions_met, failed) = self.evaluate_conditions(&permission.conditions, context).await?;
            if !conditions_met {
                failed_conditions.extend(failed);
                continue;
            }

            matched_permissions.push(permission.id.clone());

            // Check if this permission requires approval
            if self.requires_approval(permission, context) {
                requires_approval = true;
                approval_reason = Some(format!("High-risk operation requiring approval: {}", permission.permission_name));
            }
        }

        // Check inherited permissions
        let inherited = self.check_inherited_permissions(
            subject, permission_name, resource_type, resource_id, context
        ).await?;

        matched_permissions.extend(inherited.matched_permissions);
        failed_conditions.extend(inherited.failed_conditions);

        let granted = !matched_permissions.is_empty();
        let reason = if granted {
            format!("Permission granted via {} direct permission(s)", matched_permissions.len())
        } else if !failed_conditions.is_empty() {
            format!("Permission denied: conditions failed - {}", failed_conditions.join(", "))
        } else {
            "Permission denied: no matching permissions found".to_string()
        };

        let evaluation_time = start_time.elapsed().as_millis() as u64;

        Ok(PermissionEvaluationResult {
            granted,
            reason,
            matched_permissions,
            failed_conditions,
            evaluation_time_ms: evaluation_time,
            requires_approval,
            approval_reason,
        })
    }

    fn subject_matches(&self, granted_to: &PermissionSubject, subject: &PermissionSubject) -> bool {
        match (granted_to, subject) {
            (PermissionSubject::User(granted_user), PermissionSubject::User(check_user)) => {
                granted_user == check_user
            },
            (PermissionSubject::Role(granted_role), PermissionSubject::Role(check_role)) => {
                granted_role == check_role
            },
            (PermissionSubject::Team(granted_team), PermissionSubject::Team(check_team)) => {
                granted_team == check_team
            },
            (PermissionSubject::ApiKey(granted_key), PermissionSubject::ApiKey(check_key)) => {
                granted_key == check_key
            },
            (PermissionSubject::ServiceAccount(granted_sa), PermissionSubject::ServiceAccount(check_sa)) => {
                granted_sa == check_sa
            },
            _ => false,
        }
    }

    fn resource_matches(&self, permission: &ResourcePermission, resource_type: &str, resource_id: &str) -> Result<bool> {
        // Check resource type
        if permission.resource_type != "*" && permission.resource_type != resource_type {
            return Ok(false);
        }

        // Check resource ID based on scope
        match permission.scope {
            PermissionScope::Global => Ok(true),
            PermissionScope::ResourceType => Ok(permission.resource_type == resource_type),
            PermissionScope::Resource => Ok(
                permission.resource_type == resource_type && 
                (permission.resource_id == "*" || permission.resource_id == resource_id)
            ),
            PermissionScope::ResourceAndChildren => {
                if permission.resource_type != resource_type {
                    return Ok(false);
                }
                
                // Check if current resource is the target or a child
                Ok(permission.resource_id == "*" || 
                   permission.resource_id == resource_id ||
                   resource_id.starts_with(&format!("{}:", permission.resource_id)))
            },
            PermissionScope::Namespace => {
                // Extract namespace from resource ID (e.g., "namespace:resource")
                let namespace = resource_id.split(':').next().unwrap_or("");
                let permission_namespace = permission.resource_id.split(':').next().unwrap_or("");
                Ok(namespace == permission_namespace)
            },
        }
    }

    async fn evaluate_conditions(&self, conditions: &[PermissionCondition], context: &PermissionContext) -> Result<(bool, Vec<String>)> {
        let mut failed_conditions = Vec::new();

        for condition in conditions {
            if !self.evaluate_single_condition(condition, context)? {
                failed_conditions.push(format!("{:?} {} {}", 
                    condition.condition_type, 
                    condition.operator.to_string(), 
                    condition.value
                ));
            }
        }

        Ok((failed_conditions.is_empty(), failed_conditions))
    }

    fn evaluate_single_condition(&self, condition: &PermissionCondition, context: &PermissionContext) -> Result<bool> {
        let context_value = match condition.condition_type {
            ConditionType::TimeRange => {
                let current_time = context.request_time.format("%H:%M").to_string();
                current_time
            },
            ConditionType::IpAddress => {
                context.ip_address.clone().unwrap_or_default()
            },
            ConditionType::UserAgent => {
                context.user_agent.clone().unwrap_or_default()
            },
            ConditionType::ResourceState => {
                if let Some(key) = &condition.context_key {
                    context.resource_state.get(key).cloned().unwrap_or_default()
                } else {
                    String::new()
                }
            },
            ConditionType::CustomAttribute => {
                if let Some(key) = &condition.context_key {
                    context.custom_attributes.get(key).cloned().unwrap_or_default()
                } else {
                    String::new()
                }
            },
            ConditionType::ApiKey => {
                context.api_key.clone().unwrap_or_default()
            },
            ConditionType::SessionAge => {
                // Calculate session age in minutes (simplified)
                "30".to_string() // Placeholder
            },
            ConditionType::RiskScore => {
                context.risk_score.map(|s| s.to_string()).unwrap_or_default()
            },
        };

        self.evaluate_operator(&condition.operator, &context_value, &condition.value)
    }

    fn evaluate_operator(&self, operator: &ConditionOperator, context_value: &str, condition_value: &str) -> Result<bool> {
        match operator {
            ConditionOperator::Equals => Ok(context_value == condition_value),
            ConditionOperator::NotEquals => Ok(context_value != condition_value),
            ConditionOperator::Contains => Ok(context_value.contains(condition_value)),
            ConditionOperator::NotContains => Ok(!context_value.contains(condition_value)),
            ConditionOperator::GreaterThan => {
                let ctx_val: f64 = context_value.parse().unwrap_or(0.0);
                let cond_val: f64 = condition_value.parse().unwrap_or(0.0);
                Ok(ctx_val > cond_val)
            },
            ConditionOperator::LessThan => {
                let ctx_val: f64 = context_value.parse().unwrap_or(0.0);
                let cond_val: f64 = condition_value.parse().unwrap_or(0.0);
                Ok(ctx_val < cond_val)
            },
            ConditionOperator::GreaterOrEqual => {
                let ctx_val: f64 = context_value.parse().unwrap_or(0.0);
                let cond_val: f64 = condition_value.parse().unwrap_or(0.0);
                Ok(ctx_val >= cond_val)
            },
            ConditionOperator::LessOrEqual => {
                let ctx_val: f64 = context_value.parse().unwrap_or(0.0);
                let cond_val: f64 = condition_value.parse().unwrap_or(0.0);
                Ok(ctx_val <= cond_val)
            },
            ConditionOperator::In => {
                let values: Vec<&str> = condition_value.split(',').collect();
                Ok(values.contains(&context_value))
            },
            ConditionOperator::NotIn => {
                let values: Vec<&str> = condition_value.split(',').collect();
                Ok(!values.contains(&context_value))
            },
            ConditionOperator::Matches => {
                // Regex matching
                if let Ok(regex) = regex::Regex::new(condition_value) {
                    Ok(regex.is_match(context_value))
                } else {
                    Ok(false)
                }
            },
        }
    }

    async fn check_inherited_permissions(
        &self,
        _subject: &PermissionSubject,
        _permission_name: &str,
        _resource_type: &str,
        _resource_id: &str,
        _context: &PermissionContext,
    ) -> Result<PermissionEvaluationResult> {
        // Simplified - in a real implementation, this would check inheritance rules
        Ok(PermissionEvaluationResult {
            granted: false,
            reason: "No inherited permissions".to_string(),
            matched_permissions: vec![],
            failed_conditions: vec![],
            evaluation_time_ms: 0,
            requires_approval: false,
            approval_reason: None,
        })
    }

    fn requires_approval(&self, permission: &ResourcePermission, context: &PermissionContext) -> bool {
        // Check if this is a high-risk operation
        let high_risk_permissions = ["delete", "admin", "grant", "revoke"];
        if high_risk_permissions.contains(&permission.permission_name.as_str()) {
            return true;
        }

        // Check risk score
        if let Some(risk_score) = context.risk_score {
            if risk_score > 70 {
                return true;
            }
        }

        // Check for sensitive resources
        if permission.resource_type == "financial" || 
           permission.resource_type == "sensitive" ||
           permission.resource_id.contains("prod") {
            return true;
        }

        false
    }

    /// Apply permission template
    pub async fn apply_template(
        &self,
        template_id: &str,
        subject: PermissionSubject,
        variables: HashMap<String, String>,
        granted_by: &str,
    ) -> Result<Vec<String>> {
        let templates = self.templates.read().await;
        let template = templates.get(template_id)
            .ok_or_else(|| anyhow!("Template not found: {}", template_id))?;

        let mut permission_ids = Vec::new();

        for template_perm in &template.permissions {
            // Substitute variables in resource pattern
            let mut resource_pattern = template_perm.resource_pattern.clone();
            for (var, value) in &variables {
                resource_pattern = resource_pattern.replace(&format!("${{{}}}", var), value);
            }

            // Extract resource type and ID from pattern
            let parts: Vec<&str> = resource_pattern.split(':').collect();
            let resource_type = template_perm.resource_type.replace("${resource_type}", 
                variables.get("resource_type").unwrap_or(&template_perm.resource_type));
            let resource_id = if parts.len() > 1 { parts[1..].join(":") } else { "*".to_string() };

            let permission_id = self.grant_permission(
                &template_perm.permission_name,
                &resource_type,
                &resource_id,
                subject.clone(),
                template_perm.scope.clone(),
                granted_by,
                template_perm.conditions.clone(),
                None,
            ).await?;

            permission_ids.push(permission_id);
        }

        Ok(permission_ids)
    }

    /// List permissions for a subject
    pub async fn list_permissions(&self, subject: &PermissionSubject) -> Result<Vec<ResourcePermission>> {
        let permissions = self.permissions.read().await;
        Ok(permissions.values()
            .filter(|p| p.active && self.subject_matches(&p.granted_to, subject))
            .cloned()
            .collect())
    }

    /// Get permission statistics
    pub async fn get_statistics(&self) -> Result<PermissionStatistics> {
        let permissions = self.permissions.read().await;
        let templates = self.templates.read().await;
        let inheritance_rules = self.inheritance_rules.read().await;

        let now = Utc::now();
        let expired_permissions = permissions.values()
            .filter(|p| p.expires_at.map_or(false, |exp| exp < now))
            .count();

        Ok(PermissionStatistics {
            total_permissions: permissions.len(),
            active_permissions: permissions.values().filter(|p| p.active).count(),
            expired_permissions,
            total_templates: templates.len(),
            total_inheritance_rules: inheritance_rules.len(),
        })
    }
}

impl ConditionOperator {
    fn to_string(&self) -> &str {
        match self {
            ConditionOperator::Equals => "equals",
            ConditionOperator::NotEquals => "not_equals",
            ConditionOperator::Contains => "contains",
            ConditionOperator::NotContains => "not_contains",
            ConditionOperator::GreaterThan => "greater_than",
            ConditionOperator::LessThan => "less_than",
            ConditionOperator::GreaterOrEqual => "greater_or_equal",
            ConditionOperator::LessOrEqual => "less_or_equal",
            ConditionOperator::In => "in",
            ConditionOperator::NotIn => "not_in",
            ConditionOperator::Matches => "matches",
        }
    }
}

/// Permission management statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionStatistics {
    pub total_permissions: usize,
    pub active_permissions: usize,
    pub expired_permissions: usize,
    pub total_templates: usize,
    pub total_inheritance_rules: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_grant_and_check() {
        let manager = PermissionManager::new().await.unwrap();
        
        let subject = PermissionSubject::User("test_user".to_string());
        let permission_id = manager.grant_permission(
            "read",
            "file",
            "test_file.txt",
            subject.clone(),
            PermissionScope::Resource,
            "admin",
            vec![],
            None,
        ).await.unwrap();

        let context = PermissionContext::default();
        let result = manager.check_permission(
            &subject,
            "read",
            "file",
            "test_file.txt",
            &context,
        ).await.unwrap();

        assert!(result.granted);
        assert_eq!(result.matched_permissions.len(), 1);
        assert_eq!(result.matched_permissions[0], permission_id);
    }

    #[tokio::test]
    async fn test_permission_with_conditions() {
        let manager = PermissionManager::new().await.unwrap();
        
        let conditions = vec![
            PermissionCondition {
                condition_type: ConditionType::RiskScore,
                operator: ConditionOperator::LessThan,
                value: "50".to_string(),
                context_key: None,
            }
        ];

        let subject = PermissionSubject::User("test_user".to_string());
        manager.grant_permission(
            "write",
            "file",
            "sensitive_file.txt",
            subject.clone(),
            PermissionScope::Resource,
            "admin",
            conditions,
            None,
        ).await.unwrap();

        // Test with low risk score (should pass)
        let mut context = PermissionContext::default();
        context.risk_score = Some(30);
        
        let result = manager.check_permission(
            &subject,
            "write",
            "file",
            "sensitive_file.txt",
            &context,
        ).await.unwrap();

        assert!(result.granted);

        // Test with high risk score (should fail)
        context.risk_score = Some(80);
        
        let result = manager.check_permission(
            &subject,
            "write",
            "file",
            "sensitive_file.txt",
            &context,
        ).await.unwrap();

        assert!(!result.granted);
        assert!(!result.failed_conditions.is_empty());
    }

    #[tokio::test]
    async fn test_permission_template() {
        let manager = PermissionManager::new().await.unwrap();
        manager.initialize_defaults().await.unwrap();

        let subject = PermissionSubject::User("developer".to_string());
        let variables = [("project_id".to_string(), "proj123".to_string())].into_iter().collect();

        let permission_ids = manager.apply_template(
            "developer_project_access",
            subject.clone(),
            variables,
            "admin",
        ).await.unwrap();

        assert_eq!(permission_ids.len(), 2); // Should create 2 permissions

        // Check that permissions were created correctly
        let permissions = manager.list_permissions(&subject).await.unwrap();
        assert_eq!(permissions.len(), 2);
    }

    #[tokio::test]
    async fn test_permission_scopes() {
        let manager = PermissionManager::new().await.unwrap();
        
        let subject = PermissionSubject::User("test_user".to_string());
        
        // Grant permission with ResourceAndChildren scope
        manager.grant_permission(
            "read",
            "project",
            "project1",
            subject.clone(),
            PermissionScope::ResourceAndChildren,
            "admin",
            vec![],
            None,
        ).await.unwrap();

        let context = PermissionContext::default();

        // Should have access to the project itself
        let result = manager.check_permission(
            &subject,
            "read",
            "project",
            "project1",
            &context,
        ).await.unwrap();
        assert!(result.granted);

        // Should have access to child resources
        let result = manager.check_permission(
            &subject,
            "read",
            "project",
            "project1:file1",
            &context,
        ).await.unwrap();
        assert!(result.granted);

        // Should NOT have access to different project
        let result = manager.check_permission(
            &subject,
            "read",
            "project",
            "project2",
            &context,
        ).await.unwrap();
        assert!(!result.granted);
    }
}