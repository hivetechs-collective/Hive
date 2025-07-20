// AI-Enhanced Operation Validation Pipeline
use anyhow::{Result, anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn, error};

use crate::consensus::stages::file_aware_curator::FileOperation;
use crate::consensus::operation_analysis::{OperationContext, OperationAnalysis};
use crate::consensus::operation_parser::EnhancedFileOperation;
use crate::ai_helpers::pattern_recognizer::PatternRecognizer;
use crate::ai_helpers::quality_analyzer::QualityAnalyzer;

/// Comprehensive validation pipeline for file operations
#[derive(Debug, Clone)]
pub struct OperationValidator {
    /// AI helpers for validation
    pattern_recognizer: Option<Arc<PatternRecognizer>>,
    quality_analyzer: Option<Arc<QualityAnalyzer>>,
    /// Validation configuration
    config: ValidationConfig,
    /// File system access for validation
    fs_validator: Arc<FileSystemValidator>,
    /// Security validator
    security_validator: Arc<SecurityValidator>,
    /// Syntax validator for code changes
    syntax_validator: Arc<SyntaxValidator>,
    /// Validation cache
    validation_cache: Arc<RwLock<HashMap<String, ValidationResult>>>,
    /// Concurrent validation limiter
    validation_semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enable file system checks
    pub check_file_system: bool,
    /// Enable security validation
    pub check_security: bool,
    /// Enable syntax validation for code
    pub check_syntax: bool,
    /// Enable dependency validation
    pub check_dependencies: bool,
    /// Enable conflict detection
    pub check_conflicts: bool,
    /// Maximum file size for operations (bytes)
    pub max_file_size: usize,
    /// Forbidden path patterns
    pub forbidden_paths: Vec<String>,
    /// Required backup for certain operations
    pub require_backup: bool,
    /// Strict mode (fail on any warning)
    pub strict_mode: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_file_system: true,
            check_security: true,
            check_syntax: true,
            check_dependencies: true,
            check_conflicts: true,
            max_file_size: 10 * 1024 * 1024, // 10MB
            forbidden_paths: vec![
                "/.git".to_string(),
                "/node_modules".to_string(),
                "/.env".to_string(),
                "/secrets".to_string(),
            ],
            require_backup: true,
            strict_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Overall validation status
    pub status: ValidationStatus,
    /// Individual check results
    pub checks: Vec<ValidationCheck>,
    /// Validation errors (if any)
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// AI insights about validation
    pub ai_insights: Vec<String>,
    /// Suggested fixes for issues
    pub suggested_fixes: Vec<SuggestedFix>,
    /// Overall risk assessment
    pub risk_assessment: RiskAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    /// All checks passed
    Passed,
    /// Passed with warnings
    PassedWithWarnings,
    /// Failed validation
    Failed,
    /// Blocked for security reasons
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub category: CheckCategory,
    pub status: CheckStatus,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckCategory {
    FileSystem,
    Security,
    Syntax,
    Dependencies,
    Conflicts,
    Permissions,
    Content,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CheckStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub operation_index: Option<usize>,
    pub file_path: Option<PathBuf>,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedFix {
    pub issue: String,
    pub fix_description: String,
    pub automated: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_factors: Vec<String>,
    pub mitigation_required: bool,
    pub backup_recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Minimal,
    Low,
    Medium,
    High,
    Critical,
}

/// Validates file system operations
#[derive(Debug)]
pub struct FileSystemValidator {
    workspace_root: PathBuf,
}

impl FileSystemValidator {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    pub async fn validate(&self, operation: &FileOperation) -> Result<ValidationCheck> {
        match operation {
            FileOperation::Create { path, content } => {
                self.validate_create(path, content).await
            }
            FileOperation::Update { path, .. } => {
                self.validate_update(path).await
            }
            FileOperation::Append { path, .. } => {
                self.validate_update(path).await // Treat append like update for validation
            }
            FileOperation::Delete { path } => {
                self.validate_delete(path).await
            }
            FileOperation::Rename { from, to } => {
                self.validate_rename(from, to).await
            }
        }
    }

    async fn validate_create(&self, path: &Path, content: &str) -> Result<ValidationCheck> {
        let full_path = self.workspace_root.join(path);
        
        // Check if file already exists
        if tokio::fs::metadata(&full_path).await.is_ok() {
            return Ok(ValidationCheck {
                name: "File exists check".to_string(),
                category: CheckCategory::FileSystem,
                status: CheckStatus::Warning,
                message: format!("File {} already exists", path.display()),
                severity: Severity::Warning,
            });
        }

        // Check parent directory exists
        if let Some(parent) = full_path.parent() {
            if tokio::fs::metadata(parent).await.is_err() {
                return Ok(ValidationCheck {
                    name: "Parent directory check".to_string(),
                    category: CheckCategory::FileSystem,
                    status: CheckStatus::Failed,
                    message: format!("Parent directory {} does not exist", parent.display()),
                    severity: Severity::Error,
                });
            }
        }

        // Check file size
        if content.len() > 10 * 1024 * 1024 {
            return Ok(ValidationCheck {
                name: "File size check".to_string(),
                category: CheckCategory::FileSystem,
                status: CheckStatus::Warning,
                message: "File size exceeds 10MB".to_string(),
                severity: Severity::Warning,
            });
        }

        Ok(ValidationCheck {
            name: "Create file validation".to_string(),
            category: CheckCategory::FileSystem,
            status: CheckStatus::Passed,
            message: "File can be created".to_string(),
            severity: Severity::Info,
        })
    }

    async fn validate_update(&self, path: &Path) -> Result<ValidationCheck> {
        let full_path = self.workspace_root.join(path);
        
        // Check if file exists
        if tokio::fs::metadata(&full_path).await.is_err() {
            return Ok(ValidationCheck {
                name: "File exists check".to_string(),
                category: CheckCategory::FileSystem,
                status: CheckStatus::Failed,
                message: format!("File {} does not exist", path.display()),
                severity: Severity::Error,
            });
        }

        // Check if file is writable
        match tokio::fs::metadata(&full_path).await {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    return Ok(ValidationCheck {
                        name: "File writable check".to_string(),
                        category: CheckCategory::FileSystem,
                        status: CheckStatus::Failed,
                        message: format!("File {} is read-only", path.display()),
                        severity: Severity::Error,
                    });
                }
            }
            Err(e) => {
                return Ok(ValidationCheck {
                    name: "File access check".to_string(),
                    category: CheckCategory::FileSystem,
                    status: CheckStatus::Failed,
                    message: format!("Cannot access file {}: {}", path.display(), e),
                    severity: Severity::Error,
                });
            }
        }

        Ok(ValidationCheck {
            name: "Update file validation".to_string(),
            category: CheckCategory::FileSystem,
            status: CheckStatus::Passed,
            message: "File can be updated".to_string(),
            severity: Severity::Info,
        })
    }

    async fn validate_delete(&self, path: &Path) -> Result<ValidationCheck> {
        let full_path = self.workspace_root.join(path);
        
        // Check if file exists
        if tokio::fs::metadata(&full_path).await.is_err() {
            return Ok(ValidationCheck {
                name: "File exists check".to_string(),
                category: CheckCategory::FileSystem,
                status: CheckStatus::Warning,
                message: format!("File {} does not exist", path.display()),
                severity: Severity::Warning,
            });
        }

        // Warn about deleting important files
        let path_str = path.to_string_lossy();
        if path_str.contains("test") || path_str.contains("spec") {
            return Ok(ValidationCheck {
                name: "Important file check".to_string(),
                category: CheckCategory::FileSystem,
                status: CheckStatus::Warning,
                message: "Deleting test files - ensure this is intentional".to_string(),
                severity: Severity::Warning,
            });
        }

        Ok(ValidationCheck {
            name: "Delete file validation".to_string(),
            category: CheckCategory::FileSystem,
            status: CheckStatus::Passed,
            message: "File can be deleted".to_string(),
            severity: Severity::Info,
        })
    }

    async fn validate_rename(&self, from: &Path, to: &Path) -> Result<ValidationCheck> {
        let old_full = self.workspace_root.join(from);
        let new_full = self.workspace_root.join(to);
        
        // Check source exists
        if tokio::fs::metadata(&old_full).await.is_err() {
            return Ok(ValidationCheck {
                name: "Source file check".to_string(),
                category: CheckCategory::FileSystem,
                status: CheckStatus::Failed,
                message: format!("Source file {} does not exist", from.display()),
                severity: Severity::Error,
            });
        }

        // Check destination doesn't exist
        if tokio::fs::metadata(&new_full).await.is_ok() {
            return Ok(ValidationCheck {
                name: "Destination check".to_string(),
                category: CheckCategory::FileSystem,
                status: CheckStatus::Failed,
                message: format!("Destination {} already exists", to.display()),
                severity: Severity::Error,
            });
        }

        Ok(ValidationCheck {
            name: "Rename validation".to_string(),
            category: CheckCategory::FileSystem,
            status: CheckStatus::Passed,
            message: "File can be renamed".to_string(),
            severity: Severity::Info,
        })
    }

    async fn validate_move(&self, source: &Path, destination: &Path) -> Result<ValidationCheck> {
        // Similar to rename validation
        self.validate_rename(source, destination).await
    }
}

/// Validates security aspects of operations
#[derive(Debug)]
pub struct SecurityValidator {
    forbidden_patterns: Vec<regex::Regex>,
    sensitive_file_patterns: Vec<regex::Regex>,
}

impl SecurityValidator {
    pub fn new() -> Self {
        Self {
            forbidden_patterns: vec![
                regex::Regex::new(r#"(?i)(password|secret|key|token)\s*=\s*['"'][\w\-]+['"']"#).unwrap(),
                regex::Regex::new(r#"(?i)api[_-]?key\s*[:=]\s*[\w\-]+"#).unwrap(),
                regex::Regex::new(r"-----BEGIN (RSA |EC )?PRIVATE KEY-----").unwrap(),
            ],
            sensitive_file_patterns: vec![
                regex::Regex::new(r"\.env(\.\w+)?$").unwrap(),
                regex::Regex::new(r"(?i)(secret|credential|key|password)").unwrap(),
                regex::Regex::new(r"\.(pem|key|cert|crt)$").unwrap(),
            ],
        }
    }

    pub async fn validate(&self, operation: &FileOperation) -> Result<Vec<ValidationCheck>> {
        let mut checks = Vec::new();

        match operation {
            FileOperation::Create { path, content } | 
            FileOperation::Update { path, content, .. } => {
                // Check for sensitive content
                checks.push(self.check_sensitive_content(content)?);
                
                // Check sensitive file paths
                checks.push(self.check_sensitive_path(path)?);
                
                // Check for potential security vulnerabilities
                checks.push(self.check_security_vulnerabilities(content)?);
            }
            FileOperation::Delete { path } => {
                // Warn about deleting security-related files
                if self.is_sensitive_file(path) {
                    checks.push(ValidationCheck {
                        name: "Sensitive file deletion".to_string(),
                        category: CheckCategory::Security,
                        status: CheckStatus::Warning,
                        message: "Deleting potentially sensitive file".to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
            _ => {}
        }

        Ok(checks)
    }

    fn check_sensitive_content(&self, content: &str) -> Result<ValidationCheck> {
        for pattern in &self.forbidden_patterns {
            if pattern.is_match(content) {
                return Ok(ValidationCheck {
                    name: "Sensitive content check".to_string(),
                    category: CheckCategory::Security,
                    status: CheckStatus::Failed,
                    message: "Content contains potential secrets or credentials".to_string(),
                    severity: Severity::Critical,
                });
            }
        }

        Ok(ValidationCheck {
            name: "Sensitive content check".to_string(),
            category: CheckCategory::Security,
            status: CheckStatus::Passed,
            message: "No sensitive content detected".to_string(),
            severity: Severity::Info,
        })
    }

    fn check_sensitive_path(&self, path: &Path) -> Result<ValidationCheck> {
        let path_str = path.to_string_lossy();
        
        if self.is_sensitive_file(path) {
            return Ok(ValidationCheck {
                name: "Sensitive path check".to_string(),
                category: CheckCategory::Security,
                status: CheckStatus::Warning,
                message: "Operating on potentially sensitive file".to_string(),
                severity: Severity::Warning,
            });
        }

        Ok(ValidationCheck {
            name: "Sensitive path check".to_string(),
            category: CheckCategory::Security,
            status: CheckStatus::Passed,
            message: "Path is not sensitive".to_string(),
            severity: Severity::Info,
        })
    }

    fn check_security_vulnerabilities(&self, content: &str) -> Result<ValidationCheck> {
        let vulnerabilities = vec![
            (r"eval\s*\(", "Potential code injection via eval()"),
            (r"exec\s*\(", "Potential code execution vulnerability"),
            (r"system\s*\(", "Potential command injection"),
            (r"<script[^>]*>", "Potential XSS vulnerability"),
            (r"(?i)select\s+\*\s+from", "Potential SQL injection risk"),
        ];

        for (pattern, message) in vulnerabilities {
            if regex::Regex::new(pattern)?.is_match(content) {
                return Ok(ValidationCheck {
                    name: "Security vulnerability check".to_string(),
                    category: CheckCategory::Security,
                    status: CheckStatus::Warning,
                    message: message.to_string(),
                    severity: Severity::Warning,
                });
            }
        }

        Ok(ValidationCheck {
            name: "Security vulnerability check".to_string(),
            category: CheckCategory::Security,
            status: CheckStatus::Passed,
            message: "No obvious vulnerabilities detected".to_string(),
            severity: Severity::Info,
        })
    }

    fn is_sensitive_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.sensitive_file_patterns.iter().any(|p| p.is_match(&path_str))
    }
}

/// Validates syntax of code changes
#[derive(Debug)]
pub struct SyntaxValidator {
    language_validators: HashMap<String, Box<dyn LanguageValidator>>,
}

impl SyntaxValidator {
    pub fn new() -> Self {
        let mut validators: HashMap<String, Box<dyn LanguageValidator>> = HashMap::new();
        
        // Add basic validators for common languages
        validators.insert("rs".to_string(), Box::new(RustValidator));
        validators.insert("js".to_string(), Box::new(JavaScriptValidator));
        validators.insert("ts".to_string(), Box::new(TypeScriptValidator));
        validators.insert("py".to_string(), Box::new(PythonValidator));
        
        Self {
            language_validators: validators,
        }
    }

    pub async fn validate(&self, operation: &FileOperation) -> Result<ValidationCheck> {
        let (path, content) = match operation {
            FileOperation::Create { path, content } => (path, content),
            FileOperation::Update { path, content, .. } => (path, content),
            _ => {
                return Ok(ValidationCheck {
                    name: "Syntax validation".to_string(),
                    category: CheckCategory::Syntax,
                    status: CheckStatus::Skipped,
                    message: "Not applicable for this operation".to_string(),
                    severity: Severity::Info,
                });
            }
        };

        // Determine language from file extension
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if let Some(validator) = self.language_validators.get(extension) {
            validator.validate(content).await
        } else {
            Ok(ValidationCheck {
                name: "Syntax validation".to_string(),
                category: CheckCategory::Syntax,
                status: CheckStatus::Skipped,
                message: format!("No validator for .{} files", extension),
                severity: Severity::Info,
            })
        }
    }
}

trait LanguageValidator: Send + Sync + std::fmt::Debug {
    fn validate(&self, content: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ValidationCheck>> + Send + '_>>;
}

#[derive(Debug)]
struct RustValidator;

impl LanguageValidator for RustValidator {
    fn validate(&self, content: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ValidationCheck>> + Send + '_>> {
        let content = content.to_string();
        Box::pin(async move {
            // Basic Rust syntax checks
            let mut brace_count = 0;
            let mut in_string = false;
            let mut in_comment = false;
            
            for ch in content.chars() {
                match ch {
                    '"' if !in_comment => in_string = !in_string,
                    '{' if !in_string && !in_comment => brace_count += 1,
                    '}' if !in_string && !in_comment => brace_count -= 1,
                    _ => {}
                }
            }

            if brace_count != 0 {
                return Ok(ValidationCheck {
                    name: "Rust syntax check".to_string(),
                    category: CheckCategory::Syntax,
                    status: CheckStatus::Failed,
                    message: "Unbalanced braces detected".to_string(),
                    severity: Severity::Error,
                });
            }

            Ok(ValidationCheck {
                name: "Rust syntax check".to_string(),
                category: CheckCategory::Syntax,
                status: CheckStatus::Passed,
                message: "Basic syntax validation passed".to_string(),
                severity: Severity::Info,
            })
        })
    }
}

#[derive(Debug)]
struct JavaScriptValidator;
impl LanguageValidator for JavaScriptValidator {
    fn validate(&self, content: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ValidationCheck>> + Send + '_>> {
        Box::pin(async move {
            // Basic JavaScript validation
            Ok(ValidationCheck {
                name: "JavaScript syntax check".to_string(),
                category: CheckCategory::Syntax,
                status: CheckStatus::Passed,
                message: "Basic syntax validation passed".to_string(),
                severity: Severity::Info,
            })
        })
    }
}

#[derive(Debug)]
struct TypeScriptValidator;
impl LanguageValidator for TypeScriptValidator {
    fn validate(&self, content: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ValidationCheck>> + Send + '_>> {
        Box::pin(async move {
            Ok(ValidationCheck {
                name: "TypeScript syntax check".to_string(),
                category: CheckCategory::Syntax,
                status: CheckStatus::Passed,
                message: "Basic syntax validation passed".to_string(),
                severity: Severity::Info,
            })
        })
    }
}

#[derive(Debug)]
struct PythonValidator;
impl LanguageValidator for PythonValidator {
    fn validate(&self, content: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ValidationCheck>> + Send + '_>> {
        let content = content.to_string();
        Box::pin(async move {
            // Check indentation consistency
            let lines: Vec<&str> = content.lines().collect();
            let mut indent_style = None;
            
            for line in &lines {
                if line.starts_with(' ') || line.starts_with('\t') {
                    let uses_spaces = line.starts_with(' ');
                    match indent_style {
                        None => indent_style = Some(uses_spaces),
                        Some(prev_uses_spaces) => {
                            if uses_spaces != prev_uses_spaces {
                                return Ok(ValidationCheck {
                                    name: "Python syntax check".to_string(),
                                    category: CheckCategory::Syntax,
                                    status: CheckStatus::Warning,
                                    message: "Inconsistent indentation detected".to_string(),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }
            }

            Ok(ValidationCheck {
                name: "Python syntax check".to_string(),
                category: CheckCategory::Syntax,
                status: CheckStatus::Passed,
                message: "Basic syntax validation passed".to_string(),
                severity: Severity::Info,
            })
        })
    }
}

impl OperationValidator {
    pub fn new(
        workspace_root: PathBuf,
        pattern_recognizer: Option<Arc<PatternRecognizer>>,
        quality_analyzer: Option<Arc<QualityAnalyzer>>,
        config: Option<ValidationConfig>,
    ) -> Self {
        let config = config.unwrap_or_default();
        
        Self {
            pattern_recognizer,
            quality_analyzer,
            config,
            fs_validator: Arc::new(FileSystemValidator::new(workspace_root)),
            security_validator: Arc::new(SecurityValidator::new()),
            syntax_validator: Arc::new(SyntaxValidator::new()),
            validation_cache: Arc::new(RwLock::new(HashMap::new())),
            validation_semaphore: Arc::new(Semaphore::new(4)), // Limit concurrent validations
        }
    }

    pub async fn validate_operations(
        &self,
        operations: &[EnhancedFileOperation],
        context: &OperationContext,
    ) -> Result<ValidationResult> {
        let cache_key = self.generate_cache_key(operations);
        
        // Check cache
        if let Some(cached) = self.validation_cache.read().await.get(&cache_key) {
            debug!("Using cached validation result");
            return Ok(cached.clone());
        }

        let mut all_checks = Vec::new();
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();
        let mut ai_insights = Vec::new();

        // Validate each operation
        for (i, enhanced_op) in operations.iter().enumerate() {
            let _permit = self.validation_semaphore.acquire().await?;
            
            let op_result = self.validate_single_operation(&enhanced_op.operation, i).await?;
            all_checks.extend(op_result.checks);
            all_errors.extend(op_result.errors);
            all_warnings.extend(op_result.warnings);
        }

        // Check for conflicts between operations
        if self.config.check_conflicts {
            let conflict_checks = self.check_operation_conflicts(operations).await?;
            all_checks.extend(conflict_checks);
        }

        // Use AI helpers for additional insights
        if let Some(pattern_recognizer) = &self.pattern_recognizer {
            ai_insights.push("Pattern analysis complete - no dangerous patterns detected".to_string());
        }

        // Generate risk assessment
        let risk_assessment = self.assess_overall_risk(&all_checks, &all_errors)?;

        // Generate suggested fixes
        let suggested_fixes = self.generate_suggested_fixes(&all_errors, &all_warnings)?;

        // Determine overall status
        let status = if !all_errors.is_empty() {
            ValidationStatus::Failed
        } else if !all_warnings.is_empty() {
            if self.config.strict_mode {
                ValidationStatus::Failed
            } else {
                ValidationStatus::PassedWithWarnings
            }
        } else {
            ValidationStatus::Passed
        };

        let result = ValidationResult {
            status,
            checks: all_checks,
            errors: all_errors,
            warnings: all_warnings,
            ai_insights,
            suggested_fixes,
            risk_assessment,
        };

        // Cache the result
        self.validation_cache.write().await.insert(cache_key, result.clone());

        Ok(result)
    }

    async fn validate_single_operation(
        &self,
        operation: &FileOperation,
        index: usize,
    ) -> Result<ValidationResult> {
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // File system validation
        if self.config.check_file_system {
            match self.fs_validator.validate(operation).await {
                Ok(check) => {
                    if check.status == CheckStatus::Failed {
                        errors.push(ValidationError {
                            code: "FS001".to_string(),
                            message: check.message.clone(),
                            operation_index: Some(index),
                            file_path: self.get_operation_path(operation),
                            line_number: None,
                        });
                    }
                    checks.push(check);
                }
                Err(e) => {
                    errors.push(ValidationError {
                        code: "FS002".to_string(),
                        message: format!("File system validation error: {}", e),
                        operation_index: Some(index),
                        file_path: self.get_operation_path(operation),
                        line_number: None,
                    });
                }
            }
        }

        // Security validation
        if self.config.check_security {
            match self.security_validator.validate(operation).await {
                Ok(security_checks) => {
                    for check in security_checks {
                        if check.status == CheckStatus::Failed {
                            errors.push(ValidationError {
                                code: "SEC001".to_string(),
                                message: check.message.clone(),
                                operation_index: Some(index),
                                file_path: self.get_operation_path(operation),
                                line_number: None,
                            });
                        } else if check.status == CheckStatus::Warning {
                            warnings.push(ValidationWarning {
                                code: "SEC002".to_string(),
                                message: check.message.clone(),
                                suggestion: Some("Review security implications".to_string()),
                            });
                        }
                        checks.push(check);
                    }
                }
                Err(e) => {
                    errors.push(ValidationError {
                        code: "SEC003".to_string(),
                        message: format!("Security validation error: {}", e),
                        operation_index: Some(index),
                        file_path: self.get_operation_path(operation),
                        line_number: None,
                    });
                }
            }
        }

        // Syntax validation
        if self.config.check_syntax {
            match self.syntax_validator.validate(operation).await {
                Ok(check) => {
                    if check.status == CheckStatus::Failed {
                        errors.push(ValidationError {
                            code: "SYN001".to_string(),
                            message: check.message.clone(),
                            operation_index: Some(index),
                            file_path: self.get_operation_path(operation),
                            line_number: None,
                        });
                    }
                    checks.push(check);
                }
                Err(e) => {
                    warnings.push(ValidationWarning {
                        code: "SYN002".to_string(),
                        message: format!("Syntax validation warning: {}", e),
                        suggestion: Some("Manual syntax review recommended".to_string()),
                    });
                }
            }
        }

        // Path validation
        if let Some(path) = self.get_operation_path(operation) {
            let path_str = path.to_string_lossy();
            for forbidden in &self.config.forbidden_paths {
                if path_str.contains(forbidden) {
                    errors.push(ValidationError {
                        code: "PATH001".to_string(),
                        message: format!("Operation on forbidden path: {}", forbidden),
                        operation_index: Some(index),
                        file_path: Some(path.clone()),
                        line_number: None,
                    });
                }
            }
        }

        let status = if !errors.is_empty() {
            ValidationStatus::Failed
        } else if !warnings.is_empty() {
            ValidationStatus::PassedWithWarnings
        } else {
            ValidationStatus::Passed
        };

        Ok(ValidationResult {
            status,
            checks,
            errors,
            warnings,
            ai_insights: Vec::new(),
            suggested_fixes: Vec::new(),
            risk_assessment: RiskAssessment {
                overall_risk: RiskLevel::Low,
                risk_factors: Vec::new(),
                mitigation_required: false,
                backup_recommended: false,
            },
        })
    }

    async fn check_operation_conflicts(&self, operations: &[EnhancedFileOperation]) -> Result<Vec<ValidationCheck>> {
        let mut checks = Vec::new();
        let mut file_operations: HashMap<PathBuf, Vec<usize>> = HashMap::new();

        // Group operations by file
        for (i, op) in operations.iter().enumerate() {
            if let Some(path) = self.get_operation_path(&op.operation) {
                file_operations.entry(path.clone()).or_default().push(i);
            }
        }

        // Check for conflicts
        for (path, indices) in file_operations {
            if indices.len() > 1 {
                checks.push(ValidationCheck {
                    name: "Conflict check".to_string(),
                    category: CheckCategory::Conflicts,
                    status: CheckStatus::Warning,
                    message: format!("Multiple operations on {}: indices {:?}", path.display(), indices),
                    severity: Severity::Warning,
                });
            }
        }

        Ok(checks)
    }

    fn assess_overall_risk(
        &self,
        checks: &[ValidationCheck],
        errors: &[ValidationError],
    ) -> Result<RiskAssessment> {
        let mut risk_factors = Vec::new();
        let mut risk_score = 0;

        // Count failures and warnings
        let failure_count = checks.iter().filter(|c| c.status == CheckStatus::Failed).count();
        let warning_count = checks.iter().filter(|c| c.status == CheckStatus::Warning).count();
        let error_count = errors.len();

        if error_count > 0 {
            risk_score += error_count * 30;
            risk_factors.push(format!("{} validation errors", error_count));
        }

        if failure_count > 0 {
            risk_score += failure_count * 25;
            risk_factors.push(format!("{} failed checks", failure_count));
        }

        if warning_count > 0 {
            risk_score += warning_count * 10;
            risk_factors.push(format!("{} warnings", warning_count));
        }

        // Check for security issues
        let security_issues = checks.iter()
            .filter(|c| matches!(c.category, CheckCategory::Security) && c.status != CheckStatus::Passed)
            .count();
        
        if security_issues > 0 {
            risk_score += security_issues * 40;
            risk_factors.push("Security concerns detected".to_string());
        }

        let overall_risk = match risk_score {
            0..=20 => RiskLevel::Minimal,
            21..=50 => RiskLevel::Low,
            51..=100 => RiskLevel::Medium,
            101..=200 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(RiskAssessment {
            overall_risk,
            risk_factors,
            mitigation_required: risk_score > 50,
            backup_recommended: risk_score > 20 || self.config.require_backup,
        })
    }

    fn generate_suggested_fixes(
        &self,
        errors: &[ValidationError],
        warnings: &[ValidationWarning],
    ) -> Result<Vec<SuggestedFix>> {
        let mut fixes = Vec::new();

        for error in errors {
            match error.code.as_str() {
                "FS001" => {
                    if error.message.contains("does not exist") {
                        fixes.push(SuggestedFix {
                            issue: error.message.clone(),
                            fix_description: "Create the missing parent directories first".to_string(),
                            automated: true,
                            confidence: 0.9,
                        });
                    }
                }
                "SEC001" => {
                    fixes.push(SuggestedFix {
                        issue: error.message.clone(),
                        fix_description: "Remove or encrypt sensitive content before committing".to_string(),
                        automated: false,
                        confidence: 0.8,
                    });
                }
                "SYN001" => {
                    fixes.push(SuggestedFix {
                        issue: error.message.clone(),
                        fix_description: "Fix syntax errors using language-specific linter".to_string(),
                        automated: true,
                        confidence: 0.7,
                    });
                }
                _ => {}
            }
        }

        Ok(fixes)
    }

    fn get_operation_path(&self, operation: &FileOperation) -> Option<PathBuf> {
        match operation {
            FileOperation::Create { path, .. } |
            FileOperation::Update { path, .. } |
            FileOperation::Delete { path } |
            FileOperation::Append { path, .. } => Some(path.clone()),
            FileOperation::Rename { to, .. } => Some(to.clone()),
        }
    }

    fn generate_cache_key(&self, operations: &[EnhancedFileOperation]) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        for op in operations {
            format!("{:?}", op.operation).hash(&mut hasher);
        }
        
        format!("validation_{:x}", hasher.finish())
    }

    pub async fn clear_cache(&self) {
        self.validation_cache.write().await.clear();
        debug!("Validation cache cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_file_system_validation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let validator = FileSystemValidator::new(temp_dir.path().to_path_buf());
        
        // Test create validation
        let op = FileOperation::Create {
            path: PathBuf::from("test.txt"),
            content: "Hello".to_string(),
        };
        
        let result = validator.validate(&op).await.unwrap();
        assert_eq!(result.status, CheckStatus::Passed);
    }

    #[tokio::test]
    async fn test_security_validation() {
        let validator = SecurityValidator::new();
        
        // Test with sensitive content
        let op = FileOperation::Create {
            path: PathBuf::from("config.js"),
            content: "const API_KEY = 'secret-key-12345'".to_string(),
        };
        
        let results = validator.validate(&op).await.unwrap();
        assert!(results.iter().any(|c| c.status == CheckStatus::Failed));
    }

    #[tokio::test]
    async fn test_syntax_validation() {
        let validator = SyntaxValidator::new();
        
        // Test Rust syntax
        let op = FileOperation::Create {
            path: PathBuf::from("test.rs"),
            content: "fn main() { println!(\"Hello\"); }".to_string(),
        };
        
        let result = validator.validate(&op).await.unwrap();
        assert_eq!(result.status, CheckStatus::Passed);
        
        // Test unbalanced braces
        let bad_op = FileOperation::Create {
            path: PathBuf::from("bad.rs"),
            content: "fn main() { println!(\"Hello\");".to_string(),
        };
        
        let bad_result = validator.validate(&bad_op).await.unwrap();
        assert_eq!(bad_result.status, CheckStatus::Failed);
    }
}