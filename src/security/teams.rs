//! Enterprise Team Management System
//!
//! Provides comprehensive team management including:
//! - Hierarchical team structures
//! - Team-based permissions
//! - Team access controls
//! - Team invitations and workflows

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{audit::EnterpriseAuditLogger, EnterpriseRbacManager};

/// Team manager for enterprise team operations
pub struct TeamManager {
    teams: Arc<RwLock<HashMap<String, EnterpriseTeam>>>,
    hierarchies: Arc<RwLock<HashMap<String, TeamHierarchy>>>,
    invitations: Arc<RwLock<HashMap<String, TeamInvitation>>>,
    rbac_manager: Arc<EnterpriseRbacManager>,
    audit_logger: Arc<EnterpriseAuditLogger>,
}

/// Enhanced enterprise team with advanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseTeam {
    pub id: String,
    pub name: String,
    pub description: String,
    pub team_type: TeamType,
    pub parent_team: Option<String>,
    pub child_teams: Vec<String>,
    pub members: Vec<TeamMember>,
    pub roles: Vec<TeamRole>,
    pub permissions: TeamPermissions,
    pub access_controls: TeamAccess,
    pub settings: TeamSettings,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub active: bool,
}

/// Team member with role and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id: String,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub invited_by: String,
    pub status: MemberStatus,
    pub permissions: Vec<String>,
    pub last_activity: Option<DateTime<Utc>>,
    pub temporary: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Team role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRole {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: HashSet<String>,
    pub can_invite: bool,
    pub can_remove: bool,
    pub can_manage_roles: bool,
    pub can_manage_settings: bool,
    pub inheritance_level: InheritanceLevel,
    pub created_at: DateTime<Utc>,
}

/// Team permissions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPermissions {
    pub resource_access: HashMap<String, Vec<String>>, // resource_type -> actions
    pub inherited_permissions: Vec<String>,
    pub custom_permissions: Vec<String>,
    pub permission_inheritance: bool,
    pub override_permissions: HashMap<String, Vec<String>>, // user_id -> permissions
}

/// Team access controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAccess {
    pub visibility: TeamVisibility,
    pub join_policy: JoinPolicy,
    pub approval_required: bool,
    pub approvers: Vec<String>,
    pub ip_restrictions: Vec<String>,
    pub time_restrictions: Option<TimeRestriction>,
    pub max_members: Option<u32>,
    pub allowed_domains: Vec<String>,
    pub blocked_domains: Vec<String>,
}

/// Team settings and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSettings {
    pub auto_archive_inactive_days: Option<u32>,
    pub require_mfa: bool,
    pub session_timeout: Option<u32>,
    pub notification_settings: NotificationSettings,
    pub integration_settings: HashMap<String, String>,
    pub custom_fields: HashMap<String, String>,
}

/// Team hierarchy management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamHierarchy {
    pub root_team: String,
    pub levels: HashMap<String, u32>, // team_id -> level
    pub parent_child_map: HashMap<String, Vec<String>>, // parent -> children
    pub inheritance_rules: Vec<HierarchyInheritanceRule>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Team invitation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamInvitation {
    pub id: String,
    pub team_id: String,
    pub invitee_email: String,
    pub invitee_user_id: Option<String>,
    pub invited_by: String,
    pub role: String,
    pub message: Option<String>,
    pub status: InvitationStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub permissions: Vec<String>,
    pub temporary: bool,
    pub temporary_duration: Option<Duration>,
}

// Enums and supporting types

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamType {
    Department,
    Project,
    Functional,
    CrossFunctional,
    Temporary,
    Virtual,
    External,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus {
    Active,
    Inactive,
    Suspended,
    PendingInvitation,
    PendingApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InheritanceLevel {
    None,
    Read,
    Partial,
    Full,
    Enhanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamVisibility {
    Public,
    Private,
    Internal,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinPolicy {
    Open,
    InviteOnly,
    RequestApproval,
    DomainRestricted,
    AdminOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestriction {
    pub start_time: String,
    pub end_time: String,
    pub days_of_week: Vec<u8>,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub slack_notifications: bool,
    pub webhook_url: Option<String>,
    pub notification_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyInheritanceRule {
    pub rule_type: String,
    pub from_level: u32,
    pub to_level: u32,
    pub permissions: Vec<String>,
    pub conditions: Vec<String>,
}

impl Default for TeamPermissions {
    fn default() -> Self {
        Self {
            resource_access: HashMap::new(),
            inherited_permissions: vec![],
            custom_permissions: vec![],
            permission_inheritance: true,
            override_permissions: HashMap::new(),
        }
    }
}

impl Default for TeamAccess {
    fn default() -> Self {
        Self {
            visibility: TeamVisibility::Private,
            join_policy: JoinPolicy::InviteOnly,
            approval_required: true,
            approvers: vec![],
            ip_restrictions: vec![],
            time_restrictions: None,
            max_members: None,
            allowed_domains: vec![],
            blocked_domains: vec![],
        }
    }
}

impl Default for TeamSettings {
    fn default() -> Self {
        Self {
            auto_archive_inactive_days: Some(90),
            require_mfa: false,
            session_timeout: None,
            notification_settings: NotificationSettings {
                email_notifications: true,
                slack_notifications: false,
                webhook_url: None,
                notification_events: vec![
                    "member_joined".to_string(),
                    "member_left".to_string(),
                    "role_changed".to_string(),
                ],
            },
            integration_settings: HashMap::new(),
            custom_fields: HashMap::new(),
        }
    }
}

impl TeamManager {
    pub async fn new(
        rbac_manager: Arc<EnterpriseRbacManager>,
        audit_logger: Arc<EnterpriseAuditLogger>,
    ) -> Result<Self> {
        Ok(Self {
            teams: Arc::new(RwLock::new(HashMap::new())),
            hierarchies: Arc::new(RwLock::new(HashMap::new())),
            invitations: Arc::new(RwLock::new(HashMap::new())),
            rbac_manager,
            audit_logger,
        })
    }

    /// Create a new enterprise team
    pub async fn create_team(&self, team: EnterpriseTeam) -> Result<()> {
        let mut teams = self.teams.write().await;

        if teams.contains_key(&team.id) {
            return Err(anyhow!("Team already exists: {}", team.id));
        }

        // Validate parent team exists if specified
        if let Some(parent_id) = &team.parent_team {
            if !teams.contains_key(parent_id) {
                return Err(anyhow!("Parent team not found: {}", parent_id));
            }
        }

        teams.insert(team.id.clone(), team.clone());

        // Update hierarchy if this team has a parent
        if let Some(parent_id) = &team.parent_team {
            self.update_hierarchy(&team.id, Some(parent_id)).await?;
        }

        self.audit_logger
            .log_team_event(
                super::audit::AuditEventType::TeamCreated,
                &team.id,
                format!("Team '{}' created by {}", team.name, team.created_by),
            )
            .await?;

        Ok(())
    }

    async fn update_hierarchy(&self, team_id: &str, parent_id: Option<&str>) -> Result<()> {
        let mut hierarchies = self.hierarchies.write().await;

        if let Some(parent) = parent_id {
            // Find or create hierarchy for this parent chain
            let root_team = self.find_root_team(parent).await?;
            let hierarchy_id = root_team.clone();

            let hierarchy = hierarchies
                .entry(hierarchy_id)
                .or_insert_with(|| TeamHierarchy {
                    root_team,
                    levels: HashMap::new(),
                    parent_child_map: HashMap::new(),
                    inheritance_rules: vec![],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });

            // Calculate level for new team
            let parent_level = hierarchy.levels.get(parent).copied().unwrap_or(0);
            hierarchy
                .levels
                .insert(team_id.to_string(), parent_level + 1);

            // Update parent-child mapping
            hierarchy
                .parent_child_map
                .entry(parent.to_string())
                .or_insert_with(Vec::new)
                .push(team_id.to_string());

            hierarchy.updated_at = Utc::now();
        }

        Ok(())
    }

    async fn find_root_team(&self, team_id: &str) -> Result<String> {
        let teams = self.teams.read().await;
        let mut current_id = team_id.to_string();

        // Traverse up the hierarchy to find root
        while let Some(team) = teams.get(&current_id) {
            if let Some(parent_id) = &team.parent_team {
                current_id = parent_id.clone();
            } else {
                break;
            }
        }

        Ok(current_id)
    }

    /// Add user to team
    pub async fn add_user_to_team(&self, user_id: &str, team_id: &str) -> Result<()> {
        let mut teams = self.teams.write().await;
        let team = teams
            .get_mut(team_id)
            .ok_or_else(|| anyhow!("Team not found: {}", team_id))?;

        // Check if user is already a member
        if team.members.iter().any(|m| m.user_id == user_id) {
            return Err(anyhow!("User is already a member of team: {}", team_id));
        }

        // Check team capacity
        if let Some(max_members) = team.access_controls.max_members {
            if team.members.len() >= max_members as usize {
                return Err(anyhow!("Team has reached maximum member capacity"));
            }
        }

        // Default role is "member"
        let default_role = team
            .roles
            .iter()
            .find(|r| r.name == "member")
            .map(|r| r.id.clone())
            .unwrap_or_else(|| "member".to_string());

        let member = TeamMember {
            user_id: user_id.to_string(),
            role: default_role,
            joined_at: Utc::now(),
            invited_by: "system".to_string(), // Would be the current user in real implementation
            status: MemberStatus::Active,
            permissions: vec![],
            last_activity: Some(Utc::now()),
            temporary: false,
            expires_at: None,
        };

        team.members.push(member);

        self.audit_logger
            .log_team_event(
                super::audit::AuditEventType::UserAddedToTeam,
                team_id,
                format!("User {} added to team", user_id),
            )
            .await?;

        Ok(())
    }

    /// Remove user from team
    pub async fn remove_user_from_team(&self, user_id: &str, team_id: &str) -> Result<()> {
        let mut teams = self.teams.write().await;
        let team = teams
            .get_mut(team_id)
            .ok_or_else(|| anyhow!("Team not found: {}", team_id))?;

        let initial_len = team.members.len();
        team.members.retain(|m| m.user_id != user_id);

        if team.members.len() == initial_len {
            return Err(anyhow!("User is not a member of team: {}", team_id));
        }

        self.audit_logger
            .log_team_event(
                super::audit::AuditEventType::UserRemovedFromTeam,
                team_id,
                format!("User {} removed from team", user_id),
            )
            .await?;

        Ok(())
    }

    /// Create team invitation
    pub async fn create_invitation(
        &self,
        team_id: &str,
        invitee_email: &str,
        role: &str,
        invited_by: &str,
        message: Option<String>,
        temporary: bool,
        duration: Option<Duration>,
    ) -> Result<String> {
        let teams = self.teams.read().await;
        let team = teams
            .get(team_id)
            .ok_or_else(|| anyhow!("Team not found: {}", team_id))?;

        // Validate role exists in team
        if !team.roles.iter().any(|r| r.id == role) {
            return Err(anyhow!("Role not found in team: {}", role));
        }

        let invitation_id = uuid::Uuid::new_v4().to_string();
        let invitation = TeamInvitation {
            id: invitation_id.clone(),
            team_id: team_id.to_string(),
            invitee_email: invitee_email.to_string(),
            invitee_user_id: None, // Would be resolved if user exists
            invited_by: invited_by.to_string(),
            role: role.to_string(),
            message,
            status: InvitationStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(7), // 7 day expiry
            accepted_at: None,
            declined_at: None,
            permissions: vec![],
            temporary,
            temporary_duration: duration,
        };

        let mut invitations = self.invitations.write().await;
        invitations.insert(invitation_id.clone(), invitation);

        self.audit_logger
            .log_team_event(
                super::audit::AuditEventType::TeamCreated, // Would have specific invitation event type
                team_id,
                format!("Invitation created for {} by {}", invitee_email, invited_by),
            )
            .await?;

        Ok(invitation_id)
    }

    /// Accept team invitation
    pub async fn accept_invitation(&self, invitation_id: &str, user_id: &str) -> Result<()> {
        let mut invitations = self.invitations.write().await;
        let invitation = invitations
            .get_mut(invitation_id)
            .ok_or_else(|| anyhow!("Invitation not found: {}", invitation_id))?;

        if invitation.status != InvitationStatus::Pending {
            return Err(anyhow!("Invitation is not pending"));
        }

        if invitation.expires_at < Utc::now() {
            invitation.status = InvitationStatus::Expired;
            return Err(anyhow!("Invitation has expired"));
        }

        invitation.status = InvitationStatus::Accepted;
        invitation.accepted_at = Some(Utc::now());
        invitation.invitee_user_id = Some(user_id.to_string());

        // Clone needed values before dropping the lock
        let team_id = invitation.team_id.clone();
        let is_temporary = invitation.temporary;
        let temp_duration = invitation.temporary_duration;

        // Add user to team
        drop(invitations); // Release lock before calling add_user_to_team
        self.add_user_to_team(user_id, &team_id).await?;

        // Set temporary membership if specified
        if is_temporary {
            self.set_temporary_membership(user_id, &team_id, temp_duration)
                .await?;
        }

        Ok(())
    }

    async fn set_temporary_membership(
        &self,
        user_id: &str,
        team_id: &str,
        duration: Option<Duration>,
    ) -> Result<()> {
        let mut teams = self.teams.write().await;
        let team = teams
            .get_mut(team_id)
            .ok_or_else(|| anyhow!("Team not found: {}", team_id))?;

        if let Some(member) = team.members.iter_mut().find(|m| m.user_id == user_id) {
            member.temporary = true;
            member.expires_at = duration.map(|d| Utc::now() + d);
        }

        Ok(())
    }

    /// Get team by ID
    pub async fn get_team(&self, team_id: &str) -> Result<Option<EnterpriseTeam>> {
        let teams = self.teams.read().await;
        Ok(teams.get(team_id).cloned())
    }

    /// List teams for user
    pub async fn list_user_teams(&self, user_id: &str) -> Result<Vec<EnterpriseTeam>> {
        let teams = self.teams.read().await;
        Ok(teams
            .values()
            .filter(|team| {
                team.members
                    .iter()
                    .any(|m| m.user_id == user_id && m.status == MemberStatus::Active)
            })
            .cloned()
            .collect())
    }

    /// Get team hierarchy
    pub async fn get_team_hierarchy(&self, root_team_id: &str) -> Result<Option<TeamHierarchy>> {
        let hierarchies = self.hierarchies.read().await;
        Ok(hierarchies.get(root_team_id).cloned())
    }

    /// Check if user has team permission
    pub fn check_team_permission<'a>(
        &'a self,
        user_id: &'a str,
        team_id: &'a str,
        permission: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + 'a>> {
        Box::pin(async move {
            let teams = self.teams.read().await;
            let team = teams
                .get(team_id)
                .ok_or_else(|| anyhow!("Team not found: {}", team_id))?;

            // Find user's membership
            let member = team
                .members
                .iter()
                .find(|m| m.user_id == user_id && m.status == MemberStatus::Active)
                .ok_or_else(|| anyhow!("User is not an active member of team"))?;

            // Check if user has the permission directly
            if member.permissions.contains(&permission.to_string()) {
                return Ok(true);
            }

            // Check role-based permissions
            if let Some(role) = team.roles.iter().find(|r| r.id == member.role) {
                if role.permissions.contains(permission) {
                    return Ok(true);
                }
            }

            // Check team-level permissions
            for (resource_type, actions) in &team.permissions.resource_access {
                if permission.starts_with(resource_type) {
                    let action = permission.split('.').nth(1).unwrap_or("");
                    if actions.contains(&action.to_string()) || actions.contains(&"*".to_string()) {
                        return Ok(true);
                    }
                }
            }

            // Check inherited permissions from parent teams
            if team.permissions.permission_inheritance {
                if let Some(parent_id) = &team.parent_team {
                    let parent_id = parent_id.clone();
                    drop(teams); // Release lock before recursive call
                    return self
                        .check_team_permission(user_id, &parent_id, permission)
                        .await;
                }
            }

            Ok(false)
        })
    }

    /// Get team statistics
    pub async fn get_statistics(&self) -> Result<TeamStatistics> {
        let teams = self.teams.read().await;
        let invitations = self.invitations.read().await;
        let hierarchies = self.hierarchies.read().await;

        let total_members = teams.values().map(|team| team.members.len()).sum::<usize>();

        let active_members = teams
            .values()
            .flat_map(|team| &team.members)
            .filter(|member| member.status == MemberStatus::Active)
            .count();

        let pending_invitations = invitations
            .values()
            .filter(|inv| inv.status == InvitationStatus::Pending && inv.expires_at > Utc::now())
            .count();

        Ok(TeamStatistics {
            total_teams: teams.len(),
            active_teams: teams.values().filter(|t| t.active).count(),
            total_members,
            active_members,
            pending_invitations,
            total_hierarchies: hierarchies.len(),
        })
    }

    /// Initialize default team roles
    pub async fn initialize_default_roles(&self, team_id: &str) -> Result<()> {
        let default_roles = vec![
            TeamRole {
                id: "owner".to_string(),
                name: "Owner".to_string(),
                description: "Full control over team".to_string(),
                permissions: [
                    "team.manage".to_string(),
                    "team.delete".to_string(),
                    "members.invite".to_string(),
                    "members.remove".to_string(),
                    "roles.manage".to_string(),
                    "settings.manage".to_string(),
                ]
                .into_iter()
                .collect(),
                can_invite: true,
                can_remove: true,
                can_manage_roles: true,
                can_manage_settings: true,
                inheritance_level: InheritanceLevel::Full,
                created_at: Utc::now(),
            },
            TeamRole {
                id: "admin".to_string(),
                name: "Administrator".to_string(),
                description: "Administrative access to team".to_string(),
                permissions: [
                    "members.invite".to_string(),
                    "members.remove".to_string(),
                    "settings.view".to_string(),
                    "settings.modify".to_string(),
                ]
                .into_iter()
                .collect(),
                can_invite: true,
                can_remove: true,
                can_manage_roles: false,
                can_manage_settings: true,
                inheritance_level: InheritanceLevel::Partial,
                created_at: Utc::now(),
            },
            TeamRole {
                id: "member".to_string(),
                name: "Member".to_string(),
                description: "Standard team member".to_string(),
                permissions: ["team.view".to_string(), "members.view".to_string()]
                    .into_iter()
                    .collect(),
                can_invite: false,
                can_remove: false,
                can_manage_roles: false,
                can_manage_settings: false,
                inheritance_level: InheritanceLevel::Read,
                created_at: Utc::now(),
            },
        ];

        let mut teams = self.teams.write().await;
        if let Some(team) = teams.get_mut(team_id) {
            team.roles = default_roles;
        }

        Ok(())
    }

    /// Cleanup expired memberships and invitations
    pub async fn cleanup_expired(&self) -> Result<CleanupStatistics> {
        let now = Utc::now();
        let mut expired_memberships = 0;
        let mut expired_invitations = 0;

        // Cleanup expired memberships
        {
            let mut teams = self.teams.write().await;
            for team in teams.values_mut() {
                let initial_count = team.members.len();
                team.members.retain(|member| {
                    if member.temporary {
                        if let Some(expires_at) = member.expires_at {
                            expires_at > now
                        } else {
                            true // Keep if no expiry set
                        }
                    } else {
                        true // Keep non-temporary members
                    }
                });
                expired_memberships += initial_count - team.members.len();
            }
        }

        // Cleanup expired invitations
        {
            let mut invitations = self.invitations.write().await;
            let initial_count = invitations.len();
            invitations.retain(|_, invitation| {
                if invitation.expires_at < now && invitation.status == InvitationStatus::Pending {
                    false // Remove expired pending invitations
                } else {
                    true
                }
            });
            expired_invitations = initial_count - invitations.len();
        }

        Ok(CleanupStatistics {
            expired_memberships,
            expired_invitations,
        })
    }
}

/// Team management statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct TeamStatistics {
    pub total_teams: usize,
    pub active_teams: usize,
    pub total_members: usize,
    pub active_members: usize,
    pub pending_invitations: usize,
    pub total_hierarchies: usize,
}

/// Cleanup operation statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupStatistics {
    pub expired_memberships: usize,
    pub expired_invitations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_team_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("team_test.db");

        let permission_manager = Arc::new(super::PermissionManager::new().await.unwrap());
        let audit_logger = Arc::new(
            super::EnterpriseAuditLogger::new(Some(db_path), 365)
                .await
                .unwrap(),
        );
        let rbac_manager = Arc::new(
            super::EnterpriseRbacManager::new(permission_manager, audit_logger.clone())
                .await
                .unwrap(),
        );

        let team_manager = TeamManager::new(rbac_manager, audit_logger).await.unwrap();

        let team = EnterpriseTeam {
            id: "team001".to_string(),
            name: "Engineering Team".to_string(),
            description: "Software engineering team".to_string(),
            team_type: TeamType::Department,
            parent_team: None,
            child_teams: vec![],
            members: vec![],
            roles: vec![],
            permissions: TeamPermissions::default(),
            access_controls: TeamAccess::default(),
            settings: TeamSettings::default(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            created_by: "admin".to_string(),
            active: true,
        };

        team_manager.create_team(team).await.unwrap();

        let retrieved_team = team_manager.get_team("team001").await.unwrap();
        assert!(retrieved_team.is_some());
        assert_eq!(retrieved_team.unwrap().name, "Engineering Team");
    }

    #[tokio::test]
    async fn test_team_membership() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("membership_test.db");

        let permission_manager = Arc::new(super::PermissionManager::new().await.unwrap());
        let audit_logger = Arc::new(
            super::EnterpriseAuditLogger::new(Some(db_path), 365)
                .await
                .unwrap(),
        );
        let rbac_manager = Arc::new(
            super::EnterpriseRbacManager::new(permission_manager, audit_logger.clone())
                .await
                .unwrap(),
        );

        let team_manager = TeamManager::new(rbac_manager, audit_logger).await.unwrap();

        // Create team
        let team = EnterpriseTeam {
            id: "team002".to_string(),
            name: "Test Team".to_string(),
            description: "Test team".to_string(),
            team_type: TeamType::Project,
            parent_team: None,
            child_teams: vec![],
            members: vec![],
            roles: vec![],
            permissions: TeamPermissions::default(),
            access_controls: TeamAccess::default(),
            settings: TeamSettings::default(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            created_by: "admin".to_string(),
            active: true,
        };

        team_manager.create_team(team).await.unwrap();

        // Initialize default roles
        team_manager
            .initialize_default_roles("team002")
            .await
            .unwrap();

        // Add user to team
        team_manager
            .add_user_to_team("user001", "team002")
            .await
            .unwrap();

        let team = team_manager.get_team("team002").await.unwrap().unwrap();
        assert_eq!(team.members.len(), 1);
        assert_eq!(team.members[0].user_id, "user001");
        assert_eq!(team.members[0].status, MemberStatus::Active);

        // Remove user from team
        team_manager
            .remove_user_from_team("user001", "team002")
            .await
            .unwrap();

        let team = team_manager.get_team("team002").await.unwrap().unwrap();
        assert_eq!(team.members.len(), 0);
    }

    #[tokio::test]
    async fn test_team_invitations() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("invitation_test.db");

        let permission_manager = Arc::new(super::PermissionManager::new().await.unwrap());
        let audit_logger = Arc::new(
            super::EnterpriseAuditLogger::new(Some(db_path), 365)
                .await
                .unwrap(),
        );
        let rbac_manager = Arc::new(
            super::EnterpriseRbacManager::new(permission_manager, audit_logger.clone())
                .await
                .unwrap(),
        );

        let team_manager = TeamManager::new(rbac_manager, audit_logger).await.unwrap();

        // Create team with roles
        let mut team = EnterpriseTeam {
            id: "team003".to_string(),
            name: "Invitation Test Team".to_string(),
            description: "Test team for invitations".to_string(),
            team_type: TeamType::Project,
            parent_team: None,
            child_teams: vec![],
            members: vec![],
            roles: vec![TeamRole {
                id: "member".to_string(),
                name: "Member".to_string(),
                description: "Standard member".to_string(),
                permissions: HashSet::new(),
                can_invite: false,
                can_remove: false,
                can_manage_roles: false,
                can_manage_settings: false,
                inheritance_level: InheritanceLevel::Read,
                created_at: Utc::now(),
            }],
            permissions: TeamPermissions::default(),
            access_controls: TeamAccess::default(),
            settings: TeamSettings::default(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            created_by: "admin".to_string(),
            active: true,
        };

        team_manager.create_team(team).await.unwrap();

        // Create invitation
        let invitation_id = team_manager
            .create_invitation(
                "team003",
                "newuser@company.com",
                "member",
                "admin",
                Some("Welcome to the team!".to_string()),
                false,
                None,
            )
            .await
            .unwrap();

        assert!(!invitation_id.is_empty());

        // Accept invitation
        team_manager
            .accept_invitation(&invitation_id, "user002")
            .await
            .unwrap();

        let team = team_manager.get_team("team003").await.unwrap().unwrap();
        assert_eq!(team.members.len(), 1);
        assert_eq!(team.members[0].user_id, "user002");
    }

    #[tokio::test]
    async fn test_team_permissions() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("permissions_test.db");

        let permission_manager = Arc::new(super::PermissionManager::new().await.unwrap());
        let audit_logger = Arc::new(
            super::EnterpriseAuditLogger::new(Some(db_path), 365)
                .await
                .unwrap(),
        );
        let rbac_manager = Arc::new(
            super::EnterpriseRbacManager::new(permission_manager, audit_logger.clone())
                .await
                .unwrap(),
        );

        let team_manager = TeamManager::new(rbac_manager, audit_logger).await.unwrap();

        // Create team with specific permissions
        let mut team_permissions = TeamPermissions::default();
        team_permissions.resource_access.insert(
            "project".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );

        let team = EnterpriseTeam {
            id: "team004".to_string(),
            name: "Permissions Test Team".to_string(),
            description: "Test team for permissions".to_string(),
            team_type: TeamType::Project,
            parent_team: None,
            child_teams: vec![],
            members: vec![TeamMember {
                user_id: "user003".to_string(),
                role: "member".to_string(),
                joined_at: Utc::now(),
                invited_by: "admin".to_string(),
                status: MemberStatus::Active,
                permissions: vec!["project.read".to_string()],
                last_activity: Some(Utc::now()),
                temporary: false,
                expires_at: None,
            }],
            roles: vec![],
            permissions: team_permissions,
            access_controls: TeamAccess::default(),
            settings: TeamSettings::default(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            created_by: "admin".to_string(),
            active: true,
        };

        team_manager.create_team(team).await.unwrap();

        // Test permission check
        let has_permission = team_manager
            .check_team_permission("user003", "team004", "project.read")
            .await
            .unwrap();

        assert!(has_permission);

        let no_permission = team_manager
            .check_team_permission("user003", "team004", "project.delete")
            .await
            .unwrap();

        assert!(!no_permission);
    }
}
