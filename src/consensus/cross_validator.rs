//! Cross-Validator - Multi-stage consensus verification
//!
//! This module checks if consensus stages agree on basic facts and catches
//! contradictions between different stages before finalizing results.

use crate::consensus::fact_checker::{
    ClaimType, Contradiction, FactChecker, FactClaim, ValidationResult,
};
use crate::consensus::types::{ConsensusResult, Stage};
use crate::consensus::verification::RepositoryFacts;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output from a single consensus stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageOutput {
    pub stage: Stage,
    pub content: String,
    pub confidence: f64,
    pub claims: Vec<FactClaim>,
}

/// Health status of consensus across stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusHealth {
    Healthy,
    Compromised {
        conflicting_stages: Vec<Stage>,
        recommended_action: RecommendedAction,
        discrepancies: Vec<StageDiscrepancy>,
    },
}

/// Discrepancy between stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDiscrepancy {
    pub claim_type: ClaimType,
    pub conflicting_stages: Vec<(Stage, String)>,
    pub severity: DiscrepancySeverity,
    pub explanation: String,
}

/// Severity of stage discrepancy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscrepancySeverity {
    Critical, // Completely contradictory claims
    Major,    // Significantly different interpretations
    Minor,    // Slight variations in details
}

/// Recommended action for compromised consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    RejectAndRerun,          // Start over with enhanced context
    RejectConflictingStages, // Re-run only problematic stages
    ManualReview,            // Human intervention required
    AcceptWithWarning,       // Proceed but flag issues
}

/// Semantic claim extracted from stage output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticClaim {
    pub claim_type: SemanticClaimType,
    pub value: String,
    pub confidence: f64,
    pub evidence: String,
}

/// Types of semantic claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SemanticClaimType {
    DependencyCount,
    Complexity,
    Technology,
    Maturity,
    Version,
    Architecture,
}

/// Semantic contradiction detected between stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContradiction {
    pub contradiction_type: ContradictionType,
    pub severity: ContradictionSeverity,
    pub description: String,
    pub evidence: Vec<String>,
    pub confidence: f64,
}

/// Types of contradictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContradictionType {
    Logical,  // Directly contradictory statements
    Temporal, // Timeline or maturity inconsistencies
    Scale,    // Number relationships that don't make sense
}

/// Severity of semantic contradictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContradictionSeverity {
    Critical, // Fundamental disagreement
    Major,    // Significant inconsistency
    Minor,    // Small discrepancy
}

/// Cross-validator that verifies agreement between consensus stages
pub struct CrossValidator {
    stage_outputs: HashMap<Stage, StageOutput>,
    fact_checker: FactChecker,
    consensus_threshold: f64,
}

impl CrossValidator {
    /// Create a new cross-validator
    pub fn new(repository_facts: RepositoryFacts) -> Self {
        Self {
            stage_outputs: HashMap::new(),
            fact_checker: FactChecker::new(repository_facts),
            consensus_threshold: 0.8, // 80% agreement required for healthy consensus
        }
    }

    /// Add stage output for cross-validation
    pub fn add_stage_output(
        &mut self,
        stage: Stage,
        content: String,
        confidence: f64,
    ) -> Result<()> {
        tracing::debug!("Adding {:?} stage output for cross-validation", stage);

        // Extract claims from the content
        let validation_result = self.fact_checker.validate_stage_output(stage, &content)?;

        let claims = match validation_result {
            ValidationResult::Passed {
                verified_claims, ..
            } => verified_claims,
            ValidationResult::Failed { contradictions, .. } => {
                // Extract claims from contradictions
                contradictions.into_iter().map(|c| c.claim).collect()
            }
        };

        let stage_output = StageOutput {
            stage,
            content,
            confidence,
            claims,
        };

        self.stage_outputs.insert(stage, stage_output);
        Ok(())
    }

    /// Check if stages agree on basic facts
    pub fn verify_stage_consensus(&self) -> Result<ConsensusHealth> {
        tracing::debug!(
            "Verifying consensus across {} stages",
            self.stage_outputs.len()
        );

        if self.stage_outputs.len() < 2 {
            return Ok(ConsensusHealth::Healthy); // Can't cross-validate with less than 2 stages
        }

        let discrepancies = self.find_all_discrepancies()?;

        if discrepancies.is_empty() {
            Ok(ConsensusHealth::Healthy)
        } else {
            let conflicting_stages = self.identify_outliers(&discrepancies);
            let recommended_action = self.determine_action(&discrepancies);

            Ok(ConsensusHealth::Compromised {
                conflicting_stages,
                recommended_action,
                discrepancies,
            })
        }
    }

    /// Find all discrepancies between stages
    fn find_all_discrepancies(&self) -> Result<Vec<StageDiscrepancy>> {
        let mut discrepancies = Vec::new();

        // Check each claim type for consistency across stages
        for claim_type in [
            ClaimType::ProjectName,
            ClaimType::Version,
            ClaimType::DependencyCount,
            ClaimType::ModuleCount,
            ClaimType::ProjectComplexity,
        ] {
            if let Some(discrepancy) = self.check_claim_consistency(claim_type)? {
                discrepancies.push(discrepancy);
            }
        }

        Ok(discrepancies)
    }

    /// Check consistency of a specific claim type across stages
    fn check_claim_consistency(&self, claim_type: ClaimType) -> Result<Option<StageDiscrepancy>> {
        let mut stage_claims: Vec<(Stage, String)> = Vec::new();

        // Collect claims of this type from all stages
        for (stage, output) in &self.stage_outputs {
            for claim in &output.claims {
                if claim.claim_type == claim_type {
                    stage_claims.push((*stage, claim.value.clone()));
                }
            }
        }

        if stage_claims.len() < 2 {
            return Ok(None); // Not enough claims to compare
        }

        // Check for contradictions
        let unique_values: std::collections::HashSet<_> = stage_claims
            .iter()
            .map(|(_, value)| value.clone())
            .collect();

        if unique_values.len() > 1 {
            // Found contradictions
            let severity = self.assess_discrepancy_severity(&claim_type, &stage_claims);
            let explanation = self.generate_discrepancy_explanation(&claim_type, &stage_claims);

            Ok(Some(StageDiscrepancy {
                claim_type,
                conflicting_stages: stage_claims,
                severity,
                explanation,
            }))
        } else {
            Ok(None) // All stages agree
        }
    }

    /// Assess severity of a discrepancy
    fn assess_discrepancy_severity(
        &self,
        claim_type: &ClaimType,
        stage_claims: &[(Stage, String)],
    ) -> DiscrepancySeverity {
        match claim_type {
            ClaimType::ProjectName | ClaimType::ProjectComplexity => {
                // These should never disagree - critical if they do
                DiscrepancySeverity::Critical
            }
            ClaimType::Version => {
                // Version mismatches are major issues
                DiscrepancySeverity::Major
            }
            ClaimType::DependencyCount | ClaimType::ModuleCount => {
                // Check if numbers are within reasonable tolerance
                let numbers: Vec<usize> = stage_claims
                    .iter()
                    .filter_map(|(_, value)| value.parse().ok())
                    .collect();

                if numbers.len() >= 2 {
                    let min = *numbers.iter().min().unwrap();
                    let max = *numbers.iter().max().unwrap();
                    let tolerance = (min as f64 * 0.3) as usize; // 30% tolerance

                    if max.saturating_sub(min) > tolerance {
                        DiscrepancySeverity::Major
                    } else {
                        DiscrepancySeverity::Minor
                    }
                } else {
                    DiscrepancySeverity::Major
                }
            }
            _ => DiscrepancySeverity::Minor,
        }
    }

    /// Generate explanation for discrepancy
    fn generate_discrepancy_explanation(
        &self,
        claim_type: &ClaimType,
        stage_claims: &[(Stage, String)],
    ) -> String {
        let claims_text = stage_claims
            .iter()
            .map(|(stage, value)| format!("{:?}: '{}'", stage, value))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "Consensus stages disagree on {:?}. Found conflicting claims: {}",
            claim_type, claims_text
        )
    }

    /// Identify which stages are outliers
    fn identify_outliers(&self, discrepancies: &[StageDiscrepancy]) -> Vec<Stage> {
        let mut stage_error_counts: HashMap<Stage, usize> = HashMap::new();

        // Count how many discrepancies each stage is involved in
        for discrepancy in discrepancies {
            for (stage, _) in &discrepancy.conflicting_stages {
                *stage_error_counts.entry(*stage).or_insert(0) += 1;
            }
        }

        // Identify stages with above-average error counts
        if stage_error_counts.is_empty() {
            return Vec::new();
        }

        let total_errors: usize = stage_error_counts.values().sum();
        let average_errors = total_errors as f64 / stage_error_counts.len() as f64;

        stage_error_counts
            .into_iter()
            .filter(|(_, count)| *count as f64 > average_errors)
            .map(|(stage, _)| stage)
            .collect()
    }

    /// Determine recommended action based on discrepancies
    fn determine_action(&self, discrepancies: &[StageDiscrepancy]) -> RecommendedAction {
        let critical_count = discrepancies
            .iter()
            .filter(|d| matches!(d.severity, DiscrepancySeverity::Critical))
            .count();

        let major_count = discrepancies
            .iter()
            .filter(|d| matches!(d.severity, DiscrepancySeverity::Major))
            .count();

        if critical_count > 0 {
            RecommendedAction::RejectAndRerun
        } else if major_count > 2 {
            RecommendedAction::RejectConflictingStages
        } else if major_count > 0 {
            RecommendedAction::ManualReview
        } else {
            RecommendedAction::AcceptWithWarning
        }
    }

    /// Get detailed consensus report
    pub fn get_consensus_report(&self) -> Result<ConsensusReport> {
        let health = self.verify_stage_consensus()?;
        let stage_count = self.stage_outputs.len();
        let avg_confidence = if stage_count > 0 {
            self.stage_outputs
                .values()
                .map(|output| output.confidence)
                .sum::<f64>()
                / stage_count as f64
        } else {
            0.0
        };

        Ok(ConsensusReport {
            health,
            stage_count,
            average_confidence: avg_confidence,
            agreement_score: self.calculate_agreement_score()?,
        })
    }

    /// Calculate overall agreement score
    fn calculate_agreement_score(&self) -> Result<f64> {
        let discrepancies = self.find_all_discrepancies()?;

        if discrepancies.is_empty() {
            return Ok(1.0); // Perfect agreement
        }

        // Calculate score based on severity and count of discrepancies
        let total_severity_score: f64 = discrepancies
            .iter()
            .map(|d| match d.severity {
                DiscrepancySeverity::Critical => 1.0,
                DiscrepancySeverity::Major => 0.7,
                DiscrepancySeverity::Minor => 0.3,
            })
            .sum();

        let max_possible_score = discrepancies.len() as f64;
        let agreement_score = 1.0 - (total_severity_score / max_possible_score);

        Ok(agreement_score.max(0.0))
    }

    /// Advanced contradiction detection using semantic analysis
    pub fn detect_semantic_contradictions(&self) -> Result<Vec<SemanticContradiction>> {
        let mut contradictions = Vec::new();

        // Extract semantic claims from each stage
        let stage_claims = self.extract_semantic_claims()?;

        // Check for logical contradictions
        for logical_contradiction in self.find_logical_contradictions(&stage_claims)? {
            contradictions.push(logical_contradiction);
        }

        // Check for temporal contradictions
        for temporal_contradiction in self.find_temporal_contradictions(&stage_claims)? {
            contradictions.push(temporal_contradiction);
        }

        // Check for scale contradictions (numbers that don't make sense together)
        for scale_contradiction in self.find_scale_contradictions(&stage_claims)? {
            contradictions.push(scale_contradiction);
        }

        Ok(contradictions)
    }

    /// Extract semantic claims from stage outputs
    fn extract_semantic_claims(&self) -> Result<HashMap<Stage, Vec<SemanticClaim>>> {
        let mut claims_map = HashMap::new();

        for (stage, output) in &self.stage_outputs {
            let mut claims = Vec::new();

            // Extract explicit claims using patterns
            claims.extend(self.extract_explicit_claims(&output.content)?);

            // Extract implicit claims using inference
            claims.extend(self.extract_implicit_claims(&output.content)?);

            claims_map.insert(*stage, claims);
        }

        Ok(claims_map)
    }

    /// Extract explicit claims like "This project has X dependencies"
    fn extract_explicit_claims(&self, content: &str) -> Result<Vec<SemanticClaim>> {
        let mut claims = Vec::new();

        // Pattern for explicit dependency claims
        if let Some(captures) =
            regex::Regex::new(r"(?i)(?:has|contains|includes)\s+(\d+)\s+dependencies")
                .unwrap()
                .captures(content)
        {
            if let Ok(count) = captures[1].parse::<usize>() {
                claims.push(SemanticClaim {
                    claim_type: SemanticClaimType::DependencyCount,
                    value: count.to_string(),
                    confidence: 0.9,
                    evidence: captures[0].to_string(),
                });
            }
        }

        // Pattern for complexity claims
        if content.to_lowercase().contains("enterprise")
            && content.to_lowercase().contains("complex")
        {
            claims.push(SemanticClaim {
                claim_type: SemanticClaimType::Complexity,
                value: "high".to_string(),
                confidence: 0.8,
                evidence: "Contains 'enterprise' and 'complex' keywords".to_string(),
            });
        } else if content.to_lowercase().contains("simple")
            || content.to_lowercase().contains("minimal")
        {
            claims.push(SemanticClaim {
                claim_type: SemanticClaimType::Complexity,
                value: "low".to_string(),
                confidence: 0.8,
                evidence: "Contains 'simple' or 'minimal' keywords".to_string(),
            });
        }

        // Pattern for technology stack claims
        if content.to_lowercase().contains("rust") {
            claims.push(SemanticClaim {
                claim_type: SemanticClaimType::Technology,
                value: "rust".to_string(),
                confidence: 0.95,
                evidence: "Mentions Rust programming language".to_string(),
            });
        }

        Ok(claims)
    }

    /// Extract implicit claims through inference
    fn extract_implicit_claims(&self, content: &str) -> Result<Vec<SemanticClaim>> {
        let mut claims = Vec::new();

        // Infer complexity from description length and technical terms
        let technical_terms = [
            "async",
            "parallel",
            "concurrent",
            "distributed",
            "microservice",
            "architecture",
        ];
        let tech_term_count = technical_terms
            .iter()
            .filter(|&term| content.to_lowercase().contains(term))
            .count();

        if tech_term_count >= 3 {
            claims.push(SemanticClaim {
                claim_type: SemanticClaimType::Complexity,
                value: "high".to_string(),
                confidence: 0.6,
                evidence: format!(
                    "Contains {} technical terms indicating complexity",
                    tech_term_count
                ),
            });
        }

        // Infer maturity from version patterns
        if let Some(captures) = regex::Regex::new(r"v?(\d+)\.(\d+)\.(\d+)")
            .unwrap()
            .captures(content)
        {
            if let (Ok(major), Ok(minor), Ok(patch)) = (
                captures[1].parse::<usize>(),
                captures[2].parse::<usize>(),
                captures[3].parse::<usize>(),
            ) {
                let maturity = if major >= 2 || (major >= 1 && minor >= 5) {
                    "mature"
                } else if major >= 1 || (major == 0 && minor >= 5) {
                    "stable"
                } else {
                    "early"
                };

                claims.push(SemanticClaim {
                    claim_type: SemanticClaimType::Maturity,
                    value: maturity.to_string(),
                    confidence: 0.7,
                    evidence: format!(
                        "Version {}.{}.{} indicates {} maturity",
                        major, minor, patch, maturity
                    ),
                });
            }
        }

        Ok(claims)
    }

    /// Find logical contradictions between claims
    fn find_logical_contradictions(
        &self,
        stage_claims: &HashMap<Stage, Vec<SemanticClaim>>,
    ) -> Result<Vec<SemanticContradiction>> {
        let mut contradictions = Vec::new();

        // Check for complexity contradictions
        let complexity_claims: Vec<(Stage, &SemanticClaim)> = stage_claims
            .iter()
            .flat_map(|(stage, claims)| {
                claims
                    .iter()
                    .filter(|claim| matches!(claim.claim_type, SemanticClaimType::Complexity))
                    .map(move |claim| (*stage, claim))
            })
            .collect();

        if complexity_claims.len() >= 2 {
            for i in 0..complexity_claims.len() {
                for j in i + 1..complexity_claims.len() {
                    let (stage1, claim1) = &complexity_claims[i];
                    let (stage2, claim2) = &complexity_claims[j];

                    if claim1.value != claim2.value {
                        let severity = if (claim1.value == "high" && claim2.value == "low")
                            || (claim1.value == "low" && claim2.value == "high")
                        {
                            ContradictionSeverity::Critical
                        } else {
                            ContradictionSeverity::Major
                        };

                        contradictions.push(SemanticContradiction {
                            contradiction_type: ContradictionType::Logical,
                            severity,
                            description: format!(
                                "Complexity contradiction: {:?} claims '{}' but {:?} claims '{}'",
                                stage1, claim1.value, stage2, claim2.value
                            ),
                            evidence: vec![claim1.evidence.clone(), claim2.evidence.clone()],
                            confidence: (claim1.confidence + claim2.confidence) / 2.0,
                        });
                    }
                }
            }
        }

        Ok(contradictions)
    }

    /// Find temporal contradictions (timeline issues)
    fn find_temporal_contradictions(
        &self,
        stage_claims: &HashMap<Stage, Vec<SemanticClaim>>,
    ) -> Result<Vec<SemanticContradiction>> {
        let mut contradictions = Vec::new();

        // Check for maturity vs version contradictions
        let maturity_claims: Vec<(Stage, &SemanticClaim)> = stage_claims
            .iter()
            .flat_map(|(stage, claims)| {
                claims
                    .iter()
                    .filter(|claim| matches!(claim.claim_type, SemanticClaimType::Maturity))
                    .map(move |claim| (*stage, claim))
            })
            .collect();

        let version_claims: Vec<(Stage, &SemanticClaim)> = stage_claims
            .iter()
            .flat_map(|(stage, claims)| {
                claims
                    .iter()
                    .filter(|claim| matches!(claim.claim_type, SemanticClaimType::Version))
                    .map(move |claim| (*stage, claim))
            })
            .collect();

        // Check if maturity and version claims are consistent
        for (mat_stage, mat_claim) in &maturity_claims {
            for (ver_stage, ver_claim) in &version_claims {
                if let Ok(version) = semver::Version::parse(&ver_claim.value) {
                    let expected_maturity = if version.major >= 2 {
                        "mature"
                    } else if version.major >= 1 {
                        "stable"
                    } else {
                        "early"
                    };

                    if mat_claim.value != expected_maturity {
                        contradictions.push(SemanticContradiction {
                            contradiction_type: ContradictionType::Temporal,
                            severity: ContradictionSeverity::Major,
                            description: format!(
                                "Temporal contradiction: {:?} claims maturity '{}' but version {} from {:?} suggests '{}'",
                                mat_stage, mat_claim.value, version, ver_stage, expected_maturity
                            ),
                            evidence: vec![mat_claim.evidence.clone(), ver_claim.evidence.clone()],
                            confidence: (mat_claim.confidence + ver_claim.confidence) / 2.0,
                        });
                    }
                }
            }
        }

        Ok(contradictions)
    }

    /// Find scale contradictions (numbers that don't make sense together)
    fn find_scale_contradictions(
        &self,
        stage_claims: &HashMap<Stage, Vec<SemanticClaim>>,
    ) -> Result<Vec<SemanticContradiction>> {
        let mut contradictions = Vec::new();

        // Check dependency count vs complexity claims
        let dep_claims: Vec<(Stage, &SemanticClaim)> = stage_claims
            .iter()
            .flat_map(|(stage, claims)| {
                claims
                    .iter()
                    .filter(|claim| matches!(claim.claim_type, SemanticClaimType::DependencyCount))
                    .map(move |claim| (*stage, claim))
            })
            .collect();

        let complexity_claims: Vec<(Stage, &SemanticClaim)> = stage_claims
            .iter()
            .flat_map(|(stage, claims)| {
                claims
                    .iter()
                    .filter(|claim| matches!(claim.claim_type, SemanticClaimType::Complexity))
                    .map(move |claim| (*stage, claim))
            })
            .collect();

        for (dep_stage, dep_claim) in &dep_claims {
            if let Ok(dep_count) = dep_claim.value.parse::<usize>() {
                for (comp_stage, comp_claim) in &complexity_claims {
                    let expected_complexity = if dep_count > 50 {
                        "high"
                    } else if dep_count > 20 {
                        "medium"
                    } else {
                        "low"
                    };

                    if comp_claim.value != expected_complexity
                        && !((comp_claim.value == "medium" && expected_complexity == "high")
                            || (comp_claim.value == "high" && expected_complexity == "medium"))
                    {
                        contradictions.push(SemanticContradiction {
                            contradiction_type: ContradictionType::Scale,
                            severity: ContradictionSeverity::Major,
                            description: format!(
                                "Scale contradiction: {} dependencies from {:?} suggests {} complexity, but {:?} claims '{}'",
                                dep_count, dep_stage, expected_complexity, comp_stage, comp_claim.value
                            ),
                            evidence: vec![dep_claim.evidence.clone(), comp_claim.evidence.clone()],
                            confidence: (dep_claim.confidence + comp_claim.confidence) / 2.0,
                        });
                    }
                }
            }
        }

        Ok(contradictions)
    }
}

/// Comprehensive consensus report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusReport {
    pub health: ConsensusHealth,
    pub stage_count: usize,
    pub average_confidence: f64,
    pub agreement_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_facts() -> RepositoryFacts {
        RepositoryFacts {
            name: "hive-ai".to_string(),
            version: "2.0.2".to_string(),
            dependency_count: 100,
            module_count: 25,
            total_files: 150,
            lines_of_code: 10000,
            is_enterprise: true,
            verified_at: Utc::now(),
            root_path: PathBuf::from("/test"),
            file_extensions: vec!["rs".to_string()],
            major_directories: vec!["src".to_string()],
        }
    }

    #[tokio::test]
    async fn test_healthy_consensus() {
        let facts = create_test_facts();
        let mut validator = CrossValidator::new(facts);

        // Add consistent stage outputs
        validator
            .add_stage_output(
                Stage::Generator,
                "This is the hive-ai project version 2.0.2 with 100 dependencies.".to_string(),
                0.9,
            )
            .unwrap();

        validator
            .add_stage_output(
                Stage::Validator,
                "The hive-ai project (version 2.0.2) has 100 dependencies and is enterprise-grade."
                    .to_string(),
                0.85,
            )
            .unwrap();

        let health = validator.verify_stage_consensus().unwrap();
        assert!(matches!(health, ConsensusHealth::Healthy));
    }

    #[tokio::test]
    async fn test_compromised_consensus() {
        let facts = create_test_facts();
        let mut validator = CrossValidator::new(facts);

        // Add contradictory stage outputs
        validator
            .add_stage_output(
                Stage::Generator,
                "This is the hive-ai project version 2.0.2 with 100 dependencies.".to_string(),
                0.9,
            )
            .unwrap();

        validator
            .add_stage_output(
                Stage::Validator,
                "This is a minimal project with version 0.1.0 and no dependencies.".to_string(),
                0.8,
            )
            .unwrap();

        let health = validator.verify_stage_consensus().unwrap();
        match health {
            ConsensusHealth::Compromised { discrepancies, .. } => {
                assert!(!discrepancies.is_empty());
            }
            ConsensusHealth::Healthy => {
                panic!("Expected compromised consensus for contradictory outputs");
            }
        }
    }

    #[test]
    fn test_agreement_score_calculation() {
        let facts = create_test_facts();
        let validator = CrossValidator::new(facts);

        // Test perfect agreement
        let score = validator.calculate_agreement_score().unwrap();
        assert_eq!(score, 1.0);
    }
}
