//! Enterprise Role-Based Access Control (RBAC) Manager
//! 
//! Enhanced RBAC system with enterprise features including:
//! - Hierarchical roles
//! - Dynamic role assignment
//! - Security groups
//! - Access policies
//! - Role inheritance

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

use super::{PermissionManager, audit::EnterpriseAuditLogger};

/// Enterprise RBAC manager
pub struct EnterpriseRbacManager {
    users: Arc<RwLock<HashMap<String, EnterpriseUser>>>,
    roles: Arc<RwLock<HashMap<String, EnterpriseRole>>>,
    security_groups: Arc<RwLock<HashMap<String, SecurityGroup>>>,
    access_policies: Arc<RwLock<HashMap<String, AccessPolicy>>>,
    role_assignments: Arc<RwLock<HashMap<String, Vec<RoleAssignment>>>>,
    permission_manager: Arc<PermissionManager>,
    audit_logger: Arc<EnterpriseAuditLogger>,
}

/// Enhanced enterprise user with additional security features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub roles: Vec<String>,
    pub teams: Vec<String>,
    pub security_groups: Vec<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub password_hash: Option<String>,
    pub mfa_enabled: bool,
    pub account_locked: bool,
    pub lock_reason: Option<String>,
    pub last_activity: Option<DateTime<Utc>>,
    pub failed_login_attempts: u32,
    pub password_expires_at: Option<DateTime<Utc>>,
    pub must_change_password: bool,
    pub session_timeout: Option<u32>,
    pub ip_restrictions: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Enhanced enterprise role with hierarchical support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseRole {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: HashSet<String>,
    pub parent_roles: Vec<String>,
    pub child_roles: Vec<String>,
    pub auto_assign_conditions: Vec<AutoAssignCondition>,
    pub max_assignment_duration: Option<Duration>,
    pub requires_approval: bool,
    pub approval_roles: Vec<String>,
    pub risk_level: RiskLevel,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub active: bool,
    pub metadata: HashMap<String, String>,
}

/// Security group for grouping users with similar security requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub members: HashSet<String>,
    pub security_policies: Vec<String>,
    pub access_policies: Vec<String>,
    pub default_roles: Vec<String>,
    pub isolation_level: IsolationLevel,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Access policy for fine-grained control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub policy_type: PolicyType,
    pub conditions: Vec<PolicyCondition>,
    pub effect: PolicyEffect,
    pub resources: Vec<String>,
    pub actions: Vec<String>,
    pub principals: Vec<String>,
    pub time_restrictions: Option<TimeRestriction>,
    pub ip_restrictions: Vec<String>,
    pub priority: u32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// Role assignment with temporal and conditional aspects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignment {
    pub id: String,
    pub user_id: String,
    pub role_id: String,
    pub assigned_by: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Vec<AssignmentCondition>,
    pub approval_status: ApprovalStatus,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub active: bool,
    pub source: AssignmentSource,
}

/// Role inheritance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInheritance {
    pub parent_role: String,
    pub child_role: String,
    pub inheritance_type: InheritanceType,
    pub conditions: Vec<InheritanceCondition>,
    pub created_at: DateTime<Utc>,
}

// Enums and supporting types

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationLevel {
    None,
    Basic,
    Enhanced,
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyType {
    Allow,
    Deny,
    Audit,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    Allow,
    Deny,
    RequireApproval,
    RequireMfa,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentSource {
    Manual,
    Automatic,
    Inherited,
    Policy,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InheritanceType {
    Full,
    Partial,
    Conditional,
    Temporal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoAssignCondition {
    pub condition_type: String,
    pub operator: String,
    pub value: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    pub attribute: String,
    pub operator: String,
    pub value: String,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentCondition {
    pub condition_type: String,
    pub parameters: HashMap<String, String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceCondition {
    pub condition_type: String,
    pub value: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestriction {
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
    pub days_of_week: Vec<u8>, // 0-6, Monday=0
    pub timezone: String,
}

impl EnterpriseRbacManager {
    pub async fn new(
        permission_manager: Arc<PermissionManager>,
        audit_logger: Arc<EnterpriseAuditLogger>,
    ) -> Result<Self> {
        Ok(Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            security_groups: Arc::new(RwLock::new(HashMap::new())),
            access_policies: Arc::new(RwLock::new(HashMap::new())),
            role_assignments: Arc::new(RwLock::new(HashMap::new())),
            permission_manager,
            audit_logger,
        })
    }

    /// Initialize with default enterprise roles
    pub async fn initialize_default_roles(&self) -> Result<()> {
        let default_roles = vec![
            EnterpriseRole {
                id: "super_admin".to_string(),
                name: "Super Administrator".to_string(),
                description: "Full system access with all privileges".to_string(),
                permissions: ["*".to_string()].into_iter().collect(),
                parent_roles: vec![],
                child_roles: vec!["admin".to_string()],
                auto_assign_conditions: vec![],
                max_assignment_duration: None,
                requires_approval: true,
                approval_roles: vec!["board_member".to_string()],
                risk_level: RiskLevel::Critical,
                created_at: Utc::now(),
                created_by: "system".to_string(),
                active: true,
                metadata: HashMap::new(),
            },
            EnterpriseRole {
                id: "admin".to_string(),
                name: "Administrator".to_string(),
                description: "Administrative access to most system functions".to_string(),
                permissions: [
                    "user.create".to_string(),
                    "user.read".to_string(),
                    "user.update".to_string(),
                    "user.delete".to_string(),
                    "role.create".to_string(),
                    "role.read".to_string(),
                    "role.update".to_string(),
                    "system.configure".to_string(),
                ].into_iter().collect(),
                parent_roles: vec!["super_admin".to_string()],
                child_roles: vec!["manager".to_string()],
                auto_assign_conditions: vec![],
                max_assignment_duration: Some(Duration::days(365)),
                requires_approval: true,
                approval_roles: vec!["super_admin".to_string()],
                risk_level: RiskLevel::High,
                created_at: Utc::now(),
                created_by: "system".to_string(),
                active: true,
                metadata: HashMap::new(),
            },
            EnterpriseRole {
                id: "manager".to_string(),
                name: "Manager".to_string(),
                description: "Management access to team resources".to_string(),
                permissions: [
                    "team.read".to_string(),
                    "team.update".to_string(),
                    "project.create".to_string(),
                    "project.read".to_string(),
                    "project.update".to_string(),
                    "user.read".to_string(),
                ].into_iter().collect(),
                parent_roles: vec!["admin".to_string()],
                child_roles: vec!["developer".to_string(), "analyst".to_string()],
                auto_assign_conditions: vec![
                    AutoAssignCondition {
                        condition_type: "department".to_string(),
                        operator: "equals".to_string(),
                        value: "management".to_string(),
                        metadata: HashMap::new(),
                    }
                ],
                max_assignment_duration: Some(Duration::days(180)),
                requires_approval: false,
                approval_roles: vec![],
                risk_level: RiskLevel::Medium,
                created_at: Utc::now(),
                created_by: "system".to_string(),
                active: true,
                metadata: HashMap::new(),
            },
            EnterpriseRole {
                id: "developer".to_string(),
                name: "Developer".to_string(),
                description: "Development access to assigned projects".to_string(),
                permissions: [
                    "project.read".to_string(),
                    "code.read".to_string(),
                    "code.write".to_string(),
                    "build.create".to_string(),
                    "test.run".to_string(),
                ].into_iter().collect(),
                parent_roles: vec!["manager".to_string()],
                child_roles: vec![],
                auto_assign_conditions: vec![
                    AutoAssignCondition {
                        condition_type: "department".to_string(),
                        operator: "equals".to_string(),
                        value: "engineering".to_string(),
                        metadata: HashMap::new(),
                    }
                ],
                max_assignment_duration: Some(Duration::days(90)),
                requires_approval: false,
                approval_roles: vec![],
                risk_level: RiskLevel::Low,
                created_at: Utc::now(),
                created_by: "system".to_string(),
                active: true,
                metadata: HashMap::new(),
            },
            EnterpriseRole {
                id: "analyst".to_string(),
                name: "Business Analyst".to_string(),
                description: "Read-only access to business data and reports".to_string(),
                permissions: [
                    "report.read".to_string(),
                    "data.read".to_string(),
                    "analytics.view".to_string(),
                ].into_iter().collect(),
                parent_roles: vec!["manager".to_string()],
                child_roles: vec![],
                auto_assign_conditions: vec![],
                max_assignment_duration: Some(Duration::days(60)),
                requires_approval: false,
                approval_roles: vec![],
                risk_level: RiskLevel::Low,
                created_at: Utc::now(),
                created_by: "system".to_string(),
                active: true,
                metadata: HashMap::new(),
            },
            EnterpriseRole {
                id: "auditor".to_string(),
                name: "Security Auditor".to_string(),
                description: "Read-only access to security logs and audit trails".to_string(),
                permissions: [
                    "audit.read".to_string(),
                    "security.read".to_string(),
                    "compliance.read".to_string(),
                ].into_iter().collect(),
                parent_roles: vec![],
                child_roles: vec![],
                auto_assign_conditions: vec![],
                max_assignment_duration: Some(Duration::days(30)),
                requires_approval: true,
                approval_roles: vec!["admin".to_string()],
                risk_level: RiskLevel::Medium,
                created_at: Utc::now(),
                created_by: "system".to_string(),
                active: true,
                metadata: HashMap::new(),
            },
        ];

        let mut roles = self.roles.write().await;
        for role in default_roles {
            roles.insert(role.id.clone(), role);
        }

        // Create default security groups
        self.create_default_security_groups().await?;

        // Create default access policies
        self.create_default_access_policies().await?;

        Ok(())
    }

    async fn create_default_security_groups(&self) -> Result<()> {
        let default_groups = vec![
            SecurityGroup {
                id: "executives".to_string(),
                name: "Executive Team".to_string(),
                description: "C-level executives and senior leadership".to_string(),
                members: HashSet::new(),
                security_policies: vec!["high_security".to_string()],
                access_policies: vec!["executive_access".to_string()],
                default_roles: vec!["manager".to_string()],
                isolation_level: IsolationLevel::Enhanced,
                created_at: Utc::now(),
                metadata: HashMap::new(),
            },
            SecurityGroup {
                id: "developers".to_string(),
                name: "Development Team".to_string(),
                description: "Software developers and engineers".to_string(),
                members: HashSet::new(),
                security_policies: vec!["standard_security".to_string()],
                access_policies: vec!["developer_access".to_string()],
                default_roles: vec!["developer".to_string()],
                isolation_level: IsolationLevel::Basic,
                created_at: Utc::now(),
                metadata: HashMap::new(),
            },
            SecurityGroup {
                id: "contractors".to_string(),
                name: "External Contractors".to_string(),
                description: "External contractors and temporary staff".to_string(),
                members: HashSet::new(),
                security_policies: vec!["restricted_security".to_string()],
                access_policies: vec!["contractor_access".to_string()],
                default_roles: vec![],
                isolation_level: IsolationLevel::Maximum,
                created_at: Utc::now(),
                metadata: HashMap::new(),
            },
        ];

        let mut groups = self.security_groups.write().await;
        for group in default_groups {
            groups.insert(group.id.clone(), group);
        }

        Ok(())
    }

    async fn create_default_access_policies(&self) -> Result<()> {
        let default_policies = vec![
            AccessPolicy {
                id: "business_hours_only".to_string(),
                name: "Business Hours Access Only".to_string(),
                description: "Restrict access to business hours".to_string(),
                policy_type: PolicyType::Conditional,
                conditions: vec![
                    PolicyCondition {
                        attribute: "time".to_string(),
                        operator: "between".to_string(),
                        value: "09:00-18:00".to_string(),
                        case_sensitive: false,
                    }
                ],
                effect: PolicyEffect::Allow,
                resources: vec!["*".to_string()],
                actions: vec!["*".to_string()],
                principals: vec!["contractors".to_string()],
                time_restrictions: Some(TimeRestriction {
                    start_time: "09:00".to_string(),
                    end_time: "18:00".to_string(),
                    days_of_week: vec![0, 1, 2, 3, 4], // Monday-Friday
                    timezone: "UTC".to_string(),
                }),
                ip_restrictions: vec![],
                priority: 100,
                active: true,
                created_at: Utc::now(),
            },
            AccessPolicy {
                id: "mfa_required_admin".to_string(),
                name: "MFA Required for Admin Actions".to_string(),
                description: "Require MFA for all administrative actions".to_string(),
                policy_type: PolicyType::Conditional,
                conditions: vec![
                    PolicyCondition {
                        attribute: "action".to_string(),
                        operator: "starts_with".to_string(),
                        value: "admin".to_string(),
                        case_sensitive: false,
                    }
                ],
                effect: PolicyEffect::RequireMfa,
                resources: vec!["*".to_string()],
                actions: vec!["admin.*".to_string()],
                principals: vec!["*".to_string()],
                time_restrictions: None,
                ip_restrictions: vec![],
                priority: 200,
                active: true,
                created_at: Utc::now(),
            },
            AccessPolicy {
                id: "deny_sensitive_data_contractors".to_string(),
                name: "Deny Sensitive Data Access to Contractors".to_string(),
                description: "Block contractor access to sensitive data".to_string(),
                policy_type: PolicyType::Deny,
                conditions: vec![
                    PolicyCondition {
                        attribute: "resource_type".to_string(),
                        operator: "equals".to_string(),
                        value: "sensitive_data".to_string(),
                        case_sensitive: false,
                    }
                ],
                effect: PolicyEffect::Deny,
                resources: vec!["sensitive_data:*".to_string()],
                actions: vec!["*".to_string()],
                principals: vec!["contractors".to_string()],
                time_restrictions: None,
                ip_restrictions: vec![],
                priority: 300,
                active: true,
                created_at: Utc::now(),
            },
        ];

        let mut policies = self.access_policies.write().await;
        for policy in default_policies {
            policies.insert(policy.id.clone(), policy);
        }

        Ok(())
    }

    /// Create a new enterprise user
    pub async fn create_user(&self, user: EnterpriseUser) -> Result<()> {
        let mut users = self.users.write().await;
        
        if users.contains_key(&user.id) {
            return Err(anyhow!("User already exists: {}", user.id));
        }

        // Auto-assign roles based on conditions
        let auto_assigned_roles = self.evaluate_auto_assign_roles(&user).await?;
        let mut user_with_roles = user.clone();
        user_with_roles.roles.extend(auto_assigned_roles);

        users.insert(user.id.clone(), user_with_roles);

        // Log user creation
        self.audit_logger.log_user_event(
            super::audit::AuditEventType::UserCreated,
            &user.id,
            format!("Enterprise user created: {}", user.full_name),
        ).await?;

        Ok(())
    }

    async fn evaluate_auto_assign_roles(&self, user: &EnterpriseUser) -> Result<Vec<String>> {
        let roles = self.roles.read().await;
        let mut auto_assigned = Vec::new();

        for role in roles.values() {
            if role.active && !role.auto_assign_conditions.is_empty() {
                if self.evaluate_auto_assign_conditions(&role.auto_assign_conditions, user) {
                    auto_assigned.push(role.id.clone());
                }
            }
        }

        Ok(auto_assigned)
    }

    fn evaluate_auto_assign_conditions(&self, conditions: &[AutoAssignCondition], user: &EnterpriseUser) -> bool {
        for condition in conditions {
            let user_value = match condition.condition_type.as_str() {
                "department" => user.metadata.get("department").cloned().unwrap_or_default(),
                "email_domain" => user.email.split('@').nth(1).unwrap_or("").to_string(),
                "employee_type" => user.metadata.get("employee_type").cloned().unwrap_or_default(),
                _ => continue,
            };

            match condition.operator.as_str() {
                "equals" => {
                    if user_value != condition.value {
                        return false;
                    }
                },
                "contains" => {
                    if !user_value.contains(&condition.value) {
                        return false;
                    }
                },
                _ => return false,
            }
        }

        true
    }

    /// Assign role to user with approval workflow
    pub async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        // Check if role requires approval
        let role = {
            let roles = self.roles.read().await;
            roles.get(role_id).cloned().ok_or_else(|| anyhow!("Role not found: {}", role_id))?
        };

        let assignment_id = uuid::Uuid::new_v4().to_string();
        let assignment = RoleAssignment {
            id: assignment_id.clone(),
            user_id: user_id.to_string(),
            role_id: role_id.to_string(),
            assigned_by: "system".to_string(), // In real implementation, this would be the current user
            assigned_at: Utc::now(),
            expires_at: role.max_assignment_duration.map(|d| Utc::now() + d),
            conditions: vec![],
            approval_status: if role.requires_approval {
                ApprovalStatus::Pending
            } else {
                ApprovalStatus::Approved
            },
            approved_by: None,
            approved_at: None,
            active: !role.requires_approval,
            source: AssignmentSource::Manual,
        };

        // Store assignment
        let mut assignments = self.role_assignments.write().await;
        assignments.entry(user_id.to_string()).or_insert_with(Vec::new).push(assignment);

        // If no approval required, assign immediately
        if !role.requires_approval {
            let mut users = self.users.write().await;
            if let Some(user) = users.get_mut(user_id) {
                if !user.roles.contains(&role_id.to_string()) {
                    user.roles.push(role_id.to_string());
                }
            }

            self.audit_logger.log_user_event(
                super::audit::AuditEventType::RoleAssigned,
                user_id,
                format!("Role {} assigned", role_id),
            ).await?;
        } else {
            self.audit_logger.log_user_event(
                super::audit::AuditEventType::RoleAssigned,
                user_id,
                format!("Role {} assignment pending approval", role_id),
            ).await?;
        }

        Ok(())
    }

    /// Check permission for user with enterprise policies
    pub async fn check_permission(&self, user_id: &str, permission: &str, resource: Option<&str>) -> Result<bool> {
        let user = {
            let users = self.users.read().await;
            users.get(user_id).cloned().ok_or_else(|| anyhow!("User not found: {}", user_id))?
        };

        if !user.active || user.account_locked {
            return Ok(false);
        }

        // Check access policies first
        if !self.evaluate_access_policies(&user, permission, resource).await? {
            return Ok(false);
        }

        // Get effective permissions from roles (including inherited)
        let effective_permissions = self.get_effective_permissions(&user).await?;

        // Check if user has the required permission
        Ok(effective_permissions.contains(permission) || effective_permissions.contains("*"))
    }

    async fn evaluate_access_policies(&self, user: &EnterpriseUser, permission: &str, resource: Option<&str>) -> Result<bool> {
        let policies = self.access_policies.read().await;
        let mut applicable_policies: Vec<_> = policies.values()
            .filter(|p| p.active && self.policy_applies_to_user(p, user))
            .collect();

        // Sort by priority (higher priority first)
        applicable_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        for policy in applicable_policies {
            if self.evaluate_policy_conditions(&policy.conditions, user, permission, resource) {
                match policy.effect {
                    PolicyEffect::Deny => return Ok(false),
                    PolicyEffect::Allow => return Ok(true),
                    PolicyEffect::RequireApproval => {
                        // In a real implementation, this would trigger an approval workflow
                        return Ok(false);
                    },
                    PolicyEffect::RequireMfa => {
                        // Check if user has completed MFA
                        return Ok(user.mfa_enabled);
                    },
                }
            }
        }

        // Default allow if no policies match
        Ok(true)
    }

    fn policy_applies_to_user(&self, policy: &AccessPolicy, user: &EnterpriseUser) -> bool {
        if policy.principals.contains(&"*".to_string()) {
            return true;
        }

        // Check if user's security groups are in policy principals
        for group in &user.security_groups {
            if policy.principals.contains(group) {
                return true;
            }
        }

        // Check if user's roles are in policy principals
        for role in &user.roles {
            if policy.principals.contains(role) {
                return true;
            }
        }

        // Check if user ID is directly in principals
        policy.principals.contains(&user.id)
    }

    fn evaluate_policy_conditions(&self, conditions: &[PolicyCondition], user: &EnterpriseUser, permission: &str, resource: Option<&str>) -> bool {
        for condition in conditions {
            let actual_value = match condition.attribute.as_str() {
                "permission" => permission.to_string(),
                "resource" => resource.unwrap_or("").to_string(),
                "user_id" => user.id.clone(),
                "department" => user.metadata.get("department").cloned().unwrap_or_default(),
                "time" => {
                    let now = Utc::now();
                    now.format("%H:%M").to_string()
                },
                _ => continue,
            };

            if !self.evaluate_condition_operator(&condition.operator, &actual_value, &condition.value) {
                return false;
            }
        }

        true
    }

    fn evaluate_condition_operator(&self, operator: &str, actual: &str, expected: &str) -> bool {
        match operator {
            "equals" => actual == expected,
            "not_equals" => actual != expected,
            "contains" => actual.contains(expected),
            "starts_with" => actual.starts_with(expected),
            "ends_with" => actual.ends_with(expected),
            "between" => {
                // For time ranges like "09:00-18:00"
                if let Some((start, end)) = expected.split_once('-') {
                    actual >= start && actual <= end
                } else {
                    false
                }
            },
            _ => false,
        }
    }

    async fn get_effective_permissions(&self, user: &EnterpriseUser) -> Result<HashSet<String>> {
        let mut permissions = HashSet::new();
        let roles = self.roles.read().await;

        // Collect permissions from all user roles (including inherited)
        for role_id in &user.roles {
            if let Some(role) = roles.get(role_id) {
                if role.active {
                    permissions.extend(role.permissions.clone());
                    
                    // Add inherited permissions from parent roles
                    permissions.extend(self.get_inherited_permissions_sync(role, &roles)?);
                }
            }
        }

        Ok(permissions)
    }

    fn get_inherited_permissions_sync(&self, role: &EnterpriseRole, roles: &HashMap<String, EnterpriseRole>) -> Result<HashSet<String>> {
        let mut inherited = HashSet::new();

        for parent_role_id in &role.parent_roles {
            if let Some(parent_role) = roles.get(parent_role_id) {
                inherited.extend(parent_role.permissions.clone());
                // Recursively get permissions from parent's parents
                inherited.extend(self.get_inherited_permissions_sync(parent_role, roles)?);
            }
        }

        Ok(inherited)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<Option<EnterpriseUser>> {
        let users = self.users.read().await;
        Ok(users.get(user_id).cloned())
    }

    /// List all roles
    pub async fn list_roles(&self) -> Result<Vec<EnterpriseRole>> {
        let roles = self.roles.read().await;
        Ok(roles.values().cloned().collect())
    }

    /// Get RBAC statistics
    pub async fn get_statistics(&self) -> Result<RbacStatistics> {
        let users = self.users.read().await;
        let roles = self.roles.read().await;
        let groups = self.security_groups.read().await;
        let assignments = self.role_assignments.read().await;

        let total_assignments = assignments.values().map(|v| v.len()).sum::<usize>();
        let pending_approvals = assignments.values()
            .flatten()
            .filter(|a| a.approval_status == ApprovalStatus::Pending)
            .count();

        Ok(RbacStatistics {
            total_users: users.len(),
            active_users: users.values().filter(|u| u.active).count(),
            locked_users: users.values().filter(|u| u.account_locked).count(),
            total_roles: roles.len(),
            active_roles: roles.values().filter(|r| r.active).count(),
            total_security_groups: groups.len(),
            total_role_assignments: total_assignments,
            pending_approvals,
        })
    }
}

/// RBAC statistics for monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct RbacStatistics {
    pub total_users: usize,
    pub active_users: usize,
    pub locked_users: usize,
    pub total_roles: usize,
    pub active_roles: usize,
    pub total_security_groups: usize,
    pub total_role_assignments: usize,
    pub pending_approvals: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_enterprise_rbac_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("rbac_test.db");
        
        let permission_manager = Arc::new(PermissionManager::new().await.unwrap());
        let audit_logger = Arc::new(
            EnterpriseAuditLogger::new(Some(db_path), 365).await.unwrap()
        );
        
        let rbac = EnterpriseRbacManager::new(permission_manager, audit_logger).await.unwrap();
        rbac.initialize_default_roles().await.unwrap();

        let roles = rbac.list_roles().await.unwrap();
        assert!(roles.len() >= 6); // Should have at least 6 default roles
        
        let admin_role = roles.iter().find(|r| r.id == "admin").unwrap();
        assert_eq!(admin_role.risk_level, RiskLevel::High);
        assert!(admin_role.requires_approval);
    }

    #[tokio::test]
    async fn test_user_creation_with_auto_assign() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("auto_assign_test.db");
        
        let permission_manager = Arc::new(PermissionManager::new().await.unwrap());
        let audit_logger = Arc::new(
            EnterpriseAuditLogger::new(Some(db_path), 365).await.unwrap()
        );
        
        let rbac = EnterpriseRbacManager::new(permission_manager, audit_logger).await.unwrap();
        rbac.initialize_default_roles().await.unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("department".to_string(), "engineering".to_string());

        let user = EnterpriseUser {
            id: "dev001".to_string(),
            username: "developer1".to_string(),
            email: "dev1@company.com".to_string(),
            full_name: "Developer One".to_string(),
            roles: vec![],
            teams: vec![],
            security_groups: vec![],
            active: true,
            created_at: Utc::now(),
            last_login: None,
            password_hash: None,
            mfa_enabled: false,
            account_locked: false,
            lock_reason: None,
            last_activity: None,
            failed_login_attempts: 0,
            password_expires_at: None,
            must_change_password: false,
            session_timeout: None,
            ip_restrictions: vec![],
            metadata,
        };

        rbac.create_user(user).await.unwrap();

        let created_user = rbac.get_user("dev001").await.unwrap().unwrap();
        assert!(created_user.roles.contains(&"developer".to_string()));
    }

    #[tokio::test]
    async fn test_permission_check_with_policies() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("policy_test.db");
        
        let permission_manager = Arc::new(PermissionManager::new().await.unwrap());
        let audit_logger = Arc::new(
            EnterpriseAuditLogger::new(Some(db_path), 365).await.unwrap()
        );
        
        let rbac = EnterpriseRbacManager::new(permission_manager, audit_logger).await.unwrap();
        rbac.initialize_default_roles().await.unwrap();

        let user = EnterpriseUser {
            id: "admin001".to_string(),
            username: "admin1".to_string(),
            email: "admin@company.com".to_string(),
            full_name: "Admin User".to_string(),
            roles: vec!["admin".to_string()],
            teams: vec![],
            security_groups: vec![],
            active: true,
            created_at: Utc::now(),
            last_login: None,
            password_hash: None,
            mfa_enabled: true,
            account_locked: false,
            lock_reason: None,
            last_activity: None,
            failed_login_attempts: 0,
            password_expires_at: None,
            must_change_password: false,
            session_timeout: None,
            ip_restrictions: vec![],
            metadata: HashMap::new(),
        };

        rbac.create_user(user).await.unwrap();

        // Test permission check
        let has_permission = rbac.check_permission(
            "admin001",
            "user.create",
            Some("user:test")
        ).await.unwrap();

        assert!(has_permission);
    }

    #[tokio::test]
    async fn test_role_inheritance() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("inheritance_test.db");
        
        let permission_manager = Arc::new(PermissionManager::new().await.unwrap());
        let audit_logger = Arc::new(
            EnterpriseAuditLogger::new(Some(db_path), 365).await.unwrap()
        );
        
        let rbac = EnterpriseRbacManager::new(permission_manager, audit_logger).await.unwrap();
        rbac.initialize_default_roles().await.unwrap();

        let user = EnterpriseUser {
            id: "mgr001".to_string(),
            username: "manager1".to_string(),
            email: "mgr@company.com".to_string(),
            full_name: "Manager User".to_string(),
            roles: vec!["manager".to_string()],
            teams: vec![],
            security_groups: vec![],
            active: true,
            created_at: Utc::now(),
            last_login: None,
            password_hash: None,
            mfa_enabled: false,
            account_locked: false,
            lock_reason: None,
            last_activity: None,
            failed_login_attempts: 0,
            password_expires_at: None,
            must_change_password: false,
            session_timeout: None,
            ip_restrictions: vec![],
            metadata: HashMap::new(),
        };

        rbac.create_user(user).await.unwrap();

        // Manager should inherit permissions from admin role
        let effective_permissions = rbac.get_effective_permissions(
            &rbac.get_user("mgr001").await.unwrap().unwrap()
        ).await.unwrap();

        // Should have manager permissions
        assert!(effective_permissions.contains("team.read"));
        
        // Should also have inherited admin permissions
        assert!(effective_permissions.contains("user.create"));
    }
}