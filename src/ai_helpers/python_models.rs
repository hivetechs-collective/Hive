//! Python Model Integration
//!
//! This module provides integration with Python-based AI models through subprocess
//! communication, allowing us to use transformers and other Python ML libraries.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};

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

        // Production mode: Use bundled Python from Electron app
        // Development mode: Fall back to system Python
        let python_path = if let Ok(bundled_python) = std::env::var("HIVE_BUNDLED_PYTHON") {
            // Use the Python bundled with Electron app
            tracing::info!("Using bundled Python from Electron: {}", bundled_python);
            bundled_python
        } else if let Ok(current_dir) = std::env::current_dir() {
            // Development fallback: Check for venv in current directory
            let venv_python = current_dir.join("venv").join("bin").join("python3");
            if venv_python.exists() {
                tracing::info!("Using venv Python: {}", venv_python.display());
                venv_python.to_string_lossy().to_string()
            } else {
                tracing::info!("Using system Python");
                "python3".to_string()
            }
        } else {
            "python3".to_string()
        };

        // Use bundled model service script if running from Electron
        let service_script = if let Ok(bundled_script) = std::env::var("HIVE_BUNDLED_MODEL_SCRIPT")
        {
            tracing::info!("Using bundled model service script: {}", bundled_script);
            bundled_script
        } else {
            models_dir
                .join("model_service.py")
                .to_string_lossy()
                .to_string()
        };

        Self {
            python_path,
            service_script,
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
    HealthCheck { request_id: String },
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
    GenerateResult { text: String, request_id: String },

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
    Error { error: String, request_id: String },
}

/// Python model service manager
pub struct PythonModelService {
    config: PythonModelConfig,
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    response_handlers:
        Arc<RwLock<std::collections::HashMap<String, tokio::sync::oneshot::Sender<ModelResponse>>>>,
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
            let script_dir = script_path.parent().context("Invalid script path")?;
            tokio::fs::create_dir_all(script_dir).await?;

            let script_content = include_str!("../../python/model_service.py");
            tokio::fs::write(&config.service_script, script_content)
                .await
                .context("Failed to write model service script")?;
        }

        Ok(())
    }

    /// Start the Python process
    async fn start(&self) -> Result<()> {
        tracing::info!("🐍 Starting Python model service...");
        tracing::info!("Python path: {}", self.config.python_path);
        tracing::info!("Script path: {}", self.config.service_script);
        tracing::info!("Model cache dir: {}", self.config.model_cache_dir);

        // Verify files exist before spawning
        if !std::path::Path::new(&self.config.python_path).exists() {
            tracing::error!(
                "❌ Python executable not found at: {}",
                self.config.python_path
            );
            anyhow::bail!(
                "Python executable not found at: {}",
                self.config.python_path
            );
        }

        if !std::path::Path::new(&self.config.service_script).exists() {
            tracing::error!(
                "❌ Service script not found at: {}",
                self.config.service_script
            );
            anyhow::bail!(
                "Service script not found at: {}",
                self.config.service_script
            );
        }

        tracing::info!(
            "✅ Python executable exists at: {}",
            self.config.python_path
        );
        tracing::info!(
            "✅ Service script exists at: {}",
            self.config.service_script
        );

        // Log current working directory
        if let Ok(cwd) = std::env::current_dir() {
            tracing::info!("Current working directory: {:?}", cwd);
        }

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
        tracing::info!(
            "Command: {} {} --model-cache-dir {}",
            self.config.python_path,
            self.config.service_script,
            self.config.model_cache_dir
        );

        let mut child = match cmd.spawn() {
            Ok(child) => {
                tracing::info!("✅ Python subprocess spawned successfully");
                child
            }
            Err(e) => {
                tracing::error!("❌ Failed to spawn Python subprocess: {}", e);
                tracing::error!("Error kind: {:?}", e.kind());
                tracing::error!("Raw OS error: {:?}", e.raw_os_error());
                return Err(anyhow::anyhow!("Failed to spawn Python subprocess: {}", e));
            }
        };

        tracing::info!("✅ Python subprocess spawned with PID: {:?}", child.id());

        // Get stdin for sending requests
        let stdin = child.stdin.take().context("Failed to get stdin")?;

        // Get stdout for receiving responses
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        // Get stderr for error monitoring
        let stderr = child.stderr.take().context("Failed to get stderr")?;

        // Start stderr handler to capture Python errors
        tokio::spawn(async move {
            tracing::info!("📝 Starting Python stderr handler...");
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                // Log all stderr output, even empty lines (for debugging)
                tracing::warn!("🐍 Python stderr: [{}]", line);
            }

            tracing::warn!("⚠️ Python stderr handler stopped (process likely exited)");
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

        // First check if process is still running
        if let Some(child) = self.process.lock().await.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    tracing::error!("❌ Python process already exited with status: {:?}", status);
                    anyhow::bail!(
                        "Python process exited immediately with status: {:?}",
                        status
                    );
                }
                Ok(None) => {
                    tracing::info!("✅ Python process is still running");
                }
                Err(e) => {
                    tracing::warn!("⚠️ Could not check Python process status: {}", e);
                }
            }
        }

        let request_id = uuid::Uuid::new_v4().to_string();
        tracing::info!("📤 Sending health check request with ID: {}", request_id);

        // Add timeout to health check to prevent hanging
        let response = match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            self.send_request(ModelRequest::HealthCheck {
                request_id: request_id.clone(),
            }),
        )
        .await
        {
            Ok(Ok(response)) => {
                tracing::info!("📥 Received health check response");
                response
            }
            Ok(Err(e)) => {
                tracing::error!("❌ Health check request failed: {}", e);
                return Err(anyhow::anyhow!("Health check request failed: {}", e));
            }
            Err(_) => {
                tracing::error!("❌ Health check timed out after 10 seconds");

                // Check if process is still alive after timeout
                if let Some(child) = self.process.lock().await.as_mut() {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            tracing::error!(
                                "Python process died during health check with status: {:?}",
                                status
                            );
                        }
                        Ok(None) => {
                            tracing::error!("Python process is still running but not responding");
                        }
                        Err(e) => {
                            tracing::error!("Could not check process status: {}", e);
                        }
                    }
                }

                return Err(anyhow::anyhow!(
                    "Python service health check timed out after 10 seconds"
                ));
            }
        };

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
        let response = self
            .send_request(ModelRequest::Embed {
                model: model.to_string(),
                texts,
                request_id,
            })
            .await?;

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
        let response = self
            .send_request(ModelRequest::Generate {
                model: model.to_string(),
                prompt: prompt.to_string(),
                max_tokens,
                temperature,
                request_id,
            })
            .await?;

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
        let response = self
            .send_request(ModelRequest::Analyze {
                model: model.to_string(),
                code: code.to_string(),
                task: task.to_string(),
                request_id,
            })
            .await?;

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

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_python_service_lifecycle() {
        // Test service creation and shutdown
    }
}
