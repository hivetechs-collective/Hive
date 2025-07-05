//! Compliance Management System
//! 
//! Provides comprehensive compliance monitoring and reporting for:
//! - SOX (Sarbanes-Oxley Act)
//! - GDPR (General Data Protection Regulation)
//! - ISO 27001 (Information Security Management)
//! - Custom compliance frameworks

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use tokio::sync::RwLock;

use super::audit::{EnterpriseAuditLogger, AuditEvent, AuditEventType};

/// Compliance manager
pub struct ComplianceManager {
    standards: Vec<ComplianceStandard>,
    rules: Arc<RwLock<HashMap<String, Vec<ComplianceRule>>>>,
    violations: Arc<RwLock<Vec<ComplianceViolation>>>,
    audit_logger: Arc<EnterpriseAuditLogger>,
}

/// Compliance standard definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStandard {
    pub name: String,
    pub version: String,
    pub description: String,
    pub requirements: Vec<ComplianceRequirement>,
    pub severity_mapping: HashMap<String, ViolationSeverity>,
}

/// Compliance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub mandatory: bool,
    pub evidence_required: bool,
    pub remediation_time_days: Option<u32>,
}

/// Compliance rule for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    pub id: String,
    pub standard: String,
    pub requirement_id: String,
    pub name: String,
    pub description: String,
    pub rule_type: ComplianceRuleType,
    pub condition: RuleCondition,
    pub severity: ViolationSeverity,
    pub auto_remediate: bool,
    pub notification_required: bool,
}

/// Types of compliance rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceRuleType {
    AccessControl,
    DataProtection,
    AuditLogging,
    ChangeManagement,
    IncidentResponse,
    BusinessContinuity,
    RiskManagement,
    Custom(String),
}

/// Rule condition for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub event_types: Option<Vec<AuditEventType>>,
    pub time_window_hours: Option<u32>,
    pub frequency_threshold: Option<u32>,
    pub user_pattern: Option<String>,
    pub resource_pattern: Option<String>,
    pub custom_expression: Option<String>,
}

/// Compliance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub id: String,
    pub standard: String,
    pub rule_id: String,
    pub requirement_id: String,
    pub severity: ViolationSeverity,
    pub title: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub related_events: Vec<String>,
    pub affected_users: Vec<String>,
    pub affected_resources: Vec<String>,
    pub status: ViolationStatus,
    pub assigned_to: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub resolution: Option<ViolationResolution>,
    pub risk_score: u32,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Violation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationStatus {
    Open,
    InProgress,
    Resolved,
    Dismissed,
    Escalated,
}

/// Violation resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationResolution {
    pub resolved_at: DateTime<Utc>,
    pub resolved_by: String,
    pub resolution_type: ResolutionType,
    pub description: String,
    pub evidence: Vec<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionType {
    Fixed,
    Mitigated,
    Accepted,
    Transferred,
    FalsePositive,
}

/// Compliance status overview
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub standard: String,
    pub overall_score: f64,
    pub total_requirements: u32,
    pub met_requirements: u32,
    pub violations: ComplianceViolationSummary,
    pub last_assessment: DateTime<Utc>,
    pub next_assessment: DateTime<Utc>,
    pub trend: ComplianceTrend,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceViolationSummary {
    pub total: u32,
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub open: u32,
    pub overdue: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceTrend {
    Improving,
    Stable,
    Declining,
    Unknown,
}

/// Comprehensive compliance report
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: String,
    pub standard: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub executive_summary: ExecutiveSummary,
    pub detailed_findings: Vec<ComplianceFinding>,
    pub violations: Vec<ComplianceViolation>,
    pub recommendations: Vec<ComplianceRecommendation>,
    pub evidence: Vec<EvidenceItem>,
    pub attestation: Option<ComplianceAttestation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub compliance_score: f64,
    pub total_controls: u32,
    pub effective_controls: u32,
    pub critical_findings: u32,
    pub improvement_areas: Vec<String>,
    pub certification_status: CertificationStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificationStatus {
    Compliant,
    NonCompliant,
    PartiallyCompliant,
    UnderReview,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub id: String,
    pub requirement_id: String,
    pub control_id: String,
    pub finding_type: FindingType,
    pub severity: ViolationSeverity,
    pub description: String,
    pub evidence: Vec<String>,
    pub remediation: String,
    pub timeline: u32, // Days to remediate
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingType {
    ControlDeficiency,
    ImplementationGap,
    Documentation,
    Testing,
    Monitoring,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceRecommendation {
    pub priority: u32,
    pub category: String,
    pub description: String,
    pub impact: String,
    pub effort: EffortLevel,
    pub timeline_days: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: String,
    pub requirement_id: String,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub location: String,
    pub collected_at: DateTime<Utc>,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    AuditLog,
    Configuration,
    Policy,
    Procedure,
    TestResult,
    Certification,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceAttestation {
    pub attested_by: String,
    pub attested_at: DateTime<Utc>,
    pub period_covered: (DateTime<Utc>, DateTime<Utc>),
    pub statement: String,
    pub limitations: Option<String>,
}

impl ComplianceManager {
    pub async fn new(standards: Vec<String>, audit_logger: Arc<EnterpriseAuditLogger>) -> Result<Self> {
        let compliance_standards = Self::load_standards(standards).await?;
        let rules = Arc::new(RwLock::new(HashMap::new()));
        let violations = Arc::new(RwLock::new(Vec::new()));

        Ok(Self {
            standards: compliance_standards,
            rules,
            violations,
            audit_logger,
        })
    }

    async fn load_standards(standard_names: Vec<String>) -> Result<Vec<ComplianceStandard>> {
        let mut standards = Vec::new();

        for name in standard_names {
            match name.to_uppercase().as_str() {
                "SOX" => standards.push(Self::create_sox_standard()),
                "GDPR" => standards.push(Self::create_gdpr_standard()),
                "ISO27001" => standards.push(Self::create_iso27001_standard()),
                _ => return Err(anyhow!("Unknown compliance standard: {}", name)),
            }
        }

        Ok(standards)
    }

    fn create_sox_standard() -> ComplianceStandard {
        ComplianceStandard {
            name: "SOX".to_string(),
            version: "2002".to_string(),
            description: "Sarbanes-Oxley Act compliance for financial reporting".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "SOX_302".to_string(),
                    title: "Corporate Responsibility for Financial Reports".to_string(),
                    description: "CEO and CFO certification of financial reports".to_string(),
                    category: "Financial Reporting".to_string(),
                    mandatory: true,
                    evidence_required: true,
                    remediation_time_days: Some(30),
                },
                ComplianceRequirement {
                    id: "SOX_404".to_string(),
                    title: "Management Assessment of Internal Controls".to_string(),
                    description: "Assessment of internal control over financial reporting".to_string(),
                    category: "Internal Controls".to_string(),
                    mandatory: true,
                    evidence_required: true,
                    remediation_time_days: Some(90),
                },
            ],
            severity_mapping: [
                ("financial_access".to_string(), ViolationSeverity::Critical),
                ("audit_log".to_string(), ViolationSeverity::High),
                ("change_control".to_string(), ViolationSeverity::High),
            ].into_iter().collect(),
        }
    }

    fn create_gdpr_standard() -> ComplianceStandard {
        ComplianceStandard {
            name: "GDPR".to_string(),
            version: "2018".to_string(),
            description: "General Data Protection Regulation compliance".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "GDPR_25".to_string(),
                    title: "Data Protection by Design and by Default".to_string(),
                    description: "Implement appropriate technical and organizational measures".to_string(),
                    category: "Data Protection".to_string(),
                    mandatory: true,
                    evidence_required: true,
                    remediation_time_days: Some(72), // 72 hours for breach notification
                },
                ComplianceRequirement {
                    id: "GDPR_30".to_string(),
                    title: "Records of Processing Activities".to_string(),
                    description: "Maintain records of all data processing activities".to_string(),
                    category: "Documentation".to_string(),
                    mandatory: true,
                    evidence_required: true,
                    remediation_time_days: Some(30),
                },
            ],
            severity_mapping: [
                ("data_breach".to_string(), ViolationSeverity::Critical),
                ("data_access".to_string(), ViolationSeverity::High),
                ("consent".to_string(), ViolationSeverity::High),
            ].into_iter().collect(),
        }
    }

    fn create_iso27001_standard() -> ComplianceStandard {
        ComplianceStandard {
            name: "ISO27001".to_string(),
            version: "2013".to_string(),
            description: "Information Security Management System".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "ISO_A9_1_1".to_string(),
                    title: "Access Control Policy".to_string(),
                    description: "Establish and maintain access control policy".to_string(),
                    category: "Access Control".to_string(),
                    mandatory: true,
                    evidence_required: true,
                    remediation_time_days: Some(60),
                },
                ComplianceRequirement {
                    id: "ISO_A12_4_1".to_string(),
                    title: "Event Logging".to_string(),
                    description: "Log events and protect log information".to_string(),
                    category: "Logging and Monitoring".to_string(),
                    mandatory: true,
                    evidence_required: true,
                    remediation_time_days: Some(30),
                },
            ],
            severity_mapping: [
                ("security_incident".to_string(), ViolationSeverity::Critical),
                ("access_violation".to_string(), ViolationSeverity::High),
                ("log_tampering".to_string(), ViolationSeverity::High),
            ].into_iter().collect(),
        }
    }

    pub async fn initialize_monitoring(&self) -> Result<()> {
        // Initialize compliance rules for each standard
        for standard in &self.standards {
            self.load_rules_for_standard(standard).await?;
        }

        // Start monitoring task
        self.start_monitoring_task().await?;

        Ok(())
    }

    async fn load_rules_for_standard(&self, standard: &ComplianceStandard) -> Result<()> {
        let mut rules = self.rules.write().await;
        let standard_rules = self.create_rules_for_standard(standard);
        rules.insert(standard.name.clone(), standard_rules);
        Ok(())
    }

    fn create_rules_for_standard(&self, standard: &ComplianceStandard) -> Vec<ComplianceRule> {
        match standard.name.as_str() {
            "SOX" => vec![
                ComplianceRule {
                    id: "SOX_FINANCIAL_ACCESS".to_string(),
                    standard: "SOX".to_string(),
                    requirement_id: "SOX_404".to_string(),
                    name: "Unauthorized Financial System Access".to_string(),
                    description: "Detect unauthorized access to financial systems".to_string(),
                    rule_type: ComplianceRuleType::AccessControl,
                    condition: RuleCondition {
                        event_types: Some(vec![AuditEventType::UnauthorizedAccess]),
                        time_window_hours: Some(24),
                        frequency_threshold: Some(1),
                        user_pattern: None,
                        resource_pattern: Some("financial:*".to_string()),
                        custom_expression: None,
                    },
                    severity: ViolationSeverity::Critical,
                    auto_remediate: false,
                    notification_required: true,
                },
                ComplianceRule {
                    id: "SOX_AUDIT_LOG_TAMPERING".to_string(),
                    standard: "SOX".to_string(),
                    requirement_id: "SOX_404".to_string(),
                    name: "Audit Log Tampering".to_string(),
                    description: "Detect attempts to tamper with audit logs".to_string(),
                    rule_type: ComplianceRuleType::AuditLogging,
                    condition: RuleCondition {
                        event_types: Some(vec![AuditEventType::AuditLogAccessed]),
                        time_window_hours: Some(1),
                        frequency_threshold: Some(10),
                        user_pattern: None,
                        resource_pattern: Some("audit:*".to_string()),
                        custom_expression: None,
                    },
                    severity: ViolationSeverity::High,
                    auto_remediate: true,
                    notification_required: true,
                },
            ],
            "GDPR" => vec![
                ComplianceRule {
                    id: "GDPR_DATA_BREACH".to_string(),
                    standard: "GDPR".to_string(),
                    requirement_id: "GDPR_25".to_string(),
                    name: "Data Breach Detection".to_string(),
                    description: "Detect potential data breaches".to_string(),
                    rule_type: ComplianceRuleType::DataProtection,
                    condition: RuleCondition {
                        event_types: Some(vec![AuditEventType::DataExport, AuditEventType::UnauthorizedAccess]),
                        time_window_hours: Some(1),
                        frequency_threshold: Some(5),
                        user_pattern: None,
                        resource_pattern: Some("data:personal:*".to_string()),
                        custom_expression: None,
                    },
                    severity: ViolationSeverity::Critical,
                    auto_remediate: false,
                    notification_required: true,
                },
            ],
            "ISO27001" => vec![
                ComplianceRule {
                    id: "ISO_ACCESS_VIOLATION".to_string(),
                    standard: "ISO27001".to_string(),
                    requirement_id: "ISO_A9_1_1".to_string(),
                    name: "Access Control Violation".to_string(),
                    description: "Detect access control policy violations".to_string(),
                    rule_type: ComplianceRuleType::AccessControl,
                    condition: RuleCondition {
                        event_types: Some(vec![AuditEventType::PermissionDenied, AuditEventType::UnauthorizedAccess]),
                        time_window_hours: Some(24),
                        frequency_threshold: Some(3),
                        user_pattern: None,
                        resource_pattern: None,
                        custom_expression: None,
                    },
                    severity: ViolationSeverity::High,
                    auto_remediate: false,
                    notification_required: true,
                },
            ],
            _ => vec![],
        }
    }

    async fn start_monitoring_task(&self) -> Result<()> {
        // In a real implementation, this would start a background task
        // that continuously monitors audit events for compliance violations
        Ok(())
    }

    pub async fn scan_violations(&self) -> Result<Vec<ComplianceViolation>> {
        let violations = self.violations.read().await;
        Ok(violations.clone())
    }

    pub async fn get_violation_count(&self) -> Result<u64> {
        let violations = self.violations.read().await;
        Ok(violations.len() as u64)
    }

    pub async fn generate_report(&self, standard: &str) -> Result<ComplianceReport> {
        let standard_def = self.standards.iter()
            .find(|s| s.name == standard)
            .ok_or_else(|| anyhow!("Standard not found: {}", standard))?;

        let violations = self.get_violations_for_standard(standard).await?;
        let findings = self.generate_findings(standard_def, &violations).await?;
        let recommendations = self.generate_recommendations(standard);

        Ok(ComplianceReport {
            id: uuid::Uuid::new_v4().to_string(),
            standard: standard.to_string(),
            period_start: Utc::now() - Duration::days(30),
            period_end: Utc::now(),
            generated_at: Utc::now(),
            generated_by: "system".to_string(),
            executive_summary: ExecutiveSummary {
                compliance_score: self.calculate_compliance_score(standard, &violations),
                total_controls: standard_def.requirements.len() as u32,
                effective_controls: (standard_def.requirements.len() - violations.len()) as u32,
                critical_findings: violations.iter()
                    .filter(|v| v.severity == ViolationSeverity::Critical)
                    .count() as u32,
                improvement_areas: self.identify_improvement_areas(&violations),
                certification_status: self.determine_certification_status(standard, &violations),
            },
            detailed_findings: findings,
            violations,
            recommendations,
            evidence: vec![], // Would be populated from audit logs
            attestation: None,
        })
    }

    async fn get_violations_for_standard(&self, standard: &str) -> Result<Vec<ComplianceViolation>> {
        let violations = self.violations.read().await;
        Ok(violations.iter()
            .filter(|v| v.standard == standard)
            .cloned()
            .collect())
    }

    async fn generate_findings(&self, _standard: &ComplianceStandard, violations: &[ComplianceViolation]) -> Result<Vec<ComplianceFinding>> {
        let mut findings = Vec::new();

        for violation in violations {
            findings.push(ComplianceFinding {
                id: uuid::Uuid::new_v4().to_string(),
                requirement_id: violation.requirement_id.clone(),
                control_id: violation.rule_id.clone(),
                finding_type: FindingType::ControlDeficiency,
                severity: violation.severity.clone(),
                description: violation.description.clone(),
                evidence: violation.related_events.clone(),
                remediation: self.get_remediation_guidance(&violation.rule_id),
                timeline: 30, // Default 30 days
            });
        }

        Ok(findings)
    }

    fn get_remediation_guidance(&self, rule_id: &str) -> String {
        match rule_id {
            "SOX_FINANCIAL_ACCESS" => "Review and update access controls for financial systems. Implement principle of least privilege.".to_string(),
            "SOX_AUDIT_LOG_TAMPERING" => "Implement tamper-proof audit logging. Review log access permissions.".to_string(),
            "GDPR_DATA_BREACH" => "Implement data loss prevention controls. Review data access patterns.".to_string(),
            "ISO_ACCESS_VIOLATION" => "Review access control policies and implement proper authentication mechanisms.".to_string(),
            _ => "Review control implementation and update as necessary.".to_string(),
        }
    }

    fn calculate_compliance_score(&self, _standard: &str, violations: &[ComplianceViolation]) -> f64 {
        if violations.is_empty() {
            return 100.0;
        }

        let total_risk = violations.iter().map(|v| v.risk_score as f64).sum::<f64>();
        let max_possible_risk = violations.len() as f64 * 100.0;
        
        ((max_possible_risk - total_risk) / max_possible_risk * 100.0).max(0.0)
    }

    fn identify_improvement_areas(&self, violations: &[ComplianceViolation]) -> Vec<String> {
        let mut areas = Vec::new();
        
        if violations.iter().any(|v| matches!(v.rule_id.as_str(), s if s.contains("ACCESS"))) {
            areas.push("Access Control Management".to_string());
        }
        
        if violations.iter().any(|v| matches!(v.rule_id.as_str(), s if s.contains("LOG"))) {
            areas.push("Audit Logging and Monitoring".to_string());
        }
        
        if violations.iter().any(|v| matches!(v.rule_id.as_str(), s if s.contains("DATA"))) {
            areas.push("Data Protection and Privacy".to_string());
        }

        areas
    }

    fn determine_certification_status(&self, _standard: &str, violations: &[ComplianceViolation]) -> CertificationStatus {
        let critical_violations = violations.iter()
            .filter(|v| v.severity == ViolationSeverity::Critical)
            .count();
        
        let high_violations = violations.iter()
            .filter(|v| v.severity == ViolationSeverity::High)
            .count();

        if critical_violations > 0 {
            CertificationStatus::NonCompliant
        } else if high_violations > 3 {
            CertificationStatus::PartiallyCompliant
        } else if violations.is_empty() {
            CertificationStatus::Compliant
        } else {
            CertificationStatus::PartiallyCompliant
        }
    }

    fn generate_recommendations(&self, standard: &str) -> Vec<ComplianceRecommendation> {
        match standard {
            "SOX" => vec![
                ComplianceRecommendation {
                    priority: 1,
                    category: "Access Control".to_string(),
                    description: "Implement role-based access control for financial systems".to_string(),
                    impact: "Reduces risk of unauthorized financial data access".to_string(),
                    effort: EffortLevel::Medium,
                    timeline_days: 60,
                },
                ComplianceRecommendation {
                    priority: 2,
                    category: "Audit Logging".to_string(),
                    description: "Enhance audit logging for all financial transactions".to_string(),
                    impact: "Improves traceability and compliance reporting".to_string(),
                    effort: EffortLevel::Low,
                    timeline_days: 30,
                },
            ],
            "GDPR" => vec![
                ComplianceRecommendation {
                    priority: 1,
                    category: "Data Protection".to_string(),
                    description: "Implement data classification and labeling".to_string(),
                    impact: "Better protection of personal data".to_string(),
                    effort: EffortLevel::High,
                    timeline_days: 90,
                },
            ],
            "ISO27001" => vec![
                ComplianceRecommendation {
                    priority: 1,
                    category: "Security Management".to_string(),
                    description: "Establish formal security governance framework".to_string(),
                    impact: "Systematic approach to information security".to_string(),
                    effort: EffortLevel::VeryHigh,
                    timeline_days: 180,
                },
            ],
            _ => vec![],
        }
    }

    pub async fn get_compliance_status(&self, standard: &str) -> Result<ComplianceStatus> {
        let violations = self.get_violations_for_standard(standard).await?;
        let standard_def = self.standards.iter()
            .find(|s| s.name == standard)
            .ok_or_else(|| anyhow!("Standard not found: {}", standard))?;

        let violation_summary = ComplianceViolationSummary {
            total: violations.len() as u32,
            critical: violations.iter().filter(|v| v.severity == ViolationSeverity::Critical).count() as u32,
            high: violations.iter().filter(|v| v.severity == ViolationSeverity::High).count() as u32,
            medium: violations.iter().filter(|v| v.severity == ViolationSeverity::Medium).count() as u32,
            low: violations.iter().filter(|v| v.severity == ViolationSeverity::Low).count() as u32,
            open: violations.iter().filter(|v| v.status == ViolationStatus::Open).count() as u32,
            overdue: violations.iter()
                .filter(|v| v.due_date.map_or(false, |d| d < Utc::now()))
                .count() as u32,
        };

        Ok(ComplianceStatus {
            standard: standard.to_string(),
            overall_score: self.calculate_compliance_score(standard, &violations),
            total_requirements: standard_def.requirements.len() as u32,
            met_requirements: (standard_def.requirements.len() - violations.len()) as u32,
            violations: violation_summary,
            last_assessment: Utc::now() - Duration::days(7), // Last week
            next_assessment: Utc::now() + Duration::days(23), // Next month
            trend: ComplianceTrend::Stable, // Would be calculated from historical data
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_compliance_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("compliance_test.db");
        
        let audit_logger = Arc::new(
            EnterpriseAuditLogger::new(Some(db_path), 365).await.unwrap()
        );
        
        let standards = vec!["SOX".to_string(), "GDPR".to_string()];
        let manager = ComplianceManager::new(standards, audit_logger).await.unwrap();
        
        assert_eq!(manager.standards.len(), 2);
        assert!(manager.standards.iter().any(|s| s.name == "SOX"));
        assert!(manager.standards.iter().any(|s| s.name == "GDPR"));
    }

    #[tokio::test]
    async fn test_sox_compliance_rules() {
        let sox_standard = ComplianceManager::create_sox_standard();
        assert_eq!(sox_standard.name, "SOX");
        assert_eq!(sox_standard.requirements.len(), 2);
        
        let manager = ComplianceManager {
            standards: vec![sox_standard.clone()],
            rules: Arc::new(RwLock::new(HashMap::new())),
            violations: Arc::new(RwLock::new(Vec::new())),
            audit_logger: Arc::new(
                EnterpriseAuditLogger::new(None, 365).await.unwrap()
            ),
        };
        
        let rules = manager.create_rules_for_standard(&sox_standard);
        assert_eq!(rules.len(), 2);
        assert!(rules.iter().any(|r| r.id == "SOX_FINANCIAL_ACCESS"));
        assert!(rules.iter().any(|r| r.id == "SOX_AUDIT_LOG_TAMPERING"));
    }

    #[tokio::test]
    async fn test_compliance_score_calculation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("score_test.db");
        
        let audit_logger = Arc::new(
            EnterpriseAuditLogger::new(Some(db_path), 365).await.unwrap()
        );
        
        let manager = ComplianceManager::new(vec!["SOX".to_string()], audit_logger).await.unwrap();
        
        // Test with no violations (100% score)
        let score = manager.calculate_compliance_score("SOX", &[]);
        assert_eq!(score, 100.0);
        
        // Test with violations
        let violations = vec![
            ComplianceViolation {
                id: "test_violation".to_string(),
                standard: "SOX".to_string(),
                rule_id: "test_rule".to_string(),
                requirement_id: "test_req".to_string(),
                severity: ViolationSeverity::High,
                title: "Test violation".to_string(),
                description: "Test description".to_string(),
                detected_at: Utc::now(),
                related_events: vec![],
                affected_users: vec![],
                affected_resources: vec![],
                status: ViolationStatus::Open,
                assigned_to: None,
                due_date: None,
                resolution: None,
                risk_score: 80,
            }
        ];
        
        let score = manager.calculate_compliance_score("SOX", &violations);
        assert_eq!(score, 20.0); // 100 - 80 = 20%
    }
}