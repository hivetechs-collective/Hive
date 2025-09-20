//! Repository Reality Verification System
//!
//! This module ensures that all consensus stages are working with verified repository facts,
//! preventing hallucinations about project structure, dependencies, and characteristics.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;
use walkdir::WalkDir;

/// Verified facts about a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryFacts {
    /// Repository name from Cargo.toml
    pub name: String,

    /// Version from Cargo.toml
    pub version: String,

    /// Number of external dependencies
    pub dependency_count: usize,

    /// Number of Rust modules
    pub module_count: usize,

    /// Total files in the project
    pub total_files: usize,

    /// Lines of code count
    pub lines_of_code: usize,

    /// Classification based on complexity
    pub is_enterprise: bool,

    /// When verification was performed
    pub verified_at: DateTime<Utc>,

    /// Repository root path
    pub root_path: PathBuf,

    /// Key file extensions found
    pub file_extensions: Vec<String>,

    /// Major directories found
    pub major_directories: Vec<String>,
}

/// Repository structure analysis
#[derive(Debug, Clone)]
pub struct RepositoryStructure {
    pub rust_modules: Vec<String>,
    pub total_files: usize,
    pub lines_of_code: usize,
    pub file_extensions: Vec<String>,
    pub major_directories: Vec<String>,
}

/// Cargo manifest information
#[derive(Debug, Clone)]
pub struct CargoManifest {
    pub package: PackageInfo,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

/// Repository verifier that ensures facts match file system reality
pub struct RepositoryVerifier {
    root_path: PathBuf,
    manifest_cache: Option<CargoManifest>,
    structure_cache: Option<RepositoryStructure>,
}

impl RepositoryVerifier {
    /// Create a new repository verifier
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            manifest_cache: None,
            structure_cache: None,
        }
    }

    /// Verify basic repository facts before any analysis
    pub async fn verify_repository_context(&mut self) -> Result<RepositoryFacts> {
        tracing::info!(
            "Verifying repository context at: {}",
            self.root_path.display()
        );

        // Read and verify Cargo.toml
        let manifest = self
            .read_cargo_manifest()
            .await
            .context("Failed to read Cargo.toml")?;

        // Analyze file structure
        let file_structure = self
            .analyze_structure()
            .await
            .context("Failed to analyze repository structure")?;

        let dependency_count = manifest.dependencies.len();
        let is_enterprise = self.classify_complexity(&file_structure);

        let facts = RepositoryFacts {
            name: manifest.package.name.clone(),
            version: manifest.package.version.clone(),
            dependency_count,
            module_count: file_structure.rust_modules.len(),
            total_files: file_structure.total_files,
            lines_of_code: file_structure.lines_of_code,
            is_enterprise,
            verified_at: Utc::now(),
            root_path: self.root_path.clone(),
            file_extensions: file_structure.file_extensions,
            major_directories: file_structure.major_directories,
        };

        tracing::info!(
            "Repository verification complete: {} v{} ({} dependencies, {} modules, {} files)",
            facts.name,
            facts.version,
            facts.dependency_count,
            facts.module_count,
            facts.total_files
        );

        Ok(facts)
    }

    /// Read and parse Cargo.toml
    async fn read_cargo_manifest(&mut self) -> Result<CargoManifest> {
        if let Some(manifest) = &self.manifest_cache {
            return Ok(manifest.clone());
        }

        let cargo_path = self.root_path.join("Cargo.toml");
        if !cargo_path.exists() {
            anyhow::bail!("No Cargo.toml found at {}", cargo_path.display());
        }

        let content = fs::read_to_string(&cargo_path)
            .with_context(|| format!("Failed to read {}", cargo_path.display()))?;

        let toml: TomlValue = content.parse().context("Failed to parse Cargo.toml")?;

        // Extract package information
        let package = toml
            .get("package")
            .context("No [package] section in Cargo.toml")?;

        let name = package
            .get("name")
            .and_then(|v| v.as_str())
            .context("No package name in Cargo.toml")?
            .to_string();

        let version = package
            .get("version")
            .and_then(|v| v.as_str())
            .context("No package version in Cargo.toml")?
            .to_string();

        // Extract dependencies
        let mut dependencies = Vec::new();
        if let Some(deps) = toml.get("dependencies").and_then(|v| v.as_table()) {
            dependencies.extend(deps.keys().map(|k| k.to_string()));
        }
        if let Some(dev_deps) = toml.get("dev-dependencies").and_then(|v| v.as_table()) {
            dependencies.extend(dev_deps.keys().map(|k| format!("{} (dev)", k)));
        }

        let manifest = CargoManifest {
            package: PackageInfo { name, version },
            dependencies,
        };

        self.manifest_cache = Some(manifest.clone());
        Ok(manifest)
    }

    /// Analyze repository file structure
    async fn analyze_structure(&mut self) -> Result<RepositoryStructure> {
        if let Some(structure) = &self.structure_cache {
            return Ok(structure.clone());
        }

        let mut rust_modules = Vec::new();
        let mut total_files = 0;
        let mut lines_of_code = 0;
        let mut file_extensions = std::collections::HashSet::new();
        let mut major_directories = std::collections::HashSet::new();

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            total_files += 1;

            let path = entry.path();

            // Track file extensions
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                file_extensions.insert(ext.to_string());
            }

            // Track major directories
            if let Some(parent) = path.parent() {
                if let Some(dir_name) = parent.file_name().and_then(|s| s.to_str()) {
                    if !dir_name.starts_with('.') && dir_name != "target" {
                        major_directories.insert(dir_name.to_string());
                    }
                }
            }

            // Count Rust modules
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Some(relative_path) = path.strip_prefix(&self.root_path).ok() {
                    rust_modules.push(relative_path.display().to_string());
                }

                // Count lines of code in Rust files
                if let Ok(content) = fs::read_to_string(path) {
                    lines_of_code += content.lines().count();
                }
            }
        }

        let structure = RepositoryStructure {
            rust_modules,
            total_files,
            lines_of_code,
            file_extensions: file_extensions.into_iter().collect(),
            major_directories: major_directories.into_iter().collect(),
        };

        self.structure_cache = Some(structure.clone());
        Ok(structure)
    }

    /// Classify repository complexity
    fn classify_complexity(&self, structure: &RepositoryStructure) -> bool {
        // Enterprise classification criteria
        let has_many_dependencies = if let Some(ref manifest) = self.manifest_cache {
            manifest.dependencies.len() > 20
        } else {
            false
        };

        let has_many_modules = structure.rust_modules.len() > 10;
        let has_many_files = structure.total_files > 50;
        let has_significant_code = structure.lines_of_code > 5000;

        // Consider enterprise if it meets multiple criteria
        let criteria_count = [
            has_many_dependencies,
            has_many_modules,
            has_many_files,
            has_significant_code,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        criteria_count >= 2
    }
}

/// Build mandatory context prefix for consensus stages
pub fn build_stage_context(
    facts: &RepositoryFacts,
    stage: crate::consensus::types::Stage,
) -> String {
    let stage_instructions = match stage {
        crate::consensus::types::Stage::Generator => {
            "GENERATOR: You must base your analysis on the VERIFIED FACTS above. Generate comprehensive solutions that acknowledge the actual repository structure and dependencies."
        },
        crate::consensus::types::Stage::Refiner => {
            "REFINER: You must improve responses while staying true to the VERIFIED FACTS above. Any refinements must be consistent with the actual repository characteristics."
        },
        crate::consensus::types::Stage::Validator => {
            "VALIDATOR: You must validate responses against the VERIFIED FACTS above. Flag any claims that contradict the actual repository structure, dependencies, or characteristics."
        },
        crate::consensus::types::Stage::Curator => {
            "CURATOR: You must synthesize the final response using only information consistent with the VERIFIED FACTS above. Ensure the final answer accurately reflects the actual repository."
        },
    };

    format!(
        r#"
=== REPOSITORY VERIFICATION (MANDATORY CONTEXT) ===
- Name: {} v{}
- Dependencies: {} external crates
- Modules: {} Rust modules
- Files: {} total files ({} lines of code)
- Classification: {}
- Verified: {}
- Root Path: {}
- File Types: {}
- Directories: {}

🚨 CRITICAL: You are analyzing the ABOVE repository. Any analysis that contradicts these verified facts is INCORRECT.

{}
=== END VERIFICATION CONTEXT ===

"#,
        facts.name,
        facts.version,
        facts.dependency_count,
        facts.module_count,
        facts.total_files,
        facts.lines_of_code,
        if facts.is_enterprise {
            "Enterprise-grade"
        } else {
            "Simple project"
        },
        facts.verified_at.format("%Y-%m-%d %H:%M UTC"),
        facts.root_path.display(),
        facts.file_extensions.join(", "),
        facts.major_directories.join(", "),
        stage_instructions
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_repository_verification() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();

        // Create a minimal Cargo.toml
        let cargo_content = r#"
[package]
name = "test-project"
version = "1.0.0"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#;
        let mut cargo_file = File::create(root_path.join("Cargo.toml")).unwrap();
        cargo_file.write_all(cargo_content.as_bytes()).unwrap();

        // Create some Rust files
        std::fs::create_dir(root_path.join("src")).unwrap();
        let mut main_file = File::create(root_path.join("src").join("main.rs")).unwrap();
        main_file.write_all(b"fn main() {}\n").unwrap();

        let mut verifier = RepositoryVerifier::new(root_path);
        let facts = verifier.verify_repository_context().await.unwrap();

        assert_eq!(facts.name, "test-project");
        assert_eq!(facts.version, "1.0.0");
        assert_eq!(facts.dependency_count, 2);
        assert!(facts.module_count >= 1);
        assert!(!facts.is_enterprise); // Small test project
    }

    #[test]
    fn test_context_generation() {
        let facts = RepositoryFacts {
            name: "hive-ai".to_string(),
            version: "2.0.2".to_string(),
            dependency_count: 100,
            module_count: 25,
            total_files: 150,
            lines_of_code: 10000,
            is_enterprise: true,
            verified_at: Utc::now(),
            root_path: PathBuf::from("/test/hive"),
            file_extensions: vec!["rs".to_string(), "toml".to_string()],
            major_directories: vec!["src".to_string(), "tests".to_string()],
        };

        let context = build_stage_context(&facts, crate::consensus::types::Stage::Validator);

        assert!(context.contains("hive-ai v2.0.2"));
        assert!(context.contains("100 external crates"));
        assert!(context.contains("Enterprise-grade"));
        assert!(context.contains("VALIDATOR:"));
    }
}
