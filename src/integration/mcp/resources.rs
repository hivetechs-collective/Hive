//! MCP resource management
//!
//! Provides access to codebase resources via MCP

use super::protocol::{Resource, ResourceContent, ResourceData};
use crate::core::config::Config;
use crate::core::security::SecurityManager;

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::path::Path;
use std::fs;
use tracing::{info, warn};
use url::Url;
use walkdir::WalkDir;

/// Resource manager for MCP server
pub struct ResourceManager {
    config: Arc<Config>,
    security: SecurityManager,
}

impl ResourceManager {
    /// Create new resource manager
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let security = SecurityManager::new(None, true)?;
        
        Ok(Self {
            config,
            security,
        })
    }

    /// List available resources
    pub async fn list_resources(&self) -> Result<Vec<Resource>> {
        let mut resources = Vec::new();

        // Get current working directory resources
        if let Ok(cwd) = std::env::current_dir() {
            if let Ok(workspace_resources) = self.scan_workspace(&cwd).await {
                resources.extend(workspace_resources);
            }
        }

        // Add system resources
        resources.extend(self.get_system_resources().await?);

        info!("Found {} resources", resources.len());
        Ok(resources)
    }

    /// Read a resource by URI
    pub async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContent>> {
        info!("Reading resource: {}", uri);

        // Parse URI
        let url = Url::parse(uri)
            .map_err(|e| anyhow!("Invalid resource URI: {}", e))?;

        match url.scheme() {
            "file" => self.read_file_resource(&url).await,
            "hive" => self.read_hive_resource(&url).await,
            _ => Err(anyhow!("Unsupported resource scheme: {}", url.scheme())),
        }
    }

    /// Scan workspace for resources
    async fn scan_workspace(&self, path: &Path) -> Result<Vec<Resource>> {
        let mut resources = Vec::new();

        // Check if we have permission to access this directory
        if !self.security.is_path_trusted(path)? {
            warn!("Access denied to untrusted path: {}", path.display());
            return Ok(resources);
        }

        // Walk directory tree
        let walker = WalkDir::new(path)
            .follow_links(false)
            .max_depth(10);

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_path = entry.path();
            
            // Skip hidden files and directories
            if file_path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false) {
                continue;
            }

            // Skip common build/output directories
            if let Some(parent) = file_path.parent() {
                let parent_name = parent.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                if matches!(parent_name, "target" | "node_modules" | ".git" | "dist" | "build") {
                    continue;
                }
            }

            if file_path.is_file() {
                if let Some(resource) = self.create_file_resource(file_path)? {
                    resources.push(resource);
                }
            }
        }

        Ok(resources)
    }

    /// Create file resource
    fn create_file_resource(&self, path: &Path) -> Result<Option<Resource>> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Only include text files
        let mime_type = match extension {
            "rs" => "text/x-rust",
            "js" | "mjs" => "text/javascript", 
            "ts" => "text/typescript",
            "py" => "text/x-python",
            "java" => "text/x-java",
            "cpp" | "cc" | "cxx" => "text/x-c++",
            "c" => "text/x-c",
            "h" | "hpp" => "text/x-c-header",
            "go" => "text/x-go",
            "php" => "text/x-php",
            "rb" => "text/x-ruby",
            "swift" => "text/x-swift",
            "kt" => "text/x-kotlin",
            "scala" => "text/x-scala",
            "json" => "application/json",
            "xml" => "application/xml",
            "yaml" | "yml" => "application/x-yaml",
            "toml" => "application/toml",
            "md" => "text/markdown",
            "txt" => "text/plain",
            "csv" => "text/csv",
            "sql" => "text/x-sql",
            "html" => "text/html",
            "css" => "text/css",
            "scss" | "sass" => "text/x-scss",
            "sh" | "bash" => "text/x-shellscript",
            "dockerfile" => "text/x-dockerfile",
            _ => return Ok(None), // Skip unknown file types
        };

        let uri = format!("file://{}", path.display());
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Some(Resource {
            uri,
            name,
            description: Some(format!("Source file: {}", path.display())),
            mime_type: Some(mime_type.to_string()),
        }))
    }

    /// Get system resources
    async fn get_system_resources(&self) -> Result<Vec<Resource>> {
        let mut resources = Vec::new();

        // Add Hive-specific resources
        resources.push(Resource {
            uri: "hive://config".to_string(),
            name: "Hive Configuration".to_string(),
            description: Some("Current Hive AI configuration".to_string()),
            mime_type: Some("application/toml".to_string()),
        });

        resources.push(Resource {
            uri: "hive://memory/conversations".to_string(),
            name: "Conversation History".to_string(),
            description: Some("Recent conversation summaries".to_string()),
            mime_type: Some("application/json".to_string()),
        });

        resources.push(Resource {
            uri: "hive://analysis/repository".to_string(),
            name: "Repository Analysis".to_string(),
            description: Some("Current repository analysis data".to_string()),
            mime_type: Some("application/json".to_string()),
        });

        Ok(resources)
    }

    /// Read file resource
    async fn read_file_resource(&self, url: &Url) -> Result<Vec<ResourceContent>> {
        let path = Path::new(url.path());

        // Security check
        if !self.security.is_path_trusted(path)? {
            return Err(anyhow!("Access denied to untrusted file: {}", path.display()));
        }

        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read file {}: {}", path.display(), e))?;

        let mime_type = self.detect_mime_type(path);

        Ok(vec![ResourceContent {
            uri: url.to_string(),
            mime_type,
            content: ResourceData::Text { text: content },
        }])
    }

    /// Read Hive resource
    async fn read_hive_resource(&self, url: &Url) -> Result<Vec<ResourceContent>> {
        let resource_path = url.path().trim_start_matches('/');

        match resource_path {
            "config" => {
                let config_content = toml::to_string(&*self.config)
                    .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

                Ok(vec![ResourceContent {
                    uri: url.to_string(),
                    mime_type: "application/toml".to_string(),
                    content: ResourceData::Text { text: config_content },
                }])
            }
            "memory/conversations" => {
                // TODO: Implement conversation history retrieval
                let placeholder = serde_json::json!({
                    "conversations": [],
                    "note": "Conversation history integration pending"
                });

                Ok(vec![ResourceContent {
                    uri: url.to_string(),
                    mime_type: "application/json".to_string(),
                    content: ResourceData::Text { 
                        text: serde_json::to_string_pretty(&placeholder)? 
                    },
                }])
            }
            "analysis/repository" => {
                // TODO: Implement repository analysis retrieval
                let placeholder = serde_json::json!({
                    "analysis": {},
                    "note": "Repository analysis integration pending"
                });

                Ok(vec![ResourceContent {
                    uri: url.to_string(),
                    mime_type: "application/json".to_string(),
                    content: ResourceData::Text { 
                        text: serde_json::to_string_pretty(&placeholder)? 
                    },
                }])
            }
            _ => Err(anyhow!("Unknown Hive resource: {}", resource_path)),
        }
    }

    /// Detect MIME type for file
    fn detect_mime_type(&self, path: &Path) -> String {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "rs" => "text/x-rust",
            "js" | "mjs" => "text/javascript",
            "ts" => "text/typescript", 
            "py" => "text/x-python",
            "java" => "text/x-java",
            "cpp" | "cc" | "cxx" => "text/x-c++",
            "c" => "text/x-c",
            "h" | "hpp" => "text/x-c-header",
            "go" => "text/x-go",
            "php" => "text/x-php",
            "rb" => "text/x-ruby",
            "swift" => "text/x-swift",
            "kt" => "text/x-kotlin",
            "scala" => "text/x-scala",
            "json" => "application/json",
            "xml" => "application/xml",
            "yaml" | "yml" => "application/x-yaml",
            "toml" => "application/toml",
            "md" => "text/markdown",
            "txt" => "text/plain",
            "csv" => "text/csv",
            "sql" => "text/x-sql",
            "html" => "text/html",
            "css" => "text/css",
            "scss" | "sass" => "text/x-scss",
            "sh" | "bash" => "text/x-shellscript",
            "dockerfile" => "text/x-dockerfile",
            _ => "text/plain",
        }.to_string()
    }
}