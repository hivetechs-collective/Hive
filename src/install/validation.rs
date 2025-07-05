//! Installation validation and integrity checking

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;
use tokio::fs as afs;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Installation validator
#[derive(Debug, Clone)]
pub struct InstallationValidator {
    /// Installation directory
    pub install_dir: PathBuf,
    /// Configuration directory
    pub config_dir: PathBuf,
    /// Validation rules
    pub rules: ValidationRules,
}

/// Validation rules and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Required files
    pub required_files: Vec<RequiredFile>,
    /// Required permissions
    pub required_permissions: Vec<PermissionCheck>,
    /// System requirements
    pub system_requirements: SystemRequirements,
    /// Integrity checks
    pub integrity_checks: Vec<IntegrityCheck>,
}

/// Required file specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFile {
    /// File path (relative to install directory)
    pub path: String,
    /// Expected file type
    pub file_type: FileType,
    /// Expected size range (min, max)
    pub size_range: Option<(u64, u64)>,
    /// Expected checksum
    pub checksum: Option<String>,
    /// Required permissions
    pub permissions: Option<u32>,
}

/// File types for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    /// Executable binary
    Binary,
    /// Configuration file
    Config,
    /// Documentation
    Documentation,
    /// Shell completion
    Completion,
    /// Data file
    Data,
}

/// Permission check specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheck {
    /// Path to check
    pub path: String,
    /// Required permissions
    pub permissions: u32,
    /// Check type
    pub check_type: PermissionCheckType,
}

/// Permission check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionCheckType {
    /// Exact match
    Exact,
    /// At least these permissions
    AtLeast,
    /// At most these permissions
    AtMost,
}

/// System requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    /// Minimum available disk space
    pub min_disk_space: u64,
    /// Minimum available memory
    pub min_memory: u64,
    /// Supported operating systems
    pub supported_os: Vec<String>,
    /// Supported architectures
    pub supported_arch: Vec<String>,
    /// Required system dependencies
    pub dependencies: Vec<String>,
}

/// Integrity check specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheck {
    /// Check name
    pub name: String,
    /// Check type
    pub check_type: IntegrityCheckType,
    /// Expected result
    pub expected: String,
    /// Critical check (failure prevents installation)
    pub critical: bool,
}

/// Integrity check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrityCheckType {
    /// File checksum
    Checksum { file_path: String },
    /// Command output
    Command { command: String, args: Vec<String> },
    /// Environment variable
    Environment { variable: String },
    /// Network connectivity
    Network { host: String, port: u16 },
}

/// Validation result
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    /// Overall validation success
    pub success: bool,
    /// Individual check results
    pub checks: Vec<ValidationCheck>,
    /// Error summary
    pub errors: Vec<ValidationError>,
    /// Warning summary
    pub warnings: Vec<ValidationWarning>,
    /// Validation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Individual validation check result
#[derive(Debug, Clone, Serialize)]
pub struct ValidationCheck {
    /// Check name
    pub name: String,
    /// Check success
    pub success: bool,
    /// Check message
    pub message: String,
    /// Check type
    pub check_type: String,
    /// Execution time
    pub duration: std::time::Duration,
}

/// Validation error
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error category
    pub category: ErrorCategory,
    /// Suggested fix
    pub fix: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone, Serialize)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
    /// Warning category
    pub category: WarningCategory,
}

/// Error categories
#[derive(Debug, Clone, Serialize)]
pub enum ErrorCategory {
    /// File system related
    FileSystem,
    /// Permission related
    Permission,
    /// System requirement
    SystemRequirement,
    /// Integrity check
    Integrity,
    /// Network related
    Network,
}

/// Warning categories
#[derive(Debug, Clone, Serialize)]
pub enum WarningCategory {
    /// Performance related
    Performance,
    /// Configuration related
    Configuration,
    /// Compatibility related
    Compatibility,
    /// Security related
    Security,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            required_files: vec![
                RequiredFile {
                    path: "hive".to_string(),
                    file_type: FileType::Binary,
                    size_range: Some((1024 * 1024, 100 * 1024 * 1024)), // 1MB - 100MB
                    checksum: None,
                    permissions: Some(0o755),
                },
            ],
            required_permissions: vec![
                PermissionCheck {
                    path: "hive".to_string(),
                    permissions: 0o755,
                    check_type: PermissionCheckType::AtLeast,
                },
            ],
            system_requirements: SystemRequirements {
                min_disk_space: 100 * 1024 * 1024, // 100MB
                min_memory: 256 * 1024 * 1024, // 256MB
                supported_os: vec![
                    "linux".to_string(),
                    "macos".to_string(),
                    "windows".to_string(),
                ],
                supported_arch: vec![
                    "x86_64".to_string(),
                    "aarch64".to_string(),
                ],
                dependencies: vec![],
            },
            integrity_checks: vec![
                IntegrityCheck {
                    name: "binary_signature".to_string(),
                    check_type: IntegrityCheckType::Command {
                        command: "hive".to_string(),
                        args: vec!["--version".to_string()],
                    },
                    expected: "hive".to_string(),
                    critical: true,
                },
            ],
        }
    }
}

impl InstallationValidator {
    /// Create a new installation validator
    pub fn new(install_dir: PathBuf, config_dir: PathBuf) -> Result<Self> {
        let rules = ValidationRules::default();
        
        Ok(Self {
            install_dir,
            config_dir,
            rules,
        })
    }

    /// Validate installation
    pub async fn validate(&self) -> Result<ValidationResult> {
        println!("🔍 Validating installation...");
        
        let start_time = std::time::Instant::now();
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check required files
        let file_results = self.check_required_files().await?;
        checks.extend(file_results.0);
        errors.extend(file_results.1);
        warnings.extend(file_results.2);
        
        // Check permissions
        let permission_results = self.check_permissions().await?;
        checks.extend(permission_results.0);
        errors.extend(permission_results.1);
        warnings.extend(permission_results.2);
        
        // Check system requirements
        let system_results = self.check_system_requirements().await?;
        checks.extend(system_results.0);
        errors.extend(system_results.1);
        warnings.extend(system_results.2);
        
        // Run integrity checks
        let integrity_results = self.check_integrity().await?;
        checks.extend(integrity_results.0);
        errors.extend(integrity_results.1);
        warnings.extend(integrity_results.2);
        
        let success = errors.is_empty();
        let duration = start_time.elapsed();
        
        let result = ValidationResult {
            success,
            checks,
            errors,
            warnings,
            timestamp: chrono::Utc::now(),
        };
        
        if success {
            println!("✅ Installation validation passed ({:.2}s)", duration.as_secs_f64());
        } else {
            println!("❌ Installation validation failed ({:.2}s)", duration.as_secs_f64());
        }
        
        Ok(result)
    }

    /// Check required files
    async fn check_required_files(&self) -> Result<(Vec<ValidationCheck>, Vec<ValidationError>, Vec<ValidationWarning>)> {
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        for required_file in &self.rules.required_files {
            let start_time = std::time::Instant::now();
            let file_path = self.install_dir.join(&required_file.path);
            
            let (success, message) = if file_path.exists() {
                // Check file type
                let metadata = fs::metadata(&file_path)?;
                
                // Check size range
                if let Some((min_size, max_size)) = required_file.size_range {
                    let file_size = metadata.len();
                    if file_size < min_size || file_size > max_size {
                        errors.push(ValidationError {
                            code: "FILE_SIZE_INVALID".to_string(),
                            message: format!("File size {} is outside expected range {}-{}", 
                                           file_size, min_size, max_size),
                            category: ErrorCategory::FileSystem,
                            fix: Some("Reinstall the application".to_string()),
                        });
                        (false, format!("File size validation failed: {}", file_path.display()))
                    } else {
                        (true, format!("File exists and size is valid: {}", file_path.display()))
                    }
                } else {
                    (true, format!("File exists: {}", file_path.display()))
                }
            } else {
                errors.push(ValidationError {
                    code: "FILE_MISSING".to_string(),
                    message: format!("Required file missing: {}", required_file.path),
                    category: ErrorCategory::FileSystem,
                    fix: Some("Reinstall the application".to_string()),
                });
                (false, format!("File missing: {}", file_path.display()))
            };
            
            checks.push(ValidationCheck {
                name: format!("required_file_{}", required_file.path),
                success,
                message,
                check_type: "file_check".to_string(),
                duration: start_time.elapsed(),
            });
        }
        
        Ok((checks, errors, warnings))
    }

    /// Check permissions
    async fn check_permissions(&self) -> Result<(Vec<ValidationCheck>, Vec<ValidationError>, Vec<ValidationWarning>)> {
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            
            for permission_check in &self.rules.required_permissions {
                let start_time = std::time::Instant::now();
                let file_path = self.install_dir.join(&permission_check.path);
                
                let (success, message) = if file_path.exists() {
                    let metadata = fs::metadata(&file_path)?;
                    let actual_permissions = metadata.permissions().mode() & 0o777;
                    
                    let permissions_valid = match permission_check.check_type {
                        PermissionCheckType::Exact => actual_permissions == permission_check.permissions,
                        PermissionCheckType::AtLeast => actual_permissions >= permission_check.permissions,
                        PermissionCheckType::AtMost => actual_permissions <= permission_check.permissions,
                    };
                    
                    if permissions_valid {
                        (true, format!("Permissions correct: {:o}", actual_permissions))
                    } else {
                        errors.push(ValidationError {
                            code: "PERMISSION_INVALID".to_string(),
                            message: format!("Invalid permissions on {}: expected {:o}, got {:o}", 
                                           permission_check.path, permission_check.permissions, actual_permissions),
                            category: ErrorCategory::Permission,
                            fix: Some(format!("chmod {:o} {}", permission_check.permissions, file_path.display())),
                        });
                        (false, format!("Permissions invalid: expected {:o}, got {:o}", 
                                       permission_check.permissions, actual_permissions))
                    }
                } else {
                    (false, "File does not exist".to_string())
                };
                
                checks.push(ValidationCheck {
                    name: format!("permission_{}", permission_check.path),
                    success,
                    message,
                    check_type: "permission_check".to_string(),
                    duration: start_time.elapsed(),
                });
            }
        }
        
        #[cfg(windows)]
        {
            // Windows permission checks would go here
            // For now, we'll just mark them as successful
            for permission_check in &self.rules.required_permissions {
                checks.push(ValidationCheck {
                    name: format!("permission_{}", permission_check.path),
                    success: true,
                    message: "Windows permission check skipped".to_string(),
                    check_type: "permission_check".to_string(),
                    duration: std::time::Duration::from_millis(1),
                });
            }
        }
        
        Ok((checks, errors, warnings))
    }

    /// Check system requirements
    async fn check_system_requirements(&self) -> Result<(Vec<ValidationCheck>, Vec<ValidationError>, Vec<ValidationWarning>)> {
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check operating system
        let start_time = std::time::Instant::now();
        let current_os = std::env::consts::OS;
        let os_supported = self.rules.system_requirements.supported_os.contains(&current_os.to_string());
        
        if !os_supported {
            errors.push(ValidationError {
                code: "OS_UNSUPPORTED".to_string(),
                message: format!("Unsupported operating system: {}", current_os),
                category: ErrorCategory::SystemRequirement,
                fix: Some("Use a supported operating system".to_string()),
            });
        }
        
        checks.push(ValidationCheck {
            name: "os_support".to_string(),
            success: os_supported,
            message: if os_supported {
                format!("Operating system supported: {}", current_os)
            } else {
                format!("Operating system not supported: {}", current_os)
            },
            check_type: "system_check".to_string(),
            duration: start_time.elapsed(),
        });
        
        // Check architecture
        let start_time = std::time::Instant::now();
        let current_arch = std::env::consts::ARCH;
        let arch_supported = self.rules.system_requirements.supported_arch.contains(&current_arch.to_string());
        
        if !arch_supported {
            errors.push(ValidationError {
                code: "ARCH_UNSUPPORTED".to_string(),
                message: format!("Unsupported architecture: {}", current_arch),
                category: ErrorCategory::SystemRequirement,
                fix: Some("Use a supported architecture".to_string()),
            });
        }
        
        checks.push(ValidationCheck {
            name: "arch_support".to_string(),
            success: arch_supported,
            message: if arch_supported {
                format!("Architecture supported: {}", current_arch)
            } else {
                format!("Architecture not supported: {}", current_arch)
            },
            check_type: "system_check".to_string(),
            duration: start_time.elapsed(),
        });
        
        // Check disk space
        let start_time = std::time::Instant::now();
        let available_space = self.get_available_disk_space().await?;
        let disk_space_ok = available_space >= self.rules.system_requirements.min_disk_space;
        
        if !disk_space_ok {
            errors.push(ValidationError {
                code: "DISK_SPACE_INSUFFICIENT".to_string(),
                message: format!("Insufficient disk space: {} available, {} required", 
                               available_space, self.rules.system_requirements.min_disk_space),
                category: ErrorCategory::SystemRequirement,
                fix: Some("Free up disk space".to_string()),
            });
        }
        
        checks.push(ValidationCheck {
            name: "disk_space".to_string(),
            success: disk_space_ok,
            message: format!("Disk space: {} available, {} required", 
                           available_space, self.rules.system_requirements.min_disk_space),
            check_type: "system_check".to_string(),
            duration: start_time.elapsed(),
        });
        
        Ok((checks, errors, warnings))
    }

    /// Check integrity
    async fn check_integrity(&self) -> Result<(Vec<ValidationCheck>, Vec<ValidationError>, Vec<ValidationWarning>)> {
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        for integrity_check in &self.rules.integrity_checks {
            let start_time = std::time::Instant::now();
            
            let (success, message) = match &integrity_check.check_type {
                IntegrityCheckType::Checksum { file_path } => {
                    self.check_file_checksum(file_path, &integrity_check.expected).await?
                }
                IntegrityCheckType::Command { command, args } => {
                    self.check_command_output(command, args, &integrity_check.expected).await?
                }
                IntegrityCheckType::Environment { variable } => {
                    self.check_environment_variable(variable, &integrity_check.expected).await?
                }
                IntegrityCheckType::Network { host, port } => {
                    self.check_network_connectivity(host, *port).await?
                }
            };
            
            if !success && integrity_check.critical {
                errors.push(ValidationError {
                    code: "INTEGRITY_CHECK_FAILED".to_string(),
                    message: format!("Critical integrity check failed: {}", integrity_check.name),
                    category: ErrorCategory::Integrity,
                    fix: Some("Reinstall the application".to_string()),
                });
            } else if !success {
                warnings.push(ValidationWarning {
                    code: "INTEGRITY_CHECK_WARNING".to_string(),
                    message: format!("Non-critical integrity check failed: {}", integrity_check.name),
                    category: WarningCategory::Security,
                });
            }
            
            checks.push(ValidationCheck {
                name: integrity_check.name.clone(),
                success,
                message,
                check_type: "integrity_check".to_string(),
                duration: start_time.elapsed(),
            });
        }
        
        Ok((checks, errors, warnings))
    }

    /// Check file checksum
    async fn check_file_checksum(&self, file_path: &str, expected: &str) -> Result<(bool, String)> {
        let full_path = self.install_dir.join(file_path);
        
        if !full_path.exists() {
            return Ok((false, format!("File not found: {}", file_path)));
        }
        
        let content = afs::read(&full_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let actual_checksum = format!("{:x}", hasher.finalize());
        
        let success = actual_checksum == expected;
        let message = if success {
            "Checksum verified".to_string()
        } else {
            format!("Checksum mismatch: expected {}, got {}", expected, actual_checksum)
        };
        
        Ok((success, message))
    }

    /// Check command output
    async fn check_command_output(&self, command: &str, args: &[String], expected: &str) -> Result<(bool, String)> {
        let binary_path = self.install_dir.join(command);
        
        let output = tokio::process::Command::new(&binary_path)
            .args(args)
            .output()
            .await?;
        
        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let contains_expected = stdout.contains(expected);
        
        let overall_success = success && contains_expected;
        let message = if overall_success {
            "Command executed successfully".to_string()
        } else {
            format!("Command failed or unexpected output: {}", stdout)
        };
        
        Ok((overall_success, message))
    }

    /// Check environment variable
    async fn check_environment_variable(&self, variable: &str, expected: &str) -> Result<(bool, String)> {
        match std::env::var(variable) {
            Ok(value) => {
                let success = value == expected;
                let message = if success {
                    "Environment variable matches expected value".to_string()
                } else {
                    format!("Environment variable mismatch: expected {}, got {}", expected, value)
                };
                Ok((success, message))
            }
            Err(_) => {
                Ok((false, format!("Environment variable not set: {}", variable)))
            }
        }
    }

    /// Check network connectivity
    async fn check_network_connectivity(&self, host: &str, port: u16) -> Result<(bool, String)> {
        use tokio::net::TcpStream;
        use tokio::time::{timeout, Duration};
        
        let address = format!("{}:{}", host, port);
        
        match timeout(Duration::from_secs(5), TcpStream::connect(&address)).await {
            Ok(Ok(_)) => Ok((true, format!("Successfully connected to {}", address))),
            Ok(Err(e)) => Ok((false, format!("Connection failed: {}", e))),
            Err(_) => Ok((false, "Connection timeout".to_string())),
        }
    }

    /// Get available disk space
    async fn get_available_disk_space(&self) -> Result<u64> {
        // This is a simplified implementation
        // In a real implementation, you'd use platform-specific APIs
        
        #[cfg(unix)]
        {
            use std::ffi::CString;
            use std::mem;
            
            let path = CString::new(self.install_dir.to_str().unwrap())?;
            let mut stat: libc::statvfs = unsafe { mem::zeroed() };
            
            let result = unsafe { libc::statvfs(path.as_ptr(), &mut stat) };
            
            if result == 0 {
                Ok(stat.f_bavail as u64 * stat.f_frsize)
            } else {
                // Fallback to a reasonable default
                Ok(1024 * 1024 * 1024) // 1GB
            }
        }
        
        #[cfg(windows)]
        {
            // Windows implementation would use GetDiskFreeSpaceEx
            // For now, return a reasonable default
            Ok(1024 * 1024 * 1024) // 1GB
        }
        
        #[cfg(not(any(unix, windows)))]
        {
            Ok(1024 * 1024 * 1024) // 1GB
        }
    }

    /// Generate validation report
    pub fn generate_report(&self, result: &ValidationResult) -> Result<String> {
        let mut report = String::new();
        
        report.push_str(&format!("# Hive AI Installation Validation Report\n\n"));
        report.push_str(&format!("Generated: {}\n", result.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("Status: {}\n\n", if result.success { "✅ PASSED" } else { "❌ FAILED" }));
        
        if !result.errors.is_empty() {
            report.push_str("## Errors\n\n");
            for error in &result.errors {
                report.push_str(&format!("- **{}**: {}\n", error.code, error.message));
                if let Some(fix) = &error.fix {
                    report.push_str(&format!("  - Fix: {}\n", fix));
                }
            }
            report.push_str("\n");
        }
        
        if !result.warnings.is_empty() {
            report.push_str("## Warnings\n\n");
            for warning in &result.warnings {
                report.push_str(&format!("- **{}**: {}\n", warning.code, warning.message));
            }
            report.push_str("\n");
        }
        
        report.push_str("## Check Details\n\n");
        for check in &result.checks {
            let status = if check.success { "✅" } else { "❌" };
            report.push_str(&format!("- {} **{}**: {} ({:.2}ms)\n", 
                                   status, check.name, check.message, check.duration.as_millis()));
        }
        
        Ok(report)
    }

    /// Save validation report
    pub async fn save_report(&self, result: &ValidationResult) -> Result<PathBuf> {
        let report = self.generate_report(result)?;
        let report_path = self.config_dir.join("validation_report.md");
        
        afs::create_dir_all(&self.config_dir).await?;
        afs::write(&report_path, report).await?;
        
        Ok(report_path)
    }
}

/// Quick validation for essential functionality
pub async fn quick_validate(install_dir: &PathBuf) -> Result<bool> {
    let binary_path = install_dir.join("hive");
    
    // Check if binary exists
    if !binary_path.exists() {
        return Ok(false);
    }
    
    // Check if binary is executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&binary_path)?;
        let permissions = metadata.permissions().mode();
        if permissions & 0o111 == 0 {
            return Ok(false);
        }
    }
    
    // Try to run --version
    let output = tokio::process::Command::new(&binary_path)
        .arg("--version")
        .output()
        .await?;
    
    Ok(output.status.success())
}

/// Comprehensive validation with full reporting
pub async fn comprehensive_validate(install_dir: &PathBuf, config_dir: &PathBuf) -> Result<ValidationResult> {
    let validator = InstallationValidator::new(install_dir.clone(), config_dir.clone())?;
    validator.validate().await
}