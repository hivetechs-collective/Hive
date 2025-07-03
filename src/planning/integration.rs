//! Repository Intelligence Integration for Planning Engine
//! 
//! Integrates planning with repository analysis for context-aware planning

use crate::core::error::{HiveResult, HiveError};
use crate::core::semantic::SemanticIndex;
use crate::planning::types::*;
use crate::analysis::AnalysisEngine;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Repository context for planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryContext {
    pub project_structure: ProjectStructure,
    pub code_metrics: CodeMetrics,
    pub dependencies: DependencyInfo,
    pub quality_indicators: QualityIndicators,
    pub team_patterns: TeamPatterns,
}

/// Project structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub root_path: PathBuf,
    pub source_directories: Vec<PathBuf>,
    pub test_directories: Vec<PathBuf>,
    pub documentation_paths: Vec<PathBuf>,
    pub configuration_files: Vec<PathBuf>,
    pub total_files: usize,
    pub languages: HashMap<String, LanguageStats>,
}

/// Language statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub file_count: usize,
    pub line_count: usize,
    pub percentage: f32,
}

/// Code metrics for planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub complexity_score: f32,
    pub test_coverage: Option<f32>,
    pub technical_debt: TechnicalDebt,
}

/// Technical debt assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebt {
    pub score: f32,
    pub hotspots: Vec<CodeHotspot>,
    pub refactoring_candidates: Vec<RefactoringCandidate>,
}

/// Code hotspot (frequently changed, complex area)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeHotspot {
    pub file_path: PathBuf,
    pub change_frequency: f32,
    pub complexity: f32,
    pub coupled_files: Vec<PathBuf>,
}

/// Refactoring candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringCandidate {
    pub file_path: PathBuf,
    pub reason: String,
    pub priority: Priority,
    pub estimated_effort: chrono::Duration,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub external_dependencies: Vec<ExternalDependency>,
    pub internal_modules: Vec<InternalModule>,
    pub circular_dependencies: Vec<CircularDependency>,
    pub dependency_health: f32,
}

/// External dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDependency {
    pub name: String,
    pub version: String,
    pub is_outdated: bool,
    pub security_issues: Vec<SecurityIssue>,
    pub license: String,
}

/// Security issue in dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub severity: String,
    pub description: String,
    pub cve: Option<String>,
}

/// Internal module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalModule {
    pub name: String,
    pub path: PathBuf,
    pub cohesion_score: f32,
    pub coupling_score: f32,
    pub size: usize,
}

/// Circular dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub modules: Vec<String>,
    pub severity: RiskSeverity,
}

/// Quality indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIndicators {
    pub code_quality_score: f32,
    pub documentation_coverage: f32,
    pub test_quality: TestQuality,
    pub maintainability_index: f32,
    pub security_score: f32,
}

/// Test quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestQuality {
    pub coverage: f32,
    pub test_count: usize,
    pub test_success_rate: f32,
    pub test_execution_time: chrono::Duration,
}

/// Team patterns from repository history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPatterns {
    pub commit_frequency: f32,
    pub average_pr_size: usize,
    pub code_review_time: chrono::Duration,
    pub common_work_hours: Vec<u8>,
    pub collaboration_score: f32,
}

/// Repository intelligence integration
pub struct RepositoryIntelligence {
    semantic_index: Option<SemanticIndex>,
    analysis_engine: AnalysisEngine,
}

impl RepositoryIntelligence {
    /// Create new repository intelligence instance
    pub async fn new() -> HiveResult<Self> {
        Ok(Self {
            semantic_index: None,
            analysis_engine: AnalysisEngine::new(),
        })
    }

    /// Analyze repository for planning context
    pub async fn analyze_repository(&mut self, path: &Path) -> HiveResult<RepositoryContext> {
        // Initialize semantic index for the repository
        self.semantic_index = Some(SemanticIndex::new().await?);
        
        // Gather project structure
        let project_structure = self.analyze_structure(path).await?;
        
        // Calculate code metrics
        let code_metrics = self.calculate_metrics(path).await?;
        
        // Analyze dependencies
        let dependencies = self.analyze_dependencies(path).await?;
        
        // Assess quality indicators
        let quality_indicators = self.assess_quality(path).await?;
        
        // Analyze team patterns
        let team_patterns = self.analyze_team_patterns(path).await?;
        
        Ok(RepositoryContext {
            project_structure,
            code_metrics,
            dependencies,
            quality_indicators,
            team_patterns,
        })
    }

    /// Get relevant context for a specific task
    pub async fn get_task_context(&self, task: &Task, repository_context: &RepositoryContext) -> HiveResult<TaskContext> {
        let mut relevant_files: Vec<PathBuf> = Vec::new();
        let mut related_modules = Vec::new();
        let mut potential_impacts = Vec::new();
        
        // Find files related to the task
        if let Some(ref index) = self.semantic_index {
            // Note: find_related_files method not implemented yet
            relevant_files = vec![];
        }
        
        // Identify related modules
        for file in &relevant_files {
            if let Some(module) = self.find_module_for_file(file, repository_context) {
                if !related_modules.contains(&module) {
                    related_modules.push(module);
                }
            }
        }
        
        // Assess potential impacts
        potential_impacts = self.assess_task_impacts(task, &relevant_files, repository_context)?;
        
        Ok(TaskContext {
            relevant_files,
            related_modules,
            potential_impacts,
            existing_patterns: self.find_existing_patterns(&relevant_files)?,
            suggested_approach: self.suggest_approach(task, repository_context)?,
        })
    }

    /// Enhance planning context with repository intelligence
    pub fn enhance_planning_context(
        &self,
        mut context: PlanningContext,
        repository_context: &RepositoryContext,
    ) -> HiveResult<PlanningContext> {
        // Update technology stack from actual repository
        context.technology_stack = repository_context.project_structure.languages
            .keys()
            .cloned()
            .collect();
        
        // Set existing codebase flag
        context.existing_codebase = true;
        
        // Add repository path
        context.repository_path = Some(repository_context.project_structure.root_path.to_string_lossy().to_string());
        
        // Infer project type from structure
        if context.project_type == ProjectType::Other("Unknown".to_string()) {
            context.project_type = self.infer_project_type(repository_context)?;
        }
        
        Ok(context)
    }

    // Private helper methods

    async fn analyze_structure(&self, path: &Path) -> HiveResult<ProjectStructure> {
        let mut structure = ProjectStructure {
            root_path: path.to_path_buf(),
            source_directories: Vec::new(),
            test_directories: Vec::new(),
            documentation_paths: Vec::new(),
            configuration_files: Vec::new(),
            total_files: 0,
            languages: HashMap::new(),
        };

        // Use analysis engine to scan repository
        // Note: scan_directory method not implemented yet
        let files = vec![];
        structure.total_files = files.len();

        // Categorize files and directories
        for file in files {
            let file_path = Path::new(&file.path);
            
            // Detect source directories
            if file_path.to_string_lossy().contains("src") || 
               file_path.to_string_lossy().contains("lib") {
                if let Some(parent) = file_path.parent() {
                    if !structure.source_directories.contains(&parent.to_path_buf()) {
                        structure.source_directories.push(parent.to_path_buf());
                    }
                }
            }
            
            // Detect test directories
            if file_path.to_string_lossy().contains("test") ||
               file_path.to_string_lossy().contains("spec") {
                if let Some(parent) = file_path.parent() {
                    if !structure.test_directories.contains(&parent.to_path_buf()) {
                        structure.test_directories.push(parent.to_path_buf());
                    }
                }
            }
            
            // Detect documentation
            if file_path.extension().and_then(|s| s.to_str()) == Some("md") ||
               file_path.to_string_lossy().contains("docs") {
                structure.documentation_paths.push(file_path.to_path_buf());
            }
            
            // Detect configuration files
            if self.is_config_file(file_path) {
                structure.configuration_files.push(file_path.to_path_buf());
            }
            
            // Update language statistics
            if let Some(lang) = self.detect_language(file_path) {
                let stats = structure.languages.entry(lang).or_insert(LanguageStats {
                    file_count: 0,
                    line_count: 0,
                    percentage: 0.0,
                });
                stats.file_count += 1;
                stats.line_count += file.metrics.lines;
            }
        }

        // Calculate language percentages
        let total_files = structure.total_files as f32;
        for stats in structure.languages.values_mut() {
            stats.percentage = (stats.file_count as f32 / total_files) * 100.0;
        }

        Ok(structure)
    }

    async fn calculate_metrics(&self, path: &Path) -> HiveResult<CodeMetrics> {
        // Note: scan_directory method not implemented yet
        let files = vec![];
        
        let mut total_lines = 0;
        let mut code_lines = 0;
        let mut comment_lines = 0;
        let mut complexity_sum = 0.0;
        let mut file_count = 0;
        
        for file in &files {
            total_lines += file.metrics.lines;
            code_lines += file.metrics.code_lines;
            comment_lines += file.metrics.comment_lines;
            complexity_sum += file.metrics.complexity as f32;
            file_count += 1;
        }
        
        let complexity_score = if file_count > 0 {
            complexity_sum / file_count as f32
        } else {
            0.0
        };
        
        // Analyze technical debt
        let technical_debt = self.analyze_technical_debt(&files)?;
        
        Ok(CodeMetrics {
            total_lines,
            code_lines,
            comment_lines,
            complexity_score,
            test_coverage: None, // Would need test results
            technical_debt,
        })
    }

    async fn analyze_dependencies(&self, path: &Path) -> HiveResult<DependencyInfo> {
        // This would analyze package.json, Cargo.toml, requirements.txt, etc.
        Ok(DependencyInfo {
            external_dependencies: Vec::new(),
            internal_modules: Vec::new(),
            circular_dependencies: Vec::new(),
            dependency_health: 0.8, // Placeholder
        })
    }

    async fn assess_quality(&self, path: &Path) -> HiveResult<QualityIndicators> {
        Ok(QualityIndicators {
            code_quality_score: 0.85,
            documentation_coverage: 0.7,
            test_quality: TestQuality {
                coverage: 0.75,
                test_count: 100,
                test_success_rate: 0.98,
                test_execution_time: chrono::Duration::minutes(5),
            },
            maintainability_index: 0.8,
            security_score: 0.9,
        })
    }

    async fn analyze_team_patterns(&self, _path: &Path) -> HiveResult<TeamPatterns> {
        // This would analyze git history
        Ok(TeamPatterns {
            commit_frequency: 5.2,
            average_pr_size: 150,
            code_review_time: chrono::Duration::hours(4),
            common_work_hours: vec![9, 10, 11, 12, 13, 14, 15, 16, 17],
            collaboration_score: 0.85,
        })
    }

    fn analyze_technical_debt(&self, files: &[crate::core::FileInfo]) -> HiveResult<TechnicalDebt> {
        let mut hotspots = Vec::new();
        let mut refactoring_candidates = Vec::new();
        
        for file in files {
            // High complexity files are hotspots
            if file.metrics.complexity > 20 {
                hotspots.push(CodeHotspot {
                    file_path: PathBuf::from(&file.path),
                    change_frequency: 0.0, // Would need git history
                    complexity: file.metrics.complexity as f32,
                    coupled_files: Vec::new(),
                });
            }
            
            // Large files are refactoring candidates
            if file.metrics.lines > 500 {
                refactoring_candidates.push(RefactoringCandidate {
                    file_path: PathBuf::from(&file.path),
                    reason: "File too large - consider splitting".to_string(),
                    priority: Priority::Medium,
                    estimated_effort: chrono::Duration::hours(4),
                });
            }
        }
        
        let score = (hotspots.len() + refactoring_candidates.len()) as f32 / files.len() as f32;
        
        Ok(TechnicalDebt {
            score,
            hotspots,
            refactoring_candidates,
        })
    }

    fn find_module_for_file(&self, file: &Path, context: &RepositoryContext) -> Option<String> {
        // Simple heuristic: use parent directory as module name
        file.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    fn assess_task_impacts(&self, task: &Task, files: &[PathBuf], context: &RepositoryContext) -> HiveResult<Vec<Impact>> {
        let mut impacts = Vec::new();
        
        // Check if task affects hotspots
        for hotspot in &context.code_metrics.technical_debt.hotspots {
            if files.contains(&hotspot.file_path) {
                impacts.push(Impact {
                    description: format!("Modifies high-complexity hotspot: {:?}", hotspot.file_path),
                    severity: ImpactSeverity::High,
                    affected_areas: vec![hotspot.file_path.to_string_lossy().to_string()],
                });
            }
        }
        
        // Check if task affects core modules
        if task.task_type == TaskType::Refactoring || task.task_type == TaskType::Implementation {
            impacts.push(Impact {
                description: "May affect dependent modules".to_string(),
                severity: ImpactSeverity::Medium,
                affected_areas: files.iter().map(|f| f.to_string_lossy().to_string()).collect(),
            });
        }
        
        Ok(impacts)
    }

    fn find_existing_patterns(&self, files: &[PathBuf]) -> HiveResult<Vec<Pattern>> {
        // This would analyze code patterns in the files
        Ok(vec![
            Pattern {
                name: "Error Handling".to_string(),
                description: "Uses Result<T, E> pattern for error handling".to_string(),
                examples: vec!["src/core/error.rs".to_string()],
            }
        ])
    }

    fn suggest_approach(&self, task: &Task, context: &RepositoryContext) -> HiveResult<String> {
        let suggestion = match task.task_type {
            TaskType::Implementation => {
                "Follow existing patterns in the codebase. Consider test-driven development."
            }
            TaskType::Refactoring => {
                "Start with comprehensive tests. Refactor incrementally with frequent commits."
            }
            TaskType::BugFix => {
                "Write a failing test first. Fix the issue, then verify all tests pass."
            }
            _ => {
                "Follow the project's established conventions and patterns."
            }
        };
        
        Ok(suggestion.to_string())
    }

    fn infer_project_type(&self, context: &RepositoryContext) -> HiveResult<ProjectType> {
        // Simple heuristics based on languages and files
        if context.project_structure.languages.contains_key("JavaScript") ||
           context.project_structure.languages.contains_key("TypeScript") {
            if context.project_structure.configuration_files.iter()
                .any(|f| f.file_name().and_then(|n| n.to_str()) == Some("package.json")) {
                return Ok(ProjectType::WebApplication);
            }
        }
        
        if context.project_structure.languages.contains_key("Rust") {
            if context.project_structure.configuration_files.iter()
                .any(|f| f.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml")) {
                return Ok(ProjectType::Library);
            }
        }
        
        Ok(ProjectType::Other("General".to_string()))
    }

    fn is_config_file(&self, path: &Path) -> bool {
        let config_names = ["Cargo.toml", "package.json", "requirements.txt", 
                           "Gemfile", "pom.xml", "build.gradle", ".env",
                           "config.toml", "config.yaml", "config.json"];
        
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            config_names.contains(&name)
        } else {
            false
        }
    }

    fn detect_language(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "rs" => Some("Rust"),
                "js" => Some("JavaScript"),
                "ts" | "tsx" => Some("TypeScript"),
                "py" => Some("Python"),
                "java" => Some("Java"),
                "go" => Some("Go"),
                "cpp" | "cc" | "cxx" => Some("C++"),
                "c" => Some("C"),
                "rb" => Some("Ruby"),
                "php" => Some("PHP"),
                "swift" => Some("Swift"),
                "kt" => Some("Kotlin"),
                _ => None,
            })
            .map(|s| s.to_string())
    }
}

/// Context specific to a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub relevant_files: Vec<PathBuf>,
    pub related_modules: Vec<String>,
    pub potential_impacts: Vec<Impact>,
    pub existing_patterns: Vec<Pattern>,
    pub suggested_approach: String,
}

/// Impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    pub description: String,
    pub severity: ImpactSeverity,
    pub affected_areas: Vec<String>,
}

/// Impact severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactSeverity {
    High,
    Medium,
    Low,
}

/// Code pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub examples: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_repository_intelligence_creation() {
        let intelligence = RepositoryIntelligence::new().await.unwrap();
        assert!(intelligence.semantic_index.is_none());
    }

    #[test]
    fn test_config_file_detection() {
        let intelligence = RepositoryIntelligence {
            semantic_index: None,
            analysis_engine: AnalysisEngine::new().unwrap(),
        };
        
        assert!(intelligence.is_config_file(Path::new("Cargo.toml")));
        assert!(intelligence.is_config_file(Path::new("package.json")));
        assert!(!intelligence.is_config_file(Path::new("main.rs")));
    }

    #[test]
    fn test_language_detection() {
        let intelligence = RepositoryIntelligence {
            semantic_index: None,
            analysis_engine: AnalysisEngine::new().unwrap(),
        };
        
        assert_eq!(intelligence.detect_language(Path::new("main.rs")), Some("Rust".to_string()));
        assert_eq!(intelligence.detect_language(Path::new("app.js")), Some("JavaScript".to_string()));
        assert_eq!(intelligence.detect_language(Path::new("test.py")), Some("Python".to_string()));
        assert_eq!(intelligence.detect_language(Path::new("README.md")), None);
    }
}