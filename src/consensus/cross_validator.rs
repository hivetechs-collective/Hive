//! Cross-Validator - Multi-stage consensus verification
//!
//! This module checks if consensus stages agree on basic facts and catches
//! contradictions between different stages before finalizing results.

use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use crate::consensus::types::{Stage, ConsensusResult};
use crate::consensus::verification::RepositoryFacts;
use crate::consensus::fact_checker::{FactChecker, FactClaim, ClaimType, ValidationResult, Contradiction};

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
    Critical,  // Completely contradictory claims
    Major,     // Significantly different interpretations
    Minor,     // Slight variations in details
}

/// Recommended action for compromised consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    RejectAndRerun,           // Start over with enhanced context
    RejectConflictingStages,  // Re-run only problematic stages
    ManualReview,             // Human intervention required
    AcceptWithWarning,        // Proceed but flag issues
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
    pub fn add_stage_output(&mut self, stage: Stage, content: String, confidence: f64) -> Result<()> {
        tracing::debug!("Adding {:?} stage output for cross-validation", stage);
        
        // Extract claims from the content
        let validation_result = self.fact_checker.validate_stage_output(stage, &content)?;
        
        let claims = match validation_result {
            ValidationResult::Passed { verified_claims, .. } => verified_claims,
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
        tracing::debug!("Verifying consensus across {} stages", self.stage_outputs.len());
        
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
        let unique_values: std::collections::HashSet<_> = stage_claims.iter()
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
            },
            ClaimType::Version => {
                // Version mismatches are major issues
                DiscrepancySeverity::Major
            },
            ClaimType::DependencyCount | ClaimType::ModuleCount => {
                // Check if numbers are within reasonable tolerance
                let numbers: Vec<usize> = stage_claims.iter()
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
            },
            _ => DiscrepancySeverity::Minor,
        }
    }
    
    /// Generate explanation for discrepancy
    fn generate_discrepancy_explanation(
        &self,
        claim_type: &ClaimType,
        stage_claims: &[(Stage, String)],
    ) -> String {
        let claims_text = stage_claims.iter()
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
        
        stage_error_counts.into_iter()
            .filter(|(_, count)| *count as f64 > average_errors)
            .map(|(stage, _)| stage)
            .collect()
    }
    
    /// Determine recommended action based on discrepancies
    fn determine_action(&self, discrepancies: &[StageDiscrepancy]) -> RecommendedAction {
        let critical_count = discrepancies.iter()
            .filter(|d| matches!(d.severity, DiscrepancySeverity::Critical))
            .count();
            
        let major_count = discrepancies.iter()
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
            self.stage_outputs.values()
                .map(|output| output.confidence)
                .sum::<f64>() / stage_count as f64
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
        let total_severity_score: f64 = discrepancies.iter().map(|d| {
            match d.severity {
                DiscrepancySeverity::Critical => 1.0,
                DiscrepancySeverity::Major => 0.7,
                DiscrepancySeverity::Minor => 0.3,
            }
        }).sum();
        
        let max_possible_score = discrepancies.len() as f64;
        let agreement_score = 1.0 - (total_severity_score / max_possible_score);
        
        Ok(agreement_score.max(0.0))
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
        validator.add_stage_output(
            Stage::Generator,
            "This is the hive-ai project version 2.0.2 with 100 dependencies.".to_string(),
            0.9
        ).unwrap();
        
        validator.add_stage_output(
            Stage::Validator,
            "The hive-ai project (version 2.0.2) has 100 dependencies and is enterprise-grade.".to_string(),
            0.85
        ).unwrap();
        
        let health = validator.verify_stage_consensus().unwrap();
        assert!(matches!(health, ConsensusHealth::Healthy));
    }
    
    #[tokio::test]
    async fn test_compromised_consensus() {
        let facts = create_test_facts();
        let mut validator = CrossValidator::new(facts);
        
        // Add contradictory stage outputs
        validator.add_stage_output(
            Stage::Generator,
            "This is the hive-ai project version 2.0.2 with 100 dependencies.".to_string(),
            0.9
        ).unwrap();
        
        validator.add_stage_output(
            Stage::Validator,
            "This is a minimal project with version 0.1.0 and no dependencies.".to_string(),
            0.8
        ).unwrap();
        
        let health = validator.verify_stage_consensus().unwrap();
        match health {
            ConsensusHealth::Compromised { discrepancies, .. } => {
                assert!(!discrepancies.is_empty());
            },
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