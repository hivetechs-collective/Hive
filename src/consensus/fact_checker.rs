//! Fact Checker - Cross-validation between consensus stages
//!
//! This module validates stage outputs against verified repository facts,
//! detecting contradictions and hallucinations before they propagate.

use crate::consensus::types::Stage;
use crate::consensus::verification::RepositoryFacts;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A claim extracted from stage output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactClaim {
    /// Type of claim (version, dependency_count, file_count, etc.)
    pub claim_type: ClaimType,

    /// The claimed value
    pub value: String,

    /// Confidence in the extraction (0.0-1.0)
    pub confidence: f64,

    /// Source text where claim was found
    pub source_text: String,
}

/// Types of factual claims we can extract and verify
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ClaimType {
    ProjectName,
    Version,
    DependencyCount,
    ModuleCount,
    FileCount,
    LinesOfCode,
    ProjectComplexity,
    FileExtension,
    Directory,
}

/// A contradiction between claimed and verified facts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub claim: FactClaim,
    pub verified_value: String,
    pub severity: ContradictionSeverity,
    pub explanation: String,
}

/// Severity of contradiction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContradictionSeverity {
    Critical, // Completely wrong information
    Major,    // Significantly incorrect
    Minor,    // Slightly off or outdated
}

/// Result of fact checking validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    Passed {
        stage: Stage,
        confidence: f64,
        verified_claims: Vec<FactClaim>,
    },
    Failed {
        stage: Stage,
        contradictions: Vec<Contradiction>,
        confidence: f64,
        recommended_action: RecommendedAction,
    },
}

/// Recommended action when validation fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    RejectAndRetry,
    RetryWithEnhancedContext,
    ManualReview,
    Accept,
}

/// Fact checker that validates stage outputs against repository reality
pub struct FactChecker {
    repository_facts: RepositoryFacts,
    tolerance_threshold: f64,
    claim_extractors: HashMap<ClaimType, Regex>,
}

impl FactChecker {
    /// Create a new fact checker
    pub fn new(repository_facts: RepositoryFacts) -> Self {
        let mut claim_extractors = HashMap::new();

        // Build regex patterns for extracting different types of claims
        claim_extractors.insert(
            ClaimType::ProjectName,
            Regex::new(r"(?i)(?:project|package|crate)(?:\s+name)?\s+(?:is\s+)?([a-zA-Z0-9_-]+)")
                .unwrap(),
        );

        claim_extractors.insert(
            ClaimType::Version,
            Regex::new(r"(?i)version\s+(?:is\s+)?(\d+\.\d+\.\d+(?:-[a-zA-Z0-9]+)?)").unwrap(),
        );

        claim_extractors.insert(
            ClaimType::DependencyCount,
            Regex::new(r"(?i)(\d+)\s+(?:external\s+)?dependencies").unwrap(),
        );

        claim_extractors.insert(
            ClaimType::ModuleCount,
            Regex::new(r"(?i)(\d+)\s+(?:rust\s+)?modules").unwrap(),
        );

        claim_extractors.insert(
            ClaimType::FileCount,
            Regex::new(r"(?i)(\d+)\s+(?:total\s+)?files").unwrap(),
        );

        claim_extractors.insert(
            ClaimType::ProjectComplexity,
            Regex::new(r"(?i)(?:this is|it's|the project is)\s+(?:a|an)?\s*(minimal|simple|basic|enterprise|complex|large)").unwrap()
        );

        Self {
            repository_facts,
            tolerance_threshold: 0.2, // 20% tolerance for numeric values
            claim_extractors,
        }
    }

    /// Validate stage output against repository facts
    pub fn validate_stage_output(&self, stage: Stage, output: &str) -> Result<ValidationResult> {
        tracing::debug!("Fact-checking {:?} stage output", stage);

        let extracted_claims = self.extract_claims(output)?;
        let contradictions = self.find_contradictions(&extracted_claims)?;

        if contradictions.is_empty() {
            Ok(ValidationResult::Passed {
                stage,
                confidence: self.calculate_confidence(&extracted_claims),
                verified_claims: extracted_claims,
            })
        } else {
            let severity_score = self.calculate_severity_score(&contradictions);
            let recommended_action = self.determine_action(&contradictions, severity_score);

            Ok(ValidationResult::Failed {
                stage,
                contradictions,
                confidence: self.calculate_accuracy(&extracted_claims),
                recommended_action,
            })
        }
    }

    /// Extract factual claims from stage output
    fn extract_claims(&self, output: &str) -> Result<Vec<FactClaim>> {
        let mut claims = Vec::new();

        for (claim_type, regex) in &self.claim_extractors {
            for capture in regex.captures_iter(output) {
                if let Some(value_match) = capture.get(1) {
                    let value = value_match.as_str().to_string();
                    let source_text = capture.get(0).unwrap().as_str().to_string();

                    claims.push(FactClaim {
                        claim_type: claim_type.clone(),
                        value,
                        confidence: 0.8, // Default confidence
                        source_text,
                    });
                }
            }
        }

        tracing::debug!("Extracted {} claims from output", claims.len());
        Ok(claims)
    }

    /// Find contradictions between claims and verified facts
    fn find_contradictions(&self, claims: &[FactClaim]) -> Result<Vec<Contradiction>> {
        let mut contradictions = Vec::new();

        for claim in claims {
            if let Some(contradiction) = self.check_claim_against_facts(claim)? {
                contradictions.push(contradiction);
            }
        }

        Ok(contradictions)
    }

    /// Check a single claim against verified facts
    fn check_claim_against_facts(&self, claim: &FactClaim) -> Result<Option<Contradiction>> {
        match claim.claim_type {
            ClaimType::ProjectName => {
                if claim.value.to_lowercase() != self.repository_facts.name.to_lowercase() {
                    return Ok(Some(Contradiction {
                        claim: claim.clone(),
                        verified_value: self.repository_facts.name.clone(),
                        severity: ContradictionSeverity::Critical,
                        explanation: format!(
                            "Claimed project name '{}' does not match verified name '{}'",
                            claim.value, self.repository_facts.name
                        ),
                    }));
                }
            }

            ClaimType::Version => {
                if claim.value != self.repository_facts.version {
                    return Ok(Some(Contradiction {
                        claim: claim.clone(),
                        verified_value: self.repository_facts.version.clone(),
                        severity: ContradictionSeverity::Major,
                        explanation: format!(
                            "Claimed version '{}' does not match verified version '{}'",
                            claim.value, self.repository_facts.version
                        ),
                    }));
                }
            }

            ClaimType::DependencyCount => {
                if let Ok(claimed_count) = claim.value.parse::<usize>() {
                    let actual_count = self.repository_facts.dependency_count;
                    let tolerance = (actual_count as f64 * self.tolerance_threshold) as usize;

                    if claimed_count.abs_diff(actual_count) > tolerance {
                        return Ok(Some(Contradiction {
                            claim: claim.clone(),
                            verified_value: actual_count.to_string(),
                            severity: if claimed_count.abs_diff(actual_count) > actual_count / 2 {
                                ContradictionSeverity::Critical
                            } else {
                                ContradictionSeverity::Major
                            },
                            explanation: format!(
                                "Claimed {} dependencies, but verified count is {} (difference: {})",
                                claimed_count, actual_count, claimed_count.abs_diff(actual_count)
                            ),
                        }));
                    }
                }
            }

            ClaimType::ModuleCount => {
                if let Ok(claimed_count) = claim.value.parse::<usize>() {
                    let actual_count = self.repository_facts.module_count;
                    let tolerance = (actual_count as f64 * self.tolerance_threshold) as usize;

                    if claimed_count.abs_diff(actual_count) > tolerance {
                        return Ok(Some(Contradiction {
                            claim: claim.clone(),
                            verified_value: actual_count.to_string(),
                            severity: ContradictionSeverity::Major,
                            explanation: format!(
                                "Claimed {} modules, but verified count is {}",
                                claimed_count, actual_count
                            ),
                        }));
                    }
                }
            }

            ClaimType::ProjectComplexity => {
                let claimed_is_enterprise = matches!(
                    claim.value.to_lowercase().as_str(),
                    "enterprise" | "complex" | "large"
                );

                if claimed_is_enterprise != self.repository_facts.is_enterprise {
                    return Ok(Some(Contradiction {
                        claim: claim.clone(),
                        verified_value: if self.repository_facts.is_enterprise {
                            "enterprise".to_string()
                        } else {
                            "simple".to_string()
                        },
                        severity: ContradictionSeverity::Critical,
                        explanation: format!(
                            "Claimed project is '{}' but verification shows it's {}",
                            claim.value,
                            if self.repository_facts.is_enterprise {
                                "enterprise-grade"
                            } else {
                                "simple"
                            }
                        ),
                    }));
                }
            }

            _ => {
                // For other claim types, no specific validation yet
            }
        }

        Ok(None)
    }

    /// Calculate confidence score based on verified claims
    fn calculate_confidence(&self, claims: &[FactClaim]) -> f64 {
        if claims.is_empty() {
            return 0.5;
        }

        let total_confidence: f64 = claims.iter().map(|c| c.confidence).sum();
        total_confidence / claims.len() as f64
    }

    /// Calculate accuracy based on contradiction rate
    fn calculate_accuracy(&self, claims: &[FactClaim]) -> f64 {
        if claims.is_empty() {
            return 0.0;
        }

        let contradictions = self.find_contradictions(claims).unwrap_or_default();
        let accuracy = 1.0 - (contradictions.len() as f64 / claims.len() as f64);
        accuracy.max(0.0)
    }

    /// Calculate overall severity score
    fn calculate_severity_score(&self, contradictions: &[Contradiction]) -> f64 {
        if contradictions.is_empty() {
            return 0.0;
        }

        let total_score: f64 = contradictions
            .iter()
            .map(|c| match c.severity {
                ContradictionSeverity::Critical => 1.0,
                ContradictionSeverity::Major => 0.7,
                ContradictionSeverity::Minor => 0.3,
            })
            .sum();

        total_score / contradictions.len() as f64
    }

    /// Determine recommended action based on contradictions
    fn determine_action(
        &self,
        contradictions: &[Contradiction],
        severity_score: f64,
    ) -> RecommendedAction {
        let critical_count = contradictions
            .iter()
            .filter(|c| matches!(c.severity, ContradictionSeverity::Critical))
            .count();

        if critical_count > 0 {
            RecommendedAction::RejectAndRetry
        } else if severity_score > 0.7 {
            RecommendedAction::RetryWithEnhancedContext
        } else if severity_score > 0.4 {
            RecommendedAction::ManualReview
        } else {
            RecommendedAction::Accept
        }
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
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

    #[test]
    fn test_correct_claims_pass_validation() {
        let facts = create_test_facts();
        let checker = FactChecker::new(facts);

        let output =
            "This is the hive-ai project version 2.0.2 with 100 dependencies and 25 modules. \
                      It's an enterprise-grade application with 150 files.";

        let result = checker
            .validate_stage_output(Stage::Generator, output)
            .unwrap();

        match result {
            ValidationResult::Passed { confidence, .. } => {
                assert!(confidence > 0.5);
            }
            ValidationResult::Failed { .. } => {
                panic!("Validation should have passed for correct claims");
            }
        }
    }

    #[test]
    fn test_incorrect_claims_fail_validation() {
        let facts = create_test_facts();
        let checker = FactChecker::new(facts);

        let output =
            "This is a minimal project called simple-app version 0.1.0 with no dependencies.";

        let result = checker
            .validate_stage_output(Stage::Validator, output)
            .unwrap();

        match result {
            ValidationResult::Failed { contradictions, .. } => {
                assert!(!contradictions.is_empty());
                assert!(contradictions
                    .iter()
                    .any(|c| c.claim.claim_type == ClaimType::ProjectName));
            }
            ValidationResult::Passed { .. } => {
                panic!("Validation should have failed for incorrect claims");
            }
        }
    }

    #[test]
    fn test_claim_extraction() {
        let facts = create_test_facts();
        let checker = FactChecker::new(facts);

        let output = "The project hive-ai version 2.0.2 has 100 dependencies and 25 modules.";
        let claims = checker.extract_claims(output).unwrap();

        assert!(!claims.is_empty());
        assert!(claims
            .iter()
            .any(|c| c.claim_type == ClaimType::ProjectName));
        assert!(claims.iter().any(|c| c.claim_type == ClaimType::Version));
        assert!(claims
            .iter()
            .any(|c| c.claim_type == ClaimType::DependencyCount));
    }
}
