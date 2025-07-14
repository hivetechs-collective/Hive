//! NPM package distribution for global installation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tokio::fs as afs;

/// NPM package manager for Hive AI
#[derive(Debug, Clone)]
pub struct NpmPackageManager {
    /// Package name
    pub package_name: String,
    /// Package version
    pub version: String,
    /// Package directory
    pub package_dir: PathBuf,
}

/// NPM package configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmPackageConfig {
    /// Package name (@hivetechs/hive-ai)
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    pub description: String,
    /// Package keywords
    pub keywords: Vec<String>,
    /// Package homepage
    pub homepage: String,
    /// Package repository
    pub repository: RepositoryConfig,
    /// Package author
    pub author: AuthorConfig,
    /// Package license
    pub license: String,
    /// Binary configuration
    pub bin: std::collections::HashMap<String, String>,
    /// Package files to include
    pub files: Vec<String>,
    /// Package engines
    pub engines: EngineConfig,
    /// Package scripts
    pub scripts: std::collections::HashMap<String, String>,
    /// Package dependencies
    pub dependencies: std::collections::HashMap<String, String>,
    /// Package dev dependencies
    pub dev_dependencies: std::collections::HashMap<String, String>,
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    #[serde(rename = "type")]
    pub repo_type: String,
    pub url: String,
}

/// Author configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorConfig {
    pub name: String,
    pub email: String,
    pub url: String,
}

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub node: String,
    pub npm: String,
}

impl Default for NpmPackageConfig {
    fn default() -> Self {
        let mut bin = std::collections::HashMap::new();
        bin.insert("hive".to_string(), "bin/hive".to_string());

        let mut scripts = std::collections::HashMap::new();
        scripts.insert("postinstall".to_string(), "node install.js".to_string());
        scripts.insert("preuninstall".to_string(), "node uninstall.js".to_string());

        Self {
            name: "@hivetechs/hive-ai".to_string(),
            version: crate::VERSION.to_string(),
            description: "AI-powered codebase intelligence platform with 4-stage consensus engine"
                .to_string(),
            keywords: vec![
                "ai".to_string(),
                "codebase".to_string(),
                "intelligence".to_string(),
                "consensus".to_string(),
                "analysis".to_string(),
                "rust".to_string(),
                "cli".to_string(),
            ],
            homepage: "https://github.com/hivetechs/hive-ai".to_string(),
            repository: RepositoryConfig {
                repo_type: "git".to_string(),
                url: "https://github.com/hivetechs/hive-ai.git".to_string(),
            },
            author: AuthorConfig {
                name: "HiveTechs".to_string(),
                email: "info@hivetechs.com".to_string(),
                url: "https://hivetechs.com".to_string(),
            },
            license: "MIT".to_string(),
            bin,
            files: vec![
                "bin/".to_string(),
                "install.js".to_string(),
                "uninstall.js".to_string(),
                "README.md".to_string(),
                "LICENSE".to_string(),
            ],
            engines: EngineConfig {
                node: ">=16.0.0".to_string(),
                npm: ">=8.0.0".to_string(),
            },
            scripts,
            dependencies: std::collections::HashMap::new(),
            dev_dependencies: std::collections::HashMap::new(),
        }
    }
}

impl NpmPackageManager {
    /// Create a new NPM package manager
    pub fn new(package_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            package_name: "@hivetechs/hive-ai".to_string(),
            version: crate::VERSION.to_string(),
            package_dir,
        })
    }

    /// Create NPM package structure
    pub async fn create_package(&self) -> Result<()> {
        println!("📦 Creating NPM package structure...");

        // Create package directory
        afs::create_dir_all(&self.package_dir).await?;

        // Create package.json
        self.create_package_json().await?;

        // Create binary directory
        let bin_dir = self.package_dir.join("bin");
        afs::create_dir_all(&bin_dir).await?;

        // Copy binary
        self.copy_binary().await?;

        // Create install scripts
        self.create_install_scripts().await?;

        // Create README
        self.create_readme().await?;

        // Create LICENSE
        self.create_license().await?;

        println!("✅ NPM package created at: {}", self.package_dir.display());

        Ok(())
    }

    /// Create package.json file
    async fn create_package_json(&self) -> Result<()> {
        let config = NpmPackageConfig::default();
        let json = serde_json::to_string_pretty(&config)?;

        let package_json_path = self.package_dir.join("package.json");
        afs::write(package_json_path, json).await?;

        Ok(())
    }

    /// Copy binary to package
    async fn copy_binary(&self) -> Result<()> {
        let current_exe =
            std::env::current_exe().context("Failed to get current executable path")?;

        let bin_dir = self.package_dir.join("bin");
        let target_binary = bin_dir.join("hive");

        fs::copy(&current_exe, &target_binary).context("Failed to copy binary")?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&target_binary)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target_binary, perms)?;
        }

        Ok(())
    }

    /// Create install scripts
    async fn create_install_scripts(&self) -> Result<()> {
        // Create install.js
        let install_script = r#"#!/usr/bin/env node
// Hive AI NPM Package Installer

const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

const BINARY_NAME = 'hive';
const PACKAGE_DIR = __dirname;
const BINARY_PATH = path.join(PACKAGE_DIR, 'bin', BINARY_NAME);

// Platform-specific installation
const platform = os.platform();
const arch = os.arch();

console.log('🐝 Installing Hive AI globally...');
console.log(`Platform: ${platform}-${arch}`);

// Check if binary exists
if (!fs.existsSync(BINARY_PATH)) {
    console.error('❌ Binary not found in package');
    process.exit(1);
}

// Platform-specific installation
try {
    if (platform === 'win32') {
        installWindows();
    } else {
        installUnix();
    }

    // Test installation
    try {
        execSync(`${BINARY_NAME} --version`, { stdio: 'pipe' });
        console.log('✅ Hive AI installed successfully!');
        console.log('   Run "hive" to get started');
    } catch (error) {
        console.log('⚠️  Installation completed but verification failed');
        console.log('   You may need to restart your terminal or update your PATH');
    }
} catch (error) {
    console.error('❌ Installation failed:', error.message);
    process.exit(1);
}

function installWindows() {
    // Windows installation logic
    const programFiles = process.env.ProgramFiles || 'C:\\Program Files';
    const installDir = path.join(programFiles, 'Hive');
    const targetPath = path.join(installDir, 'hive.exe');

    // Create directory
    if (!fs.existsSync(installDir)) {
        fs.mkdirSync(installDir, { recursive: true });
    }

    // Copy binary
    fs.copyFileSync(BINARY_PATH, targetPath);

    // Add to PATH (requires admin privileges)
    try {
        execSync(`setx PATH "%PATH%;${installDir}" /M`, { stdio: 'inherit' });
        console.log('✅ Added Hive to system PATH');
    } catch (error) {
        console.log('⚠️  Could not add to system PATH automatically');
        console.log(`   Please add ${installDir} to your PATH manually`);
    }
}

function installUnix() {
    // Unix installation logic
    const installDir = '/usr/local/bin';
    const targetPath = path.join(installDir, BINARY_NAME);

    // Check if we have write permissions
    try {
        fs.accessSync(installDir, fs.constants.W_OK);
    } catch (error) {
        console.log('⚠️  No write permission to /usr/local/bin');
        console.log('   Trying with sudo...');

        try {
            execSync(`sudo cp "${BINARY_PATH}" "${targetPath}"`, { stdio: 'inherit' });
            execSync(`sudo chmod +x "${targetPath}"`, { stdio: 'inherit' });
            return;
        } catch (sudoError) {
            throw new Error('Failed to install with sudo');
        }
    }

    // Copy binary
    fs.copyFileSync(BINARY_PATH, targetPath);

    // Make executable
    fs.chmodSync(targetPath, 0o755);
}
"#;

        let install_path = self.package_dir.join("install.js");
        afs::write(install_path, install_script).await?;

        // Create uninstall.js
        let uninstall_script = r#"#!/usr/bin/env node
// Hive AI NPM Package Uninstaller

const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

const BINARY_NAME = 'hive';
const platform = os.platform();

console.log('🗑️  Uninstalling Hive AI...');

try {
    if (platform === 'win32') {
        uninstallWindows();
    } else {
        uninstallUnix();
    }

    console.log('✅ Hive AI uninstalled successfully!');
} catch (error) {
    console.error('❌ Uninstallation failed:', error.message);
    process.exit(1);
}

function uninstallWindows() {
    const programFiles = process.env.ProgramFiles || 'C:\\Program Files';
    const installDir = path.join(programFiles, 'Hive');
    const targetPath = path.join(installDir, 'hive.exe');

    if (fs.existsSync(targetPath)) {
        fs.unlinkSync(targetPath);
    }

    // Try to remove directory if empty
    try {
        fs.rmdirSync(installDir);
    } catch (error) {
        // Directory not empty or doesn't exist
    }
}

function uninstallUnix() {
    const targetPath = '/usr/local/bin/hive';

    if (fs.existsSync(targetPath)) {
        try {
            fs.unlinkSync(targetPath);
        } catch (error) {
            // Try with sudo
            try {
                execSync(`sudo rm "${targetPath}"`, { stdio: 'inherit' });
            } catch (sudoError) {
                throw new Error('Failed to remove binary');
            }
        }
    }
}
"#;

        let uninstall_path = self.package_dir.join("uninstall.js");
        afs::write(uninstall_path, uninstall_script).await?;

        Ok(())
    }

    /// Create README for NPM package
    async fn create_readme(&self) -> Result<()> {
        let readme_content = format!(
            r#"# Hive AI - NPM Package

AI-powered codebase intelligence platform with 4-stage consensus engine.

## Installation

```bash
npm install -g @hivetechs/hive-ai
```

## Usage

After installation, the `hive` command will be available globally:

```bash
# Get started
hive

# Ask questions about your codebase
hive ask "What does this function do?"

# Analyze a repository
hive analyze .

# Check status
hive status
```

## Features

- 🧠 **4-Stage Consensus Engine**: Generator → Refiner → Validator → Curator
- 🚀 **10-40x Performance**: Lightning-fast Rust implementation
- 🔍 **Repository Intelligence**: Deep codebase understanding
- 📊 **Analytics & Insights**: Advanced metrics and reporting
- 🎯 **Planning Mode**: AI-powered task decomposition
- 🖥️ **TUI Interface**: VS Code-like terminal experience

## Configuration

Configuration is stored in `~/.hive/config.toml`. Run `hive quickstart` for initial setup.

## Requirements

- Node.js 16.0.0 or higher
- npm 8.0.0 or higher

## Version

Current version: {}

## License

MIT License - see LICENSE file for details.

## Support

- GitHub: https://github.com/hivetechs/hive-ai
- Documentation: https://docs.hivetechs.com
- Issues: https://github.com/hivetechs/hive-ai/issues

---

**Note**: This is a binary distribution package. The main application is written in Rust for maximum performance.
"#,
            crate::VERSION
        );

        let readme_path = self.package_dir.join("README.md");
        afs::write(readme_path, readme_content).await?;

        Ok(())
    }

    /// Create LICENSE file
    async fn create_license(&self) -> Result<()> {
        let license_content = r#"MIT License

Copyright (c) 2024 HiveTechs

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;

        let license_path = self.package_dir.join("LICENSE");
        afs::write(license_path, license_content).await?;

        Ok(())
    }

    /// Publish package to NPM
    pub async fn publish(&self, dry_run: bool) -> Result<()> {
        println!("📤 Publishing to NPM...");

        // Validate package
        self.validate_package().await?;

        // Build package
        self.build_package().await?;

        // Publish (or dry run)
        let publish_cmd = if dry_run {
            "npm publish --dry-run"
        } else {
            "npm publish"
        };

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(publish_cmd)
            .current_dir(&self.package_dir)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("NPM publish failed: {}", stderr));
        }

        if dry_run {
            println!("✅ Package validation successful (dry run)");
        } else {
            println!("✅ Package published successfully!");
        }

        Ok(())
    }

    /// Validate package before publishing
    async fn validate_package(&self) -> Result<()> {
        println!("🔍 Validating package...");

        // Check required files
        let required_files = vec![
            "package.json",
            "README.md",
            "LICENSE",
            "install.js",
            "uninstall.js",
            "bin/hive",
        ];

        for file in required_files {
            let file_path = self.package_dir.join(file);
            if !file_path.exists() {
                return Err(anyhow::anyhow!("Required file missing: {}", file));
            }
        }

        // Validate package.json
        let package_json = self.package_dir.join("package.json");
        let content = afs::read_to_string(package_json).await?;
        let _: serde_json::Value = serde_json::from_str(&content)?;

        // Check binary executable
        let binary_path = self.package_dir.join("bin/hive");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&binary_path)?;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 == 0 {
                return Err(anyhow::anyhow!("Binary is not executable"));
            }
        }

        println!("✅ Package validation passed");

        Ok(())
    }

    /// Build package for distribution
    async fn build_package(&self) -> Result<()> {
        println!("🏗️  Building package...");

        // Update package.json with latest version
        let mut config = NpmPackageConfig::default();
        config.version = crate::VERSION.to_string();

        let json = serde_json::to_string_pretty(&config)?;
        let package_json_path = self.package_dir.join("package.json");
        afs::write(package_json_path, json).await?;

        // Copy latest binary
        self.copy_binary().await?;

        println!("✅ Package built successfully");

        Ok(())
    }

    /// Get package information
    pub async fn get_package_info(&self) -> Result<NpmPackageInfo> {
        let package_json_path = self.package_dir.join("package.json");

        if !package_json_path.exists() {
            return Err(anyhow::anyhow!("Package not found"));
        }

        let content = afs::read_to_string(package_json_path).await?;
        let config: NpmPackageConfig = serde_json::from_str(&content)?;

        let binary_path = self.package_dir.join("bin/hive");
        let binary_size = if binary_path.exists() {
            fs::metadata(&binary_path)?.len()
        } else {
            0
        };

        Ok(NpmPackageInfo {
            name: config.name,
            version: config.version,
            description: config.description,
            binary_size,
            package_size: self.calculate_package_size().await?,
            created_at: fs::metadata(&self.package_dir)?.created().ok(),
        })
    }

    /// Calculate total package size
    async fn calculate_package_size(&self) -> Result<u64> {
        let mut total_size = 0;

        fn visit_dir(dir: &std::path::Path, total: &mut u64) -> Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dir(&path, total)?;
                    } else {
                        *total += entry.metadata()?.len();
                    }
                }
            }
            Ok(())
        }

        visit_dir(&self.package_dir, &mut total_size)?;

        Ok(total_size)
    }
}

/// NPM package information
#[derive(Debug, Clone, Serialize)]
pub struct NpmPackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub binary_size: u64,
    pub package_size: u64,
    pub created_at: Option<std::time::SystemTime>,
}

/// Create a complete NPM package
pub async fn create_npm_package(output_dir: &PathBuf) -> Result<()> {
    let package_manager = NpmPackageManager::new(output_dir.clone())?;
    package_manager.create_package().await?;

    Ok(())
}

/// Publish NPM package
pub async fn publish_npm_package(package_dir: &PathBuf, dry_run: bool) -> Result<()> {
    let package_manager = NpmPackageManager::new(package_dir.clone())?;
    package_manager.publish(dry_run).await?;

    Ok(())
}
