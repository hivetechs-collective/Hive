//! Custom Dashboard Builder
//!
//! Enables users to create personalized analytics dashboards:
//! - Drag-and-drop widget placement
//! - Customizable visualizations
//! - Real-time data binding
//! - Template marketplace integration
//! - Export and sharing capabilities

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Dashboard definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub layout: DashboardLayout,
    pub widgets: Vec<Widget>,
    pub theme: DashboardTheme,
    pub data_sources: Vec<DataSource>,
    pub refresh_interval: u64, // seconds
    pub filters: Vec<DashboardFilter>,
    pub permissions: DashboardPermissions,
    pub metadata: DashboardMetadata,
}

/// Dashboard layout types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardLayout {
    Grid,
    Flex,
    Fixed,
    Responsive,
}

/// Widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub type_: WidgetType,
    pub title: String,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub data_binding: DataBinding,
    pub visualization: VisualizationConfig,
    pub interactions: Vec<WidgetInteraction>,
    pub refresh_override: Option<u64>,
}

/// Widget types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WidgetType {
    Chart {
        chart_type: ChartType,
        options: ChartOptions,
    },
    Metric {
        format: MetricFormat,
        comparison: Option<ComparisonConfig>,
    },
    Table {
        columns: Vec<TableColumn>,
        pagination: bool,
        sorting: bool,
    },
    Text {
        markdown: bool,
        template: Option<String>,
    },
    Map {
        map_type: MapType,
        layers: Vec<MapLayer>,
    },
    Timeline {
        event_source: String,
        grouping: Option<String>,
    },
    Custom {
        component: String,
        props: HashMap<String, serde_json::Value>,
    },
}

/// Chart types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Donut,
    Area,
    Scatter,
    Bubble,
    Radar,
    Heatmap,
    Treemap,
    Sankey,
    Gauge,
    Funnel,
}

/// Chart options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    pub stacked: bool,
    pub show_legend: bool,
    pub show_grid: bool,
    pub animations: bool,
    pub tooltips: bool,
    pub zoom: bool,
    pub export: bool,
}

/// Metric format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricFormat {
    pub value_format: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub decimal_places: u8,
    pub abbreviate: bool,
}

/// Comparison configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    pub comparison_type: ComparisonType,
    pub show_change: bool,
    pub show_trend: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonType {
    PreviousPeriod,
    YearOverYear,
    Target,
    Average,
}

/// Table column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub id: String,
    pub label: String,
    pub field: String,
    pub type_: ColumnType,
    pub format: Option<String>,
    pub width: Option<u32>,
    pub sortable: bool,
    pub filterable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    Text,
    Number,
    Date,
    Boolean,
    Link,
    Badge,
    Progress,
}

/// Map types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapType {
    World,
    Country,
    Region,
    Custom,
}

/// Map layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLayer {
    pub id: String,
    pub type_: LayerType,
    pub data_source: String,
    pub style: LayerStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    Points,
    Heatmap,
    Choropleth,
    Lines,
    Polygons,
}

/// Layer style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStyle {
    pub color_scheme: String,
    pub opacity: f32,
    pub stroke_width: f32,
}

/// Widget position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub z: u32, // layer order
}

/// Widget size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

/// Data binding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBinding {
    pub source: String,
    pub query: Option<String>,
    pub transform: Option<DataTransform>,
    pub cache: bool,
    pub real_time: bool,
}

/// Data transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransform {
    pub type_: TransformType,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformType {
    Aggregate,
    Filter,
    Sort,
    Pivot,
    Join,
    Calculate,
    Custom,
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub color_palette: String,
    pub font_family: String,
    pub font_size: u8,
    pub padding: Padding,
    pub background: Option<String>,
    pub border: Option<BorderConfig>,
}

/// Padding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Padding {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

/// Border configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderConfig {
    pub width: u32,
    pub color: String,
    pub radius: u32,
}

/// Widget interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WidgetInteraction {
    Click {
        action: InteractionAction,
    },
    Hover {
        show_tooltip: bool,
        highlight: bool,
    },
    Drill {
        target_dashboard: Option<String>,
        target_widget: Option<String>,
    },
    Export {
        formats: Vec<ExportFormat>,
    },
}

/// Interaction action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InteractionAction {
    NavigateTo { url: String },
    UpdateFilter { filter_id: String },
    ExecuteQuery { query: String },
    ShowDetails { template: String },
}

/// Export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    PNG,
    SVG,
    PDF,
    CSV,
    Excel,
    JSON,
}

/// Dashboard theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTheme {
    pub name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: String,
    pub text_color: String,
    pub font_family: String,
    pub custom_css: Option<String>,
}

/// Data source definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: String,
    pub name: String,
    pub type_: DataSourceType,
    pub connection: ConnectionConfig,
    pub refresh_interval: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DataSourceType {
    Analytics,
    Database { driver: String },
    API { base_url: String },
    File { path: String },
    Stream { protocol: String },
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub parameters: HashMap<String, String>,
    pub auth: Option<AuthConfig>,
    pub timeout: u64,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthConfig {
    None,
    ApiKey { key: String },
    Basic { username: String, password: String },
    OAuth { token: String },
}

/// Dashboard filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardFilter {
    pub id: String,
    pub name: String,
    pub type_: FilterType,
    pub default_value: Option<serde_json::Value>,
    pub options: Option<Vec<FilterOption>>,
    pub connected_widgets: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterType {
    DateRange,
    Select,
    MultiSelect,
    Text,
    Number,
    Toggle,
}

/// Filter option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOption {
    pub value: serde_json::Value,
    pub label: String,
}

/// Dashboard permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPermissions {
    pub owner: String,
    pub viewers: Vec<String>,
    pub editors: Vec<String>,
    pub public: bool,
    pub share_link: Option<String>,
}

/// Dashboard metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub thumbnail: Option<String>,
}

/// Dashboard builder
pub struct DashboardBuilder {
    dashboards: Arc<RwLock<HashMap<String, Dashboard>>>,
    templates_path: PathBuf,
    marketplace_client: MarketplaceClient,
}

impl DashboardBuilder {
    /// Create new dashboard builder
    pub fn new(templates_path: Option<PathBuf>) -> Self {
        Self {
            dashboards: Arc::new(RwLock::new(HashMap::new())),
            templates_path: templates_path.unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".hive")
                    .join("dashboards")
            }),
            marketplace_client: MarketplaceClient::new(),
        }
    }

    /// Create new dashboard
    pub async fn create_dashboard(&self, config: DashboardConfig) -> Result<Dashboard> {
        let dashboard = Dashboard {
            id: uuid::Uuid::new_v4().to_string(),
            name: config.name,
            description: config.description,
            layout: config.layout,
            widgets: Vec::new(),
            theme: config.theme.unwrap_or_else(|| self.default_theme()),
            data_sources: Vec::new(),
            refresh_interval: config.refresh_interval.unwrap_or(30),
            filters: Vec::new(),
            permissions: DashboardPermissions {
                owner: config.owner,
                viewers: Vec::new(),
                editors: Vec::new(),
                public: false,
                share_link: None,
            },
            metadata: DashboardMetadata {
                created_at: Utc::now(),
                updated_at: Utc::now(),
                version: "1.0.0".to_string(),
                tags: config.tags,
                category: config.category,
                thumbnail: None,
            },
        };

        // Save dashboard
        self.save_dashboard(&dashboard).await?;

        Ok(dashboard)
    }

    /// Add widget to dashboard
    pub async fn add_widget(
        &self,
        dashboard_id: &str,
        widget_config: WidgetConfig,
    ) -> Result<Widget> {
        let mut dashboards = self.dashboards.write().await;
        let dashboard = dashboards
            .get_mut(dashboard_id)
            .ok_or_else(|| anyhow::anyhow!("Dashboard not found"))?;

        let widget = Widget {
            id: uuid::Uuid::new_v4().to_string(),
            type_: widget_config.type_,
            title: widget_config.title,
            position: widget_config.position,
            size: widget_config.size,
            data_binding: widget_config.data_binding,
            visualization: widget_config
                .visualization
                .unwrap_or_else(|| self.default_visualization()),
            interactions: widget_config.interactions,
            refresh_override: widget_config.refresh_override,
        };

        dashboard.widgets.push(widget.clone());
        dashboard.metadata.updated_at = Utc::now();

        // Save updated dashboard
        self.save_dashboard(dashboard).await?;

        Ok(widget)
    }

    /// Update widget
    pub async fn update_widget(
        &self,
        dashboard_id: &str,
        widget_id: &str,
        updates: WidgetUpdate,
    ) -> Result<()> {
        let mut dashboards = self.dashboards.write().await;
        let dashboard = dashboards
            .get_mut(dashboard_id)
            .ok_or_else(|| anyhow::anyhow!("Dashboard not found"))?;

        let widget = dashboard
            .widgets
            .iter_mut()
            .find(|w| w.id == widget_id)
            .ok_or_else(|| anyhow::anyhow!("Widget not found"))?;

        // Apply updates
        if let Some(title) = updates.title {
            widget.title = title;
        }
        if let Some(position) = updates.position {
            widget.position = position;
        }
        if let Some(size) = updates.size {
            widget.size = size;
        }
        if let Some(data_binding) = updates.data_binding {
            widget.data_binding = data_binding;
        }

        dashboard.metadata.updated_at = Utc::now();

        // Save updated dashboard
        self.save_dashboard(dashboard).await?;

        Ok(())
    }

    /// Remove widget
    pub async fn remove_widget(&self, dashboard_id: &str, widget_id: &str) -> Result<()> {
        let mut dashboards = self.dashboards.write().await;
        let dashboard = dashboards
            .get_mut(dashboard_id)
            .ok_or_else(|| anyhow::anyhow!("Dashboard not found"))?;

        dashboard.widgets.retain(|w| w.id != widget_id);
        dashboard.metadata.updated_at = Utc::now();

        // Save updated dashboard
        self.save_dashboard(dashboard).await?;

        Ok(())
    }

    /// Load dashboard
    pub async fn load_dashboard(&self, dashboard_id: &str) -> Result<Dashboard> {
        // Check memory first
        let dashboards = self.dashboards.read().await;
        if let Some(dashboard) = dashboards.get(dashboard_id) {
            return Ok(dashboard.clone());
        }
        drop(dashboards);

        // Load from disk
        let file_path = self.templates_path.join(format!("{}.json", dashboard_id));
        let content = fs::read_to_string(&file_path)
            .await
            .context("Failed to read dashboard file")?;

        let dashboard: Dashboard =
            serde_json::from_str(&content).context("Failed to parse dashboard")?;

        // Cache in memory
        let mut dashboards = self.dashboards.write().await;
        dashboards.insert(dashboard_id.to_string(), dashboard.clone());

        Ok(dashboard)
    }

    /// Save dashboard
    async fn save_dashboard(&self, dashboard: &Dashboard) -> Result<()> {
        // Save to memory
        let mut dashboards = self.dashboards.write().await;
        dashboards.insert(dashboard.id.clone(), dashboard.clone());
        drop(dashboards);

        // Save to disk
        if !self.templates_path.exists() {
            fs::create_dir_all(&self.templates_path).await?;
        }

        let file_path = self.templates_path.join(format!("{}.json", dashboard.id));
        let content = serde_json::to_string_pretty(dashboard)?;
        fs::write(&file_path, content).await?;

        Ok(())
    }

    /// List dashboards
    pub async fn list_dashboards(&self) -> Result<Vec<DashboardSummary>> {
        let mut summaries = Vec::new();

        // List from disk
        let mut entries = fs::read_dir(&self.templates_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path).await?;
                let dashboard: Dashboard = serde_json::from_str(&content)?;

                summaries.push(DashboardSummary {
                    id: dashboard.id,
                    name: dashboard.name,
                    description: dashboard.description,
                    widget_count: dashboard.widgets.len(),
                    created_at: dashboard.metadata.created_at,
                    updated_at: dashboard.metadata.updated_at,
                    tags: dashboard.metadata.tags,
                });
            }
        }

        Ok(summaries)
    }

    /// Delete dashboard
    pub async fn delete_dashboard(&self, dashboard_id: &str) -> Result<()> {
        // Remove from memory
        let mut dashboards = self.dashboards.write().await;
        dashboards.remove(dashboard_id);
        drop(dashboards);

        // Remove from disk
        let file_path = self.templates_path.join(format!("{}.json", dashboard_id));
        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }

        Ok(())
    }

    /// Export dashboard
    pub async fn export_dashboard(
        &self,
        dashboard_id: &str,
        format: ExportFormat,
    ) -> Result<Vec<u8>> {
        let dashboard = self.load_dashboard(dashboard_id).await?;

        match format {
            ExportFormat::JSON => {
                let json = serde_json::to_vec_pretty(&dashboard)?;
                Ok(json)
            }
            ExportFormat::PNG => {
                // Would render dashboard to PNG
                Ok(Vec::new())
            }
            ExportFormat::PDF => {
                // Would render dashboard to PDF
                Ok(Vec::new())
            }
            _ => Err(anyhow::anyhow!(
                "Export format not supported for dashboards"
            )),
        }
    }

    /// Import dashboard
    pub async fn import_dashboard(&self, data: &[u8]) -> Result<Dashboard> {
        let mut dashboard: Dashboard =
            serde_json::from_slice(data).context("Failed to parse dashboard")?;

        // Generate new ID
        dashboard.id = uuid::Uuid::new_v4().to_string();
        dashboard.metadata.created_at = Utc::now();
        dashboard.metadata.updated_at = Utc::now();

        // Save dashboard
        self.save_dashboard(&dashboard).await?;

        Ok(dashboard)
    }

    /// Share dashboard
    pub async fn share_dashboard(&self, dashboard_id: &str, public: bool) -> Result<String> {
        let mut dashboards = self.dashboards.write().await;
        let dashboard = dashboards
            .get_mut(dashboard_id)
            .ok_or_else(|| anyhow::anyhow!("Dashboard not found"))?;

        dashboard.permissions.public = public;

        if public && dashboard.permissions.share_link.is_none() {
            dashboard.permissions.share_link = Some(format!(
                "https://hive.ai/dashboards/shared/{}",
                uuid::Uuid::new_v4()
            ));
        } else if !public {
            dashboard.permissions.share_link = None;
        }

        let share_link = dashboard.permissions.share_link.clone().unwrap_or_default();

        // Save updated dashboard
        self.save_dashboard(dashboard).await?;

        Ok(share_link)
    }

    /// Get dashboard from template marketplace
    pub async fn get_marketplace_template(&self, template_id: &str) -> Result<Dashboard> {
        self.marketplace_client.get_template(template_id).await
    }

    /// List marketplace templates
    pub async fn list_marketplace_templates(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<MarketplaceTemplate>> {
        self.marketplace_client.list_templates(category).await
    }

    // Helper methods

    fn default_theme(&self) -> DashboardTheme {
        DashboardTheme {
            name: "Default".to_string(),
            primary_color: "#1a73e8".to_string(),
            secondary_color: "#5f6368".to_string(),
            background_color: "#ffffff".to_string(),
            text_color: "#202124".to_string(),
            font_family: "Roboto, sans-serif".to_string(),
            custom_css: None,
        }
    }

    fn default_visualization(&self) -> VisualizationConfig {
        VisualizationConfig {
            color_palette: "default".to_string(),
            font_family: "inherit".to_string(),
            font_size: 14,
            padding: Padding {
                top: 16,
                right: 16,
                bottom: 16,
                left: 16,
            },
            background: None,
            border: None,
        }
    }
}

/// Dashboard configuration for creation
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub name: String,
    pub description: String,
    pub layout: DashboardLayout,
    pub theme: Option<DashboardTheme>,
    pub refresh_interval: Option<u64>,
    pub owner: String,
    pub tags: Vec<String>,
    pub category: Option<String>,
}

/// Widget configuration for creation
#[derive(Debug, Clone)]
pub struct WidgetConfig {
    pub type_: WidgetType,
    pub title: String,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub data_binding: DataBinding,
    pub visualization: Option<VisualizationConfig>,
    pub interactions: Vec<WidgetInteraction>,
    pub refresh_override: Option<u64>,
}

/// Widget update parameters
#[derive(Debug, Clone)]
pub struct WidgetUpdate {
    pub title: Option<String>,
    pub position: Option<WidgetPosition>,
    pub size: Option<WidgetSize>,
    pub data_binding: Option<DataBinding>,
}

/// Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub widget_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

/// Marketplace client (placeholder)
struct MarketplaceClient;

impl MarketplaceClient {
    fn new() -> Self {
        Self
    }

    async fn get_template(&self, template_id: &str) -> Result<Dashboard> {
        // Would fetch from marketplace API
        Err(anyhow::anyhow!("Marketplace not implemented"))
    }

    async fn list_templates(&self, category: Option<&str>) -> Result<Vec<MarketplaceTemplate>> {
        // Would fetch from marketplace API
        Ok(Vec::new())
    }
}

/// Marketplace template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub preview_url: String,
    pub author: String,
    pub downloads: u32,
    pub rating: f32,
    pub price: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_creation() -> Result<()> {
        let builder = DashboardBuilder::new(Some(PathBuf::from("/tmp/hive_dashboards")));

        let config = DashboardConfig {
            name: "Test Dashboard".to_string(),
            description: "Test dashboard description".to_string(),
            layout: DashboardLayout::Grid,
            theme: None,
            refresh_interval: None,
            owner: "test_user".to_string(),
            tags: vec!["test".to_string()],
            category: None,
        };

        let dashboard = builder.create_dashboard(config).await?;

        assert_eq!(dashboard.name, "Test Dashboard");
        assert_eq!(dashboard.layout, DashboardLayout::Grid);

        Ok(())
    }

    #[tokio::test]
    async fn test_widget_management() -> Result<()> {
        let builder = DashboardBuilder::new(Some(PathBuf::from("/tmp/hive_dashboards")));

        // Create dashboard
        let dashboard = builder
            .create_dashboard(DashboardConfig {
                name: "Widget Test".to_string(),
                description: "Testing widgets".to_string(),
                layout: DashboardLayout::Grid,
                theme: None,
                refresh_interval: None,
                owner: "test_user".to_string(),
                tags: vec![],
                category: None,
            })
            .await?;

        // Add widget
        let widget_config = WidgetConfig {
            type_: WidgetType::Chart {
                chart_type: ChartType::Line,
                options: ChartOptions {
                    stacked: false,
                    show_legend: true,
                    show_grid: true,
                    animations: true,
                    tooltips: true,
                    zoom: false,
                    export: true,
                },
            },
            title: "Test Chart".to_string(),
            position: WidgetPosition { x: 0, y: 0, z: 0 },
            size: WidgetSize {
                width: 6,
                height: 4,
                min_width: None,
                min_height: None,
                max_width: None,
                max_height: None,
            },
            data_binding: DataBinding {
                source: "analytics.trends".to_string(),
                query: None,
                transform: None,
                cache: true,
                real_time: false,
            },
            visualization: None,
            interactions: vec![],
            refresh_override: None,
        };

        let widget = builder.add_widget(&dashboard.id, widget_config).await?;

        assert_eq!(widget.title, "Test Chart");

        // Update widget
        let updates = WidgetUpdate {
            title: Some("Updated Chart".to_string()),
            position: None,
            size: None,
            data_binding: None,
        };

        builder
            .update_widget(&dashboard.id, &widget.id, updates)
            .await?;

        // Load and verify
        let updated_dashboard = builder.load_dashboard(&dashboard.id).await?;
        let updated_widget = updated_dashboard
            .widgets
            .iter()
            .find(|w| w.id == widget.id)
            .unwrap();

        assert_eq!(updated_widget.title, "Updated Chart");

        Ok(())
    }
}
