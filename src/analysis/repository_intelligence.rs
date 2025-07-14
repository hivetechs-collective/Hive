//! Repository intelligence with architecture pattern detection and quality assessment
//!
//! This module provides comprehensive repository analysis including:
//! - Architecture pattern detection (MVC, Clean, Layered, etc.)
//! - Code quality scoring (0-10 scale)
//! - Security vulnerability detection
//! - Performance hotspot identification
//! - Technical debt quantification

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use crate::analysis::{
    dependency::{DependencyAnalysis, DependencyAnalyzer},
    symbol_index::{SymbolEntry, SymbolIndexer},
};
use crate::core::ast::{CodeMetrics, ParseResult};
use crate::core::{Position, SymbolKind};

/// Repository analyzer for comprehensive codebase intelligence
pub struct RepositoryAnalyzer {
    /// Symbol indexer
    symbol_indexer: Arc<SymbolIndexer>,
    /// Dependency analyzer
    dependency_analyzer: Arc<DependencyAnalyzer>,
    /// Pattern detectors
    pattern_detectors: Arc<RwLock<Vec<Box<dyn ArchitectureDetector>>>>,
    /// Quality assessors
    quality_assessors: Arc<RwLock<Vec<Box<dyn QualityAssessor>>>>,
    /// Security scanners
    security_scanners: Arc<RwLock<Vec<Box<dyn SecurityScanner>>>>,
}

/// Repository analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAnalysis {
    pub architecture: ArchitectureInfo,
    pub quality: QualityReport,
    pub security: SecurityReport,
    pub performance: PerformanceReport,
    pub technical_debt: TechnicalDebtReport,
    pub recommendations: Vec<Recommendation>,
}

impl fmt::Display for RepositoryAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Repository Analysis:\n")?;
        write!(
            f,
            "  Architecture: {:?} (confidence: {:.1}%)\n",
            self.architecture.primary_pattern,
            self.architecture.confidence * 100.0
        )?;
        write!(f, "  Quality Score: {:.1}/10\n", self.quality.overall_score)?;
        write!(
            f,
            "  Security Issues: {} ({} critical)\n",
            self.security.issues.len(),
            self.security
                .issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Critical))
                .count()
        )?;
        write!(
            f,
            "  Performance Issues: {}\n",
            self.performance.issues.len()
        )?;
        write!(
            f,
            "  Technical Debt: {:.1} hours\n",
            self.technical_debt.total_hours
        )?;
        write!(f, "  Recommendations: {}", self.recommendations.len())
    }
}

/// Architecture information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureInfo {
    pub detected_patterns: Vec<ArchitecturePattern>,
    pub primary_pattern: ArchitecturePattern,
    pub confidence: f32,
    pub layers: Vec<ArchitectureLayer>,
    pub components: Vec<Component>,
    pub adherence_score: f32,
}

/// Architecture patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchitecturePattern {
    MVC,
    MVP,
    MVVM,
    CleanArchitecture,
    HexagonalArchitecture,
    LayeredArchitecture,
    Microservices,
    Monolith,
    EventDriven,
    PipeAndFilter,
    PluginBased,
    Unknown,
}

/// Architecture layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureLayer {
    pub name: String,
    pub modules: Vec<PathBuf>,
    pub responsibilities: Vec<String>,
    pub violations: Vec<LayerViolation>,
}

/// Layer violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerViolation {
    pub from_module: PathBuf,
    pub to_module: PathBuf,
    pub from_layer: String,
    pub to_layer: String,
    pub severity: Severity,
}

/// Component in architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub kind: ComponentKind,
    pub modules: Vec<PathBuf>,
    pub interfaces: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Component types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentKind {
    Controller,
    Service,
    Repository,
    Model,
    View,
    UseCase,
    Entity,
    Gateway,
    Presenter,
    Middleware,
}

/// Quality report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub overall_score: f32, // 0-10 scale
    pub metrics: QualityMetrics,
    pub issues: Vec<QualityIssue>,
    pub hotspots: Vec<QualityHotspot>,
    pub trends: QualityTrends,
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub maintainability_index: f32,
    pub cyclomatic_complexity: f32,
    pub cognitive_complexity: f32,
    pub duplication_ratio: f32,
    pub test_coverage: f32,
    pub documentation_coverage: f32,
    pub code_smell_density: f32,
    pub technical_debt_ratio: f32,
}

/// Quality issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub kind: QualityIssueKind,
    pub location: PathBuf,
    pub line: usize,
    pub severity: Severity,
    pub message: String,
    pub remediation_time: u32, // minutes
}

/// Quality issue types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityIssueKind {
    HighComplexity,
    LongMethod,
    LargeClass,
    DuplicateCode,
    MissingDocumentation,
    PoorNaming,
    UnusedCode,
    GodClass,
    FeatureEnvy,
    DataClump,
}

/// Quality hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityHotspot {
    pub file: PathBuf,
    pub score: f32,
    pub issues_count: usize,
    pub change_frequency: f32,
    pub contributors: usize,
}

/// Quality trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrends {
    pub improving_files: Vec<PathBuf>,
    pub degrading_files: Vec<PathBuf>,
    pub complexity_trend: Trend,
    pub duplication_trend: Trend,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
}

/// Security report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub vulnerability_count: usize,
    pub risk_score: f32,
    pub vulnerabilities: Vec<SecurityVulnerability>,
    pub dependency_risks: Vec<DependencyRisk>,
    pub issues: Vec<SecurityVulnerability>, // Added for compatibility
}

/// Security vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub kind: VulnerabilityKind,
    pub location: PathBuf,
    pub line: usize,
    pub severity: Severity,
    pub description: String,
    pub cwe_id: Option<String>,
    pub remediation: String,
}

/// Vulnerability types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilityKind {
    SqlInjection,
    XSS,
    PathTraversal,
    CommandInjection,
    HardcodedSecret,
    InsecureRandom,
    WeakCrypto,
    UnvalidatedInput,
    SSRF,
    XXE,
}

/// Dependency risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRisk {
    pub dependency: String,
    pub version: String,
    pub vulnerabilities: Vec<String>,
    pub severity: Severity,
    pub update_available: Option<String>,
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub hotspots: Vec<PerformanceHotspot>,
    pub bottlenecks: Vec<Bottleneck>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
    pub issues: Vec<PerformanceHotspot>, // Added for compatibility
}

/// Performance hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHotspot {
    pub location: PathBuf,
    pub function: String,
    pub issue_kind: PerformanceIssueKind,
    pub impact: Impact,
    pub description: String,
}

/// Performance issue types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceIssueKind {
    NestedLoop,
    UnboundedRecursion,
    SynchronousIO,
    InefficientAlgorithm,
    MemoryLeak,
    ExcessiveAllocation,
    Blocking,
    N1Query,
}

/// Bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub component: String,
    pub kind: BottleneckKind,
    pub severity: Severity,
    pub affected_operations: Vec<String>,
}

/// Bottleneck types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottleneckKind {
    CPU,
    Memory,
    IO,
    Network,
    Database,
    Synchronization,
}

/// Optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub location: PathBuf,
    pub kind: OptimizationKind,
    pub estimated_improvement: f32,
    pub effort: Effort,
    pub description: String,
}

/// Optimization types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationKind {
    Caching,
    Parallelization,
    AlgorithmImprovement,
    DataStructureOptimization,
    LazyLoading,
    Batching,
    IndexAddition,
}

/// Technical debt report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtReport {
    pub total_debt_hours: f32,
    pub debt_ratio: f32,
    pub estimated_cost: f32,
    pub items: Vec<TechnicalDebtItem>,
    pub debt_by_category: HashMap<String, f32>,
    pub payment_plan: Vec<DebtPaymentTask>,
    pub total_hours: f32, // Added for compatibility (alias for total_debt_hours)
}

/// Technical debt item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtItem {
    pub kind: TechnicalDebtKind,
    pub location: PathBuf,
    pub principal: f32, // hours
    pub interest: f32,  // hours/month
    pub description: String,
    pub created_date: Option<String>,
}

/// Technical debt types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechnicalDebtKind {
    CodeDuplication,
    MissingTests,
    OutdatedDependencies,
    PoorArchitecture,
    MissingDocumentation,
    Workarounds,
    LegacyCode,
    InconsistentNaming,
}

/// Debt payment task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtPaymentTask {
    pub priority: Priority,
    pub items: Vec<TechnicalDebtItem>,
    pub estimated_hours: f32,
    pub roi: f32,
    pub dependencies: Vec<String>,
}

/// Recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub impact: Impact,
    pub effort: Effort,
    pub specific_actions: Vec<String>,
}

/// Recommendation categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Architecture,
    Quality,
    Security,
    Performance,
    Maintainability,
    Testing,
    Documentation,
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

/// Impact levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Impact {
    Minimal,
    Moderate,
    Significant,
    Major,
}

/// Effort levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Effort {
    Trivial,
    Small,
    Medium,
    Large,
    Massive,
}

/// Architecture detector trait
pub trait ArchitectureDetector: Send + Sync {
    fn detect(
        &self,
        symbols: &[SymbolEntry],
        dependencies: &DependencyAnalysis,
    ) -> Option<(ArchitecturePattern, f32)>;
}

/// Quality assessor trait
pub trait QualityAssessor: Send + Sync {
    fn assess(&self, symbols: &[SymbolEntry], metrics: &CodeMetrics) -> Vec<QualityIssue>;
}

/// Security scanner trait
pub trait SecurityScanner: Send + Sync {
    fn scan(&self, content: &str, path: &Path) -> Vec<SecurityVulnerability>;
}

impl RepositoryAnalyzer {
    /// Create a new repository analyzer
    pub async fn new(
        symbol_indexer: Arc<SymbolIndexer>,
        dependency_analyzer: Arc<DependencyAnalyzer>,
    ) -> Result<Self> {
        let mut analyzer = Self {
            symbol_indexer,
            dependency_analyzer,
            pattern_detectors: Arc::new(RwLock::new(Vec::new())),
            quality_assessors: Arc::new(RwLock::new(Vec::new())),
            security_scanners: Arc::new(RwLock::new(Vec::new())),
        };

        // Register default detectors and assessors
        analyzer.register_default_components().await?;

        Ok(analyzer)
    }

    /// Analyze a repository
    #[instrument(skip(self))]
    pub async fn analyze_repository(&self, root_path: &Path) -> Result<RepositoryAnalysis> {
        info!("Analyzing repository at {}", root_path.display());

        // Build symbol index
        self.symbol_indexer.index_file(root_path, "").await?;

        // Analyze dependencies
        let dependency_analysis = self.dependency_analyzer.analyze_project(root_path).await?;

        // Get all symbols
        let symbols = self.get_all_symbols().await?;

        // Detect architecture
        let architecture = self
            .detect_architecture(&symbols, &dependency_analysis)
            .await?;

        // Assess quality
        let quality = self.assess_quality(&symbols).await?;

        // Scan security
        let security = self.scan_security(root_path).await?;

        // Analyze performance
        let performance = self.analyze_performance(&symbols).await?;

        // Calculate technical debt
        let technical_debt = self.calculate_technical_debt(&quality, &security).await?;

        // Generate recommendations
        let recommendations = self
            .generate_recommendations(
                &architecture,
                &quality,
                &security,
                &performance,
                &technical_debt,
            )
            .await?;

        Ok(RepositoryAnalysis {
            architecture,
            quality,
            security,
            performance,
            technical_debt,
            recommendations,
        })
    }

    /// Register default components
    async fn register_default_components(&self) -> Result<()> {
        // Register architecture detectors
        {
            let mut detectors = self.pattern_detectors.write().await;
            detectors.push(Box::new(MVCDetector));
            detectors.push(Box::new(CleanArchitectureDetector));
            detectors.push(Box::new(LayeredArchitectureDetector));
            detectors.push(Box::new(MicroservicesDetector));
        }

        // Register quality assessors
        {
            let mut assessors = self.quality_assessors.write().await;
            assessors.push(Box::new(ComplexityAssessor));
            assessors.push(Box::new(DuplicationAssessor));
            assessors.push(Box::new(NamingAssessor));
        }

        // Register security scanners
        {
            let mut scanners = self.security_scanners.write().await;
            scanners.push(Box::new(BasicSecurityScanner));
        }

        Ok(())
    }

    /// Get all symbols from index
    async fn get_all_symbols(&self) -> Result<Vec<SymbolEntry>> {
        // Query the symbol index for all symbols
        self.symbol_indexer
            .get_all_symbols()
            .await
            .map_err(|e| anyhow!("Failed to get symbols: {}", e))
    }

    /// Detect architecture patterns
    async fn detect_architecture(
        &self,
        symbols: &[SymbolEntry],
        dependencies: &DependencyAnalysis,
    ) -> Result<ArchitectureInfo> {
        let detectors = self.pattern_detectors.read().await;

        let mut detected_patterns = Vec::new();

        for detector in detectors.iter() {
            if let Some((pattern, confidence)) = detector.detect(symbols, dependencies) {
                detected_patterns.push((pattern, confidence));
            }
        }

        // Sort by confidence
        detected_patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let primary_pattern = detected_patterns
            .first()
            .map(|(p, _)| *p)
            .unwrap_or(ArchitecturePattern::Unknown);

        let confidence = detected_patterns.first().map(|(_, c)| *c).unwrap_or(0.0);

        // Analyze layers
        let layers = self.analyze_layers(symbols, dependencies)?;

        // Identify components
        let components = self.identify_components(symbols)?;

        // Calculate adherence score
        let adherence_score = self.calculate_adherence_score(&layers, &components)?;

        Ok(ArchitectureInfo {
            detected_patterns: detected_patterns.into_iter().map(|(p, _)| p).collect(),
            primary_pattern,
            confidence,
            layers,
            components,
            adherence_score,
        })
    }

    /// Assess code quality
    async fn assess_quality(&self, symbols: &[SymbolEntry]) -> Result<QualityReport> {
        let assessors = self.quality_assessors.read().await;

        let mut all_issues = Vec::new();
        let metrics = self.calculate_quality_metrics(symbols)?;

        for assessor in assessors.iter() {
            let issues = assessor.assess(symbols, &CodeMetrics::default());
            all_issues.extend(issues);
        }

        // Calculate overall score
        let overall_score = self.calculate_quality_score(&metrics, &all_issues);

        // Identify hotspots
        let hotspots = self.identify_quality_hotspots(&all_issues)?;

        // Analyze trends
        let trends = self.analyze_quality_trends()?;

        Ok(QualityReport {
            overall_score,
            metrics,
            issues: all_issues,
            hotspots,
            trends,
        })
    }

    /// Calculate quality metrics
    fn calculate_quality_metrics(&self, symbols: &[SymbolEntry]) -> Result<QualityMetrics> {
        let total_complexity: u32 = symbols.iter().map(|s| s.complexity).sum();
        let avg_complexity = if symbols.is_empty() {
            0.0
        } else {
            total_complexity as f32 / symbols.len() as f32
        };

        let documented_symbols = symbols.iter().filter(|s| s.documentation.is_some()).count();
        let doc_coverage = if symbols.is_empty() {
            0.0
        } else {
            documented_symbols as f32 / symbols.len() as f32
        };

        Ok(QualityMetrics {
            maintainability_index: 85.0, // Placeholder
            cyclomatic_complexity: avg_complexity,
            cognitive_complexity: avg_complexity * 1.2, // Rough estimate
            duplication_ratio: 0.05,                    // Placeholder
            test_coverage: 0.75,                        // Placeholder
            documentation_coverage: doc_coverage,
            code_smell_density: 0.1,    // Placeholder
            technical_debt_ratio: 0.15, // Placeholder
        })
    }

    /// Calculate overall quality score
    fn calculate_quality_score(&self, metrics: &QualityMetrics, issues: &[QualityIssue]) -> f32 {
        let mut score = 10.0;

        // Deduct for poor metrics
        if metrics.cyclomatic_complexity > 10.0 {
            score -= (metrics.cyclomatic_complexity - 10.0) * 0.1;
        }

        if metrics.duplication_ratio > 0.1 {
            score -= (metrics.duplication_ratio - 0.1) * 10.0;
        }

        if metrics.documentation_coverage < 0.5 {
            score -= (0.5 - metrics.documentation_coverage) * 2.0;
        }

        // Deduct for issues
        let critical_issues = issues
            .iter()
            .filter(|i| i.severity == Severity::Critical)
            .count();
        let high_issues = issues
            .iter()
            .filter(|i| i.severity == Severity::High)
            .count();

        score -= (critical_issues as f32 * 0.5) + (high_issues as f32 * 0.2);

        score.max(0.0).min(10.0)
    }

    /// Scan for security vulnerabilities
    async fn scan_security(&self, root_path: &Path) -> Result<SecurityReport> {
        let scanners = self.security_scanners.read().await;
        let mut vulnerabilities = Vec::new();

        // Walk through files and scan
        for entry in std::fs::read_dir(root_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                // Only scan certain file types
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                let scannable_extensions = vec![
                    "rs", "js", "ts", "py", "java", "go", "php", "rb", "cpp", "c", "h",
                ];

                if scannable_extensions.contains(&extension) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        for scanner in scanners.iter() {
                            let mut file_vulns = scanner.scan(&content, &path);
                            vulnerabilities.append(&mut file_vulns);
                        }
                    }
                }
            } else if path.is_dir()
                && !path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or("")
                    .starts_with('.')
            {
                // Recursively scan subdirectories (basic implementation)
                let sub_report = Box::pin(self.scan_security(&path)).await?;
                vulnerabilities.extend(sub_report.vulnerabilities);
            }
        }

        let vulnerability_count = vulnerabilities.len();
        let risk_score = self.calculate_risk_score(&vulnerabilities);

        Ok(SecurityReport {
            vulnerability_count,
            risk_score,
            vulnerabilities: vulnerabilities.clone(),
            dependency_risks: vec![], // Placeholder - would need dependency analysis
            issues: vulnerabilities,  // Use same vulnerabilities for issues field
        })
    }

    /// Calculate risk score
    fn calculate_risk_score(&self, vulnerabilities: &[SecurityVulnerability]) -> f32 {
        let critical = vulnerabilities
            .iter()
            .filter(|v| v.severity == Severity::Critical)
            .count();
        let high = vulnerabilities
            .iter()
            .filter(|v| v.severity == Severity::High)
            .count();
        let medium = vulnerabilities
            .iter()
            .filter(|v| v.severity == Severity::Medium)
            .count();

        ((critical as f32 * 10.0) + (high as f32 * 5.0) + (medium as f32 * 2.0)).min(100.0)
    }

    /// Analyze performance
    async fn analyze_performance(&self, symbols: &[SymbolEntry]) -> Result<PerformanceReport> {
        // Identify performance hotspots
        let hotspots = self.identify_performance_hotspots(symbols)?;

        // Find bottlenecks
        let bottlenecks = self.find_bottlenecks(symbols)?;

        // Find optimization opportunities
        let optimization_opportunities = self.find_optimization_opportunities(symbols)?;

        Ok(PerformanceReport {
            hotspots: hotspots.clone(),
            bottlenecks,
            optimization_opportunities,
            issues: hotspots, // Use hotspots for issues field
        })
    }

    /// Calculate technical debt using SQALE methodology
    async fn calculate_technical_debt(
        &self,
        quality: &QualityReport,
        security: &SecurityReport,
    ) -> Result<TechnicalDebtReport> {
        let mut debt_items = Vec::new();
        let mut total_debt_hours = 0.0;

        // SQALE characteristics and remediation costs
        let sqale_characteristics = HashMap::from([
            (TechnicalDebtKind::CodeDuplication, (0.05, 0.15)), // (base_cost_per_line, interest_rate)
            (TechnicalDebtKind::MissingTests, (0.1, 0.20)),
            (TechnicalDebtKind::OutdatedDependencies, (2.0, 0.25)),
            (TechnicalDebtKind::PoorArchitecture, (8.0, 0.30)),
            (TechnicalDebtKind::MissingDocumentation, (0.25, 0.10)),
            (TechnicalDebtKind::Workarounds, (4.0, 0.35)),
            (TechnicalDebtKind::LegacyCode, (12.0, 0.40)),
            (TechnicalDebtKind::InconsistentNaming, (0.15, 0.05)),
        ]);

        // Convert quality issues to debt using SQALE
        for issue in &quality.issues {
            let debt_kind = self.quality_issue_to_debt_kind(issue.kind);
            let (base_cost, interest_rate) = sqale_characteristics
                .get(&debt_kind)
                .unwrap_or(&(1.0, 0.15));

            // Calculate principal based on issue severity and type
            let severity_multiplier = match issue.severity {
                Severity::Critical => 4.0,
                Severity::High => 2.5,
                Severity::Medium => 1.5,
                Severity::Low => 1.0,
            };

            let principal = base_cost * severity_multiplier;
            let interest = interest_rate * principal;

            let debt_item = TechnicalDebtItem {
                kind: debt_kind,
                location: issue.location.clone(),
                principal,
                interest,
                description: issue.message.clone(),
                created_date: Some("2024-01-01".to_string()), // Placeholder - would use chrono in real implementation
            };

            total_debt_hours += debt_item.principal;
            debt_items.push(debt_item);
        }

        // Add security vulnerabilities as high-priority debt
        for vuln in &security.vulnerabilities {
            let (base_cost, interest_rate) = match vuln.severity {
                Severity::Critical => (12.0, 0.50), // Critical security issues are very expensive
                Severity::High => (8.0, 0.40),
                Severity::Medium => (4.0, 0.30),
                Severity::Low => (2.0, 0.20),
            };

            let debt_item = TechnicalDebtItem {
                kind: TechnicalDebtKind::Workarounds, // Security issues as workarounds
                location: vuln.location.clone(),
                principal: base_cost,
                interest: interest_rate * base_cost,
                description: format!("Security vulnerability: {}", vuln.description),
                created_date: Some("2024-01-01".to_string()),
            };

            total_debt_hours += debt_item.principal;
            debt_items.push(debt_item);
        }

        // Calculate total development effort (estimate based on codebase size)
        let total_dev_hours = self.estimate_total_development_effort(quality).await?;

        // Calculate debt ratio using SQALE formula
        let debt_ratio = if total_dev_hours > 0.0 {
            total_debt_hours / total_dev_hours
        } else {
            0.0
        };

        // Estimate cost with regional adjustments (assuming $100/hour base rate)
        let hourly_rate = 100.0;
        let estimated_cost = total_debt_hours * hourly_rate;

        // Group by category for analysis
        let mut debt_by_category = HashMap::new();
        for item in &debt_items {
            let category_key = format!("{:?}", item.kind);
            *debt_by_category.entry(category_key).or_insert(0.0) += item.principal;
        }

        // Create ROI-based payment plan
        let payment_plan = self.create_debt_payment_plan(&debt_items)?;

        Ok(TechnicalDebtReport {
            total_debt_hours,
            debt_ratio,
            estimated_cost,
            items: debt_items,
            debt_by_category,
            payment_plan,
            total_hours: total_debt_hours, // Alias for backward compatibility
        })
    }

    /// Estimate total development effort for SQALE calculation
    async fn estimate_total_development_effort(&self, quality: &QualityReport) -> Result<f32> {
        // SQALE estimation based on maintainability index and codebase size
        let base_effort = 1000.0; // Base hours for small project

        // Adjust based on maintainability index
        let maintainability_factor = if quality.metrics.maintainability_index > 80.0 {
            0.8 // Well-maintained code requires less effort
        } else if quality.metrics.maintainability_index > 60.0 {
            1.0 // Average maintenance effort
        } else if quality.metrics.maintainability_index > 40.0 {
            1.5 // Higher maintenance effort
        } else {
            2.0 // Very high maintenance effort
        };

        // Adjust based on complexity
        let complexity_factor = if quality.metrics.cyclomatic_complexity > 20.0 {
            2.0
        } else if quality.metrics.cyclomatic_complexity > 10.0 {
            1.5
        } else {
            1.0
        };

        // Adjust based on test coverage
        let test_coverage_factor = if quality.metrics.test_coverage < 0.3 {
            1.8 // Low test coverage increases effort
        } else if quality.metrics.test_coverage < 0.7 {
            1.2
        } else {
            0.9 // High test coverage reduces effort
        };

        Ok(base_effort * maintainability_factor * complexity_factor * test_coverage_factor)
    }

    /// Generate recommendations
    async fn generate_recommendations(
        &self,
        architecture: &ArchitectureInfo,
        quality: &QualityReport,
        security: &SecurityReport,
        performance: &PerformanceReport,
        debt: &TechnicalDebtReport,
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Architecture recommendations
        if architecture.adherence_score < 0.7 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Architecture,
                priority: Priority::High,
                title: "Improve Architecture Adherence".to_string(),
                description: format!(
                    "The codebase shows {} architecture pattern but adherence is only {:.1}%",
                    format!("{:?}", architecture.primary_pattern),
                    architecture.adherence_score * 100.0
                ),
                impact: Impact::Significant,
                effort: Effort::Large,
                specific_actions: vec![
                    "Review layer violations and fix dependencies".to_string(),
                    "Refactor components to match architectural boundaries".to_string(),
                ],
            });
        }

        // Quality recommendations
        if quality.overall_score < 7.0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Quality,
                priority: Priority::High,
                title: "Improve Code Quality".to_string(),
                description: format!(
                    "Overall quality score is {:.1}/10. Focus on reducing complexity and improving documentation.",
                    quality.overall_score
                ),
                impact: Impact::Major,
                effort: Effort::Medium,
                specific_actions: vec![
                    "Refactor complex methods (cyclomatic complexity > 10)".to_string(),
                    "Add missing documentation to public APIs".to_string(),
                    "Remove code duplication".to_string(),
                ],
            });
        }

        // Security recommendations
        if security.risk_score > 20.0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Security,
                priority: Priority::Urgent,
                title: "Address Security Vulnerabilities".to_string(),
                description: format!(
                    "Found {} security vulnerabilities with risk score {:.1}",
                    security.vulnerability_count, security.risk_score
                ),
                impact: Impact::Major,
                effort: Effort::Small,
                specific_actions: vec![
                    "Fix critical vulnerabilities immediately".to_string(),
                    "Update vulnerable dependencies".to_string(),
                    "Implement input validation".to_string(),
                ],
            });
        }

        // Performance recommendations
        if !performance.hotspots.is_empty() {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Performance,
                priority: Priority::Medium,
                title: "Optimize Performance Hotspots".to_string(),
                description: format!(
                    "Found {} performance hotspots that could be optimized",
                    performance.hotspots.len()
                ),
                impact: Impact::Significant,
                effort: Effort::Medium,
                specific_actions: performance
                    .optimization_opportunities
                    .iter()
                    .take(3)
                    .map(|o| o.description.clone())
                    .collect(),
            });
        }

        // Technical debt recommendations
        if debt.debt_ratio > 0.2 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Maintainability,
                priority: Priority::High,
                title: "Reduce Technical Debt".to_string(),
                description: format!(
                    "Technical debt is {:.1}% of development effort (${:.0} estimated cost)",
                    debt.debt_ratio * 100.0,
                    debt.estimated_cost
                ),
                impact: Impact::Major,
                effort: Effort::Large,
                specific_actions: vec![
                    "Prioritize debt payment based on ROI".to_string(),
                    "Allocate 20% of sprints to debt reduction".to_string(),
                    "Focus on high-interest debt items first".to_string(),
                ],
            });
        }

        Ok(recommendations)
    }

    // Helper methods

    fn analyze_layers(
        &self,
        symbols: &[SymbolEntry],
        dependencies: &DependencyAnalysis,
    ) -> Result<Vec<ArchitectureLayer>> {
        let mut layers = Vec::new();
        let mut layer_modules: HashMap<String, Vec<PathBuf>> = HashMap::new();

        // Group symbols by potential layer based on file paths
        for symbol in symbols {
            let path_str = symbol.file_path.to_string_lossy().to_lowercase();

            let layer_name = if path_str.contains("controller")
                || path_str.contains("handler")
                || path_str.contains("api")
            {
                "presentation"
            } else if path_str.contains("service")
                || path_str.contains("business")
                || path_str.contains("logic")
            {
                "business"
            } else if path_str.contains("repository")
                || path_str.contains("data")
                || path_str.contains("persistence")
            {
                "data"
            } else if path_str.contains("model")
                || path_str.contains("entity")
                || path_str.contains("domain")
            {
                "domain"
            } else if path_str.contains("infrastructure") || path_str.contains("external") {
                "infrastructure"
            } else {
                "core"
            };

            layer_modules
                .entry(layer_name.to_string())
                .or_insert_with(Vec::new)
                .push(symbol.file_path.clone());
        }

        // Create layer objects
        for (layer_name, modules) in layer_modules {
            let responsibilities = match layer_name.as_str() {
                "presentation" => vec![
                    "UI logic".to_string(),
                    "Request handling".to_string(),
                    "Response formatting".to_string(),
                ],
                "business" => vec![
                    "Business logic".to_string(),
                    "Use cases".to_string(),
                    "Workflows".to_string(),
                ],
                "data" => vec![
                    "Data access".to_string(),
                    "Persistence".to_string(),
                    "External APIs".to_string(),
                ],
                "domain" => vec![
                    "Domain models".to_string(),
                    "Business entities".to_string(),
                    "Value objects".to_string(),
                ],
                "infrastructure" => vec![
                    "Cross-cutting concerns".to_string(),
                    "Framework integration".to_string(),
                ],
                _ => vec!["Core functionality".to_string()],
            };

            // Detect layer violations
            let violations = self.detect_layer_violations(&layer_name, &modules, dependencies)?;

            layers.push(ArchitectureLayer {
                name: layer_name,
                modules: modules
                    .into_iter()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect(),
                responsibilities,
                violations,
            });
        }

        Ok(layers)
    }

    fn identify_components(&self, symbols: &[SymbolEntry]) -> Result<Vec<Component>> {
        let mut components = Vec::new();
        let mut component_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

        // Group symbols by component type based on naming patterns
        for symbol in symbols {
            let name_lower = symbol.name.to_lowercase();
            let path_str = symbol.file_path.to_string_lossy().to_lowercase();

            let (component_kind, component_name) =
                if name_lower.ends_with("controller") || path_str.contains("controller") {
                    (
                        ComponentKind::Controller,
                        symbol.name.replace("Controller", ""),
                    )
                } else if name_lower.ends_with("service") || path_str.contains("service") {
                    (ComponentKind::Service, symbol.name.replace("Service", ""))
                } else if name_lower.ends_with("repository") || path_str.contains("repository") {
                    (
                        ComponentKind::Repository,
                        symbol.name.replace("Repository", ""),
                    )
                } else if name_lower.ends_with("model") || path_str.contains("model") {
                    (ComponentKind::Model, symbol.name.replace("Model", ""))
                } else if name_lower.contains("view") || path_str.contains("view") {
                    (ComponentKind::View, symbol.name.replace("View", ""))
                } else if name_lower.contains("usecase") || name_lower.contains("interactor") {
                    (ComponentKind::UseCase, symbol.name.clone())
                } else if name_lower.contains("entity") || path_str.contains("entity") {
                    (ComponentKind::Entity, symbol.name.replace("Entity", ""))
                } else if name_lower.contains("gateway") || path_str.contains("gateway") {
                    (ComponentKind::Gateway, symbol.name.replace("Gateway", ""))
                } else if name_lower.contains("presenter") || path_str.contains("presenter") {
                    (
                        ComponentKind::Presenter,
                        symbol.name.replace("Presenter", ""),
                    )
                } else if name_lower.contains("middleware") || path_str.contains("middleware") {
                    (
                        ComponentKind::Middleware,
                        symbol.name.replace("Middleware", ""),
                    )
                } else {
                    continue; // Skip symbols that don't match known patterns
                };

            let key = format!("{:?}-{}", component_kind, component_name);
            component_map
                .entry(key)
                .or_insert_with(Vec::new)
                .push(symbol.file_path.clone());
        }

        // Create component objects
        for (key, modules) in component_map {
            let parts: Vec<&str> = key.split('-').collect();
            if parts.len() >= 2 {
                let kind_str = parts[0];
                let name = parts[1..].join("-");

                let kind = match kind_str {
                    "Controller" => ComponentKind::Controller,
                    "Service" => ComponentKind::Service,
                    "Repository" => ComponentKind::Repository,
                    "Model" => ComponentKind::Model,
                    "View" => ComponentKind::View,
                    "UseCase" => ComponentKind::UseCase,
                    "Entity" => ComponentKind::Entity,
                    "Gateway" => ComponentKind::Gateway,
                    "Presenter" => ComponentKind::Presenter,
                    "Middleware" => ComponentKind::Middleware,
                    _ => continue,
                };

                components.push(Component {
                    name,
                    kind,
                    modules: modules
                        .into_iter()
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .collect(),
                    interfaces: vec![],   // Would need deeper analysis
                    dependencies: vec![], // Would need dependency graph analysis
                });
            }
        }

        Ok(components)
    }

    fn calculate_adherence_score(
        &self,
        layers: &[ArchitectureLayer],
        components: &[Component],
    ) -> Result<f32> {
        let mut total_violations = 0;
        let mut total_dependencies = 0;

        // Count layer violations
        for layer in layers {
            total_violations += layer.violations.len();
            total_dependencies += layer.modules.len() * 2; // Rough estimate
        }

        // Calculate adherence based on violation ratio
        if total_dependencies == 0 {
            return Ok(1.0); // Perfect adherence if no dependencies
        }

        let violation_ratio = total_violations as f32 / total_dependencies as f32;
        let adherence_score = (1.0 - violation_ratio).max(0.0).min(1.0);

        // Adjust score based on component organization
        let component_bonus = if components.len() > 5 {
            0.1 // Well-organized components get a bonus
        } else {
            0.0
        };

        Ok((adherence_score + component_bonus).min(1.0))
    }

    fn identify_quality_hotspots(&self, issues: &[QualityIssue]) -> Result<Vec<QualityHotspot>> {
        let mut hotspots = Vec::new();
        let mut file_issues: HashMap<PathBuf, Vec<&QualityIssue>> = HashMap::new();

        // Group issues by file
        for issue in issues {
            file_issues
                .entry(issue.location.clone())
                .or_insert_with(Vec::new)
                .push(issue);
        }

        // Calculate hotspot scores
        for (file, file_issues_list) in file_issues {
            let issues_count = file_issues_list.len();
            let severity_score: f32 = file_issues_list
                .iter()
                .map(|issue| match issue.severity {
                    Severity::Critical => 4.0,
                    Severity::High => 3.0,
                    Severity::Medium => 2.0,
                    Severity::Low => 1.0,
                })
                .sum();

            let remediation_time: u32 = file_issues_list
                .iter()
                .map(|issue| issue.remediation_time)
                .sum();

            // Score based on issues count, severity, and remediation effort
            let score =
                (severity_score + (issues_count as f32 * 0.5) + (remediation_time as f32 / 60.0))
                    / 10.0;

            // Estimate change frequency and contributors (would need git analysis)
            let change_frequency = 0.5; // Placeholder
            let contributors = 2; // Placeholder

            if score > 0.3 {
                // Only include significant hotspots
                hotspots.push(QualityHotspot {
                    file,
                    score: score.min(10.0),
                    issues_count,
                    change_frequency,
                    contributors,
                });
            }
        }

        // Sort by score descending
        hotspots.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(hotspots)
    }

    fn analyze_quality_trends(&self) -> Result<QualityTrends> {
        // This would require historical data analysis
        // For now, return reasonable defaults
        Ok(QualityTrends {
            improving_files: vec![], // Would need git history analysis
            degrading_files: vec![], // Would need git history analysis
            complexity_trend: Trend::Stable,
            duplication_trend: Trend::Stable,
        })
    }

    fn identify_performance_hotspots(
        &self,
        symbols: &[SymbolEntry],
    ) -> Result<Vec<PerformanceHotspot>> {
        let mut hotspots = Vec::new();

        for symbol in symbols {
            // Check for nested loops (O(n²) or worse)
            if let Some(signature) = &symbol.signature {
                let loop_count = self.count_nested_loops(signature);
                if loop_count >= 2 {
                    hotspots.push(PerformanceHotspot {
                        location: symbol.file_path.clone(),
                        function: symbol.name.clone(),
                        issue_kind: PerformanceIssueKind::NestedLoop,
                        impact: if loop_count >= 3 {
                            Impact::Major
                        } else {
                            Impact::Significant
                        },
                        description: format!(
                            "Function '{}' contains {} nested loops, potentially O(n^{})",
                            symbol.name, loop_count, loop_count
                        ),
                    });
                }

                // Check for recursive patterns without tail optimization
                if self.has_unbounded_recursion(signature, &symbol.name) {
                    hotspots.push(PerformanceHotspot {
                        location: symbol.file_path.clone(),
                        function: symbol.name.clone(),
                        issue_kind: PerformanceIssueKind::UnboundedRecursion,
                        impact: Impact::Major,
                        description: format!(
                            "Function '{}' may have unbounded recursion without proper termination",
                            symbol.name
                        ),
                    });
                }

                // Check for synchronous I/O operations
                if self.has_synchronous_io(signature) {
                    hotspots.push(PerformanceHotspot {
                        location: symbol.file_path.clone(),
                        function: symbol.name.clone(),
                        issue_kind: PerformanceIssueKind::SynchronousIO,
                        impact: Impact::Significant,
                        description: format!(
                            "Function '{}' performs synchronous I/O operations",
                            symbol.name
                        ),
                    });
                }

                // Check for inefficient algorithms
                if self.has_inefficient_algorithm(signature) {
                    hotspots.push(PerformanceHotspot {
                        location: symbol.file_path.clone(),
                        function: symbol.name.clone(),
                        issue_kind: PerformanceIssueKind::InefficientAlgorithm,
                        impact: Impact::Moderate,
                        description: format!(
                            "Function '{}' uses potentially inefficient algorithms",
                            symbol.name
                        ),
                    });
                }

                // Check for excessive memory allocation
                if self.has_excessive_allocation(signature) {
                    hotspots.push(PerformanceHotspot {
                        location: symbol.file_path.clone(),
                        function: symbol.name.clone(),
                        issue_kind: PerformanceIssueKind::ExcessiveAllocation,
                        impact: Impact::Moderate,
                        description: format!(
                            "Function '{}' may allocate excessive memory",
                            symbol.name
                        ),
                    });
                }

                // Check for N+1 query patterns
                if self.has_n_plus_one_pattern(signature) {
                    hotspots.push(PerformanceHotspot {
                        location: symbol.file_path.clone(),
                        function: symbol.name.clone(),
                        issue_kind: PerformanceIssueKind::N1Query,
                        impact: Impact::Major,
                        description: format!(
                            "Function '{}' may have N+1 query problem",
                            symbol.name
                        ),
                    });
                }
            }

            // Check for blocking operations based on function names
            if self.is_blocking_operation(&symbol.name) {
                hotspots.push(PerformanceHotspot {
                    location: symbol.file_path.clone(),
                    function: symbol.name.clone(),
                    issue_kind: PerformanceIssueKind::Blocking,
                    impact: Impact::Significant,
                    description: format!(
                        "Function '{}' appears to be a blocking operation",
                        symbol.name
                    ),
                });
            }
        }

        // Sort by impact
        hotspots.sort_by(|a, b| b.impact.cmp(&a.impact));

        Ok(hotspots)
    }

    fn find_bottlenecks(&self, symbols: &[SymbolEntry]) -> Result<Vec<Bottleneck>> {
        let mut bottlenecks = Vec::new();
        let mut component_usage: HashMap<String, Vec<&SymbolEntry>> = HashMap::new();

        // Group symbols by component/module
        for symbol in symbols {
            let component = symbol
                .file_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            component_usage
                .entry(component)
                .or_insert_with(Vec::new)
                .push(symbol);
        }

        // Analyze each component for bottlenecks
        for (component, component_symbols) in component_usage {
            let total_complexity: u32 = component_symbols.iter().map(|s| s.complexity).sum();
            let avg_complexity = if component_symbols.is_empty() {
                0.0
            } else {
                total_complexity as f32 / component_symbols.len() as f32
            };

            // CPU bottlenecks (high complexity)
            if avg_complexity > 15.0 {
                bottlenecks.push(Bottleneck {
                    component: component.clone(),
                    kind: BottleneckKind::CPU,
                    severity: if avg_complexity > 25.0 {
                        Severity::High
                    } else {
                        Severity::Medium
                    },
                    affected_operations: component_symbols.iter().map(|s| s.name.clone()).collect(),
                });
            }

            // Memory bottlenecks (based on function patterns)
            let memory_intensive_ops = component_symbols
                .iter()
                .filter(|s| self.is_memory_intensive(&s.name))
                .count();

            if memory_intensive_ops > 2 {
                bottlenecks.push(Bottleneck {
                    component: component.clone(),
                    kind: BottleneckKind::Memory,
                    severity: Severity::Medium,
                    affected_operations: component_symbols
                        .iter()
                        .filter(|s| self.is_memory_intensive(&s.name))
                        .map(|s| s.name.clone())
                        .collect(),
                });
            }

            // I/O bottlenecks (based on function patterns)
            let io_operations = component_symbols
                .iter()
                .filter(|s| self.is_io_operation(&s.name))
                .count();

            if io_operations > 3 {
                bottlenecks.push(Bottleneck {
                    component: component.clone(),
                    kind: BottleneckKind::IO,
                    severity: Severity::Medium,
                    affected_operations: component_symbols
                        .iter()
                        .filter(|s| self.is_io_operation(&s.name))
                        .map(|s| s.name.clone())
                        .collect(),
                });
            }

            // Database bottlenecks
            let db_operations = component_symbols
                .iter()
                .filter(|s| self.is_database_operation(&s.name))
                .count();

            if db_operations > 5 {
                bottlenecks.push(Bottleneck {
                    component: component.clone(),
                    kind: BottleneckKind::Database,
                    severity: Severity::High,
                    affected_operations: component_symbols
                        .iter()
                        .filter(|s| self.is_database_operation(&s.name))
                        .map(|s| s.name.clone())
                        .collect(),
                });
            }
        }

        Ok(bottlenecks)
    }

    fn find_optimization_opportunities(
        &self,
        symbols: &[SymbolEntry],
    ) -> Result<Vec<OptimizationOpportunity>> {
        let mut opportunities = Vec::new();

        for symbol in symbols {
            // Caching opportunities
            if let Some(signature) = &symbol.signature {
                if self.can_benefit_from_caching(signature) {
                    opportunities.push(OptimizationOpportunity {
                        location: symbol.file_path.clone(),
                        kind: OptimizationKind::Caching,
                        estimated_improvement: 0.3, // 30% improvement
                        effort: Effort::Small,
                        description: format!(
                            "Function '{}' could benefit from result caching",
                            symbol.name
                        ),
                    });
                }

                // Parallelization opportunities
                if self.can_be_parallelized(signature) {
                    opportunities.push(OptimizationOpportunity {
                        location: symbol.file_path.clone(),
                        kind: OptimizationKind::Parallelization,
                        estimated_improvement: 0.5, // 50% improvement
                        effort: Effort::Medium,
                        description: format!(
                            "Function '{}' contains loops that could be parallelized",
                            symbol.name
                        ),
                    });
                }

                // Algorithm improvements
                if self.has_improvable_algorithm(signature) {
                    opportunities.push(OptimizationOpportunity {
                        location: symbol.file_path.clone(),
                        kind: OptimizationKind::AlgorithmImprovement,
                        estimated_improvement: 0.7, // 70% improvement
                        effort: Effort::Large,
                        description: format!(
                            "Function '{}' uses inefficient algorithms that can be improved",
                            symbol.name
                        ),
                    });
                }

                // Data structure optimization
                if self.has_suboptimal_data_structures(signature) {
                    opportunities.push(OptimizationOpportunity {
                        location: symbol.file_path.clone(),
                        kind: OptimizationKind::DataStructureOptimization,
                        estimated_improvement: 0.4, // 40% improvement
                        effort: Effort::Medium,
                        description: format!(
                            "Function '{}' could use more efficient data structures",
                            symbol.name
                        ),
                    });
                }

                // Lazy loading opportunities
                if self.can_use_lazy_loading(signature) {
                    opportunities.push(OptimizationOpportunity {
                        location: symbol.file_path.clone(),
                        kind: OptimizationKind::LazyLoading,
                        estimated_improvement: 0.6, // 60% improvement in some cases
                        effort: Effort::Small,
                        description: format!(
                            "Function '{}' loads data that could be lazy-loaded",
                            symbol.name
                        ),
                    });
                }

                // Batching opportunities
                if self.can_use_batching(signature) {
                    opportunities.push(OptimizationOpportunity {
                        location: symbol.file_path.clone(),
                        kind: OptimizationKind::Batching,
                        estimated_improvement: 0.8, // 80% improvement for I/O operations
                        effort: Effort::Medium,
                        description: format!(
                            "Function '{}' performs operations that could be batched",
                            symbol.name
                        ),
                    });
                }
            }
        }

        // Sort by estimated improvement descending
        opportunities.sort_by(|a, b| {
            b.estimated_improvement
                .partial_cmp(&a.estimated_improvement)
                .unwrap()
        });

        Ok(opportunities)
    }

    fn quality_issue_to_debt_kind(&self, issue_kind: QualityIssueKind) -> TechnicalDebtKind {
        match issue_kind {
            QualityIssueKind::DuplicateCode => TechnicalDebtKind::CodeDuplication,
            QualityIssueKind::MissingDocumentation => TechnicalDebtKind::MissingDocumentation,
            QualityIssueKind::PoorNaming => TechnicalDebtKind::InconsistentNaming,
            _ => TechnicalDebtKind::PoorArchitecture,
        }
    }

    fn create_debt_payment_plan(
        &self,
        items: &[TechnicalDebtItem],
    ) -> Result<Vec<DebtPaymentTask>> {
        let mut tasks = Vec::new();
        let mut remaining_items = items.to_vec();

        // Sort by ROI (principal / interest ratio)
        remaining_items.sort_by(|a, b| {
            let roi_a = if a.interest > 0.0 {
                a.principal / a.interest
            } else {
                a.principal
            };
            let roi_b = if b.interest > 0.0 {
                b.principal / b.interest
            } else {
                b.principal
            };
            roi_b
                .partial_cmp(&roi_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Group into tasks by priority
        let critical_items: Vec<_> = remaining_items
            .iter()
            .filter(|item| item.principal > 8.0) // High-effort items
            .cloned()
            .collect();

        if !critical_items.is_empty() {
            let total_hours: f32 = critical_items.iter().map(|i| i.principal).sum();
            let avg_roi = critical_items
                .iter()
                .map(|i| {
                    if i.interest > 0.0 {
                        i.principal / i.interest
                    } else {
                        i.principal
                    }
                })
                .sum::<f32>()
                / critical_items.len() as f32;

            tasks.push(DebtPaymentTask {
                priority: Priority::Urgent,
                items: critical_items,
                estimated_hours: total_hours,
                roi: avg_roi,
                dependencies: vec![],
            });
        }

        let medium_items: Vec<_> = remaining_items
            .iter()
            .filter(|item| item.principal > 2.0 && item.principal <= 8.0)
            .cloned()
            .collect();

        if !medium_items.is_empty() {
            let total_hours: f32 = medium_items.iter().map(|i| i.principal).sum();
            let avg_roi = medium_items
                .iter()
                .map(|i| {
                    if i.interest > 0.0 {
                        i.principal / i.interest
                    } else {
                        i.principal
                    }
                })
                .sum::<f32>()
                / medium_items.len() as f32;

            tasks.push(DebtPaymentTask {
                priority: Priority::High,
                items: medium_items,
                estimated_hours: total_hours,
                roi: avg_roi,
                dependencies: vec![],
            });
        }

        let low_items: Vec<_> = remaining_items
            .iter()
            .filter(|item| item.principal <= 2.0)
            .cloned()
            .collect();

        if !low_items.is_empty() {
            let total_hours: f32 = low_items.iter().map(|i| i.principal).sum();
            let avg_roi = low_items
                .iter()
                .map(|i| {
                    if i.interest > 0.0 {
                        i.principal / i.interest
                    } else {
                        i.principal
                    }
                })
                .sum::<f32>()
                / low_items.len() as f32;

            tasks.push(DebtPaymentTask {
                priority: Priority::Medium,
                items: low_items,
                estimated_hours: total_hours,
                roi: avg_roi,
                dependencies: vec![],
            });
        }

        Ok(tasks)
    }

    /// Detect layer violations in architecture
    fn detect_layer_violations(
        &self,
        layer_name: &str,
        modules: &[PathBuf],
        dependencies: &DependencyAnalysis,
    ) -> Result<Vec<LayerViolation>> {
        let mut violations = Vec::new();

        // Define allowed dependency directions
        let allowed_dependencies = match layer_name {
            "presentation" => vec!["business", "domain"],
            "business" => vec!["data", "domain", "infrastructure"],
            "data" => vec!["domain", "infrastructure"],
            "domain" => vec![], // Domain should not depend on other layers
            "infrastructure" => vec!["domain"],
            _ => vec!["domain", "infrastructure"],
        };

        // Check each module's dependencies
        for module in modules {
            // This would require analyzing the actual dependency graph
            // For now, return empty violations - would need full implementation
        }

        Ok(violations)
    }

    // Performance analysis helper methods

    fn count_nested_loops(&self, code: &str) -> usize {
        let loop_patterns = vec![
            r#"\bfor\s*\("#,
            r#"\bwhile\s*\("#,
            r#"\bfor\s+\w+\s+in\s+"#,
            r#"\bloop\s*\{"#,
        ];

        let mut max_nesting = 0usize;
        let mut current_nesting = 0usize;

        for line in code.lines() {
            for pattern in &loop_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(line) {
                        current_nesting += 1;
                        max_nesting = max_nesting.max(current_nesting);
                    }
                }
            }

            // Count closing braces to decrease nesting
            let closing_braces = line.matches('}').count();
            current_nesting = current_nesting.saturating_sub(closing_braces);
        }

        max_nesting
    }

    fn has_unbounded_recursion(&self, code: &str, function_name: &str) -> bool {
        let recursive_call_pattern = format!(r#"\b{}\s*\("#, regex::escape(function_name));
        if let Ok(re) = Regex::new(&recursive_call_pattern) {
            let has_recursive_call = re.is_match(code);

            // Check for termination conditions
            let termination_patterns = vec![
                r#"\bif\s*\([^)]*\)\s*\{[^}]*return"#,
                r#"\bif\s*\([^)]*\)\s*return"#,
                r#"\bmatch\s+"#,
                r#"\bswitch\s*\("#,
            ];

            let has_termination = termination_patterns.iter().any(|pattern| {
                if let Ok(re) = Regex::new(pattern) {
                    re.is_match(code)
                } else {
                    false
                }
            });

            has_recursive_call && !has_termination
        } else {
            false
        }
    }

    fn has_synchronous_io(&self, code: &str) -> bool {
        let sync_io_patterns = vec![
            r#"\bfile_get_contents\("#,
            r#"\bfopen\("#,
            r#"\bfread\("#,
            r#"\bfwrite\("#,
            r#"\bcurl_exec\("#,
            r#"\bfs\.readFileSync\("#,
            r#"\bfs\.writeFileSync\("#,
            r#"\brequests\.get\("#,
            r#"\brequests\.post\("#,
            r#"\bopen\(.*["']r["']"#,
        ];

        sync_io_patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        })
    }

    fn has_inefficient_algorithm(&self, code: &str) -> bool {
        let inefficient_patterns = vec![
            r#"\bsort\(.*\).*sort\("#,                 // Multiple sorts
            r#"\b\.find\(.*\)\.find\("#,               // Nested finds
            r#"\bfor\s*\([^}]*for\s*\([^}]*for\s*\("#, // Triple nested loops
            r#"\b\.indexOf\(.*\).*\.indexOf\("#,       // Multiple indexOf calls
        ];

        inefficient_patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        })
    }

    fn has_excessive_allocation(&self, code: &str) -> bool {
        let allocation_patterns = vec![
            r#"\bnew\s+\w+\["#, // Array allocations in loops
            r#"\bmalloc\("#,
            r#"\bcalloc\("#,
            r#"\bvec!\["#, // Rust vector allocations
            r#"\bString::new\(\)"#,
            r#"\bVec::new\(\)"#,
        ];

        let allocation_count = allocation_patterns
            .iter()
            .map(|pattern| {
                if let Ok(re) = Regex::new(pattern) {
                    re.find_iter(code).count()
                } else {
                    0
                }
            })
            .sum::<usize>();

        allocation_count > 5 // More than 5 allocations might be excessive
    }

    fn has_n_plus_one_pattern(&self, code: &str) -> bool {
        let n_plus_one_patterns = vec![
            r#"\bfor\s*\([^}]*\{[^}]*\bselect\b"#, // SQL queries in loops
            r#"\bfor\s*\([^}]*\{[^}]*\bfind\("#,   // Database finds in loops
            r#"\bfor\s*\([^}]*\{[^}]*\bquery\("#,  // Generic queries in loops
            r#"\bforEach\([^}]*\{[^}]*\bfetch\("#, // Fetch in forEach
        ];

        n_plus_one_patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        })
    }

    fn is_blocking_operation(&self, function_name: &str) -> bool {
        let blocking_keywords = vec![
            "sync", "wait", "block", "sleep", "delay", "join", "lock", "mutex",
        ];

        let name_lower = function_name.to_lowercase();
        blocking_keywords
            .iter()
            .any(|keyword| name_lower.contains(keyword))
    }

    fn is_memory_intensive(&self, function_name: &str) -> bool {
        let memory_keywords = vec![
            "cache",
            "buffer",
            "load",
            "parse",
            "process",
            "transform",
            "serialize",
            "deserialize",
        ];

        let name_lower = function_name.to_lowercase();
        memory_keywords
            .iter()
            .any(|keyword| name_lower.contains(keyword))
    }

    fn is_io_operation(&self, function_name: &str) -> bool {
        let io_keywords = vec![
            "read", "write", "save", "load", "file", "stream", "fetch", "download", "upload",
        ];

        let name_lower = function_name.to_lowercase();
        io_keywords
            .iter()
            .any(|keyword| name_lower.contains(keyword))
    }

    fn is_database_operation(&self, function_name: &str) -> bool {
        let db_keywords = vec![
            "query", "select", "insert", "update", "delete", "find", "save", "create", "execute",
            "fetch",
        ];

        let name_lower = function_name.to_lowercase();
        db_keywords
            .iter()
            .any(|keyword| name_lower.contains(keyword))
    }

    fn can_benefit_from_caching(&self, code: &str) -> bool {
        let caching_indicators = vec![
            r#"\breturn\s+[^;]*\(\s*\)"#, // Pure functions
            r#"\bcompute\w*\("#,
            r#"\bcalculate\w*\("#,
            r#"\bprocess\w*\("#,
            r#"\btransform\w*\("#,
        ];

        caching_indicators.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        })
    }

    fn can_be_parallelized(&self, code: &str) -> bool {
        let parallel_patterns = vec![
            r#"\bfor\s*\([^}]*\{[^}]*\}\s*\}"#, // Simple for loops
            r#"\bmap\("#,                       // Map operations
            r#"\bfilter\("#,                    // Filter operations
            r#"\breduce\("#,                    // Reduce operations
        ];

        let has_loop = parallel_patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        });

        // Check for absence of dependencies that prevent parallelization
        let blocking_patterns = vec![
            r#"\bmutex"#,
            r#"\block"#,
            r#"\bshared\s+"#,
            r#"\bglobal\s+"#,
        ];

        let has_blocking = blocking_patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        });

        has_loop && !has_blocking
    }

    fn has_improvable_algorithm(&self, code: &str) -> bool {
        let improvable_patterns = vec![
            r#"\bbubble.*sort"#,
            r#"\blinear.*search"#,
            r#"\bn\s*\*\s*n"#, // O(n²) mentions
            r#"\bbrute.*force"#,
        ];

        improvable_patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        })
    }

    fn has_suboptimal_data_structures(&self, code: &str) -> bool {
        let suboptimal_patterns = vec![
            r#"\bArray\(\)"#,  // Should use specific types
            r#"\bObject\(\)"#, // Should use Map for key-value
            r#"\blinkedlist"#, // Often arrays are better
        ];

        suboptimal_patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        })
    }

    fn can_use_lazy_loading(&self, code: &str) -> bool {
        let lazy_loading_patterns = vec![
            r#"\bload.*all"#,
            r#"\bfetch.*all"#,
            r#"\bget.*all"#,
            r#"\bselect\s+\*"#,
        ];

        lazy_loading_patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        })
    }

    fn can_use_batching(&self, code: &str) -> bool {
        let batching_patterns = vec![
            r#"\bfor\s*\([^}]*\{[^}]*\binsert\("#, // Multiple inserts
            r#"\bfor\s*\([^}]*\{[^}]*\bupdate\("#, // Multiple updates
            r#"\bfor\s*\([^}]*\{[^}]*\bfetch\("#,  // Multiple fetches
            r#"\bfor\s*\([^}]*\{[^}]*\bsend\("#,   // Multiple sends
        ];

        batching_patterns.iter().any(|pattern| {
            if let Ok(re) = Regex::new(pattern) {
                re.is_match(code)
            } else {
                false
            }
        })
    }
}

// Architecture pattern detectors

struct MVCDetector;

impl ArchitectureDetector for MVCDetector {
    fn detect(
        &self,
        symbols: &[SymbolEntry],
        dependencies: &DependencyAnalysis,
    ) -> Option<(ArchitecturePattern, f32)> {
        let mut mvc_score = 0.0;
        let mut indicators = 0;

        // Check for controller pattern
        let controllers = symbols
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains("controller")
                    || s.file_path.to_string_lossy().contains("controller")
            })
            .count();

        if controllers > 0 {
            mvc_score += 0.3;
            indicators += 1;
        }

        // Check for model pattern
        let models = symbols
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains("model")
                    || s.file_path.to_string_lossy().contains("model")
            })
            .count();

        if models > 0 {
            mvc_score += 0.3;
            indicators += 1;
        }

        // Check for view pattern
        let views = symbols
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains("view")
                    || s.file_path.to_string_lossy().contains("view")
                    || s.file_path.to_string_lossy().contains("template")
            })
            .count();

        if views > 0 {
            mvc_score += 0.3;
            indicators += 1;
        }

        if indicators >= 2 {
            Some((ArchitecturePattern::MVC, mvc_score))
        } else {
            None
        }
    }
}

struct CleanArchitectureDetector;

impl ArchitectureDetector for CleanArchitectureDetector {
    fn detect(
        &self,
        symbols: &[SymbolEntry],
        dependencies: &DependencyAnalysis,
    ) -> Option<(ArchitecturePattern, f32)> {
        let mut clean_score = 0.0;

        // Check for use cases
        let use_cases = symbols
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains("usecase")
                    || s.file_path.to_string_lossy().contains("usecase")
                    || s.name.to_lowercase().contains("interactor")
            })
            .count();

        if use_cases > 0 {
            clean_score += 0.4;
        }

        // Check for entities
        let entities = symbols
            .iter()
            .filter(|s| {
                s.file_path.to_string_lossy().contains("entity")
                    || s.file_path.to_string_lossy().contains("domain")
            })
            .count();

        if entities > 0 {
            clean_score += 0.3;
        }

        // Check for gateways/repositories
        let gateways = symbols
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains("gateway")
                    || s.name.to_lowercase().contains("repository")
            })
            .count();

        if gateways > 0 {
            clean_score += 0.3;
        }

        if clean_score >= 0.6 {
            Some((ArchitecturePattern::CleanArchitecture, clean_score))
        } else {
            None
        }
    }
}

struct LayeredArchitectureDetector;

impl ArchitectureDetector for LayeredArchitectureDetector {
    fn detect(
        &self,
        symbols: &[SymbolEntry],
        dependencies: &DependencyAnalysis,
    ) -> Option<(ArchitecturePattern, f32)> {
        // Check for common layer names
        let presentation_layer = symbols.iter().any(|s| {
            s.file_path.to_string_lossy().contains("presentation")
                || s.file_path.to_string_lossy().contains("ui")
        });

        let business_layer = symbols.iter().any(|s| {
            s.file_path.to_string_lossy().contains("business")
                || s.file_path.to_string_lossy().contains("service")
                || s.file_path.to_string_lossy().contains("logic")
        });

        let data_layer = symbols.iter().any(|s| {
            s.file_path.to_string_lossy().contains("data")
                || s.file_path.to_string_lossy().contains("persistence")
                || s.file_path.to_string_lossy().contains("repository")
        });

        let layers_found = [presentation_layer, business_layer, data_layer]
            .iter()
            .filter(|&&found| found)
            .count();

        if layers_found >= 2 {
            let confidence = layers_found as f32 / 3.0;
            Some((ArchitecturePattern::LayeredArchitecture, confidence))
        } else {
            None
        }
    }
}

struct MicroservicesDetector;

impl ArchitectureDetector for MicroservicesDetector {
    fn detect(
        &self,
        symbols: &[SymbolEntry],
        dependencies: &DependencyAnalysis,
    ) -> Option<(ArchitecturePattern, f32)> {
        // Check for service boundaries
        let service_indicators = symbols
            .iter()
            .filter(|s| {
                s.file_path.to_string_lossy().contains("service")
                    || s.name.to_lowercase().ends_with("service")
                    || s.file_path.to_string_lossy().contains("api")
            })
            .count();

        // Check for messaging/events
        let messaging_indicators = symbols
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains("event")
                    || s.name.to_lowercase().contains("message")
                    || s.name.to_lowercase().contains("queue")
            })
            .count();

        if service_indicators > 5 && messaging_indicators > 0 {
            let confidence = ((service_indicators + messaging_indicators) as f32 / 20.0).min(1.0);
            Some((ArchitecturePattern::Microservices, confidence))
        } else {
            None
        }
    }
}

// Quality assessors

struct ComplexityAssessor;

impl QualityAssessor for ComplexityAssessor {
    fn assess(&self, symbols: &[SymbolEntry], _metrics: &CodeMetrics) -> Vec<QualityIssue> {
        let mut issues = Vec::new();

        for symbol in symbols {
            // Check cyclomatic complexity
            if symbol.complexity > 10 {
                let severity = match symbol.complexity {
                    c if c > 30 => Severity::Critical,
                    c if c > 20 => Severity::High,
                    c if c > 15 => Severity::Medium,
                    _ => Severity::Low,
                };

                issues.push(QualityIssue {
                    kind: QualityIssueKind::HighComplexity,
                    location: symbol.file_path.clone(),
                    line: symbol.start_pos.line,
                    severity,
                    message: format!(
                        "Function '{}' has cyclomatic complexity of {} (threshold: 10)",
                        symbol.name, symbol.complexity
                    ),
                    remediation_time: symbol.complexity * 5,
                });
            }

            // Check for long methods based on signature or type
            if let Some(signature) = &symbol.signature {
                let line_count = signature.lines().count();
                if line_count > 50 {
                    issues.push(QualityIssue {
                        kind: QualityIssueKind::LongMethod,
                        location: symbol.file_path.clone(),
                        line: symbol.start_pos.line,
                        severity: if line_count > 100 {
                            Severity::High
                        } else {
                            Severity::Medium
                        },
                        message: format!(
                            "Method '{}' is too long ({} lines, threshold: 50)",
                            symbol.name, line_count
                        ),
                        remediation_time: (line_count / 10) as u32 * 15, // 15 min per 10 lines over threshold
                    });
                }
            }

            // Check for large classes (based on number of methods)
            if symbol.kind == SymbolKind::Class {
                // This would need proper class analysis - placeholder for now
                let method_count = 1; // Placeholder
                if method_count > 20 {
                    issues.push(QualityIssue {
                        kind: QualityIssueKind::LargeClass,
                        location: symbol.file_path.clone(),
                        line: symbol.start_pos.line,
                        severity: Severity::Medium,
                        message: format!(
                            "Class '{}' has too many methods ({}, threshold: 20)",
                            symbol.name, method_count
                        ),
                        remediation_time: 120, // 2 hours to refactor large class
                    });
                }
            }

            // Check for missing documentation
            if symbol.documentation.is_none()
                && (symbol.kind == SymbolKind::Function || symbol.kind == SymbolKind::Class)
                && symbol.visibility.as_deref() == Some("public")
            {
                issues.push(QualityIssue {
                    kind: QualityIssueKind::MissingDocumentation,
                    location: symbol.file_path.clone(),
                    line: symbol.start_pos.line,
                    severity: Severity::Low,
                    message: format!(
                        "Public {} '{}' lacks documentation",
                        format!("{:?}", symbol.kind).to_lowercase(),
                        symbol.name
                    ),
                    remediation_time: 15, // 15 minutes to add documentation
                });
            }
        }

        issues
    }
}

struct DuplicationAssessor;

impl QualityAssessor for DuplicationAssessor {
    fn assess(&self, symbols: &[SymbolEntry], _metrics: &CodeMetrics) -> Vec<QualityIssue> {
        let mut issues = Vec::new();
        let mut name_counts: HashMap<String, Vec<&SymbolEntry>> = HashMap::new();

        // Group symbols by name to detect potential duplicates
        for symbol in symbols {
            name_counts
                .entry(symbol.name.clone())
                .or_insert_with(Vec::new)
                .push(symbol);
        }

        // Check for duplicate names (simplified duplication detection)
        for (name, symbol_list) in name_counts {
            if symbol_list.len() > 1 {
                // Check if they're actually different implementations
                let different_files: HashSet<&PathBuf> =
                    symbol_list.iter().map(|s| &s.file_path).collect();

                if different_files.len() > 1 && symbol_list.len() > 2 {
                    // Multiple symbols with same name in different files - potential duplication
                    for symbol in symbol_list {
                        issues.push(QualityIssue {
                            kind: QualityIssueKind::DuplicateCode,
                            location: symbol.file_path.clone(),
                            line: symbol.start_pos.line,
                            severity: Severity::Medium,
                            message: format!(
                                "Potential duplicate symbol '{}' found in {} files",
                                name,
                                different_files.len()
                            ),
                            remediation_time: 30, // 30 minutes to review and refactor
                        });
                    }
                }
            }
        }

        // Check for similar signatures (basic implementation)
        let mut signature_groups: HashMap<String, Vec<&SymbolEntry>> = HashMap::new();
        for symbol in symbols {
            if let Some(sig) = &symbol.signature {
                // Normalize signature for comparison
                let normalized = sig
                    .to_lowercase()
                    .replace(" ", "")
                    .replace("\t", "")
                    .replace("\n", "");

                signature_groups
                    .entry(normalized)
                    .or_insert_with(Vec::new)
                    .push(symbol);
            }
        }

        // Check for duplicate signatures
        for (signature, symbol_list) in signature_groups {
            if symbol_list.len() > 1 && signature.len() > 20 {
                // Only check substantial signatures
                let different_files: HashSet<&PathBuf> =
                    symbol_list.iter().map(|s| &s.file_path).collect();

                if different_files.len() > 1 {
                    for symbol in symbol_list {
                        issues.push(QualityIssue {
                            kind: QualityIssueKind::DuplicateCode,
                            location: symbol.file_path.clone(),
                            line: symbol.start_pos.line,
                            severity: Severity::High,
                            message: format!(
                                "Duplicate function signature detected for '{}'",
                                symbol.name
                            ),
                            remediation_time: 60, // 1 hour to refactor duplicated logic
                        });
                    }
                }
            }
        }

        issues
    }
}

struct NamingAssessor;

impl QualityAssessor for NamingAssessor {
    fn assess(&self, symbols: &[SymbolEntry], _metrics: &CodeMetrics) -> Vec<QualityIssue> {
        let mut issues = Vec::new();

        for symbol in symbols {
            // Check for poor naming
            if symbol.name.len() < 3 || symbol.name.chars().all(|c| c.is_uppercase()) {
                issues.push(QualityIssue {
                    kind: QualityIssueKind::PoorNaming,
                    location: symbol.file_path.clone(),
                    line: symbol.start_pos.line,
                    severity: Severity::Low,
                    message: format!("Symbol '{}' has poor naming convention", symbol.name),
                    remediation_time: 5,
                });
            }
        }

        issues
    }
}

// Security scanner

struct BasicSecurityScanner;

impl SecurityScanner for BasicSecurityScanner {
    fn scan(&self, content: &str, path: &Path) -> Vec<SecurityVulnerability> {
        let mut vulnerabilities = Vec::new();

        // OWASP Top 10 2021 patterns
        let patterns = vec![
            // A01:2021 – Broken Access Control
            (
                r#"(?i)\$_GET\[\s*["'][^"']*["']\s*\]"#,
                VulnerabilityKind::UnvalidatedInput,
                "CWE-79",
                "Validate and sanitize all user input",
            ),
            (
                r#"(?i)\$_POST\[\s*["'][^"']*["']\s*\]"#,
                VulnerabilityKind::UnvalidatedInput,
                "CWE-79",
                "Validate and sanitize all user input",
            ),
            // A02:2021 – Cryptographic Failures
            (
                r#"(?i)password\s*=\s*["'][^"']+["']"#,
                VulnerabilityKind::HardcodedSecret,
                "CWE-798",
                "Use environment variables or secure vaults for secrets",
            ),
            (
                r#"(?i)api[_-]?key\s*[=:]\s*["'][^"']+["']"#,
                VulnerabilityKind::HardcodedSecret,
                "CWE-798",
                "Use environment variables or secure vaults for API keys",
            ),
            (
                r#"(?i)secret\s*[=:]\s*["'][^"']+["']"#,
                VulnerabilityKind::HardcodedSecret,
                "CWE-798",
                "Use secure secret management",
            ),
            (
                r#"(?i)token\s*[=:]\s*["'][^"']+["']"#,
                VulnerabilityKind::HardcodedSecret,
                "CWE-798",
                "Use secure token management",
            ),
            (
                r#"(?i)md5\("#,
                VulnerabilityKind::WeakCrypto,
                "CWE-327",
                "Use stronger hashing algorithms like bcrypt or SHA-256",
            ),
            (
                r#"(?i)sha1\("#,
                VulnerabilityKind::WeakCrypto,
                "CWE-327",
                "Use stronger hashing algorithms like SHA-256 or better",
            ),
            // A03:2021 – Injection
            (
                r#"(?i)select\s+.*\s+from\s+.*\s+where.*["']\s*\+\s*"#,
                VulnerabilityKind::SqlInjection,
                "CWE-89",
                "Use parameterized queries or prepared statements",
            ),
            (
                r#"(?i)\$\{[^}]*\}"#,
                VulnerabilityKind::SqlInjection,
                "CWE-89",
                "Use parameterized queries instead of string interpolation",
            ),
            (
                r#"(?i)exec\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
                "CWE-78",
                "Validate and sanitize command inputs",
            ),
            (
                r#"(?i)system\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
                "CWE-78",
                "Avoid using system() with user input",
            ),
            (
                r#"(?i)eval\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
                "CWE-95",
                "Avoid using eval() with user input",
            ),
            // A04:2021 – Insecure Design (Path Traversal)
            (
                r#"(?i)\.\.[\\/]"#,
                VulnerabilityKind::PathTraversal,
                "CWE-22",
                "Validate file paths and use absolute paths",
            ),
            (
                r#"(?i)file_get_contents\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::PathTraversal,
                "CWE-22",
                "Validate file paths before reading",
            ),
            // A05:2021 – Security Misconfiguration
            (
                r#"(?i)debug\s*[=:]\s*true"#,
                VulnerabilityKind::UnvalidatedInput,
                "CWE-489",
                "Disable debug mode in production",
            ),
            (
                r#"(?i)error_reporting\s*\(\s*E_ALL"#,
                VulnerabilityKind::UnvalidatedInput,
                "CWE-209",
                "Disable detailed error reporting in production",
            ),
            // A06:2021 – Vulnerable and Outdated Components (would need dependency analysis)

            // A07:2021 – Identification and Authentication Failures
            (
                r#"(?i)rand\s*\(\s*\)"#,
                VulnerabilityKind::InsecureRandom,
                "CWE-330",
                "Use cryptographically secure random number generators",
            ),
            (
                r#"(?i)mt_rand\s*\(\s*\)"#,
                VulnerabilityKind::InsecureRandom,
                "CWE-330",
                "Use cryptographically secure random number generators",
            ),
            // A08:2021 – Software and Data Integrity Failures

            // A09:2021 – Security Logging and Monitoring Failures

            // A10:2021 – Server-Side Request Forgery (SSRF)
            (
                r#"(?i)curl_setopt\s*\([^,]*,\s*CURLOPT_URL\s*,\s*\$[^)]*\)"#,
                VulnerabilityKind::SSRF,
                "CWE-918",
                "Validate and whitelist URLs for external requests",
            ),
            (
                r#"(?i)file_get_contents\s*\(["']https?://[^"']*\$[^"']*["']\)"#,
                VulnerabilityKind::SSRF,
                "CWE-918",
                "Validate external URLs",
            ),
            // Cross-Site Scripting (XSS)
            (
                r#"(?i)echo\s+\$[^;]*;"#,
                VulnerabilityKind::XSS,
                "CWE-79",
                "Use proper output encoding/escaping",
            ),
            (
                r#"(?i)print\s+\$[^;]*;"#,
                VulnerabilityKind::XSS,
                "CWE-79",
                "Use proper output encoding/escaping",
            ),
            (
                r#"(?i)innerHTML\s*=\s*[^;]*\$[^;]*;"#,
                VulnerabilityKind::XSS,
                "CWE-79",
                "Use proper DOM manipulation methods",
            ),
            // XXE (XML External Entity)
            (
                r#"(?i)simplexml_load_string\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::XXE,
                "CWE-611",
                "Disable external entity processing in XML parsers",
            ),
            (
                r#"(?i)DOMDocument\s*\(\s*\)"#,
                VulnerabilityKind::XXE,
                "CWE-611",
                "Configure XML parser to prevent XXE attacks",
            ),
        ];

        for (pattern, kind, cwe, remediation) in patterns {
            if let Ok(re) = Regex::new(pattern) {
                for (line_no, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        let severity = match kind {
                            VulnerabilityKind::HardcodedSecret => Severity::Critical,
                            VulnerabilityKind::SqlInjection => Severity::Critical,
                            VulnerabilityKind::CommandInjection => Severity::Critical,
                            VulnerabilityKind::XSS => Severity::High,
                            VulnerabilityKind::PathTraversal => Severity::High,
                            VulnerabilityKind::SSRF => Severity::High,
                            VulnerabilityKind::XXE => Severity::High,
                            VulnerabilityKind::WeakCrypto => Severity::Medium,
                            VulnerabilityKind::InsecureRandom => Severity::Medium,
                            VulnerabilityKind::UnvalidatedInput => Severity::Medium,
                        };

                        vulnerabilities.push(SecurityVulnerability {
                            kind,
                            location: path.to_path_buf(),
                            line: line_no + 1, // 1-based line numbers
                            severity,
                            description: format!(
                                "{:?} vulnerability detected in line: {}",
                                kind,
                                line.trim()
                            ),
                            cwe_id: Some(cwe.to_string()),
                            remediation: remediation.to_string(),
                        });
                    }
                }
            }
        }

        // Additional language-specific patterns
        self.scan_language_specific(content, path, &mut vulnerabilities);

        vulnerabilities
    }
}

impl BasicSecurityScanner {
    fn scan_language_specific(
        &self,
        content: &str,
        path: &Path,
        vulnerabilities: &mut Vec<SecurityVulnerability>,
    ) {
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        match extension {
            "js" | "ts" | "jsx" | "tsx" => self.scan_javascript(content, path, vulnerabilities),
            "py" => self.scan_python(content, path, vulnerabilities),
            "java" => self.scan_java(content, path, vulnerabilities),
            "rs" => self.scan_rust(content, path, vulnerabilities),
            "go" => self.scan_go(content, path, vulnerabilities),
            _ => {}
        }
    }

    fn scan_javascript(
        &self,
        content: &str,
        path: &Path,
        vulnerabilities: &mut Vec<SecurityVulnerability>,
    ) {
        let js_patterns = vec![
            (
                r#"(?i)document\.write\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::XSS,
            ),
            (
                r#"(?i)eval\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
            ),
            (
                r#"(?i)setTimeout\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
            ),
            (
                r#"(?i)setInterval\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
            ),
        ];

        self.apply_patterns(js_patterns, content, path, vulnerabilities);
    }

    fn scan_python(
        &self,
        content: &str,
        path: &Path,
        vulnerabilities: &mut Vec<SecurityVulnerability>,
    ) {
        let py_patterns = vec![
            (
                r#"(?i)exec\s*\([^)]*input[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
            ),
            (
                r#"(?i)eval\s*\([^)]*input[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
            ),
            (
                r#"(?i)pickle\.loads\s*\([^)]*\)"#,
                VulnerabilityKind::UnvalidatedInput,
            ),
        ];

        self.apply_patterns(py_patterns, content, path, vulnerabilities);
    }

    fn scan_java(
        &self,
        content: &str,
        path: &Path,
        vulnerabilities: &mut Vec<SecurityVulnerability>,
    ) {
        let java_patterns = vec![
            (
                r#"(?i)Runtime\.getRuntime\(\)\.exec"#,
                VulnerabilityKind::CommandInjection,
            ),
            (
                r#"(?i)ProcessBuilder\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
            ),
        ];

        self.apply_patterns(java_patterns, content, path, vulnerabilities);
    }

    fn scan_rust(
        &self,
        content: &str,
        path: &Path,
        vulnerabilities: &mut Vec<SecurityVulnerability>,
    ) {
        let rust_patterns = vec![
            (r#"(?i)unsafe\s*\{"#, VulnerabilityKind::UnvalidatedInput),
            (r#"(?i)transmute\s*\("#, VulnerabilityKind::UnvalidatedInput),
        ];

        self.apply_patterns(rust_patterns, content, path, vulnerabilities);
    }

    fn scan_go(
        &self,
        content: &str,
        path: &Path,
        vulnerabilities: &mut Vec<SecurityVulnerability>,
    ) {
        let go_patterns = vec![
            (
                r#"(?i)exec\.Command\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::CommandInjection,
            ),
            (
                r#"(?i)fmt\.Sprintf\s*\([^)]*\$[^)]*\)"#,
                VulnerabilityKind::UnvalidatedInput,
            ),
        ];

        self.apply_patterns(go_patterns, content, path, vulnerabilities);
    }

    fn apply_patterns(
        &self,
        patterns: Vec<(&str, VulnerabilityKind)>,
        content: &str,
        path: &Path,
        vulnerabilities: &mut Vec<SecurityVulnerability>,
    ) {
        for (pattern, kind) in patterns {
            if let Ok(re) = Regex::new(pattern) {
                for (line_no, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        vulnerabilities.push(SecurityVulnerability {
                            kind,
                            location: path.to_path_buf(),
                            line: line_no + 1,
                            severity: Severity::Medium,
                            description: format!(
                                "Language-specific {:?} pattern detected: {}",
                                kind,
                                line.trim()
                            ),
                            cwe_id: None,
                            remediation: "Review code for security implications".to_string(),
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvc_detector() {
        let detector = MVCDetector;
        let symbols = vec![
            SymbolEntry {
                id: "1".to_string(),
                name: "UserController".to_string(),
                file_path: PathBuf::from("controllers/user.rs"),
                ..Default::default()
            },
            SymbolEntry {
                id: "2".to_string(),
                name: "UserModel".to_string(),
                file_path: PathBuf::from("models/user.rs"),
                ..Default::default()
            },
        ];

        let result = detector.detect(&symbols, &DependencyAnalysis::default());
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, ArchitecturePattern::MVC);
    }

    #[test]
    fn test_quality_score_calculation() {
        let analyzer = RepositoryAnalyzer::new(
            Arc::new(
                SymbolIndexer::new(Arc::new(DatabaseManager::default()))
                    .await
                    .unwrap(),
            ),
            Arc::new(DependencyAnalyzer::new().await.unwrap()),
        )
        .await
        .unwrap();

        let metrics = QualityMetrics {
            cyclomatic_complexity: 15.0,
            duplication_ratio: 0.15,
            documentation_coverage: 0.3,
            ..Default::default()
        };

        let score = analyzer.calculate_quality_score(&metrics, &[]);
        assert!(score < 10.0);
        assert!(score > 0.0);
    }
}
