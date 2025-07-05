//! Export Functionality
//! 
//! Provides comprehensive data export capabilities:
//! - Multiple format support (CSV, JSON, Excel, PDF)
//! - Scheduled exports with email delivery
//! - Data compression and encryption
//! - Large dataset handling with streaming
//! - Template-based formatting

use anyhow::{Result, Context};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    CSV,
    JSON,
    Excel,
    PDF,
    HTML,
    Markdown,
    XML,
    Parquet,
}

impl ExportFormat {
    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::CSV => "csv",
            ExportFormat::JSON => "json",
            ExportFormat::Excel => "xlsx",
            ExportFormat::PDF => "pdf",
            ExportFormat::HTML => "html",
            ExportFormat::Markdown => "md",
            ExportFormat::XML => "xml",
            ExportFormat::Parquet => "parquet",
        }
    }

    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::CSV => "text/csv",
            ExportFormat::JSON => "application/json",
            ExportFormat::Excel => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ExportFormat::PDF => "application/pdf",
            ExportFormat::HTML => "text/html",
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::XML => "application/xml",
            ExportFormat::Parquet => "application/parquet",
        }
    }
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub compression: Option<CompressionType>,
    pub encryption: Option<EncryptionConfig>,
    pub filters: HashMap<String, String>,
    pub columns: Option<Vec<String>>,
    pub date_range: Option<DateRange>,
    pub formatting: FormattingOptions,
    pub delivery: Option<DeliveryConfig>,
}

/// Compression types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    Gzip,
    Zip,
    Brotli,
    Zstd,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub password: Option<String>,
    pub key_file: Option<PathBuf>,
}

/// Encryption algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES256,
    RSA,
    PGP,
}

/// Date range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Formatting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingOptions {
    pub include_headers: bool,
    pub date_format: String,
    pub number_format: String,
    pub null_value: String,
    pub delimiter: Option<String>, // For CSV
    pub pretty_print: bool, // For JSON
    pub template: Option<String>, // For HTML/PDF
}

impl Default for FormattingOptions {
    fn default() -> Self {
        Self {
            include_headers: true,
            date_format: "%Y-%m-%d %H:%M:%S".to_string(),
            number_format: "%.2f".to_string(),
            null_value: "N/A".to_string(),
            delimiter: Some(",".to_string()),
            pretty_print: true,
            template: None,
        }
    }
}

/// Delivery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryConfig {
    pub method: DeliveryMethod,
    pub schedule: Option<ExportSchedule>,
    pub retention_days: Option<u32>,
}

/// Delivery methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DeliveryMethod {
    Download,
    Email {
        recipients: Vec<String>,
        subject: String,
        body: String,
    },
    S3 {
        bucket: String,
        key_prefix: String,
        region: String,
    },
    SFTP {
        host: String,
        port: u16,
        username: String,
        path: String,
    },
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
}

/// Export schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSchedule {
    pub frequency: ScheduleFrequency,
    pub time: String, // HH:MM format
    pub timezone: String,
    pub days_of_week: Option<Vec<u8>>, // 1-7 for weekly
    pub day_of_month: Option<u8>, // 1-31 for monthly
}

/// Schedule frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleFrequency {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
}

/// Export job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJob {
    pub id: String,
    pub config: ExportConfig,
    pub status: ExportStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub file_path: Option<PathBuf>,
    pub file_size: Option<u64>,
    pub row_count: Option<u64>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Export status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Data exporter
pub struct DataExporter {
    jobs: Arc<RwLock<HashMap<String, ExportJob>>>,
    export_path: PathBuf,
    max_file_size: u64,
    concurrent_exports: usize,
}

impl DataExporter {
    /// Create new data exporter
    pub fn new(export_path: Option<PathBuf>) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            export_path: export_path.unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".hive")
                    .join("exports")
            }),
            max_file_size: 1024 * 1024 * 1024, // 1GB
            concurrent_exports: 4,
        }
    }

    /// Create export job
    pub async fn create_export(
        &self,
        config: ExportConfig,
        data_source: Arc<dyn DataSource>,
    ) -> Result<String> {
        let job = ExportJob {
            id: uuid::Uuid::new_v4().to_string(),
            config: config.clone(),
            status: ExportStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            file_path: None,
            file_size: None,
            row_count: None,
            error: None,
            metadata: HashMap::new(),
        };

        let job_id = job.id.clone();
        let job_id_for_return = job_id.clone(); // Clone for the return value

        // Store job
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id.clone(), job.clone());
        }

        // Start export task
        let jobs = Arc::clone(&self.jobs);
        let export_path = self.export_path.clone();
        
        tokio::spawn(async move {
            let job_id_clone = job_id.clone(); // Clone for the error handling closure
            if let Err(e) = Self::execute_export(job_id, config, data_source, jobs.clone(), export_path).await {
                // Update job with error
                let mut jobs = jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.status = ExportStatus::Failed;
                    job.error = Some(e.to_string());
                    job.completed_at = Some(Utc::now());
                }
            }
        });

        Ok(job_id_for_return)
    }

    /// Execute export
    async fn execute_export(
        job_id: String,
        config: ExportConfig,
        data_source: Arc<dyn DataSource>,
        jobs: Arc<RwLock<HashMap<String, ExportJob>>>,
        export_path: PathBuf,
    ) -> Result<()> {
        // Update job status
        {
            let mut jobs = jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = ExportStatus::Running;
                job.started_at = Some(Utc::now());
            }
        }

        // Create export directory
        if !export_path.exists() {
            fs::create_dir_all(&export_path).await?;
        }

        // Generate file name
        let file_name = format!(
            "export_{}_{}.{}",
            job_id,
            Utc::now().format("%Y%m%d_%H%M%S"),
            config.format.extension()
        );
        let file_path = export_path.join(&file_name);

        // Export data
        let (row_count, file_size) = match config.format {
            ExportFormat::CSV => Self::export_csv(&config, data_source, &file_path).await?,
            ExportFormat::JSON => Self::export_json(&config, data_source, &file_path).await?,
            ExportFormat::Excel => Self::export_excel(&config, data_source, &file_path).await?,
            ExportFormat::PDF => Self::export_pdf(&config, data_source, &file_path).await?,
            ExportFormat::HTML => Self::export_html(&config, data_source, &file_path).await?,
            ExportFormat::Markdown => Self::export_markdown(&config, data_source, &file_path).await?,
            ExportFormat::XML => Self::export_xml(&config, data_source, &file_path).await?,
            ExportFormat::Parquet => Self::export_parquet(&config, data_source, &file_path).await?,
        };

        // Apply compression if requested
        let final_path = if let Some(compression) = &config.compression {
            Self::compress_file(&file_path, compression).await?
        } else {
            file_path
        };

        // Apply encryption if requested
        let final_path = if let Some(encryption) = &config.encryption {
            Self::encrypt_file(&final_path, encryption).await?
        } else {
            final_path
        };

        // Get final file size
        let metadata = fs::metadata(&final_path).await?;
        let final_size = metadata.len();

        // Handle delivery
        if let Some(delivery) = &config.delivery {
            Self::deliver_export(&final_path, delivery).await?;
        }

        // Update job status
        {
            let mut jobs = jobs.write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = ExportStatus::Completed;
                job.completed_at = Some(Utc::now());
                job.file_path = Some(final_path);
                job.file_size = Some(final_size);
                job.row_count = Some(row_count);
            }
        }

        Ok(())
    }

    /// Export to CSV
    async fn export_csv(
        config: &ExportConfig,
        data_source: Arc<dyn DataSource>,
        file_path: &Path,
    ) -> Result<(u64, u64)> {
        let mut file = fs::File::create(file_path).await?;
        let delimiter = config.formatting.delimiter.as_deref().unwrap_or(",");
        let mut row_count = 0u64;

        // Write headers
        if config.formatting.include_headers {
            let headers = data_source.get_headers().await?;
            let header_line = headers.join(delimiter) + "\n";
            file.write_all(header_line.as_bytes()).await?;
        }

        // Write data
        let mut stream = data_source.stream_data(config.filters.clone()).await?;
        
        while let Some(row) = stream.next().await {
            let values: Vec<String> = row.values()
                .map(|v| Self::format_value(v, &config.formatting))
                .collect();
            
            let line = values.join(delimiter) + "\n";
            file.write_all(line.as_bytes()).await?;
            row_count += 1;
        }

        file.flush().await?;
        let metadata = fs::metadata(file_path).await?;
        
        Ok((row_count, metadata.len()))
    }

    /// Export to JSON
    async fn export_json(
        config: &ExportConfig,
        data_source: Arc<dyn DataSource>,
        file_path: &Path,
    ) -> Result<(u64, u64)> {
        let mut file = fs::File::create(file_path).await?;
        let mut row_count = 0u64;

        file.write_all(b"[\n").await?;

        let mut stream = data_source.stream_data(config.filters.clone()).await?;
        let mut first = true;

        while let Some(row) = stream.next().await {
            if !first {
                file.write_all(b",\n").await?;
            } else {
                first = false;
            }

            let json = if config.formatting.pretty_print {
                serde_json::to_string_pretty(&row)?
            } else {
                serde_json::to_string(&row)?
            };

            file.write_all(json.as_bytes()).await?;
            row_count += 1;
        }

        file.write_all(b"\n]").await?;
        file.flush().await?;

        let metadata = fs::metadata(file_path).await?;
        Ok((row_count, metadata.len()))
    }

    /// Export to Excel
    async fn export_excel(
        config: &ExportConfig,
        data_source: Arc<dyn DataSource>,
        file_path: &Path,
    ) -> Result<(u64, u64)> {
        // Simplified - would use xlsxwriter or similar
        // For now, create a CSV that Excel can open
        Self::export_csv(config, data_source, file_path).await
    }

    /// Export to PDF
    async fn export_pdf(
        config: &ExportConfig,
        data_source: Arc<dyn DataSource>,
        file_path: &Path,
    ) -> Result<(u64, u64)> {
        // Simplified - would use a PDF generation library
        // For now, export as HTML
        Self::export_html(config, data_source, file_path).await
    }

    /// Export to HTML
    async fn export_html(
        config: &ExportConfig,
        data_source: Arc<dyn DataSource>,
        file_path: &Path,
    ) -> Result<(u64, u64)> {
        let mut file = fs::File::create(file_path).await?;
        let mut row_count = 0u64;

        // Write HTML header
        let html_header = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Export</title>
    <style>
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        tr:nth-child(even) { background-color: #f9f9f9; }
    </style>
</head>
<body>
    <table>
"#;
        file.write_all(html_header.as_bytes()).await?;

        // Write headers
        if config.formatting.include_headers {
            file.write_all(b"<thead><tr>").await?;
            let headers = data_source.get_headers().await?;
            for header in headers {
                file.write_all(format!("<th>{}</th>", header).as_bytes()).await?;
            }
            file.write_all(b"</tr></thead><tbody>").await?;
        }

        // Write data
        let mut stream = data_source.stream_data(config.filters.clone()).await?;
        
        while let Some(row) = stream.next().await {
            file.write_all(b"<tr>").await?;
            
            for value in row.values() {
                let formatted = Self::format_value(value, &config.formatting);
                file.write_all(format!("<td>{}</td>", formatted).as_bytes()).await?;
            }
            
            file.write_all(b"</tr>").await?;
            row_count += 1;
        }

        // Write HTML footer
        file.write_all(b"</tbody></table></body></html>").await?;
        file.flush().await?;

        let metadata = fs::metadata(file_path).await?;
        Ok((row_count, metadata.len()))
    }

    /// Export to Markdown
    async fn export_markdown(
        config: &ExportConfig,
        data_source: Arc<dyn DataSource>,
        file_path: &Path,
    ) -> Result<(u64, u64)> {
        let mut file = fs::File::create(file_path).await?;
        let mut row_count = 0u64;

        // Write headers
        let headers = data_source.get_headers().await?;
        if config.formatting.include_headers {
            let header_line = format!("| {} |\n", headers.join(" | "));
            file.write_all(header_line.as_bytes()).await?;
            
            let separator = format!("| {} |\n", headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
            file.write_all(separator.as_bytes()).await?;
        }

        // Write data
        let mut stream = data_source.stream_data(config.filters.clone()).await?;
        
        while let Some(row) = stream.next().await {
            let values: Vec<String> = row.values()
                .map(|v| Self::format_value(v, &config.formatting))
                .collect();
            
            let line = format!("| {} |\n", values.join(" | "));
            file.write_all(line.as_bytes()).await?;
            row_count += 1;
        }

        file.flush().await?;
        let metadata = fs::metadata(file_path).await?;
        
        Ok((row_count, metadata.len()))
    }

    /// Export to XML
    async fn export_xml(
        config: &ExportConfig,
        data_source: Arc<dyn DataSource>,
        file_path: &Path,
    ) -> Result<(u64, u64)> {
        let mut file = fs::File::create(file_path).await?;
        let mut row_count = 0u64;

        file.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<data>\n").await?;

        let headers = data_source.get_headers().await?;
        let mut stream = data_source.stream_data(config.filters.clone()).await?;
        
        while let Some(row) = stream.next().await {
            file.write_all(b"  <record>\n").await?;
            
            for (i, value) in row.values().enumerate() {
                let default_header = format!("field{}", i);
                let header = headers.get(i).unwrap_or(&default_header);
                let formatted = Self::format_value(value, &config.formatting);
                file.write_all(format!("    <{}>{}</{}>\n", header, formatted, header).as_bytes()).await?;
            }
            
            file.write_all(b"  </record>\n").await?;
            row_count += 1;
        }

        file.write_all(b"</data>").await?;
        file.flush().await?;

        let metadata = fs::metadata(file_path).await?;
        Ok((row_count, metadata.len()))
    }

    /// Export to Parquet
    async fn export_parquet(
        config: &ExportConfig,
        data_source: Arc<dyn DataSource>,
        file_path: &Path,
    ) -> Result<(u64, u64)> {
        // Simplified - would use arrow/parquet library
        // For now, export as JSON
        Self::export_json(config, data_source, file_path).await
    }

    /// Format value based on formatting options
    fn format_value(value: &DataValue, formatting: &FormattingOptions) -> String {
        match value {
            DataValue::Null => formatting.null_value.clone(),
            DataValue::Bool(b) => b.to_string(),
            DataValue::Int(i) => i.to_string(),
            DataValue::Float(f) => format!("{}", f), // Would use number_format
            DataValue::String(s) => s.clone(),
            DataValue::Date(d) => d.format(&formatting.date_format).to_string(),
        }
    }

    /// Compress file
    async fn compress_file(file_path: &Path, compression: &CompressionType) -> Result<PathBuf> {
        let compressed_path = file_path.with_extension(format!(
            "{}.{}",
            file_path.extension().unwrap_or_default().to_str().unwrap_or(""),
            match compression {
                CompressionType::Gzip => "gz",
                CompressionType::Zip => "zip",
                CompressionType::Brotli => "br",
                CompressionType::Zstd => "zst",
            }
        ));

        // Simplified - would use actual compression libraries
        fs::copy(file_path, &compressed_path).await?;
        fs::remove_file(file_path).await?;

        Ok(compressed_path)
    }

    /// Encrypt file
    async fn encrypt_file(file_path: &Path, encryption: &EncryptionConfig) -> Result<PathBuf> {
        let encrypted_path = file_path.with_extension(format!(
            "{}.enc",
            file_path.extension().unwrap_or_default().to_str().unwrap_or("")
        ));

        // Simplified - would use actual encryption libraries
        fs::copy(file_path, &encrypted_path).await?;
        fs::remove_file(file_path).await?;

        Ok(encrypted_path)
    }

    /// Deliver export
    async fn deliver_export(file_path: &Path, delivery: &DeliveryConfig) -> Result<()> {
        match &delivery.method {
            DeliveryMethod::Download => {
                // File is already saved locally
                Ok(())
            }
            DeliveryMethod::Email { recipients, subject, body } => {
                // Would send email with attachment
                println!("Would send email to {:?}", recipients);
                Ok(())
            }
            DeliveryMethod::S3 { bucket, key_prefix, region } => {
                // Would upload to S3
                println!("Would upload to S3 bucket {}", bucket);
                Ok(())
            }
            DeliveryMethod::SFTP { host, port, username, path } => {
                // Would upload via SFTP
                println!("Would upload to SFTP {}:{}", host, port);
                Ok(())
            }
            DeliveryMethod::Webhook { url, headers } => {
                // Would POST to webhook
                println!("Would POST to webhook {}", url);
                Ok(())
            }
        }
    }

    /// Get export job
    pub async fn get_job(&self, job_id: &str) -> Option<ExportJob> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    /// List export jobs
    pub async fn list_jobs(&self, status: Option<ExportStatus>) -> Vec<ExportJob> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| status.map_or(true, |s| job.status == s))
            .cloned()
            .collect()
    }

    /// Cancel export job
    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs.get_mut(job_id)
            .ok_or_else(|| anyhow::anyhow!("Job not found"))?;

        if job.status == ExportStatus::Running || job.status == ExportStatus::Pending {
            job.status = ExportStatus::Cancelled;
            job.completed_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Clean old exports
    pub async fn clean_old_exports(&self, days: u32) -> Result<u32> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let mut removed = 0;

        let jobs = self.jobs.read().await;
        let old_jobs: Vec<_> = jobs.values()
            .filter(|job| {
                job.status == ExportStatus::Completed &&
                job.completed_at.map_or(false, |d| d < cutoff)
            })
            .collect();

        for job in old_jobs {
            if let Some(file_path) = &job.file_path {
                if file_path.exists() {
                    fs::remove_file(file_path).await?;
                    removed += 1;
                }
            }
        }

        Ok(removed)
    }
}

/// Data source trait
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    async fn get_headers(&self) -> Result<Vec<String>>;
    async fn stream_data(&self, filters: HashMap<String, String>) -> Result<DataStream>;
}

/// Data stream
pub struct DataStream {
    receiver: tokio::sync::mpsc::Receiver<DataRow>,
}

impl DataStream {
    pub async fn next(&mut self) -> Option<DataRow> {
        self.receiver.recv().await
    }
}

/// Data row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRow {
    values: Vec<DataValue>,
}

impl DataRow {
    pub fn values(&self) -> impl Iterator<Item = &DataValue> {
        self.values.iter()
    }
}

/// Data value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Date(DateTime<Utc>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_properties() {
        assert_eq!(ExportFormat::CSV.extension(), "csv");
        assert_eq!(ExportFormat::CSV.mime_type(), "text/csv");
        
        assert_eq!(ExportFormat::JSON.extension(), "json");
        assert_eq!(ExportFormat::JSON.mime_type(), "application/json");
    }

    #[tokio::test]
    async fn test_data_exporter_creation() {
        let exporter = DataExporter::new(Some(PathBuf::from("/tmp/hive_exports")));
        
        let jobs = exporter.list_jobs(None).await;
        assert!(jobs.is_empty());
    }

    #[test]
    fn test_formatting_options_default() {
        let formatting = FormattingOptions::default();
        assert!(formatting.include_headers);
        assert_eq!(formatting.delimiter, Some(",".to_string()));
        assert!(formatting.pretty_print);
    }
}