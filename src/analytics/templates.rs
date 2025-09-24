//! Enterprise Reporting Templates
//!
//! Professional report templates for various business needs:
//! - Executive summaries with strategic insights
//! - Financial reports with cost breakdowns
//! - Performance reports with KPIs
//! - Compliance reports for auditing
//! - Custom templates with branding

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Report template types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateType {
    ExecutiveSummary,
    FinancialReport,
    PerformanceReport,
    ComplianceAudit,
    TeamProductivity,
    ProjectStatus,
    CustomTemplate,
}

/// Report format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Markdown,
    HTML,
    PDF,
    Excel,
    PowerPoint,
    JSON,
    CSV,
}

/// Report template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub id: String,
    pub name: String,
    pub template_type: TemplateType,
    pub description: String,
    pub sections: Vec<TemplateSection>,
    pub styling: ReportStyling,
    pub metadata: TemplateMetadata,
    pub tags: Vec<String>,
}

/// Template section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSection {
    pub id: String,
    pub title: String,
    pub content_type: SectionContentType,
    pub data_sources: Vec<String>,
    pub visualizations: Vec<VisualizationType>,
    pub conditional: Option<ConditionalRule>,
}

/// Section content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SectionContentType {
    Summary { max_length: usize },
    Table { columns: Vec<TableColumn> },
    Chart { chart_type: ChartType },
    Metrics { layout: MetricsLayout },
    Timeline { period: String },
    Comparison { baseline: String },
    Custom { template: String },
}

/// Table column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub id: String,
    pub header: String,
    pub data_type: DataType,
    pub format: Option<String>,
    pub aggregation: Option<AggregationType>,
}

/// Data types for columns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    String,
    Number,
    Currency,
    Percentage,
    Date,
    Boolean,
}

/// Aggregation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Count,
    Min,
    Max,
    Median,
}

/// Chart types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    LineChart,
    BarChart,
    PieChart,
    AreaChart,
    ScatterPlot,
    Heatmap,
    Gauge,
}

/// Visualization types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualizationType {
    Chart(ChartType),
    Table,
    KPICard,
    Timeline,
    Map,
    WordCloud,
}

/// Metrics layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricsLayout {
    Grid,
    List,
    Cards,
    Dashboard,
}

/// Conditional rules for sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRule {
    pub condition: String,
    pub show_when: bool,
}

/// Report styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStyling {
    pub theme: String,
    pub logo_url: Option<String>,
    pub color_scheme: ColorScheme,
    pub fonts: FontSettings,
    pub layout: LayoutSettings,
}

/// Color scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
}

/// Font settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    pub heading: String,
    pub body: String,
    pub monospace: String,
}

/// Layout settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub page_size: String,
    pub margins: Margins,
    pub header_footer: bool,
}

/// Page margins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub version: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub department: Option<String>,
}

/// Report generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    pub template_id: String,
    pub title: String,
    pub period: ReportPeriod,
    pub filters: HashMap<String, String>,
    pub format: ReportFormat,
    pub recipients: Vec<String>,
    pub schedule: Option<ReportSchedule>,
}

/// Report period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub comparison: Option<ComparisonPeriod>,
}

/// Comparison period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonPeriod {
    pub period_type: PeriodType,
    pub offset: i32,
}

/// Period types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeriodType {
    Day,
    Week,
    Month,
    Quarter,
    Year,
    Custom,
}

/// Report schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSchedule {
    pub frequency: ScheduleFrequency,
    pub time: String,
    pub timezone: String,
    pub next_run: DateTime<Utc>,
}

/// Schedule frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

/// Template manager
pub struct TemplateManager {
    templates: HashMap<String, ReportTemplate>,
    custom_templates_path: PathBuf,
}

impl TemplateManager {
    /// Create new template manager
    pub fn new(custom_path: Option<PathBuf>) -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            custom_templates_path: custom_path.unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".hive")
                    .join("templates")
            }),
        };

        // Load built-in templates
        manager.load_builtin_templates();

        manager
    }

    /// Load built-in templates
    fn load_builtin_templates(&mut self) {
        // Executive Summary Template
        self.templates.insert(
            "executive_summary".to_string(),
            ReportTemplate {
                id: "executive_summary".to_string(),
                name: "Executive Summary Report".to_string(),
                template_type: TemplateType::ExecutiveSummary,
                description: "High-level overview for executive stakeholders".to_string(),
                sections: vec![
                    TemplateSection {
                        id: "key_metrics".to_string(),
                        title: "Key Performance Indicators".to_string(),
                        content_type: SectionContentType::Metrics {
                            layout: MetricsLayout::Cards,
                        },
                        data_sources: vec!["analytics.kpi".to_string()],
                        visualizations: vec![VisualizationType::KPICard],
                        conditional: None,
                    },
                    TemplateSection {
                        id: "trend_analysis".to_string(),
                        title: "Trend Analysis".to_string(),
                        content_type: SectionContentType::Chart {
                            chart_type: ChartType::LineChart,
                        },
                        data_sources: vec!["analytics.trends".to_string()],
                        visualizations: vec![VisualizationType::Chart(ChartType::LineChart)],
                        conditional: None,
                    },
                    TemplateSection {
                        id: "cost_summary".to_string(),
                        title: "Cost Overview".to_string(),
                        content_type: SectionContentType::Table {
                            columns: vec![
                                TableColumn {
                                    id: "category".to_string(),
                                    header: "Category".to_string(),
                                    data_type: DataType::String,
                                    format: None,
                                    aggregation: None,
                                },
                                TableColumn {
                                    id: "current".to_string(),
                                    header: "Current Period".to_string(),
                                    data_type: DataType::Currency,
                                    format: Some("$#,##0.00".to_string()),
                                    aggregation: Some(AggregationType::Sum),
                                },
                                TableColumn {
                                    id: "previous".to_string(),
                                    header: "Previous Period".to_string(),
                                    data_type: DataType::Currency,
                                    format: Some("$#,##0.00".to_string()),
                                    aggregation: Some(AggregationType::Sum),
                                },
                                TableColumn {
                                    id: "change".to_string(),
                                    header: "Change".to_string(),
                                    data_type: DataType::Percentage,
                                    format: Some("#0.0%".to_string()),
                                    aggregation: None,
                                },
                            ],
                        },
                        data_sources: vec!["analytics.costs".to_string()],
                        visualizations: vec![VisualizationType::Table],
                        conditional: None,
                    },
                    TemplateSection {
                        id: "recommendations".to_string(),
                        title: "Strategic Recommendations".to_string(),
                        content_type: SectionContentType::Summary { max_length: 500 },
                        data_sources: vec!["analytics.insights".to_string()],
                        visualizations: vec![],
                        conditional: None,
                    },
                ],
                styling: ReportStyling {
                    theme: "professional".to_string(),
                    logo_url: None,
                    color_scheme: ColorScheme {
                        primary: "#1a73e8".to_string(),
                        secondary: "#5f6368".to_string(),
                        accent: "#34a853".to_string(),
                        background: "#ffffff".to_string(),
                        text: "#202124".to_string(),
                    },
                    fonts: FontSettings {
                        heading: "Roboto".to_string(),
                        body: "Open Sans".to_string(),
                        monospace: "Roboto Mono".to_string(),
                    },
                    layout: LayoutSettings {
                        page_size: "A4".to_string(),
                        margins: Margins {
                            top: 2.5,
                            bottom: 2.5,
                            left: 2.5,
                            right: 2.5,
                        },
                        header_footer: true,
                    },
                },
                metadata: TemplateMetadata {
                    version: "1.0.0".to_string(),
                    author: "Hive AI".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    tags: vec!["executive".to_string(), "summary".to_string()],
                    department: None,
                },
                tags: vec![
                    "executive".to_string(),
                    "summary".to_string(),
                    "overview".to_string(),
                ],
            },
        );

        // Financial Report Template
        self.templates.insert(
            "financial_report".to_string(),
            ReportTemplate {
                id: "financial_report".to_string(),
                name: "Financial Analysis Report".to_string(),
                template_type: TemplateType::FinancialReport,
                description: "Detailed financial analysis with cost breakdowns".to_string(),
                sections: vec![
                    TemplateSection {
                        id: "budget_overview".to_string(),
                        title: "Budget Overview".to_string(),
                        content_type: SectionContentType::Chart {
                            chart_type: ChartType::Gauge,
                        },
                        data_sources: vec!["finance.budget".to_string()],
                        visualizations: vec![VisualizationType::Chart(ChartType::Gauge)],
                        conditional: None,
                    },
                    TemplateSection {
                        id: "spend_by_category".to_string(),
                        title: "Spending by Category".to_string(),
                        content_type: SectionContentType::Chart {
                            chart_type: ChartType::PieChart,
                        },
                        data_sources: vec!["finance.categories".to_string()],
                        visualizations: vec![VisualizationType::Chart(ChartType::PieChart)],
                        conditional: None,
                    },
                    TemplateSection {
                        id: "cost_trend".to_string(),
                        title: "Cost Trend Analysis".to_string(),
                        content_type: SectionContentType::Chart {
                            chart_type: ChartType::AreaChart,
                        },
                        data_sources: vec!["finance.trends".to_string()],
                        visualizations: vec![VisualizationType::Chart(ChartType::AreaChart)],
                        conditional: None,
                    },
                    TemplateSection {
                        id: "forecast".to_string(),
                        title: "Financial Forecast".to_string(),
                        content_type: SectionContentType::Chart {
                            chart_type: ChartType::LineChart,
                        },
                        data_sources: vec!["finance.forecast".to_string()],
                        visualizations: vec![VisualizationType::Chart(ChartType::LineChart)],
                        conditional: None,
                    },
                ],
                styling: ReportStyling {
                    theme: "financial".to_string(),
                    logo_url: None,
                    color_scheme: ColorScheme {
                        primary: "#0f4c81".to_string(),
                        secondary: "#1e88e5".to_string(),
                        accent: "#4caf50".to_string(),
                        background: "#f5f5f5".to_string(),
                        text: "#212121".to_string(),
                    },
                    fonts: FontSettings {
                        heading: "Arial".to_string(),
                        body: "Helvetica".to_string(),
                        monospace: "Courier New".to_string(),
                    },
                    layout: LayoutSettings {
                        page_size: "Letter".to_string(),
                        margins: Margins {
                            top: 2.0,
                            bottom: 2.0,
                            left: 2.0,
                            right: 2.0,
                        },
                        header_footer: true,
                    },
                },
                metadata: TemplateMetadata {
                    version: "1.0.0".to_string(),
                    author: "Hive AI".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    tags: vec!["financial".to_string(), "budget".to_string()],
                    department: Some("Finance".to_string()),
                },
                tags: vec![
                    "financial".to_string(),
                    "budget".to_string(),
                    "analysis".to_string(),
                ],
            },
        );

        // Compliance Audit Template
        self.templates.insert(
            "compliance_audit".to_string(),
            ReportTemplate {
                id: "compliance_audit".to_string(),
                name: "Compliance Audit Report".to_string(),
                template_type: TemplateType::ComplianceAudit,
                description: "Audit-ready compliance report with detailed tracking".to_string(),
                sections: vec![
                    TemplateSection {
                        id: "compliance_summary".to_string(),
                        title: "Compliance Summary".to_string(),
                        content_type: SectionContentType::Metrics {
                            layout: MetricsLayout::Grid,
                        },
                        data_sources: vec!["compliance.summary".to_string()],
                        visualizations: vec![VisualizationType::KPICard],
                        conditional: None,
                    },
                    TemplateSection {
                        id: "audit_trail".to_string(),
                        title: "Audit Trail".to_string(),
                        content_type: SectionContentType::Table {
                            columns: vec![
                                TableColumn {
                                    id: "timestamp".to_string(),
                                    header: "Timestamp".to_string(),
                                    data_type: DataType::Date,
                                    format: Some("YYYY-MM-DD HH:mm:ss".to_string()),
                                    aggregation: None,
                                },
                                TableColumn {
                                    id: "user".to_string(),
                                    header: "User".to_string(),
                                    data_type: DataType::String,
                                    format: None,
                                    aggregation: None,
                                },
                                TableColumn {
                                    id: "action".to_string(),
                                    header: "Action".to_string(),
                                    data_type: DataType::String,
                                    format: None,
                                    aggregation: None,
                                },
                                TableColumn {
                                    id: "status".to_string(),
                                    header: "Status".to_string(),
                                    data_type: DataType::String,
                                    format: None,
                                    aggregation: None,
                                },
                            ],
                        },
                        data_sources: vec!["compliance.audit_log".to_string()],
                        visualizations: vec![VisualizationType::Table],
                        conditional: None,
                    },
                    TemplateSection {
                        id: "policy_adherence".to_string(),
                        title: "Policy Adherence".to_string(),
                        content_type: SectionContentType::Chart {
                            chart_type: ChartType::BarChart,
                        },
                        data_sources: vec!["compliance.policies".to_string()],
                        visualizations: vec![VisualizationType::Chart(ChartType::BarChart)],
                        conditional: None,
                    },
                ],
                styling: ReportStyling {
                    theme: "formal".to_string(),
                    logo_url: None,
                    color_scheme: ColorScheme {
                        primary: "#000000".to_string(),
                        secondary: "#424242".to_string(),
                        accent: "#1976d2".to_string(),
                        background: "#ffffff".to_string(),
                        text: "#000000".to_string(),
                    },
                    fonts: FontSettings {
                        heading: "Times New Roman".to_string(),
                        body: "Georgia".to_string(),
                        monospace: "Consolas".to_string(),
                    },
                    layout: LayoutSettings {
                        page_size: "Legal".to_string(),
                        margins: Margins {
                            top: 3.0,
                            bottom: 3.0,
                            left: 2.5,
                            right: 2.5,
                        },
                        header_footer: true,
                    },
                },
                metadata: TemplateMetadata {
                    version: "1.0.0".to_string(),
                    author: "Hive AI".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    tags: vec!["compliance".to_string(), "audit".to_string()],
                    department: Some("Legal".to_string()),
                },
                tags: vec![
                    "compliance".to_string(),
                    "audit".to_string(),
                    "legal".to_string(),
                ],
            },
        );
    }

    /// Load custom templates from disk
    pub async fn load_custom_templates(&mut self) -> Result<()> {
        if !self.custom_templates_path.exists() {
            fs::create_dir_all(&self.custom_templates_path).await?;
        }

        let mut entries = fs::read_dir(&self.custom_templates_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path).await?;
                let template: ReportTemplate =
                    serde_json::from_str(&content).context("Failed to parse template")?;

                self.templates.insert(template.id.clone(), template);
            }
        }

        Ok(())
    }

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&ReportTemplate> {
        self.templates.get(id)
    }

    /// List all templates
    pub fn list_templates(&self) -> Vec<&ReportTemplate> {
        self.templates.values().collect()
    }

    /// Create custom template
    pub async fn create_custom_template(&mut self, template: ReportTemplate) -> Result<()> {
        // Validate template
        self.validate_template(&template)?;

        // Save to disk
        let file_path = self
            .custom_templates_path
            .join(format!("{}.json", template.id));
        let content = serde_json::to_string_pretty(&template)?;
        fs::write(&file_path, content).await?;

        // Add to memory
        self.templates.insert(template.id.clone(), template);

        Ok(())
    }

    /// Update template
    pub async fn update_template(&mut self, id: &str, template: ReportTemplate) -> Result<()> {
        if !self.templates.contains_key(id) {
            return Err(anyhow::anyhow!("Template not found"));
        }

        // Validate template
        self.validate_template(&template)?;

        // Update on disk if custom
        let file_path = self.custom_templates_path.join(format!("{}.json", id));
        if file_path.exists() {
            let content = serde_json::to_string_pretty(&template)?;
            fs::write(&file_path, content).await?;
        }

        // Update in memory
        self.templates.insert(id.to_string(), template);

        Ok(())
    }

    /// Delete custom template
    pub async fn delete_template(&mut self, id: &str) -> Result<()> {
        // Check if it's a built-in template
        if matches!(
            id,
            "executive_summary" | "financial_report" | "compliance_audit"
        ) {
            return Err(anyhow::anyhow!("Cannot delete built-in template"));
        }

        // Remove from disk
        let file_path = self.custom_templates_path.join(format!("{}.json", id));
        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }

        // Remove from memory
        self.templates.remove(id);

        Ok(())
    }

    /// Validate template
    fn validate_template(&self, template: &ReportTemplate) -> Result<()> {
        // Check required fields
        if template.id.is_empty() {
            return Err(anyhow::anyhow!("Template ID is required"));
        }
        if template.name.is_empty() {
            return Err(anyhow::anyhow!("Template name is required"));
        }
        if template.sections.is_empty() {
            return Err(anyhow::anyhow!("Template must have at least one section"));
        }

        // Validate sections
        for section in &template.sections {
            if section.id.is_empty() {
                return Err(anyhow::anyhow!("Section ID is required"));
            }
            if section.title.is_empty() {
                return Err(anyhow::anyhow!("Section title is required"));
            }
        }

        Ok(())
    }

    /// Get templates by type
    pub fn get_templates_by_type(&self, template_type: TemplateType) -> Vec<&ReportTemplate> {
        self.templates
            .values()
            .filter(|t| t.template_type == template_type)
            .collect()
    }

    /// Search templates
    pub fn search_templates(&self, query: &str) -> Vec<&ReportTemplate> {
        let query_lower = query.to_lowercase();

        self.templates
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower)
                    || t.description.to_lowercase().contains(&query_lower)
                    || t.tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Export template marketplace format
    pub fn export_marketplace_format(&self, template_id: &str) -> Result<String> {
        let template = self
            .templates
            .get(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template not found"))?;

        let marketplace_format = serde_json::json!({
            "template": template,
            "preview": self.generate_preview(template),
            "compatibility": {
                "min_version": "2.0.0",
                "max_version": "3.0.0",
            },
            "pricing": {
                "model": "free",
                "price": 0.0,
            },
            "stats": {
                "downloads": 0,
                "rating": 0.0,
                "reviews": 0,
            },
        });

        Ok(serde_json::to_string_pretty(&marketplace_format)?)
    }

    /// Generate template preview
    fn generate_preview(&self, template: &ReportTemplate) -> String {
        format!(
            "# {}\n\n{}\n\n## Sections:\n{}",
            template.name,
            template.description,
            template
                .sections
                .iter()
                .map(|s| format!("- {}", s.title))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// Report builder for generating reports from templates
pub struct ReportBuilder {
    template_manager: TemplateManager,
}

impl ReportBuilder {
    /// Create new report builder
    pub fn new(template_manager: TemplateManager) -> Self {
        Self { template_manager }
    }

    /// Build report from request
    pub async fn build_report(&self, request: &ReportRequest) -> Result<Report> {
        let template = self
            .template_manager
            .get_template(&request.template_id)
            .ok_or_else(|| anyhow::anyhow!("Template not found"))?;

        let mut report = Report {
            id: uuid::Uuid::new_v4().to_string(),
            title: request.title.clone(),
            template_id: template.id.clone(),
            generated_at: Utc::now(),
            period: request.period.clone(),
            sections: Vec::new(),
            metadata: ReportMetadata {
                template_version: template.metadata.version.clone(),
                generator_version: "2.0.0".to_string(),
                filters: request.filters.clone(),
                data_sources: self.collect_data_sources(template),
            },
        };

        // Build each section
        for section_template in &template.sections {
            if self.should_include_section(section_template, &request.filters) {
                let section = self
                    .build_section(section_template, &request.period)
                    .await?;
                report.sections.push(section);
            }
        }

        Ok(report)
    }

    /// Check if section should be included
    fn should_include_section(
        &self,
        section: &TemplateSection,
        filters: &HashMap<String, String>,
    ) -> bool {
        if let Some(conditional) = &section.conditional {
            // Evaluate conditional rule
            // Simplified - in production would use proper expression evaluation
            conditional.show_when
        } else {
            true
        }
    }

    /// Build individual section
    async fn build_section(
        &self,
        template: &TemplateSection,
        period: &ReportPeriod,
    ) -> Result<ReportSection> {
        Ok(ReportSection {
            id: template.id.clone(),
            title: template.title.clone(),
            content: self.generate_section_content(template, period).await?,
            visualizations: template.visualizations.clone(),
        })
    }

    /// Generate section content
    async fn generate_section_content(
        &self,
        template: &TemplateSection,
        period: &ReportPeriod,
    ) -> Result<SectionContent> {
        match &template.content_type {
            SectionContentType::Summary { max_length } => Ok(SectionContent::Text {
                content:
                    "Executive summary content would be generated here based on data analysis."
                        .to_string(),
            }),
            SectionContentType::Table { columns } => Ok(SectionContent::Table {
                headers: columns.iter().map(|c| c.header.clone()).collect(),
                rows: vec![
                    vec![
                        "Model Usage".to_string(),
                        "$1,234.56".to_string(),
                        "$987.65".to_string(),
                        "+25.0%".to_string(),
                    ],
                    vec![
                        "API Calls".to_string(),
                        "$543.21".to_string(),
                        "$456.78".to_string(),
                        "+18.9%".to_string(),
                    ],
                ],
            }),
            SectionContentType::Chart { chart_type } => Ok(SectionContent::Chart {
                chart_type: *chart_type,
                data: serde_json::json!({
                    "labels": ["Jan", "Feb", "Mar", "Apr", "May"],
                    "datasets": [{
                        "label": "Cost",
                        "data": [1200, 1400, 1100, 1600, 1800],
                    }],
                }),
            }),
            SectionContentType::Metrics { layout } => Ok(SectionContent::Metrics {
                metrics: vec![
                    Metric {
                        label: "Total Cost".to_string(),
                        value: "$5,432.10".to_string(),
                        change: Some("+15.2%".to_string()),
                        trend: Some("up".to_string()),
                    },
                    Metric {
                        label: "API Calls".to_string(),
                        value: "1.2M".to_string(),
                        change: Some("+8.5%".to_string()),
                        trend: Some("up".to_string()),
                    },
                ],
                layout: *layout,
            }),
            _ => Ok(SectionContent::Text {
                content: "Content generation not implemented for this type".to_string(),
            }),
        }
    }

    /// Collect all data sources from template
    fn collect_data_sources(&self, template: &ReportTemplate) -> Vec<String> {
        template
            .sections
            .iter()
            .flat_map(|s| s.data_sources.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

/// Generated report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub title: String,
    pub template_id: String,
    pub generated_at: DateTime<Utc>,
    pub period: ReportPeriod,
    pub sections: Vec<ReportSection>,
    pub metadata: ReportMetadata,
}

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub id: String,
    pub title: String,
    pub content: SectionContent,
    pub visualizations: Vec<VisualizationType>,
}

/// Section content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SectionContent {
    Text {
        content: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Chart {
        chart_type: ChartType,
        data: serde_json::Value,
    },
    Metrics {
        metrics: Vec<Metric>,
        layout: MetricsLayout,
    },
}

/// Metric data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub label: String,
    pub value: String,
    pub change: Option<String>,
    pub trend: Option<String>,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub template_version: String,
    pub generator_version: String,
    pub filters: HashMap<String, String>,
    pub data_sources: Vec<String>,
}

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_template_manager_creation() {
        let manager = TemplateManager::new(None);

        // Check built-in templates are loaded
        assert!(manager.get_template("executive_summary").is_some());
        assert!(manager.get_template("financial_report").is_some());
        assert!(manager.get_template("compliance_audit").is_some());
    }

    #[tokio::test]
    async fn test_custom_template_creation() -> Result<()> {
        let mut manager = TemplateManager::new(Some(PathBuf::from("/tmp/hive_templates")));

        let template = ReportTemplate {
            id: "test_template".to_string(),
            name: "Test Template".to_string(),
            template_type: TemplateType::CustomTemplate,
            description: "Test template".to_string(),
            sections: vec![TemplateSection {
                id: "test_section".to_string(),
                title: "Test Section".to_string(),
                content_type: SectionContentType::Summary { max_length: 100 },
                data_sources: vec![],
                visualizations: vec![],
                conditional: None,
            }],
            styling: ReportStyling {
                theme: "default".to_string(),
                logo_url: None,
                color_scheme: ColorScheme {
                    primary: "#000000".to_string(),
                    secondary: "#ffffff".to_string(),
                    accent: "#0000ff".to_string(),
                    background: "#ffffff".to_string(),
                    text: "#000000".to_string(),
                },
                fonts: FontSettings {
                    heading: "Arial".to_string(),
                    body: "Arial".to_string(),
                    monospace: "Courier".to_string(),
                },
                layout: LayoutSettings {
                    page_size: "A4".to_string(),
                    margins: Margins {
                        top: 2.5,
                        bottom: 2.5,
                        left: 2.5,
                        right: 2.5,
                    },
                    header_footer: true,
                },
            },
            metadata: TemplateMetadata {
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                tags: vec!["test".to_string()],
                department: None,
            },
            tags: vec!["test".to_string()],
        };

        manager.create_custom_template(template).await?;
        assert!(manager.get_template("test_template").is_some());

        Ok(())
    }

    #[test]
    fn test_template_search() {
        let manager = TemplateManager::new(None);

        let results = manager.search_templates("executive");
        assert!(!results.is_empty());

        let results = manager.search_templates("financial");
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_report_building() -> Result<()> {
        let manager = TemplateManager::new(None);
        let builder = ReportBuilder::new(manager);

        let request = ReportRequest {
            template_id: "executive_summary".to_string(),
            title: "Q1 2024 Executive Summary".to_string(),
            period: ReportPeriod {
                start: Utc::now() - chrono::Duration::days(90),
                end: Utc::now(),
                comparison: None,
            },
            filters: HashMap::new(),
            format: ReportFormat::Markdown,
            recipients: vec![],
            schedule: None,
        };

        let report = builder.build_report(&request).await?;

        assert_eq!(report.title, "Q1 2024 Executive Summary");
        assert!(!report.sections.is_empty());

        Ok(())
    }
}
