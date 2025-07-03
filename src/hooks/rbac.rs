//! Role-Based Access Control for Enterprise Hooks

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use super::{HookId, SecurityPolicy};

/// User role definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: HashSet<Permission>,
    pub inherits: Vec<String>, // Roles this role inherits from
}

/// Permission types for hook operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    // Hook management permissions
    CreateHook,
    DeleteHook,
    ModifyHook,
    EnableHook,
    DisableHook,
    ViewHook,
    
    // Hook execution permissions
    ExecuteHook,
    ApproveHookExecution,
    DenyHookExecution,
    
    // Security permissions
    ManageSecurity,
    ViewAuditLogs,
    ManageApprovals,
    
    // Administrative permissions
    ManageUsers,
    ManageRoles,
    ManageTeams,
    
    // System permissions
    SystemAdmin,
    
    // Custom permissions
    Custom(String),
}

impl Permission {
    pub fn as_str(&self) -> &str {
        match self {
            Permission::CreateHook => "create_hook",
            Permission::DeleteHook => "delete_hook",
            Permission::ModifyHook => "modify_hook",
            Permission::EnableHook => "enable_hook",
            Permission::DisableHook => "disable_hook",
            Permission::ViewHook => "view_hook",
            Permission::ExecuteHook => "execute_hook",
            Permission::ApproveHookExecution => "approve_hook_execution",
            Permission::DenyHookExecution => "deny_hook_execution",
            Permission::ManageSecurity => "manage_security",
            Permission::ViewAuditLogs => "view_audit_logs",
            Permission::ManageApprovals => "manage_approvals",
            Permission::ManageUsers => "manage_users",
            Permission::ManageRoles => "manage_roles",
            Permission::ManageTeams => "manage_teams",
            Permission::SystemAdmin => "system_admin",
            Permission::Custom(name) => name,
        }
    }
}

/// User definition with roles and teams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub roles: HashSet<String>,
    pub teams: HashSet<String>,
    pub direct_permissions: HashSet<Permission>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Team definition for group-based permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub description: String,
    pub roles: HashSet<String>,
    pub members: HashSet<String>,
    pub hook_access: HookAccess,
}

/// Hook access control per team/user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookAccess {
    pub allowed_hooks: HashSet<HookId>,
    pub denied_hooks: HashSet<HookId>,
    pub hook_patterns: Vec<String>, // Glob patterns for hook names
}

impl Default for HookAccess {
    fn default() -> Self {
        Self {
            allowed_hooks: HashSet::new(),
            denied_hooks: HashSet::new(),
            hook_patterns: vec!["*".to_string()], // Allow all by default
        }
    }
}

/// RBAC manager for hooks
pub struct HookRbacManager {
    users: Arc<RwLock<HashMap<String, User>>>,
    roles: Arc<RwLock<HashMap<String, Role>>>,
    teams: Arc<RwLock<HashMap<String, Team>>>,
}

impl HookRbacManager {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            teams: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize default roles (should be called after creation)
    pub async fn initialize(&self) -> Result<()> {
        self.initialize_default_roles().await
    }
    
    /// Initialize default roles
    async fn initialize_default_roles(&self) -> Result<()> {
        let default_roles = vec![
            Role {
                name: "admin".to_string(),
                description: "Full system administrator".to_string(),
                permissions: [Permission::SystemAdmin].into_iter().collect(),
                inherits: vec![],
            },
            Role {
                name: "hook_admin".to_string(),
                description: "Hook system administrator".to_string(),
                permissions: [
                    Permission::CreateHook,
                    Permission::DeleteHook,
                    Permission::ModifyHook,
                    Permission::EnableHook,
                    Permission::DisableHook,
                    Permission::ViewHook,
                    Permission::ManageSecurity,
                    Permission::ViewAuditLogs,
                ].into_iter().collect(),
                inherits: vec![],
            },
            Role {
                name: "developer".to_string(),
                description: "Developer with hook creation and modification rights".to_string(),
                permissions: [
                    Permission::CreateHook,
                    Permission::ModifyHook,
                    Permission::ViewHook,
                    Permission::ExecuteHook,
                ].into_iter().collect(),
                inherits: vec![],
            },
            Role {
                name: "approver".to_string(),
                description: "Can approve hook executions".to_string(),
                permissions: [
                    Permission::ViewHook,
                    Permission::ApproveHookExecution,
                    Permission::DenyHookExecution,
                    Permission::ViewAuditLogs,
                ].into_iter().collect(),
                inherits: vec![],
            },
            Role {
                name: "viewer".to_string(),
                description: "Read-only access to hooks".to_string(),
                permissions: [
                    Permission::ViewHook,
                    Permission::ViewAuditLogs,
                ].into_iter().collect(),
                inherits: vec![],
            },
        ];
        
        let mut roles = self.roles.write().await;
        for role in default_roles {
            roles.insert(role.name.clone(), role);
        }
        
        Ok(())
    }
    
    /// Check if user has permission for hook operation
    pub async fn check_permission(&self, user_id: &str, permission: &Permission, hook_id: Option<&HookId>) -> Result<bool> {
        let user = self.get_user(user_id).await?
            .ok_or_else(|| anyhow!("User not found: {}", user_id))?;
        
        if !user.active {
            return Ok(false);
        }
        
        // Check if user has direct permission
        if user.direct_permissions.contains(permission) {
            return Ok(true);
        }
        
        // Check role-based permissions
        let user_permissions = self.get_user_permissions(&user).await?;
        if user_permissions.contains(permission) || user_permissions.contains(&Permission::SystemAdmin) {
            return Ok(true);
        }
        
        // Check hook-specific access if hook_id provided
        if let Some(hook_id) = hook_id {
            return self.check_hook_access(user_id, hook_id).await;
        }
        
        Ok(false)
    }
    
    /// Check if user has access to specific hook
    pub async fn check_hook_access(&self, user_id: &str, hook_id: &HookId) -> Result<bool> {
        let user = self.get_user(user_id).await?
            .ok_or_else(|| anyhow!("User not found: {}", user_id))?;
        
        // Check team-based hook access
        for team_name in &user.teams {
            if let Some(team) = self.get_team(team_name).await? {
                // Check explicit denials first
                if team.hook_access.denied_hooks.contains(hook_id) {
                    return Ok(false);
                }
                
                // Check explicit allowances
                if team.hook_access.allowed_hooks.contains(hook_id) {
                    return Ok(true);
                }
                
                // Check pattern matching
                for pattern in &team.hook_access.hook_patterns {
                    if self.matches_pattern(&hook_id.0, pattern) {
                        return Ok(true);
                    }
                }
            }
        }
        
        // Default deny if no explicit access granted
        Ok(false)
    }
    
    /// Get all permissions for a user (including inherited)
    async fn get_user_permissions(&self, user: &User) -> Result<HashSet<Permission>> {
        let mut permissions = user.direct_permissions.clone();
        
        let roles = self.roles.read().await;
        
        for role_name in &user.roles {
            if let Some(role) = roles.get(role_name) {
                permissions.extend(self.get_role_permissions_sync(role, &roles)?);
            }
        }
        
        // Add team-based permissions
        for team_name in &user.teams {
            if let Some(team) = self.get_team(team_name).await? {
                for role_name in &team.roles {
                    if let Some(role) = roles.get(role_name) {
                        permissions.extend(self.get_role_permissions_sync(role, &roles)?);
                    }
                }
            }
        }
        
        Ok(permissions)
    }
    
    /// Get permissions for a role (including inherited)
    fn get_role_permissions_sync(&self, role: &Role, roles: &HashMap<String, Role>) -> Result<HashSet<Permission>> {
        let mut permissions = role.permissions.clone();
        
        for inherited_role_name in &role.inherits {
            if let Some(inherited_role) = roles.get(inherited_role_name) {
                permissions.extend(self.get_role_permissions_sync(inherited_role, roles)?);
            }
        }
        
        Ok(permissions)
    }
    
    /// Create a new user
    pub async fn create_user(&self, user: User) -> Result<()> {
        let mut users = self.users.write().await;
        
        if users.contains_key(&user.id) {
            return Err(anyhow!("User already exists: {}", user.id));
        }
        
        users.insert(user.id.clone(), user);
        Ok(())
    }
    
    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.get(user_id).cloned())
    }
    
    /// Update user
    pub async fn update_user(&self, user: User) -> Result<()> {
        let mut users = self.users.write().await;
        users.insert(user.id.clone(), user);
        Ok(())
    }
    
    /// Delete user
    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let mut users = self.users.write().await;
        users.remove(user_id);
        Ok(())
    }
    
    /// Create a new role
    pub async fn create_role(&self, role: Role) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        if roles.contains_key(&role.name) {
            return Err(anyhow!("Role already exists: {}", role.name));
        }
        
        roles.insert(role.name.clone(), role);
        Ok(())
    }
    
    /// Get role by name
    pub async fn get_role(&self, role_name: &str) -> Result<Option<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.get(role_name).cloned())
    }
    
    /// Update role
    pub async fn update_role(&self, role: Role) -> Result<()> {
        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role);
        Ok(())
    }
    
    /// Delete role
    pub async fn delete_role(&self, role_name: &str) -> Result<()> {
        let mut roles = self.roles.write().await;
        roles.remove(role_name);
        Ok(())
    }
    
    /// Create a new team
    pub async fn create_team(&self, team: Team) -> Result<()> {
        let mut teams = self.teams.write().await;
        
        if teams.contains_key(&team.name) {
            return Err(anyhow!("Team already exists: {}", team.name));
        }
        
        teams.insert(team.name.clone(), team);
        Ok(())
    }
    
    /// Get team by name
    pub async fn get_team(&self, team_name: &str) -> Result<Option<Team>> {
        let teams = self.teams.read().await;
        Ok(teams.get(team_name).cloned())
    }
    
    /// Update team
    pub async fn update_team(&self, team: Team) -> Result<()> {
        let mut teams = self.teams.write().await;
        teams.insert(team.name.clone(), team);
        Ok(())
    }
    
    /// Delete team
    pub async fn delete_team(&self, team_name: &str) -> Result<()> {
        let mut teams = self.teams.write().await;
        teams.remove(team_name);
        Ok(())
    }
    
    /// Add user to team
    pub async fn add_user_to_team(&self, user_id: &str, team_name: &str) -> Result<()> {
        // Add user to team members
        {
            let mut teams = self.teams.write().await;
            if let Some(team) = teams.get_mut(team_name) {
                team.members.insert(user_id.to_string());
            } else {
                return Err(anyhow!("Team not found: {}", team_name));
            }
        }
        
        // Add team to user
        {
            let mut users = self.users.write().await;
            if let Some(user) = users.get_mut(user_id) {
                user.teams.insert(team_name.to_string());
            } else {
                return Err(anyhow!("User not found: {}", user_id));
            }
        }
        
        Ok(())
    }
    
    /// Remove user from team
    pub async fn remove_user_from_team(&self, user_id: &str, team_name: &str) -> Result<()> {
        // Remove user from team members
        {
            let mut teams = self.teams.write().await;
            if let Some(team) = teams.get_mut(team_name) {
                team.members.remove(user_id);
            }
        }
        
        // Remove team from user
        {
            let mut users = self.users.write().await;
            if let Some(user) = users.get_mut(user_id) {
                user.teams.remove(team_name);
            }
        }
        
        Ok(())
    }
    
    /// Assign role to user
    pub async fn assign_role(&self, user_id: &str, role_name: &str) -> Result<()> {
        let mut users = self.users.write().await;
        let user = users.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found: {}", user_id))?;
        
        // Verify role exists
        {
            let roles = self.roles.read().await;
            if !roles.contains_key(role_name) {
                return Err(anyhow!("Role not found: {}", role_name));
            }
        }
        
        user.roles.insert(role_name.to_string());
        Ok(())
    }
    
    /// Remove role from user
    pub async fn remove_role(&self, user_id: &str, role_name: &str) -> Result<()> {
        let mut users = self.users.write().await;
        let user = users.get_mut(user_id)
            .ok_or_else(|| anyhow!("User not found: {}", user_id))?;
        
        user.roles.remove(role_name);
        Ok(())
    }
    
    /// List all users
    pub async fn list_users(&self) -> Result<Vec<User>> {
        let users = self.users.read().await;
        Ok(users.values().cloned().collect())
    }
    
    /// List all roles
    pub async fn list_roles(&self) -> Result<Vec<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.values().cloned().collect())
    }
    
    /// List all teams
    pub async fn list_teams(&self) -> Result<Vec<Team>> {
        let teams = self.teams.read().await;
        Ok(teams.values().cloned().collect())
    }
    
    /// Simple pattern matching for hook names
    fn matches_pattern(&self, hook_name: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        // Simple glob matching
        if pattern.contains('*') {
            let regex_pattern = pattern
                .replace('.', r"\.")
                .replace('*', ".*")
                .replace('?', ".");
            
            if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
                return regex.is_match(hook_name);
            }
        }
        
        hook_name == pattern
    }
    
    /// Get RBAC statistics
    pub async fn get_statistics(&self) -> RbacStatistics {
        let users = self.users.read().await;
        let roles = self.roles.read().await;
        let teams = self.teams.read().await;
        
        RbacStatistics {
            total_users: users.len(),
            active_users: users.values().filter(|u| u.active).count(),
            total_roles: roles.len(),
            total_teams: teams.len(),
            users_with_teams: users.values().filter(|u| !u.teams.is_empty()).count(),
            users_with_roles: users.values().filter(|u| !u.roles.is_empty()).count(),
        }
    }
}

/// RBAC statistics
#[derive(Debug, Serialize)]
pub struct RbacStatistics {
    pub total_users: usize,
    pub active_users: usize,
    pub total_roles: usize,
    pub total_teams: usize,
    pub users_with_teams: usize,
    pub users_with_roles: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_permission_checking() {
        let rbac = HookRbacManager::new();
        
        // Create test user with developer role
        let user = User {
            id: "test_user".to_string(),
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            roles: ["developer".to_string()].into_iter().collect(),
            teams: HashSet::new(),
            direct_permissions: HashSet::new(),
            active: true,
            created_at: chrono::Utc::now(),
        };
        
        rbac.create_user(user).await.unwrap();
        
        // Test permission checking
        let has_permission = rbac.check_permission(
            "test_user",
            &Permission::CreateHook,
            None
        ).await.unwrap();
        
        assert!(has_permission);
        
        let no_permission = rbac.check_permission(
            "test_user",
            &Permission::DeleteHook,
            None
        ).await.unwrap();
        
        assert!(!no_permission);
    }
}