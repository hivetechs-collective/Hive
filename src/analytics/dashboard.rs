//! Real-time Analytics Dashboard with Live Metrics
//!
//! This module provides:
//! - Real-time metric streaming
//! - Dashboard layouts and widgets
//! - ASCII visualization for CLI
//! - Enhanced visualization for TUI
//! - Live data updates
//! - Interactive controls

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;
use tracing::{debug, info};

use crate::analytics::AdvancedAnalyticsConfig;
use crate::core::database::{get_database, ActivityLog};

/// Dashboard engine for real-time analytics
pub struct DashboardEngine {
    config: Arc<RwLock<AdvancedAnalyticsConfig>>,
    layout_manager: Arc<LayoutManager>,
    widget_factory: Arc<WidgetFactory>,
    data_streamer: Arc<DataStreamer>,
    renderer: Arc<DashboardRenderer>,
}

/// Dashboard layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub name: String,
    pub grid: GridLayout,
    pub widgets: Vec<WidgetConfig>,
    pub refresh_rate: RefreshRate,
    pub theme: DashboardTheme,
}

/// Grid layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLayout {
    pub rows: u16,
    pub cols: u16,
    pub gap: u16,
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub id: String,
    pub widget_type: WidgetType,
    pub position: Position,
    pub size: Size,
    pub data_source: String,
    pub options: WidgetOptions,
}

/// Widget position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

/// Widget size
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Size {
    pub rows: u16,
    pub cols: u16,
}

/// Widget types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    GaugeChart,
    Sparkline,
    HeatMap,
    Table,
    StatCard,
    AlertList,
    LogStream,
}

/// Widget options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetOptions {
    pub title: String,
    pub subtitle: Option<String>,
    pub show_legend: bool,
    pub show_grid: bool,
    pub animation: bool,
    pub custom: HashMap<String, serde_json::Value>,
}

/// Refresh rate options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefreshRate {
    Realtime, // < 1 second
    Fast,     // 1 second
    Normal,   // 5 seconds
    Slow,     // 30 seconds
    Manual,   // User triggered
}

impl RefreshRate {
    pub fn to_duration(&self) -> std::time::Duration {
        match self {
            Self::Realtime => std::time::Duration::from_millis(500),
            Self::Fast => std::time::Duration::from_secs(1),
            Self::Normal => std::time::Duration::from_secs(5),
            Self::Slow => std::time::Duration::from_secs(30),
            Self::Manual => std::time::Duration::from_secs(3600), // Effectively never
        }
    }
}

/// Dashboard theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTheme {
    pub name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: String,
    pub text_color: String,
    pub accent_colors: Vec<String>,
}

impl Default for DashboardTheme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            primary_color: "#3498db".to_string(),
            secondary_color: "#2ecc71".to_string(),
            background_color: "#1a1a1a".to_string(),
            text_color: "#ffffff".to_string(),
            accent_colors: vec![
                "#e74c3c".to_string(),
                "#f39c12".to_string(),
                "#9b59b6".to_string(),
                "#1abc9c".to_string(),
            ],
        }
    }
}

/// Dashboard widget with data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub config: WidgetConfig,
    pub data: WidgetData,
    pub last_updated: DateTime<Utc>,
}

/// Widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetData {
    pub primary_value: Option<f64>,
    pub secondary_value: Option<f64>,
    pub series: Option<Vec<Series>>,
    pub table_data: Option<TableData>,
    pub alerts: Option<Vec<Alert>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Time series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub name: String,
    pub data_points: Vec<DataPoint>,
    pub color: Option<String>,
}

/// Data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub label: Option<String>,
}

/// Table data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub sortable: bool,
}

/// Alert data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Alert levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Real-time metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

/// Data stream for real-time updates
#[derive(Debug, Clone)]
pub struct DataStream {
    pub stream_id: String,
    pub metric_name: String,
    pub buffer_size: usize,
}

/// Layout manager
struct LayoutManager {
    layouts: Arc<RwLock<HashMap<String, DashboardLayout>>>,
    active_layout: Arc<RwLock<String>>,
}

/// Widget factory
struct WidgetFactory {
    widget_templates: HashMap<WidgetType, WidgetTemplate>,
}

/// Widget template
struct WidgetTemplate {
    default_options: WidgetOptions,
    required_data_fields: Vec<String>,
    renderer: Box<dyn WidgetRenderer + Send + Sync>,
}

/// Widget renderer trait
trait WidgetRenderer: Send + Sync {
    fn render_ascii(&self, widget: &Widget, width: u16, height: u16) -> Result<String>;
    fn render_tui(&self, widget: &Widget) -> Result<serde_json::Value>;
}

/// Data streamer for real-time updates
struct DataStreamer {
    streams: Arc<RwLock<HashMap<String, StreamState>>>,
    sender: broadcast::Sender<RealTimeMetric>,
}

/// Stream state
struct StreamState {
    stream: DataStream,
    buffer: VecDeque<RealTimeMetric>,
    subscribers: u32,
}

/// Dashboard renderer
struct DashboardRenderer {
    ascii_mode: bool,
    color_support: bool,
}

impl DashboardEngine {
    /// Create a new dashboard engine
    pub async fn new(config: Arc<RwLock<AdvancedAnalyticsConfig>>) -> Result<Self> {
        info!("Initializing real-time dashboard engine");

        let layout_manager = Arc::new(LayoutManager::new());
        let widget_factory = Arc::new(WidgetFactory::new());
        let data_streamer = Arc::new(DataStreamer::new());
        let renderer = Arc::new(DashboardRenderer::new());

        // Initialize default layout
        layout_manager.initialize_default_layout().await?;

        // Start data streaming
        data_streamer.start_streaming(Arc::clone(&config)).await?;

        Ok(Self {
            config,
            layout_manager,
            widget_factory,
            data_streamer,
            renderer,
        })
    }

    /// Get current dashboard
    pub async fn get_dashboard(&self) -> Result<Dashboard> {
        let layout = self.layout_manager.get_active_layout().await?;
        let widgets = self.create_widgets(&layout).await?;

        Ok(Dashboard {
            layout: layout.clone(),
            widgets,
            last_updated: Utc::now(),
        })
    }

    /// Update widget data
    pub async fn update_widget(&self, widget_id: &str) -> Result<Widget> {
        let layout = self.layout_manager.get_active_layout().await?;
        let widget_config = layout
            .widgets
            .iter()
            .find(|w| w.id == widget_id)
            .ok_or_else(|| anyhow::anyhow!("Widget not found: {}", widget_id))?;

        self.create_widget(widget_config).await
    }

    /// Set active layout
    pub async fn set_layout(&self, layout_name: &str) -> Result<()> {
        self.layout_manager.set_active_layout(layout_name).await
    }

    /// Get available layouts
    pub async fn get_layouts(&self) -> Result<Vec<String>> {
        self.layout_manager.get_layout_names().await
    }

    /// Subscribe to real-time updates
    pub async fn subscribe(&self) -> broadcast::Receiver<RealTimeMetric> {
        self.data_streamer.subscribe()
    }

    /// Render dashboard as ASCII
    pub async fn render_ascii(&self, width: u16, height: u16) -> Result<String> {
        let dashboard = self.get_dashboard().await?;
        self.renderer
            .render_dashboard_ascii(&dashboard, width, height)
    }

    /// Reload configuration
    pub async fn reload_config(&self) -> Result<()> {
        debug!("Reloading dashboard configuration");
        Ok(())
    }

    // Private helper methods

    async fn create_widgets(&self, layout: &DashboardLayout) -> Result<Vec<Widget>> {
        let mut widgets = Vec::new();

        for config in &layout.widgets {
            let widget = self.create_widget(config).await?;
            widgets.push(widget);
        }

        Ok(widgets)
    }

    async fn create_widget(&self, config: &WidgetConfig) -> Result<Widget> {
        let data = self.fetch_widget_data(config).await?;

        Ok(Widget {
            config: config.clone(),
            data,
            last_updated: Utc::now(),
        })
    }

    async fn fetch_widget_data(&self, config: &WidgetConfig) -> Result<WidgetData> {
        match config.widget_type {
            WidgetType::LineChart => self.fetch_line_chart_data(&config.data_source).await,
            WidgetType::BarChart => self.fetch_bar_chart_data(&config.data_source).await,
            WidgetType::GaugeChart => self.fetch_gauge_data(&config.data_source).await,
            WidgetType::StatCard => self.fetch_stat_card_data(&config.data_source).await,
            WidgetType::Table => self.fetch_table_data(&config.data_source).await,
            WidgetType::AlertList => self.fetch_alerts_data(&config.data_source).await,
            _ => Ok(WidgetData::default()),
        }
    }

    async fn fetch_line_chart_data(&self, data_source: &str) -> Result<WidgetData> {
        // Fetch time series data
        let activities = ActivityLog::get_recent(100).await?;
        let mut data_points = Vec::new();

        for activity in activities {
            if let Ok(timestamp) = activity.created_at.parse::<DateTime<Utc>>() {
                data_points.push(DataPoint {
                    timestamp,
                    value: 1.0, // Count queries
                    label: None,
                });
            }
        }

        let series = vec![Series {
            name: "Query Volume".to_string(),
            data_points,
            color: Some("#3498db".to_string()),
        }];

        Ok(WidgetData {
            primary_value: None,
            secondary_value: None,
            series: Some(series),
            table_data: None,
            alerts: None,
            metadata: HashMap::new(),
        })
    }

    async fn fetch_bar_chart_data(&self, data_source: &str) -> Result<WidgetData> {
        // Model usage distribution
        let activities = ActivityLog::get_recent(1000).await?;
        let mut model_counts: HashMap<String, f64> = HashMap::new();

        for activity in activities {
            if let Some(model) = &activity.model_used {
                *model_counts.entry(model.clone()).or_insert(0.0) += 1.0;
            }
        }

        let mut series = Vec::new();
        for (model, count) in model_counts {
            series.push(Series {
                name: model,
                data_points: vec![DataPoint {
                    timestamp: Utc::now(),
                    value: count,
                    label: None,
                }],
                color: None,
            });
        }

        Ok(WidgetData {
            primary_value: None,
            secondary_value: None,
            series: Some(series),
            table_data: None,
            alerts: None,
            metadata: HashMap::new(),
        })
    }

    async fn fetch_gauge_data(&self, data_source: &str) -> Result<WidgetData> {
        // Performance gauge
        let db = get_database().await?;
        let stats = db.get_statistics().await?;

        // Calculate average response time (placeholder)
        let avg_response_time = 245.0;

        let mut metadata = HashMap::new();
        metadata.insert("min".to_string(), serde_json::json!(0));
        metadata.insert("max".to_string(), serde_json::json!(1000));
        metadata.insert("unit".to_string(), serde_json::json!("ms"));
        metadata.insert(
            "thresholds".to_string(),
            serde_json::json!({
                "good": 200,
                "warning": 500,
                "critical": 1000
            }),
        );

        Ok(WidgetData {
            primary_value: Some(avg_response_time),
            secondary_value: None,
            series: None,
            table_data: None,
            alerts: None,
            metadata,
        })
    }

    async fn fetch_stat_card_data(&self, data_source: &str) -> Result<WidgetData> {
        let db = get_database().await?;
        let stats = db.get_statistics().await?;

        let value = match data_source {
            "total_conversations" => stats.conversation_count as f64,
            "total_messages" => stats.message_count as f64,
            "active_users" => stats.user_count as f64,
            "models_available" => stats.model_count as f64,
            _ => 0.0,
        };

        Ok(WidgetData {
            primary_value: Some(value),
            secondary_value: None,
            series: None,
            table_data: None,
            alerts: None,
            metadata: HashMap::new(),
        })
    }

    async fn fetch_table_data(&self, data_source: &str) -> Result<WidgetData> {
        // Recent activity table
        let activities = ActivityLog::get_recent(10).await?;

        let headers = vec![
            "Time".to_string(),
            "User".to_string(),
            "Model".to_string(),
            "Cost".to_string(),
        ];

        let mut rows = Vec::new();
        for activity in activities {
            let time = activity.created_at.split('T').next().unwrap_or("-");
            let user = activity.user_id.as_deref().unwrap_or("-");
            let model = activity.model_used.as_deref().unwrap_or("-");
            let cost = activity
                .cost
                .map(|c| format!("${:.3}", c))
                .unwrap_or("-".to_string());

            rows.push(vec![
                time.to_string(),
                user.to_string(),
                model.to_string(),
                cost,
            ]);
        }

        Ok(WidgetData {
            primary_value: None,
            secondary_value: None,
            series: None,
            table_data: Some(TableData {
                headers,
                rows,
                sortable: true,
            }),
            alerts: None,
            metadata: HashMap::new(),
        })
    }

    async fn fetch_alerts_data(&self, data_source: &str) -> Result<WidgetData> {
        let mut alerts = Vec::new();

        // Check for high costs
        let activities = ActivityLog::get_recent(100).await?;
        let total_cost: f64 = activities.iter().filter_map(|a| a.cost).sum();

        if total_cost > 50.0 {
            alerts.push(Alert {
                level: AlertLevel::Warning,
                title: "High Usage Costs".to_string(),
                message: format!("Total costs reached ${:.2} today", total_cost),
                timestamp: Utc::now(),
            });
        }

        Ok(WidgetData {
            primary_value: None,
            secondary_value: None,
            series: None,
            table_data: None,
            alerts: Some(alerts),
            metadata: HashMap::new(),
        })
    }
}

/// Dashboard with widgets
#[derive(Debug, Clone)]
pub struct Dashboard {
    pub layout: DashboardLayout,
    pub widgets: Vec<Widget>,
    pub last_updated: DateTime<Utc>,
}

impl Default for WidgetData {
    fn default() -> Self {
        Self {
            primary_value: None,
            secondary_value: None,
            series: None,
            table_data: None,
            alerts: None,
            metadata: HashMap::new(),
        }
    }
}

impl LayoutManager {
    fn new() -> Self {
        Self {
            layouts: Arc::new(RwLock::new(HashMap::new())),
            active_layout: Arc::new(RwLock::new("default".to_string())),
        }
    }

    async fn initialize_default_layout(&self) -> Result<()> {
        let default_layout = DashboardLayout {
            name: "default".to_string(),
            grid: GridLayout {
                rows: 6,
                cols: 4,
                gap: 1,
            },
            widgets: vec![
                // Query volume line chart
                WidgetConfig {
                    id: "query_volume".to_string(),
                    widget_type: WidgetType::LineChart,
                    position: Position { row: 0, col: 0 },
                    size: Size { rows: 2, cols: 2 },
                    data_source: "query_volume".to_string(),
                    options: WidgetOptions {
                        title: "Query Volume (24h)".to_string(),
                        subtitle: Some("Queries per hour".to_string()),
                        show_legend: true,
                        show_grid: true,
                        animation: true,
                        custom: HashMap::new(),
                    },
                },
                // Performance gauge
                WidgetConfig {
                    id: "performance".to_string(),
                    widget_type: WidgetType::GaugeChart,
                    position: Position { row: 0, col: 2 },
                    size: Size { rows: 1, cols: 1 },
                    data_source: "avg_response_time".to_string(),
                    options: WidgetOptions {
                        title: "Avg Response Time".to_string(),
                        subtitle: None,
                        show_legend: false,
                        show_grid: false,
                        animation: true,
                        custom: HashMap::new(),
                    },
                },
                // Cost stat card
                WidgetConfig {
                    id: "total_cost".to_string(),
                    widget_type: WidgetType::StatCard,
                    position: Position { row: 0, col: 3 },
                    size: Size { rows: 1, cols: 1 },
                    data_source: "total_cost".to_string(),
                    options: WidgetOptions {
                        title: "Total Cost".to_string(),
                        subtitle: Some("This month".to_string()),
                        show_legend: false,
                        show_grid: false,
                        animation: false,
                        custom: HashMap::new(),
                    },
                },
                // Model usage bar chart
                WidgetConfig {
                    id: "model_usage".to_string(),
                    widget_type: WidgetType::BarChart,
                    position: Position { row: 2, col: 0 },
                    size: Size { rows: 2, cols: 2 },
                    data_source: "model_usage".to_string(),
                    options: WidgetOptions {
                        title: "Model Usage".to_string(),
                        subtitle: Some("By query count".to_string()),
                        show_legend: true,
                        show_grid: true,
                        animation: true,
                        custom: HashMap::new(),
                    },
                },
                // Recent activity table
                WidgetConfig {
                    id: "recent_activity".to_string(),
                    widget_type: WidgetType::Table,
                    position: Position { row: 2, col: 2 },
                    size: Size { rows: 2, cols: 2 },
                    data_source: "recent_activity".to_string(),
                    options: WidgetOptions {
                        title: "Recent Activity".to_string(),
                        subtitle: None,
                        show_legend: false,
                        show_grid: false,
                        animation: false,
                        custom: HashMap::new(),
                    },
                },
                // Alerts list
                WidgetConfig {
                    id: "alerts".to_string(),
                    widget_type: WidgetType::AlertList,
                    position: Position { row: 4, col: 0 },
                    size: Size { rows: 2, cols: 4 },
                    data_source: "system_alerts".to_string(),
                    options: WidgetOptions {
                        title: "System Alerts".to_string(),
                        subtitle: None,
                        show_legend: false,
                        show_grid: false,
                        animation: false,
                        custom: HashMap::new(),
                    },
                },
            ],
            refresh_rate: RefreshRate::Normal,
            theme: DashboardTheme::default(),
        };

        let mut layouts = self.layouts.write().await;
        layouts.insert("default".to_string(), default_layout);

        Ok(())
    }

    async fn get_active_layout(&self) -> Result<DashboardLayout> {
        let active = self.active_layout.read().await;
        let layouts = self.layouts.read().await;

        layouts
            .get(active.as_str())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Active layout not found"))
    }

    async fn set_active_layout(&self, name: &str) -> Result<()> {
        let layouts = self.layouts.read().await;
        if !layouts.contains_key(name) {
            return Err(anyhow::anyhow!("Layout not found: {}", name));
        }

        let mut active = self.active_layout.write().await;
        *active = name.to_string();

        Ok(())
    }

    async fn get_layout_names(&self) -> Result<Vec<String>> {
        let layouts = self.layouts.read().await;
        Ok(layouts.keys().cloned().collect())
    }
}

impl WidgetFactory {
    fn new() -> Self {
        let mut widget_templates = HashMap::new();

        // Register widget templates
        widget_templates.insert(
            WidgetType::LineChart,
            WidgetTemplate {
                default_options: WidgetOptions {
                    title: "Line Chart".to_string(),
                    subtitle: None,
                    show_legend: true,
                    show_grid: true,
                    animation: true,
                    custom: HashMap::new(),
                },
                required_data_fields: vec!["series".to_string()],
                renderer: Box::new(LineChartRenderer),
            },
        );

        widget_templates.insert(
            WidgetType::GaugeChart,
            WidgetTemplate {
                default_options: WidgetOptions {
                    title: "Gauge".to_string(),
                    subtitle: None,
                    show_legend: false,
                    show_grid: false,
                    animation: true,
                    custom: HashMap::new(),
                },
                required_data_fields: vec!["primary_value".to_string()],
                renderer: Box::new(GaugeRenderer),
            },
        );

        Self { widget_templates }
    }
}

impl DataStreamer {
    fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);

        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            sender,
        }
    }

    async fn start_streaming(&self, config: Arc<RwLock<AdvancedAnalyticsConfig>>) -> Result<()> {
        let streams = Arc::clone(&self.streams);
        let sender = self.sender.clone();

        tokio::spawn(async move {
            let mut interval = interval(std::time::Duration::from_secs(1));

            loop {
                interval.tick().await;

                // Generate sample metrics
                let metric = RealTimeMetric {
                    name: "queries_per_second".to_string(),
                    value: rand::random::<f64>() * 10.0,
                    unit: "qps".to_string(),
                    timestamp: Utc::now(),
                    tags: HashMap::new(),
                };

                let _ = sender.send(metric);
            }
        });

        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<RealTimeMetric> {
        self.sender.subscribe()
    }
}

impl DashboardRenderer {
    fn new() -> Self {
        Self {
            ascii_mode: true,
            color_support: true,
        }
    }

    fn render_dashboard_ascii(
        &self,
        dashboard: &Dashboard,
        width: u16,
        height: u16,
    ) -> Result<String> {
        let mut output = String::new();

        // Dashboard header
        output.push_str(&format!(
            "╭{:─^width$}╮\n",
            " Analytics Dashboard ",
            width = width as usize - 2
        ));
        output.push_str(&format!(
            "│ Last updated: {:^width$} │\n",
            dashboard.last_updated.format("%H:%M:%S"),
            width = width as usize - 17
        ));
        output.push_str(&format!("╰{:─<width$}╯\n", "", width = width as usize - 2));

        // Render widgets in grid
        for widget in &dashboard.widgets {
            let widget_width = (width / dashboard.layout.grid.cols) - 2;
            let widget_height = (height / dashboard.layout.grid.rows) - 2;

            if let Some(renderer) = self.get_widget_renderer(widget.config.widget_type) {
                let widget_output = renderer.render_ascii(widget, widget_width, widget_height)?;
                output.push_str(&widget_output);
                output.push_str("\n");
            }
        }

        Ok(output)
    }

    fn get_widget_renderer(&self, widget_type: WidgetType) -> Option<Box<dyn WidgetRenderer>> {
        match widget_type {
            WidgetType::LineChart => Some(Box::new(LineChartRenderer)),
            WidgetType::GaugeChart => Some(Box::new(GaugeRenderer)),
            WidgetType::StatCard => Some(Box::new(StatCardRenderer)),
            WidgetType::Table => Some(Box::new(TableRenderer)),
            _ => None,
        }
    }
}

// Widget renderer implementations

struct LineChartRenderer;

impl WidgetRenderer for LineChartRenderer {
    fn render_ascii(&self, widget: &Widget, width: u16, height: u16) -> Result<String> {
        let mut output = String::new();

        output.push_str(&format!(
            "┌{:─^width$}┐\n",
            format!(" {} ", widget.config.options.title),
            width = width as usize - 2
        ));

        // Simple ASCII line chart
        if let Some(series) = &widget.data.series {
            if let Some(first_series) = series.first() {
                let points = &first_series.data_points;
                if !points.is_empty() {
                    // Find min/max values
                    let max_val = points
                        .iter()
                        .map(|p| p.value)
                        .fold(f64::NEG_INFINITY, f64::max);
                    let min_val = points.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);

                    // Create simple sparkline
                    let sparkline = create_sparkline(points, (width - 4) as usize);
                    output.push_str(&format!("│ {} │\n", sparkline));
                    output.push_str(&format!(
                        "│ Min: {:.1} Max: {:.1}{:>width$} │\n",
                        min_val,
                        max_val,
                        "",
                        width = width as usize - 20
                    ));
                }
            }
        }

        output.push_str(&format!("└{:─<width$}┘", "", width = width as usize - 2));

        Ok(output)
    }

    fn render_tui(&self, widget: &Widget) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "line_chart",
            "data": widget.data,
            "options": widget.config.options,
        }))
    }
}

struct GaugeRenderer;

impl WidgetRenderer for GaugeRenderer {
    fn render_ascii(&self, widget: &Widget, width: u16, height: u16) -> Result<String> {
        let mut output = String::new();

        output.push_str(&format!(
            "┌{:─^width$}┐\n",
            format!(" {} ", widget.config.options.title),
            width = width as usize - 2
        ));

        if let Some(value) = widget.data.primary_value {
            let max = widget
                .data
                .metadata
                .get("max")
                .and_then(|v| v.as_f64())
                .unwrap_or(100.0);

            let percentage = (value / max * 100.0).min(100.0);
            let filled = ((percentage / 100.0) * (width as f64 - 6.0)) as usize;
            let empty = (width as usize - 6) - filled;

            output.push_str(&format!(
                "│ [{}{}] │\n",
                "█".repeat(filled),
                "░".repeat(empty)
            ));

            output.push_str(&format!(
                "│ {:.1}{}{:>width$} │\n",
                value,
                widget
                    .data
                    .metadata
                    .get("unit")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                "",
                width = width as usize - 10
            ));
        }

        output.push_str(&format!("└{:─<width$}┘", "", width = width as usize - 2));

        Ok(output)
    }

    fn render_tui(&self, widget: &Widget) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "gauge",
            "data": widget.data,
            "options": widget.config.options,
        }))
    }
}

struct StatCardRenderer;

impl WidgetRenderer for StatCardRenderer {
    fn render_ascii(&self, widget: &Widget, width: u16, height: u16) -> Result<String> {
        let mut output = String::new();

        output.push_str(&format!(
            "┌{:─^width$}┐\n",
            format!(" {} ", widget.config.options.title),
            width = width as usize - 2
        ));

        if let Some(value) = widget.data.primary_value {
            output.push_str(&format!(
                "│{:^width$}│\n",
                format!("{:.0}", value),
                width = width as usize - 2
            ));

            if let Some(subtitle) = &widget.config.options.subtitle {
                output.push_str(&format!(
                    "│{:^width$}│\n",
                    subtitle,
                    width = width as usize - 2
                ));
            }
        }

        output.push_str(&format!("└{:─<width$}┘", "", width = width as usize - 2));

        Ok(output)
    }

    fn render_tui(&self, widget: &Widget) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "stat_card",
            "data": widget.data,
            "options": widget.config.options,
        }))
    }
}

struct TableRenderer;

impl WidgetRenderer for TableRenderer {
    fn render_ascii(&self, widget: &Widget, width: u16, height: u16) -> Result<String> {
        let mut output = String::new();

        output.push_str(&format!(
            "┌{:─^width$}┐\n",
            format!(" {} ", widget.config.options.title),
            width = width as usize - 2
        ));

        if let Some(table) = &widget.data.table_data {
            // Render headers
            if !table.headers.is_empty() {
                let col_width = (width as usize - 2) / table.headers.len();
                output.push_str("│");
                for header in &table.headers {
                    output.push_str(&format!("{:^width$}", header, width = col_width));
                }
                output.push_str("│\n");

                // Separator
                output.push_str(&format!("├{:─<width$}┤\n", "", width = width as usize - 2));

                // Render rows (limited by height)
                let max_rows = (height as usize).saturating_sub(5);
                for row in table.rows.iter().take(max_rows) {
                    output.push_str("│");
                    for (i, cell) in row.iter().enumerate() {
                        if i < table.headers.len() {
                            output.push_str(&format!("{:^width$}", cell, width = col_width));
                        }
                    }
                    output.push_str("│\n");
                }
            }
        }

        output.push_str(&format!("└{:─<width$}┘", "", width = width as usize - 2));

        Ok(output)
    }

    fn render_tui(&self, widget: &Widget) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "table",
            "data": widget.data,
            "options": widget.config.options,
        }))
    }
}

// Helper functions

fn create_sparkline(points: &[DataPoint], width: usize) -> String {
    if points.is_empty() || width == 0 {
        return String::new();
    }

    let values: Vec<f64> = points.iter().map(|p| p.value).collect();
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let range = max - min;

    if range == 0.0 {
        return "─".repeat(width);
    }

    let spark_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let mut sparkline = String::new();

    // Sample points to fit width
    let step = values.len() / width;
    for i in 0..width {
        let idx = i * step;
        if idx < values.len() {
            let normalized = (values[idx] - min) / range;
            let char_idx = ((normalized * 7.0) as usize).min(7);
            sparkline.push(spark_chars[char_idx]);
        }
    }

    sparkline
}

// External dependency placeholder
mod rand {
    pub fn random<T>() -> T
    where
        T: Default,
    {
        T::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_engine_creation() -> Result<()> {
        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let engine = DashboardEngine::new(config).await?;

        let layouts = engine.get_layouts().await?;
        assert!(layouts.contains(&"default".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_dashboard_rendering() -> Result<()> {
        let config = Arc::new(RwLock::new(AdvancedAnalyticsConfig::default()));
        let engine = DashboardEngine::new(config).await?;

        let ascii = engine.render_ascii(80, 24).await?;
        assert!(!ascii.is_empty());
        assert!(ascii.contains("Analytics Dashboard"));

        Ok(())
    }

    #[test]
    fn test_sparkline_creation() {
        let points = vec![
            DataPoint {
                timestamp: Utc::now(),
                value: 1.0,
                label: None,
            },
            DataPoint {
                timestamp: Utc::now(),
                value: 5.0,
                label: None,
            },
            DataPoint {
                timestamp: Utc::now(),
                value: 3.0,
                label: None,
            },
            DataPoint {
                timestamp: Utc::now(),
                value: 7.0,
                label: None,
            },
            DataPoint {
                timestamp: Utc::now(),
                value: 2.0,
                label: None,
            },
        ];

        let sparkline = create_sparkline(&points, 5);
        assert_eq!(sparkline.len(), 5);
        assert!(sparkline.chars().all(|c| "▁▂▃▄▅▆▇█".contains(c)));
    }

    #[test]
    fn test_refresh_rate_duration() {
        assert_eq!(
            RefreshRate::Realtime.to_duration(),
            std::time::Duration::from_millis(500)
        );
        assert_eq!(
            RefreshRate::Fast.to_duration(),
            std::time::Duration::from_secs(1)
        );
        assert_eq!(
            RefreshRate::Normal.to_duration(),
            std::time::Duration::from_secs(5)
        );
    }
}
