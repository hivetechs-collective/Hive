//! Model Downloader for AI Helpers
//!
//! This module handles downloading and setting up AI models on first run,
//! ensuring all required models are available locally.

use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Mutex};

/// Model registry with information about available models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub model_id: String,
    pub download_url: String,
    pub size_mb: u64,
    pub checksum: String,
    pub required: bool,
    pub description: String,
}

/// Download progress event
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started {
        model: String,
        size: u64,
    },
    Progress {
        model: String,
        downloaded: u64,
        total: u64,
    },
    Completed {
        model: String,
    },
    Failed {
        model: String,
        error: String,
    },
}

/// Model downloader configuration
#[derive(Debug, Clone)]
pub struct DownloaderConfig {
    /// Base directory for models
    pub models_dir: PathBuf,

    /// Maximum concurrent downloads
    pub max_concurrent: usize,

    /// Timeout for downloads
    pub timeout: std::time::Duration,

    /// Whether to verify checksums
    pub verify_checksums: bool,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            models_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("hive")
                .join("models"),
            max_concurrent: 2,
            timeout: std::time::Duration::from_secs(3600), // 1 hour
            verify_checksums: true,
        }
    }
}

/// Model downloader for AI helpers
pub struct ModelDownloader {
    config: DownloaderConfig,
    client: Client,
    models_registry: Vec<ModelInfo>,
    progress: Arc<Mutex<MultiProgress>>,
}

impl ModelDownloader {
    /// Create a new model downloader
    pub async fn new(config: DownloaderConfig) -> Result<Self> {
        // Ensure models directory exists
        tokio::fs::create_dir_all(&config.models_dir)
            .await
            .context("Failed to create models directory")?;

        let client = Client::builder().timeout(config.timeout).build()?;

        let models_registry = Self::load_models_registry();
        let progress = Arc::new(Mutex::new(MultiProgress::new()));

        Ok(Self {
            config,
            client,
            models_registry,
            progress,
        })
    }

    /// Load the models registry
    fn load_models_registry() -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                name: "CodeBERT Base".to_string(),
                model_id: "microsoft/codebert-base".to_string(),
                download_url: "https://huggingface.co/microsoft/codebert-base/resolve/main/pytorch_model.bin".to_string(),
                size_mb: 420,
                checksum: "abc123".to_string(), // TODO: Real checksum
                required: true,
                description: "Pre-trained model for code understanding".to_string(),
            },
            ModelInfo {
                name: "CodeT5+ 110M Embedding".to_string(),
                model_id: "Salesforce/codet5p-110m-embedding".to_string(),
                download_url: "https://huggingface.co/Salesforce/codet5p-110m-embedding/resolve/main/pytorch_model.bin".to_string(),
                size_mb: 110,
                checksum: "def456".to_string(), // TODO: Real checksum
                required: true,
                description: "Code embedding model for semantic search".to_string(),
            },
            ModelInfo {
                name: "GraphCodeBERT".to_string(),
                model_id: "microsoft/graphcodebert-base".to_string(),
                download_url: "https://huggingface.co/microsoft/graphcodebert-base/resolve/main/pytorch_model.bin".to_string(),
                size_mb: 420,
                checksum: "ghi789".to_string(), // TODO: Real checksum
                required: true,
                description: "Code understanding with graph structure".to_string(),
            },
            ModelInfo {
                name: "UniXcoder Base".to_string(),
                model_id: "microsoft/unixcoder-base".to_string(),
                download_url: "https://huggingface.co/microsoft/unixcoder-base/resolve/main/pytorch_model.bin".to_string(),
                size_mb: 420,
                checksum: "jkl012".to_string(), // TODO: Real checksum
                required: true,
                description: "Cross-language code representation".to_string(),
            },
            ModelInfo {
                name: "Mistral 7B Instruct Q4".to_string(),
                model_id: "mistral-7b-instruct-v0.1.Q4_K_M.gguf".to_string(),
                download_url: "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.1-GGUF/resolve/main/mistral-7b-instruct-v0.1.Q4_K_M.gguf".to_string(),
                size_mb: 4370,
                checksum: "mno345".to_string(), // TODO: Real checksum
                required: false,
                description: "Local LLM for knowledge synthesis (4-bit quantized)".to_string(),
            },
        ]
    }

    /// Check which models need to be downloaded
    pub async fn check_missing_models(&self) -> Vec<ModelInfo> {
        let mut missing = Vec::new();

        for model in &self.models_registry {
            let model_path = self.get_model_path(&model.model_id);
            if !model_path.exists() {
                missing.push(model.clone());
            }
        }

        missing
    }

    /// Download all missing required models
    pub async fn download_required_models(
        &self,
        event_sender: mpsc::Sender<DownloadEvent>,
    ) -> Result<()> {
        let missing = self.check_missing_models().await;
        let required_missing: Vec<_> = missing.into_iter().filter(|m| m.required).collect();

        if required_missing.is_empty() {
            tracing::info!("All required models are already downloaded");
            return Ok(());
        }

        tracing::info!(
            "Need to download {} required models",
            required_missing.len()
        );

        // Download models with limited concurrency
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrent));
        let mut handles = Vec::new();

        for model in required_missing {
            let semaphore = semaphore.clone();
            let client = self.client.clone();
            let config = self.config.clone();
            let sender = event_sender.clone();
            let progress = self.progress.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await?;
                Self::download_model(model, client, config, sender, progress).await
            });

            handles.push(handle);
        }

        // Wait for all downloads to complete
        for handle in handles {
            handle.await??;
        }

        Ok(())
    }

    /// Download a single model
    async fn download_model(
        model: ModelInfo,
        client: Client,
        config: DownloaderConfig,
        event_sender: mpsc::Sender<DownloadEvent>,
        multi_progress: Arc<Mutex<MultiProgress>>,
    ) -> Result<()> {
        let model_path = config.models_dir.join(&model.model_id);

        // Create model directory
        if let Some(parent) = model_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Send start event
        event_sender
            .send(DownloadEvent::Started {
                model: model.name.clone(),
                size: model.size_mb * 1024 * 1024,
            })
            .await?;

        // Create progress bar
        let pb = {
            let mp = multi_progress.lock().await;
            let pb = mp.add(ProgressBar::new(model.size_mb * 1024 * 1024));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} {msg}")
                    .unwrap()
                    .progress_chars("##-"),
            );
            pb.set_message(format!("Downloading {}", model.name));
            pb
        };

        // Download with progress
        let response = client.get(&model.download_url).send().await?;
        let total_size = response
            .content_length()
            .unwrap_or(model.size_mb * 1024 * 1024);

        let temp_path = model_path.with_extension("tmp");
        let mut file = tokio::fs::File::create(&temp_path).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;

            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);

            // Send progress event
            event_sender
                .send(DownloadEvent::Progress {
                    model: model.name.clone(),
                    downloaded,
                    total: total_size,
                })
                .await?;
        }

        file.flush().await?;
        drop(file);

        // Verify checksum if enabled
        if config.verify_checksums {
            // TODO: Implement actual checksum verification
            tracing::debug!("Checksum verification not yet implemented");
        }

        // Move temp file to final location
        tokio::fs::rename(temp_path, model_path).await?;

        pb.finish_with_message(format!("{} downloaded successfully", model.name));

        // Send completion event
        event_sender
            .send(DownloadEvent::Completed {
                model: model.name.clone(),
            })
            .await?;

        Ok(())
    }

    /// Get the path where a model should be stored
    fn get_model_path(&self, model_id: &str) -> PathBuf {
        self.config.models_dir.join(model_id)
    }

    /// Initialize models (download if needed)
    pub async fn initialize_models(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(100);

        // Spawn task to handle events
        let event_task = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    DownloadEvent::Started { model, size } => {
                        tracing::info!(
                            "Starting download of {} ({:.1} MB)",
                            model,
                            size as f64 / 1_048_576.0
                        );
                    }
                    DownloadEvent::Progress {
                        model,
                        downloaded,
                        total,
                    } => {
                        let percent = (downloaded as f64 / total as f64) * 100.0;
                        tracing::debug!("{}: {:.1}%", model, percent);
                    }
                    DownloadEvent::Completed { model } => {
                        tracing::info!("✓ {} downloaded successfully", model);
                    }
                    DownloadEvent::Failed { model, error } => {
                        tracing::error!("✗ Failed to download {}: {}", model, error);
                    }
                }
            }
        });

        // Download required models
        self.download_required_models(tx).await?;

        event_task.await?;

        Ok(())
    }

    /// Get paths to all downloaded models
    pub async fn get_model_paths(&self) -> Result<std::collections::HashMap<String, PathBuf>> {
        let mut paths = std::collections::HashMap::new();

        for model in &self.models_registry {
            let path = self.get_model_path(&model.model_id);
            if path.exists() {
                paths.insert(model.model_id.clone(), path);
            }
        }

        Ok(paths)
    }
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_registry() {
        let config = DownloaderConfig::default();
        let downloader = ModelDownloader::new(config).await.unwrap();
        assert!(!downloader.models_registry.is_empty());
    }
}
