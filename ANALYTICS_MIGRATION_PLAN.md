# Analytics & Reporting Migration Plan: TypeScript to Rust (1-Day Sprint)

## 📋 Executive Summary

This document outlines the focused plan to rapidly migrate core analytics and reporting capabilities from TypeScript to Rust in a single day, prioritizing the most critical features for immediate user value through the TUI Reports panel.

## 🎯 Day Objectives

1. **Core Analytics**: Port essential analytics engine functionality
2. **TUI Reports Panel**: Create functional Reports panel with 3 main views
3. **Real-time Integration**: Connect to live consensus pipeline data
4. **Basic Export**: Enable JSON/CSV export from TUI
5. **Working Demo**: Fully functional analytics in TUI by end of day

## 📊 Current State Analysis

### TypeScript Implementation Strengths
- Comprehensive analytics engine with 6 metric categories
- Real-time data collection and aggregation
- Multiple export formats (JSON, CSV, Excel, HTML, PDF)
- ASCII chart visualization
- Integrated CLI commands
- Cloud sync via Cloudflare D1

### Rust Implementation Status
- ✅ Analytics module structure exists
- ✅ Database schema with analytics tables
- ✅ Basic CLI commands implemented
- ✅ Configuration system ready
- ❌ TUI Reports panel missing
- ❌ Some advanced features incomplete
- ❌ Export formats limited

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      TUI Interface Layer                     │
├─────────────────────────────────────────────────────────────┤
│  Explorer │  Editor  │  Consensus  │  Reports  │ Terminal   │
└───────────┴──────────┴─────────────┴───────────┴────────────┘
                                            │
                                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   Analytics Engine Core                      │
├─────────────────────────────────────────────────────────────┤
│ • Metrics Collector    • Report Generator   • Alert System  │
│ • Trend Analyzer       • Cost Intelligence  • Export Engine │
│ • Dashboard Engine     • Performance Monitor • ML Insights  │
└─────────────────────────────────────────────────────────────┘
                                            │
                                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Data Storage Layer                        │
├─────────────────────────────────────────────────────────────┤
│ • SQLite (Local)       • Cloudflare D1 (Sync)              │
│ • Analytics Tables     • Time-series Data                   │
│ • Aggregated Metrics   • Historical Archives               │
└─────────────────────────────────────────────────────────────┘
```

## 🏃 1-Day Sprint Plan

### Morning Session (4 hours): Core Analytics Engine
**Goal**: Port essential analytics functionality from TypeScript

#### 9:00 AM - 11:00 AM: Analytics Engine Core
1. **Port Metrics Collection**
   ```rust
   // Focus on essential metrics only
   pub struct CoreMetrics {
       total_queries: u64,
       success_rate: f64,
       avg_response_time: Duration,
       total_cost: f64,
       tokens_used: u64,
   }
   ```

2. **Implement Basic Aggregation**
   - Hourly summaries
   - Daily rollups
   - Cost calculations
   - Performance metrics

3. **Connect to Existing Database**
   - Use existing analytics tables
   - Add basic queries
   - Implement caching layer

#### 11:00 AM - 1:00 PM: TUI Reports Panel Structure
1. **Create Reports Panel Module**
   ```rust
   // src/tui/advanced/reports.rs
   pub struct ReportsPanel {
       analytics: Arc<AnalyticsEngine>,
       current_view: ReportView,
       data_cache: DashboardData,
       last_refresh: Instant,
   }
   
   pub enum ReportView {
       Executive,    // KPIs and trends
       CostAnalysis, // Cost breakdown
       Performance,  // Model performance
   }
   ```

2. **Add to TUI Layout**
   - Add Reports as 5th panel type
   - Implement F5 keybinding
   - Basic panel switching

### Afternoon Session (4 hours): Views & Integration
**Goal**: Create functional dashboard views with real data

#### 1:00 PM - 3:00 PM: Implement Dashboard Views
1. **Executive Dashboard**
   ```
   ╭─ Executive Dashboard ──────────────────────╮
   │ Total Queries: 1,234    Success: 98.5%     │
   │ Avg Response: 245ms     Total Cost: $12.34 │
   │                                            │
   │ 7-Day Trend:                               │
   │ ▁▂▃▅▆▇█  +15% from last week              │
   │                                            │
   │ Top Models:                                │
   │ [████████░░] GPT-4: 45%                   │
   │ [██████░░░░] Claude: 35%                   │
   │ [████░░░░░░] Gemini: 20%                   │
   ╰────────────────────────────────────────────╯
   ```

2. **Cost Analysis View**
   ```
   ╭─ Cost Analysis ────────────────────────────╮
   │ Today: $4.56  │  This Week: $28.90        │
   │ Budget Used: [████████░░] 81%              │
   │                                            │
   │ By Provider:                               │
   │ OpenAI:     $15.23 (52%)                   │
   │ Anthropic:  $10.45 (36%)                   │
   │ Google:     $3.22  (12%)                   │
   │                                            │
   │ Cost Optimization:                         │
   │ • Switch to Claude-Instant: Save $5/day    │
   │ • Batch similar queries: Save 30%          │
   ╰────────────────────────────────────────────╯
   ```

3. **Performance View**
   ```
   ╭─ Performance Metrics ───────────────────────╮
   │ Consensus Pipeline:                        │
   │ Generator:  [████] 125ms                   │
   │ Refiner:    [██████] 189ms                 │
   │ Validator:  [███] 98ms                     │
   │ Curator:    [████] 134ms                   │
   │                                            │
   │ Model Leaderboard:                         │
   │ 1. GPT-4-Turbo    Score: 94.2  ⚡23ms     │
   │ 2. Claude-3-Opus  Score: 93.8  ⚡45ms     │
   │ 3. Gemini-Pro     Score: 91.5  ⚡18ms     │
   ╰────────────────────────────────────────────╯
   ```

#### 3:00 PM - 5:00 PM: Real-time Integration & Export
1. **Connect to Live Data**
   - Hook into consensus pipeline events
   - Update metrics in real-time
   - Auto-refresh every 10 seconds

2. **Basic Export Feature**
   ```rust
   impl ReportsPanel {
       fn export_current_view(&self) -> Result<()> {
           let data = match self.current_view {
               ReportView::Executive => self.export_executive_json(),
               ReportView::CostAnalysis => self.export_cost_csv(),
               ReportView::Performance => self.export_performance_json(),
           };
           // Save to file with timestamp
       }
   }
   ```

3. **Keyboard Controls**
   - Tab: Switch between views
   - R: Refresh data
   - E: Export current view
   - Q: Close Reports panel

### Evening Session (2 hours): Testing & Polish
**Goal**: Ensure everything works end-to-end

#### 5:00 PM - 7:00 PM: Integration Testing
1. **Test Data Flow**
   - Run consensus operations
   - Verify metrics collection
   - Check TUI updates
   - Test export functionality

2. **Polish UI**
   - Add colors and styling
   - Improve chart rendering
   - Add loading indicators
   - Handle edge cases

3. **Quick Documentation**
   - Update keybindings help
   - Add Reports panel to README
   - Document new CLI commands

## 📁 Implementation Details

### 1. Database Schema Migration

```sql
-- Performance metrics with time-series optimization
CREATE TABLE performance_metrics_v2 (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    metric_type TEXT NOT NULL,
    value REAL NOT NULL,
    dimensions TEXT, -- JSON
    conversation_id TEXT,
    INDEX idx_timestamp (timestamp),
    INDEX idx_metric_type (metric_type)
) PARTITION BY RANGE (timestamp);

-- Cost analytics with provider breakdown
CREATE TABLE cost_analytics_v2 (
    id INTEGER PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    stage TEXT NOT NULL,
    tokens_used INTEGER,
    cost_usd REAL,
    conversation_id TEXT,
    INDEX idx_provider_model (provider, model)
);

-- Business KPIs aggregation
CREATE TABLE business_kpis (
    date DATE PRIMARY KEY,
    total_queries INTEGER,
    success_rate REAL,
    avg_quality_score REAL,
    cost_per_query REAL,
    time_to_first_token REAL,
    customer_satisfaction REAL,
    consensus_effectiveness REAL,
    resource_utilization REAL
);
```

### 2. TUI Reports Panel Structure

```rust
// src/tui/advanced/reports/mod.rs
pub mod views;
pub mod charts;
pub mod widgets;
pub mod interactions;

pub struct ReportsPanel {
    state: ReportsPanelState,
    analytics: Arc<AnalyticsEngine>,
    layout: ReportsLayout,
    refresh_timer: Instant,
}

pub enum ReportView {
    Executive,
    Operational,
    CostAnalysis,
    Performance,
    Quality,
    Custom(String),
}

impl ReportsPanel {
    pub fn new(analytics: Arc<AnalyticsEngine>) -> Self {
        Self {
            state: ReportsPanelState::default(),
            analytics,
            layout: ReportsLayout::default(),
            refresh_timer: Instant::now(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        match self.state.current_view {
            ReportView::Executive => self.render_executive_dashboard(frame, area),
            ReportView::Operational => self.render_operational_view(frame, area),
            ReportView::CostAnalysis => self.render_cost_analysis(frame, area),
            ReportView::Performance => self.render_performance_view(frame, area),
            ReportView::Quality => self.render_quality_view(frame, area),
            ReportView::Custom(ref name) => self.render_custom_report(frame, area, name),
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Tab => self.next_view(),
            KeyCode::BackTab => self.previous_view(),
            KeyCode::Enter => self.drill_down(),
            KeyCode::Esc => self.go_back(),
            KeyCode::Char('e') => self.export_current(),
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Up | KeyCode::Down => self.navigate_vertical(key.code),
            KeyCode::Left | KeyCode::Right => self.navigate_horizontal(key.code),
            _ => {}
        }
        Ok(())
    }
}
```

### 3. Real-time Analytics Stream

```rust
// src/analytics/streaming.rs
pub struct AnalyticsStream {
    subscribers: Arc<RwLock<Vec<mpsc::Sender<AnalyticsEvent>>>>,
    buffer: Arc<RwLock<CircularBuffer<AnalyticsEvent>>>,
}

impl AnalyticsStream {
    pub async fn publish(&self, event: AnalyticsEvent) {
        // Buffer for replay
        self.buffer.write().await.push(event.clone());
        
        // Notify all subscribers
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.iter() {
            let _ = subscriber.send(event.clone()).await;
        }
    }

    pub async fn subscribe(&self) -> mpsc::Receiver<AnalyticsEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.subscribers.write().await.push(tx);
        
        // Send buffered events to new subscriber
        let buffer = self.buffer.read().await;
        for event in buffer.iter() {
            let _ = tx.send(event.clone()).await;
        }
        
        rx
    }
}
```

### 4. Export System Enhancement

```rust
// src/analytics/export/mod.rs
pub trait ExportFormat {
    fn export(&self, data: &AnalyticsData) -> Result<Vec<u8>>;
    fn mime_type(&self) -> &'static str;
    fn extension(&self) -> &'static str;
}

pub struct PdfExporter {
    template_engine: TemplateEngine,
    chart_renderer: ChartRenderer,
}

impl ExportFormat for PdfExporter {
    fn export(&self, data: &AnalyticsData) -> Result<Vec<u8>> {
        let mut pdf = PdfDocument::new();
        
        // Add header
        pdf.add_page()
            .set_font("Helvetica-Bold", 24)
            .text(50, 750, "Analytics Report");
        
        // Render charts
        for chart in data.charts() {
            let image = self.chart_renderer.render_to_image(chart)?;
            pdf.add_image(image);
        }
        
        // Add data tables
        for table in data.tables() {
            pdf.add_table(table);
        }
        
        Ok(pdf.to_bytes()?)
    }
    
    fn mime_type(&self) -> &'static str {
        "application/pdf"
    }
    
    fn extension(&self) -> &'static str {
        "pdf"
    }
}
```

## 🧪 Testing Strategy

### Unit Tests
- Test each analytics calculation
- Verify data aggregation logic
- Test export formats
- Validate chart rendering

### Integration Tests
- End-to-end analytics pipeline
- TUI interaction testing
- Export functionality
- Real-time streaming

### Performance Tests
- Benchmark query performance
- Memory usage under load
- TUI rendering performance
- Export generation speed

### Acceptance Tests
- Feature parity verification
- User workflow testing
- Visual regression testing
- Cross-platform compatibility

## 📈 Success Metrics

1. **Performance Targets**
   - Query response time: <50ms (10x improvement)
   - Memory usage: <50MB for analytics
   - TUI refresh rate: 60 FPS
   - Export generation: <2s for any format

2. **Feature Completeness**
   - 100% of advertised features implemented
   - All TypeScript features ported
   - Enhanced with Rust-specific optimizations
   - New TUI-exclusive features

3. **User Experience**
   - Intuitive TUI navigation
   - Real-time updates without lag
   - Professional report quality
   - Seamless data migration

## 🚀 Deliverables by End of Day

### Core Features (Must Have)
1. ✅ Working Reports panel in TUI (F5 to access)
2. ✅ Three functional views: Executive, Cost Analysis, Performance
3. ✅ Real-time data from consensus operations
4. ✅ Basic export (JSON/CSV)
5. ✅ Auto-refresh functionality

### Nice to Have (If Time Permits)
- Additional chart types
- Historical data comparison
- More export formats
- Custom time range selection

### Known Limitations (Future Work)
- Advanced ML insights
- PDF/Excel export
- Custom report builder
- Enterprise integrations
- Full feature parity (remaining 9 analytics categories)

## 📝 Technical Decisions

1. **Use Ratatui for TUI**: Already integrated, perfect for Reports panel
2. **SQLite for Analytics**: Fast, embedded, perfect for time-series
3. **Async Streams**: For real-time data flow
4. **SIMD for Calculations**: Leverage Rust's performance capabilities
5. **Template Engine**: Use Tera for flexible report templates

## 🎯 Implementation Order (Today)

1. **Start with Analytics Engine** (Morning)
   - Port core metrics from TypeScript
   - Set up data aggregation
   - Connect to existing database

2. **Build TUI Reports Panel** (Late Morning)
   - Create panel structure
   - Add to TUI layout system
   - Implement panel switching

3. **Implement Dashboard Views** (Afternoon)
   - Executive Dashboard first (most important)
   - Cost Analysis second
   - Performance Metrics third

4. **Connect Real-time Data** (Late Afternoon)
   - Hook into consensus events
   - Implement auto-refresh
   - Add export functionality

5. **Test & Polish** (Evening)
   - End-to-end testing
   - UI improvements
   - Quick documentation

## 📋 Success Criteria

By end of day, users should be able to:
1. Press F5 in the TUI to open Reports panel
2. View real-time analytics from their consensus operations
3. Switch between Executive, Cost, and Performance views
4. Export current view data as JSON or CSV
5. See auto-updating metrics every 10 seconds

This focused 1-day sprint delivers immediate value with a working analytics system in the TUI, setting the foundation for future enhancements.