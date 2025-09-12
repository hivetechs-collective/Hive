// Analytics Dashboard Module
// Comprehensive analytics and reporting system for Hive Consensus

interface AnalyticsData {
  // Today's metrics
  todayQueries: number;
  todayCost: number;
  todayAvgResponseTime: number;
  todayTokenUsage: {
    total: number;
    input: number;
    output: number;
  };
  
  // All-time metrics
  totalQueries: number;
  totalCost: number;
  successRate: number;
  avgResponseTime: number;
  modelUsage: { [model: string]: number };
  recentActivity: Activity[];
  hourlyStats: HourlyStat[];
  costByModel: { [model: string]: number };
  tokenUsage: {
    total: number;
    input: number;
    output: number;
  };
  alerts: Alert[];
}

interface Activity {
  timestamp: string;
  question?: string;
  model: string;
  cost: number;
  duration: number;
  status: 'completed' | 'failed' | 'timeout';
  tokens: number;
  conversationId?: string;
}

interface HourlyStat {
  hour: string;
  queries: number;
  cost: number;
  avgTime: number;
}

interface Alert {
  type: 'warning' | 'error' | 'info';
  message: string;
  timestamp: string;
}

export class AnalyticsDashboard {
  private container: HTMLElement | null = null;
  private data: AnalyticsData | null = null;
  private updateInterval: NodeJS.Timeout | null = null;
  private chartUpdateInterval: NodeJS.Timeout | null = null;
  private period: '24h' | '7d' | '30d' = '24h';

  constructor() {
    this.initializeData();
  }

  private initializeData(): void {
    // Initialize with default data structure
    this.data = {
      todayQueries: 0,
      todayCost: 0,
      todayAvgResponseTime: 0,
      todayTokenUsage: {
        total: 0,
        input: 0,
        output: 0
      },
      totalQueries: 0,
      totalCost: 0,
      successRate: 0,
      avgResponseTime: 0,
      modelUsage: {},
      recentActivity: [],
      hourlyStats: [],
      costByModel: {},
      tokenUsage: {
        total: 0,
        input: 0,
        output: 0
      },
      alerts: []
    };
  }

  public mount(container: HTMLElement): void {
    this.container = container;
    this.render();
    this.startDataUpdates();
  }

  public unmount(): void {
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
    if (this.chartUpdateInterval) {
      clearInterval(this.chartUpdateInterval);
      this.chartUpdateInterval = null;
    }
    if (this.container) {
      this.container.innerHTML = '';
      this.container = null;
    }
  }

  private async fetchAnalyticsData(): Promise<void> {
    try {
      console.log('Fetching analytics data...');
      
      // Skip localStorage - go straight to database
      // Try to fetch from Electron API if available
      const electronAPI = (window as any).electronAPI;
      console.log('ElectronAPI available:', !!electronAPI);
      
      if (electronAPI && electronAPI.getAnalytics) {
        console.log('Calling getAnalytics...');
        const analyticsData = await electronAPI.getAnalytics(this.period);
        console.log('Analytics data received:', analyticsData);
        console.log('Recent activity sample:', analyticsData?.recentActivity?.slice(0, 2));
        
        if (analyticsData) {
          this.data = analyticsData;
          this.updateDashboard();
          return;
        }
      }

      // Initialize with empty data if no data available
      console.log('No analytics data, initializing empty');
      this.initializeEmptyData();
      this.updateDashboard();
    } catch (error) {
      console.error('Failed to fetch analytics data:', error);
      this.initializeEmptyData();
      this.updateDashboard();
    }
  }

  private processConsensusMetrics(metrics: any): AnalyticsData {
    // Process stored consensus metrics into analytics format
    const data: AnalyticsData = {
      todayQueries: metrics.todayQueries || 0,
      todayCost: metrics.todayCost || 0,
      todayAvgResponseTime: metrics.todayAvgResponseTime || 0,
      todayTokenUsage: metrics.todayTokenUsage || {
        total: 0,
        input: 0,
        output: 0
      },
      totalQueries: metrics.totalQueries || 0,
      totalCost: metrics.totalCost || 0,
      successRate: metrics.successRate || 100,
      avgResponseTime: metrics.avgResponseTime || 0,
      modelUsage: metrics.modelUsage || {},
      recentActivity: metrics.recentActivity || [],
      hourlyStats: metrics.hourlyStats || [],
      costByModel: metrics.costByModel || {},
      tokenUsage: metrics.tokenUsage || {
        total: 0,
        input: 0,
        output: 0
      },
      alerts: []
    };

    // Add system status alert
    if (data.totalQueries > 0) {
      data.alerts.push({
        type: 'info',
        message: `Consensus pipeline analytics active - ${data.totalQueries} queries processed`,
        timestamp: new Date().toISOString()
      });
    }

    return data;
  }

  private initializeEmptyData(): void {
    // Initialize with empty data structure instead of mock
    this.data = {
      todayQueries: 0,
      todayCost: 0,
      todayAvgResponseTime: 0,
      todayTokenUsage: {
        total: 0,
        input: 0,
        output: 0
      },
      totalQueries: 0,
      totalCost: 0,
      successRate: 100,
      avgResponseTime: 0,
      modelUsage: {},
      recentActivity: [],
      hourlyStats: this.generateEmptyHourlyStats(),
      costByModel: {},
      tokenUsage: {
        total: 0,
        input: 0,
        output: 0
      },
      alerts: [{
        type: 'info',
        message: 'Analytics system ready - waiting for consensus data',
        timestamp: new Date().toISOString()
      }]
    };
  }

  private generateEmptyHourlyStats(): HourlyStat[] {
    const stats: HourlyStat[] = [];
    const now = new Date();
    for (let i = 23; i >= 0; i--) {
      const hour = new Date(now.getTime() - i * 60 * 60 * 1000);
      const hourStr = hour.getHours().toString().padStart(2, '0') + ':00';
      stats.push({
        hour: hourStr,
        queries: 0,
        cost: 0,
        avgTime: 0
      });
    }
    return stats;
  }


  private render(): void {
    if (!this.container) return;

    this.container.innerHTML = `
      <div class="analytics-dashboard">
        <div class="analytics-header">
          <h2 class="analytics-title">
            <span class="icon icon-graph"></span>
            <span>Analytics Dashboard</span>
          </h2>
          <div class="analytics-controls">
            <select class="period-selector">
              <option value="24h">Last 24 Hours</option>
              <option value="7d">Last 7 Days</option>
              <option value="30d">Last 30 Days</option>
            </select>
            <button class="refresh-btn" title="Refresh">🔄</button>
          </div>
        </div>

        <!-- Key Metrics Row -->
        <div class="metrics-grid">
          <div class="metric-card">
              <div class="metric-icon"><span class="icon icon-calendar"></span></div>
            <div class="metric-content">
              <div class="metric-value" id="today-queries">0</div>
              <div class="metric-label" id="period-queries-label">Period Queries</div>
            </div>
          </div>
          
          <div class="metric-card">
              <div class="metric-icon"><span class="icon icon-dollar"></span></div>
            <div class="metric-content">
              <div class="metric-value" id="today-cost">$0.00</div>
              <div class="metric-label" id="period-cost-label">Period Cost</div>
            </div>
          </div>
          
          <div class="metric-card">
              <div class="metric-icon"><span class="icon icon-activity"></span></div>
            <div class="metric-content">
              <div class="metric-value" id="total-queries">0</div>
              <div class="metric-label">All-Time Queries</div>
            </div>
          </div>
          
          <div class="metric-card">
              <div class="metric-icon"><span class="icon icon-balance"></span></div>
            <div class="metric-content">
              <div class="metric-value" id="total-cost">$0.00</div>
              <div class="metric-label">Total Cost</div>
            </div>
          </div>
          
          <div class="metric-card">
              <div class="metric-icon"><span class="icon icon-check"></span></div>
            <div class="metric-content">
              <div class="metric-value" id="success-rate">0%</div>
              <div class="metric-label">Success Rate</div>
            </div>
          </div>
          
          <div class="metric-card">
              <div class="metric-icon"><span class="icon icon-bolt"></span></div>
            <div class="metric-content">
              <div class="metric-value" id="avg-response">0s</div>
              <div class="metric-label">Avg Response</div>
            </div>
          </div>
        </div>

        <!-- Charts Row -->
        <div class="charts-row">
          <div class="chart-container">
            <h3 id="volume-title">Query Volume</h3>
            <div class="line-chart" id="volume-chart">
              <canvas id="volume-canvas"></canvas>
            </div>
          </div>
          
          <div class="chart-container">
            <h3>Model Usage</h3>
            <div class="model-usage-list" id="model-usage-list">
              <!-- Will be populated with model bars -->
            </div>
          </div>
        </div>

        <!-- Token Usage and Cost Breakdown -->
        <div class="charts-row">
          <div class="chart-container">
            <h3>Token Usage</h3>
            <div class="token-stats">
              <div class="token-stat">
                <span class="token-label">Total:</span>
                <span class="token-value" id="total-tokens">0</span>
              </div>
              <div class="token-stat">
                <span class="token-label">Input:</span>
                <span class="token-value" id="input-tokens">0</span>
              </div>
              <div class="token-stat">
                <span class="token-label">Output:</span>
                <span class="token-value" id="output-tokens">0</span>
              </div>
            </div>
            <div class="gauge-chart" id="token-gauge">
              <canvas id="gauge-canvas"></canvas>
            </div>
          </div>
          
          <div class="chart-container">
            <h3>Cost by Model</h3>
            <div class="cost-breakdown" id="cost-breakdown"></div>
          </div>
        </div>

        <!-- Recent Activity Table -->
        <div class="activity-section">
          <h3>Recent Queries</h3>
          <div class="activity-table-wrapper">
            <table class="activity-table">
              <thead>
                <tr>
                  <th>Time</th>
                  <th style="width: 40%;">Question</th>
                  <th>Duration</th>
                  <th>Tokens</th>
                  <th>Cost</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody id="activity-tbody">
                <!-- Dynamic content -->
              </tbody>
            </table>
          </div>
        </div>

        <!-- Alerts Section -->
        <div class="alerts-section" id="alerts-section">
          <!-- Dynamic alerts -->
        </div>
      </div>
    `;

    this.attachEventListeners();
    // Set initial titles/labels for default period
    const volTitle = this.container?.querySelector('#volume-title');
    if (volTitle) (volTitle as HTMLElement).textContent = `Query Volume (${this.period.toUpperCase()})`;
    const pq = this.container?.querySelector('#period-queries-label');
    if (pq) (pq as HTMLElement).textContent = 'Period Queries';
    const pc = this.container?.querySelector('#period-cost-label');
    if (pc) (pc as HTMLElement).textContent = 'Period Cost';
    this.fetchAnalyticsData();
  }

  private attachEventListeners(): void {
    const refreshBtn = this.container?.querySelector('.refresh-btn') as HTMLElement | null;
    if (refreshBtn) {
      refreshBtn.addEventListener('click', () => {
        this.fetchAnalyticsData();
        this.animateRefresh(refreshBtn);
      });
    }

    const periodSelector = this.container?.querySelector('.period-selector') as HTMLSelectElement | null;
    if (periodSelector) {
      periodSelector.addEventListener('change', () => {
        const val = periodSelector.value as any;
        if (val === '24h' || val === '7d' || val === '30d') {
          this.period = val;
        } else {
          this.period = '24h';
        }
        const volTitle = this.container?.querySelector('#volume-title');
        if (volTitle) volTitle.textContent = `Query Volume (${this.period.toUpperCase()})`;
        const pq = this.container?.querySelector('#period-queries-label');
        if (pq) pq.textContent = 'Period Queries';
        const pc = this.container?.querySelector('#period-cost-label');
        if (pc) pc.textContent = 'Period Cost';
        this.fetchAnalyticsData();
      });
    }
  }

  private animateRefresh(button: HTMLElement): void {
    button.style.animation = 'spin 1s ease-in-out';
    setTimeout(() => {
      button.style.animation = '';
    }, 1000);
  }

  private updateDashboard(): void {
    if (!this.data || !this.container) return;

    // Update metrics
    this.updateMetrics();
    
    // Update charts
    this.updateVolumeChart();
    this.updateModelChart();
    this.updateTokenGauge();
    this.updateCostBreakdown();
    
    // Update activity table
    this.updateActivityTable();
    
    // Update alerts
    this.updateAlerts();
  }

  private updateMetrics(): void {
    if (!this.data) return;

    const updateMetric = (id: string, value: string) => {
      const element = this.container?.querySelector(`#${id}`);
      if (element) {
        element.textContent = value;
        element.classList.add('metric-update');
        setTimeout(() => element.classList.remove('metric-update'), 300);
      }
    };

    // Update today's metrics
    updateMetric('today-queries', this.data.todayQueries.toLocaleString());
    // Show more precision for small costs (< $0.10)
    const todayCostFormatted = this.data.todayCost < 0.10 && this.data.todayCost > 0 
      ? `$${this.data.todayCost.toFixed(4)}` 
      : `$${this.data.todayCost.toFixed(2)}`;
    updateMetric('today-cost', todayCostFormatted);
    
    // Update all-time metrics
    updateMetric('total-queries', this.data.totalQueries.toLocaleString());
    updateMetric('total-cost', `$${this.data.totalCost.toFixed(2)}`);
    updateMetric('success-rate', `${this.data.successRate.toFixed(1)}%`);
    updateMetric('avg-response', `${this.data.avgResponseTime.toFixed(1)}s`);
    
    // Update token usage (today's)
    updateMetric('total-tokens', this.formatNumber(this.data.todayTokenUsage.total));
    updateMetric('input-tokens', this.formatNumber(this.data.todayTokenUsage.input));
    updateMetric('output-tokens', this.formatNumber(this.data.todayTokenUsage.output));
  }

  private updateVolumeChart(): void {
    const canvas = this.container?.querySelector('#volume-canvas') as HTMLCanvasElement;
    if (!canvas || !this.data) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    canvas.width = canvas.offsetWidth;
    canvas.height = 150;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw line chart
    const stats = this.data.hourlyStats;
    const maxQueries = Math.max(...stats.map(s => s.queries), 1); // Ensure at least 1 to avoid division by zero
    const padding = 30; // Increased for labels
    const width = canvas.width - padding * 2;
    const height = canvas.height - padding * 2;

    // Draw grid lines and Y-axis labels
    ctx.strokeStyle = 'rgba(100, 200, 255, 0.1)';
    ctx.lineWidth = 1;
    ctx.font = '10px monospace';
    ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
    ctx.textAlign = 'right';
    
    for (let i = 0; i <= 4; i++) {
      const y = padding + (height / 4) * i;
      const value = Math.round(maxQueries * (1 - i / 4));
      
      // Draw grid line
      ctx.beginPath();
      ctx.moveTo(padding, y);
      ctx.lineTo(canvas.width - padding, y);
      ctx.stroke();
      
      // Draw Y-axis label
      ctx.fillText(value.toString(), padding - 5, y + 3);
    }

    // Draw line
    ctx.strokeStyle = '#64c8ff';
    ctx.lineWidth = 2;
    ctx.beginPath();
    
    stats.forEach((stat, index) => {
      const x = padding + (width / (stats.length - 1)) * index;
      const y = padding + height - (stat.queries / maxQueries) * height;
      
      if (index === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    });
    
    ctx.stroke();

    // Draw points
    ctx.fillStyle = '#64c8ff';
    stats.forEach((stat, index) => {
      const x = padding + (width / (stats.length - 1)) * index;
      const y = padding + height - (stat.queries / maxQueries) * height;
      
      ctx.beginPath();
      ctx.arc(x, y, 3, 0, Math.PI * 2);
      ctx.fill();
    });
    
    // Draw X-axis labels (hours)
    ctx.fillStyle = 'rgba(255, 255, 255, 0.5)';
    ctx.font = '9px monospace';
    ctx.textAlign = 'center';
    
    // Show every 4th hour to avoid crowding
    stats.forEach((stat, index) => {
      if (index % 4 === 0 || index === stats.length - 1) {
        const x = padding + (width / (stats.length - 1)) * index;
        ctx.fillText(stat.hour, x, canvas.height - 5);
      }
    });
  }

  private updateModelChart(): void {
    const container = this.container?.querySelector('#model-usage-list');
    if (!container || !this.data) return;

    // Sort models by usage count
    const models = Object.entries(this.data.modelUsage)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 10); // Show top 10 models
    
    if (models.length === 0) {
      container.innerHTML = '<div class="no-data">No model usage data available</div>';
      return;
    }

    const maxUsage = Math.max(...models.map(([_, count]) => count));
    
    let html = '';
    models.forEach(([model, count]) => {
      const percentage = (count / maxUsage) * 100;
      const cost = this.data?.costByModel[model] || 0;
      
      // Simplify model name - remove provider prefix if present
      const displayName = model.includes('/') ? 
        model.split('/').pop() : model;
      
      html += `
        <div class="model-usage-item">
          <div class="model-info">
            <span class="model-name">${displayName}</span>
            <span class="model-stats">${count} uses • $${cost.toFixed(2)}</span>
          </div>
          <div class="model-bar-wrapper">
            <div class="model-bar" style="width: ${percentage}%">
              <div class="model-bar-fill"></div>
            </div>
          </div>
        </div>
      `;
    });
    
    container.innerHTML = html;
  }

  private updateTokenGauge(): void {
    const canvas = this.container?.querySelector('#gauge-canvas') as HTMLCanvasElement;
    if (!canvas || !this.data) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    canvas.width = 200;
    canvas.height = 100;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw gauge
    const centerX = canvas.width / 2;
    const centerY = canvas.height - 20;
    const radius = 70;
    
    // Calculate percentage (assuming 1M tokens is 100%)
    const percentage = Math.min((this.data.tokenUsage.total / 1000000) * 100, 100);
    const angle = (percentage / 100) * Math.PI; // 0 to PI (half circle)

    // Draw background arc
    ctx.strokeStyle = 'rgba(100, 200, 255, 0.2)';
    ctx.lineWidth = 15;
    ctx.beginPath();
    ctx.arc(centerX, centerY, radius, Math.PI, 0);
    ctx.stroke();

    // Draw filled arc
    const gradient = ctx.createLinearGradient(centerX - radius, centerY, centerX + radius, centerY);
    gradient.addColorStop(0, '#00ff88');
    gradient.addColorStop(0.5, '#64c8ff');
    gradient.addColorStop(1, '#ff6b6b');
    
    ctx.strokeStyle = gradient;
    ctx.lineWidth = 15;
    ctx.beginPath();
    ctx.arc(centerX, centerY, radius, Math.PI, Math.PI + angle);
    ctx.stroke();

    // Draw percentage text
    ctx.fillStyle = '#ffffff';
    ctx.font = 'bold 20px monospace';
    ctx.textAlign = 'center';
    ctx.fillText(`${percentage.toFixed(0)}%`, centerX, centerY - 10);
  }

  private updateCostBreakdown(): void {
    const container = this.container?.querySelector('#cost-breakdown');
    if (!container || !this.data) return;

    const totalCost = Object.values(this.data.costByModel).reduce((sum, cost) => sum + cost, 0);
    
    if (totalCost === 0) {
      container.innerHTML = '<div class="no-data">No cost data available</div>';
      return;
    }
    
    let html = '<div class="cost-bars">';
    
    Object.entries(this.data.costByModel)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 8) // Show top 8 models
      .forEach(([model, cost]) => {
        const percentage = (cost / totalCost) * 100;
        // Simplify model name - remove provider
        const displayName = model.includes('/') ? 
          model.split('/').pop() : model;
        
        html += `
          <div class="cost-bar-item">
            <div class="cost-bar-label">
              <span class="model-name">${displayName}</span>
              <span class="model-cost">$${cost.toFixed(2)} (${percentage.toFixed(1)}%)</span>
            </div>
            <div class="cost-bar-wrapper">
              <div class="cost-bar" style="width: ${percentage}%"></div>
            </div>
          </div>
        `;
      });
    
    html += '</div>';
    container.innerHTML = html;
  }

  private updateActivityTable(): void {
    const tbody = this.container?.querySelector('#activity-tbody');
    if (!tbody || !this.data) return;

    if (this.data.recentActivity.length === 0) {
      tbody.innerHTML = '<tr><td colspan="6" class="no-data">No recent queries</td></tr>';
      return;
    }

    let html = '';
    
    this.data.recentActivity.forEach(activity => {
      const date = new Date(activity.timestamp);
      const time = date.toLocaleTimeString('en-US', { 
        hour: '2-digit', 
        minute: '2-digit'
      });
      const dateStr = date.toLocaleDateString('en-US', { 
        month: 'short', 
        day: 'numeric' 
      });
      
      const statusClass = activity.status === 'completed' ? 'status-success' : 'status-error';
      const statusIcon = activity.status === 'completed' ? '✅' : '❌';
      
      // Truncate long questions
      const question = activity.question || 'Query';
      const displayQuestion = question.length > 60 ? 
        question.substring(0, 57) + '...' : question;
      
      html += `
        <tr>
          <td class="time-cell">
            <div>${time}</div>
            <div class="date-small">${dateStr}</div>
          </td>
          <td class="question-cell" title="${question.replace(/"/g, '&quot;')}">
            ${displayQuestion}
          </td>
          <td>${activity.duration.toFixed(1)}s</td>
          <td>${activity.tokens.toLocaleString()}</td>
          <td class="cost-cell">$${activity.cost.toFixed(3)}</td>
          <td class="${statusClass}">${statusIcon}</td>
        </tr>
      `;
    });
    
    tbody.innerHTML = html;
  }

  private updateAlerts(): void {
    const container = this.container?.querySelector('#alerts-section');
    if (!container || !this.data || this.data.alerts.length === 0) return;

    let html = '<h3>System Alerts</h3><div class="alerts-list">';
    
    this.data.alerts.forEach(alert => {
      const icon = alert.type === 'error' ? '🔴' : alert.type === 'warning' ? '🟡' : '🔵';
      html += `
        <div class="alert alert-${alert.type}">
          <span class="alert-icon">${icon}</span>
          <span class="alert-message">${alert.message}</span>
          <span class="alert-time">${new Date(alert.timestamp).toLocaleTimeString()}</span>
        </div>
      `;
    });
    
    html += '</div>';
    container.innerHTML = html;
  }

  private formatNumber(num: number): string {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1) + 'M';
    } else if (num >= 1000) {
      return (num / 1000).toFixed(1) + 'K';
    }
    return num.toLocaleString();
  }

  private startDataUpdates(): void {
    // Update data every 5 seconds to fetch real data from database
    this.updateInterval = setInterval(() => {
      this.fetchAnalyticsData();
    }, 5000);

    // No longer auto-increment - only show real data
  }
}

// Export singleton instance
export const analyticsDashboard = new AnalyticsDashboard();
