//! Python Model Integration
//! 
//! This module provides integration with Python-based AI models through subprocess
//! communication, allowing us to use transformers and other Python ML libraries.

use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{Mutex, RwLock};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json;

/// Python model service configuration
#[derive(Debug, Clone)]
pub struct PythonModelConfig {
    /// Python executable path
    pub python_path: String,
    
    /// Model service script path
    pub service_script: String,
    
    /// Model cache directory
    pub model_cache_dir: String,
    
    /// Maximum concurrent requests per model
    pub max_concurrent_requests: usize,
    
    /// Request timeout
    pub request_timeout: std::time::Duration,
}

impl Default for PythonModelConfig {
    fn default() -> Self {
        let models_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("hive")
            .join("models");
        
        // Try to use virtual environment Python first
        let python_path = if let Ok(current_dir) = std::env::current_dir() {
            let venv_python = current_dir.join("venv").join("bin").join("python3");
            if venv_python.exists() {
                venv_python.to_string_lossy().to_string()
            } else {
                "python3".to_string()
            }
        } else {
            "python3".to_string()
        };
        
        Self {
            python_path,
            service_script: models_dir.join("model_service.py").to_string_lossy().to_string(),
            model_cache_dir: models_dir.join("cache").to_string_lossy().to_string(),
            max_concurrent_requests: 4,
            request_timeout: std::time::Duration::from_secs(300), // 5 minutes for model downloads
        }
    }
}

/// Request to Python model service
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ModelRequest {
    #[serde(rename = "embed")]
    Embed {
        model: String,
        texts: Vec<String>,
        request_id: String,
    },
    
    #[serde(rename = "generate")]
    Generate {
        model: String,
        prompt: String,
        max_tokens: usize,
        temperature: f64,
        request_id: String,
    },
    
    #[serde(rename = "analyze")]
    Analyze {
        model: String,
        code: String,
        task: String,
        request_id: String,
    },
    
    #[serde(rename = "health")]
    HealthCheck {
        request_id: String,
    },
}

/// Response from Python model service
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ModelResponse {
    #[serde(rename = "embed_result")]
    EmbedResult {
        embeddings: Vec<Vec<f32>>,
        request_id: String,
    },
    
    #[serde(rename = "generate_result")]
    GenerateResult {
        text: String,
        request_id: String,
    },
    
    #[serde(rename = "analyze_result")]
    AnalyzeResult {
        result: serde_json::Value,
        request_id: String,
    },
    
    #[serde(rename = "health_result")]
    HealthResult {
        status: String,
        models_loaded: Vec<String>,
        request_id: String,
    },
    
    #[serde(rename = "error")]
    Error {
        error: String,
        request_id: String,
    },
}

/// Python model service manager
pub struct PythonModelService {
    config: PythonModelConfig,
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    response_handlers: Arc<RwLock<std::collections::HashMap<String, tokio::sync::oneshot::Sender<ModelResponse>>>>,
}

impl PythonModelService {
    /// Create a new Python model service
    pub async fn new(config: PythonModelConfig) -> Result<Self> {
        // First, ensure the Python service script exists
        Self::ensure_service_script(&config).await?;
        
        let service = Self {
            config,
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            response_handlers: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };
        
        // Start the Python process
        service.start().await?;
        
        Ok(service)
    }
    
    /// Ensure the Python service script exists
    async fn ensure_service_script(config: &PythonModelConfig) -> Result<()> {
        let script_path = std::path::Path::new(&config.service_script);
        
        if !script_path.exists() {
            // Create the service script
            let script_dir = script_path.parent()
                .context("Invalid script path")?;
            tokio::fs::create_dir_all(script_dir).await?;
            
            let script_content = include_str!("../../python/model_service.py");
            tokio::fs::write(&config.service_script, script_content).await
                .context("Failed to write model service script")?;
        }
        
        Ok(())
    }
    
    /// Start the Python process
    async fn start(&self) -> Result<()> {
        tracing::info!("🐍 Starting Python model service...");
        tracing::info!("Python path: {}", self.config.python_path);
        tracing::info!("Script path: {}", self.config.service_script);
        
        let mut cmd = Command::new(&self.config.python_path);
        cmd.arg(&self.config.service_script)
            .arg("--model-cache-dir")
            .arg(&self.config.model_cache_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // Critical: Kill the subprocess if parent dies
            .kill_on_drop(true)
            // Set environment to ensure Python subprocess works correctly
            .env("PYTHONUNBUFFERED", "1")
            .env("TRANSFORMERS_OFFLINE", "0")
            .env("HF_HOME", &self.config.model_cache_dir);
        
        tracing::info!("🚀 Spawning Python subprocess with enhanced environment...");
        
        let mut child = cmd.spawn()
            .context("Failed to spawn Python model service")?;
        
        tracing::info!("✅ Python subprocess spawned with PID: {:?}", child.id());
        
        // Get stdin for sending requests
        let stdin = child.stdin.take()
            .context("Failed to get stdin")?;
        
        // Get stdout for receiving responses
        let stdout = child.stdout.take()
            .context("Failed to get stdout")?;
        
        // Get stderr for error monitoring
        let stderr = child.stderr.take()
            .context("Failed to get stderr")?;
        
        // Start stderr handler to capture Python errors
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            
            while let Ok(Some(line)) = lines.next_line().await {
                if !line.trim().is_empty() {
                    tracing::warn!("🐍 Python stderr: {}", line);
                }
            }
        });
        
        // Start response handler
        let response_handlers = self.response_handlers.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(response) = serde_json::from_str::<ModelResponse>(&line) {
                    let request_id = match &response {
                        ModelResponse::EmbedResult { request_id, .. } => request_id,
                        ModelResponse::GenerateResult { request_id, .. } => request_id,
                        ModelResponse::AnalyzeResult { request_id, .. } => request_id,
                        ModelResponse::HealthResult { request_id, .. } => request_id,
                        ModelResponse::Error { request_id, .. } => request_id,
                    };
                    
                    if let Some(sender) = response_handlers.write().await.remove(request_id) {
                        let _ = sender.send(response);
                    }
                }
            }
        });
        
        // Store process and stdin
        *self.process.lock().await = Some(child);
        *self.stdin.lock().await = Some(stdin);
        
        // Wait for service to be ready
        self.wait_for_ready().await?;
        
        Ok(())
    }
    
    /// Wait for the service to be ready
    async fn wait_for_ready(&self) -> Result<()> {
        tracing::info!("⏳ Waiting for Python service to be ready...");
        
        let request_id = uuid::Uuid::new_v4().to_string();
        
        // Add timeout to health check to prevent hanging
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            self.send_request(ModelRequest::HealthCheck { request_id })
        ).await
        .context("Python service health check timed out after 10 seconds")?
        .context("Python service health check failed")?;
        
        match response {
            ModelResponse::HealthResult { status, .. } => {
                if status != "ready" {
                    anyhow::bail!("Python model service not ready: {}", status);
                }
                Ok(())
            }
            ModelResponse::Error { error, .. } => {
                anyhow::bail!("Python model service error: {}", error)
            }
            _ => anyhow::bail!("Unexpected response from health check"),
        }
    }
    
    /// Send a request to the Python service
    async fn send_request(&self, request: ModelRequest) -> Result<ModelResponse> {
        let request_id = match &request {
            ModelRequest::Embed { request_id, .. } => request_id.clone(),
            ModelRequest::Generate { request_id, .. } => request_id.clone(),
            ModelRequest::Analyze { request_id, .. } => request_id.clone(),
            ModelRequest::HealthCheck { request_id } => request_id.clone(),
        };
        
        // Create response channel
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.response_handlers.write().await.insert(request_id, tx);
        
        // Send request
        let request_json = serde_json::to_string(&request)?;
        if let Some(stdin) = &mut *self.stdin.lock().await {
            stdin.write_all(request_json.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
        } else {
            anyhow::bail!("Python service stdin not available");
        }
        
        // Wait for response with timeout
        match tokio::time::timeout(self.config.request_timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => anyhow::bail!("Response channel closed"),
            Err(_) => anyhow::bail!("Request timeout"),
        }
    }
    
    /// Generate embeddings using CodeBERT/CodeT5+
    pub async fn generate_embeddings(
        &self,
        model: &str,
        texts: Vec<String>,
    ) -> Result<Vec<Vec<f32>>> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let response = self.send_request(ModelRequest::Embed {
            model: model.to_string(),
            texts,
            request_id,
        }).await?;
        
        match response {
            ModelResponse::EmbedResult { embeddings, .. } => Ok(embeddings),
            ModelResponse::Error { error, .. } => anyhow::bail!("Embedding error: {}", error),
            _ => anyhow::bail!("Unexpected response type"),
        }
    }
    
    /// Generate text using local LLM
    pub async fn generate_text(
        &self,
        model: &str,
        prompt: &str,
        max_tokens: usize,
        temperature: f64,
    ) -> Result<String> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let response = self.send_request(ModelRequest::Generate {
            model: model.to_string(),
            prompt: prompt.to_string(),
            max_tokens,
            temperature,
            request_id,
        }).await?;
        
        match response {
            ModelResponse::GenerateResult { text, .. } => Ok(text),
            ModelResponse::Error { error, .. } => anyhow::bail!("Generation error: {}", error),
            _ => anyhow::bail!("Unexpected response type"),
        }
    }
    
    /// Analyze code using specialized model
    pub async fn analyze_code(
        &self,
        model: &str,
        code: &str,
        task: &str,
    ) -> Result<serde_json::Value> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let response = self.send_request(ModelRequest::Analyze {
            model: model.to_string(),
            code: code.to_string(),
            task: task.to_string(),
            request_id,
        }).await?;
        
        match response {
            ModelResponse::AnalyzeResult { result, .. } => Ok(result),
            ModelResponse::Error { error, .. } => anyhow::bail!("Analysis error: {}", error),
            _ => anyhow::bail!("Unexpected response type"),
        }
    }
    
    /// Shutdown the Python service
    pub async fn shutdown(&self) -> Result<()> {
        if let Some(mut child) = self.process.lock().await.take() {
            child.kill().await?;
        }
        Ok(())
    }
}

impl Drop for PythonModelService {
    fn drop(&mut self) {
        // Attempt to shutdown the Python process
        let process = self.process.clone();
        tokio::spawn(async move {
            if let Some(mut child) = process.lock().await.take() {
                let _ = child.kill().await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_python_service_lifecycle() {
        // Test service creation and shutdown
    }
}