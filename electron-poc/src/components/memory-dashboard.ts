/**
 * Memory Service Dashboard Component
 * Shows real-time memory activity, connected tools, and statistics
 */

export class MemoryDashboard {
  private container: HTMLElement | null = null;
  private ws: WebSocket | null = null;
  private updateInterval: NodeJS.Timeout | null = null;
  private activityBuffer: any[] = [];
  private memoryServicePort: number | null = null;
  
  constructor() {
    this.initializePort();
  }
  
  private async initializePort() {
    try {
      // Get dynamic port from main process via IPC
      this.memoryServicePort = await (window as any).api.invoke('memory-service-port');
      console.log('[MemoryDashboard] Got memory service port:', this.memoryServicePort);
      this.setupWebSocket();
    } catch (error) {
      console.error('[MemoryDashboard] Failed to get memory service port:', error);
      // Retry in 2 seconds
      setTimeout(() => this.initializePort(), 2000);
    }
  }
  
  private setupWebSocket() {
    if (!this.memoryServicePort) {
      console.error('[MemoryDashboard] No port available, waiting...');
      return;
    }
    
    try {
      this.ws = new WebSocket(`ws://localhost:${this.memoryServicePort}`);
      console.log('[MemoryDashboard] Connecting to WebSocket on port', this.memoryServicePort);
      
      this.ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        this.handleWebSocketMessage(data);
      };
      
      this.ws.onerror = (error) => {
        console.error('[MemoryDashboard] WebSocket error:', error);
      };
      
      this.ws.onclose = () => {
        console.log('[MemoryDashboard] WebSocket closed, reconnecting in 5s...');
        setTimeout(() => this.setupWebSocket(), 5000);
      };
    } catch (error) {
      console.error('[MemoryDashboard] Failed to connect WebSocket:', error);
    }
  }
  
  private handleWebSocketMessage(message: any) {
    if (message.type === 'activity') {
      this.addActivityItem(message.data);
    } else if (message.type === 'stats') {
      this.updateStats(message.data);
    }
  }
  
  private addActivityItem(activity: any) {
    this.activityBuffer.push(activity);
    if (this.activityBuffer.length > 100) {
      this.activityBuffer.shift();
    }
    
    const activityStream = document.getElementById('memory-activity-stream');
    if (activityStream) {
      const activityItem = this.createActivityItem(activity);
      activityStream.insertBefore(activityItem, activityStream.firstChild);
      
      // Keep only last 50 items in DOM
      while (activityStream.children.length > 50) {
        activityStream.removeChild(activityStream.lastChild!);
      }
    }
  }
  
  private createActivityItem(activity: any): HTMLElement {
    const item = document.createElement('div');
    item.className = 'activity-item';
    
    const time = new Date(activity.timestamp).toLocaleTimeString();
    const icon = activity.type === 'query' ? '🔍' : 
                 activity.type === 'contribution' ? '📝' : '📊';
    
    item.innerHTML = `
      <span class="activity-time">${time}</span>
      <span class="activity-icon">${icon}</span>
      <span class="activity-text">${this.formatActivity(activity)}</span>
    `;
    
    return item;
  }
  
  private formatActivity(activity: any): string {
    switch (activity.type) {
      case 'query':
        return `${activity.tool || 'Unknown'} queried: "${activity.query || 'N/A'}"`;
      case 'contribution':
        return `${activity.tool || 'Unknown'} contributed ${activity.category || 'knowledge'}`;
      case 'request':
        return `${activity.client || 'Unknown'} ${activity.method} ${activity.path}`;
      default:
        return JSON.stringify(activity).substring(0, 100);
    }
  }
  
  private async updateStats(stats?: any) {
    if (!stats) {
      stats = await window.electronAPI.getMemoryStats();
    }
    
    // Update stat cards
    document.getElementById('stat-total-memories')!.textContent = stats.totalMemories.toLocaleString();
    document.getElementById('stat-queries-today')!.textContent = stats.queriesToday.toLocaleString();
    document.getElementById('stat-contributions')!.textContent = stats.contributionsToday.toLocaleString();
    document.getElementById('stat-hit-rate')!.textContent = `${stats.hitRate}%`;
    document.getElementById('stat-response-time')!.textContent = `${Math.round(stats.avgResponseTime)}ms`;
    document.getElementById('stat-connected-tools')!.textContent = stats.connectedTools.toString();
  }
  
  private async updateConnectedTools() {
    const tools = await window.electronAPI.getConnectedTools();
    const toolsList = document.getElementById('connected-tools-list');
    
    if (!toolsList) return;
    
    toolsList.innerHTML = tools.map((tool: any) => `
      <div class="tool-item">
        <div class="tool-header">
          <span class="tool-name">${tool.name}</span>
          <span class="tool-status active">Active</span>
        </div>
        <div class="tool-stats">
          <span>Last Active: ${new Date(tool.lastActivity).toLocaleTimeString()}</span>
        </div>
      </div>
    `).join('') || '<div class="no-tools">No tools connected yet</div>';
  }
  
  public async create(): Promise<HTMLElement> {
    const container = document.createElement('div');
    container.className = 'memory-dashboard';
    this.container = container;
    
    // Get initial data
    const stats = await window.electronAPI.getMemoryStats();
    const tools = await window.electronAPI.getConnectedTools();
    const activity = await window.electronAPI.getMemoryActivity(20);
    
    container.innerHTML = `
      <div class="dashboard-header">
        <h2>Universal Memory Infrastructure</h2>
        <div class="dashboard-subtitle">
          Memory-as-a-Service for AI Development Tools
        </div>
      </div>
      
      <div class="dashboard-status">
        <div class="status-item">
          <span class="status-indicator ${stats.totalMemories > 0 ? 'active' : 'inactive'}"></span>
          <span>Memory Service: ${stats.totalMemories > 0 ? 'Active' : 'Starting...'}</span>
        </div>
        <div class="status-item">
          <span>Port: ${this.memoryServicePort || 'Loading...'}</span>
        </div>
        <div class="status-item">
          <span>Database: ~/.hive/hive-ai.db</span>
        </div>
      </div>
      
      <!-- Statistics Grid -->
      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-label">Total Memories</div>
          <div class="stat-value" id="stat-total-memories">${stats.totalMemories.toLocaleString()}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Queries Today</div>
          <div class="stat-value" id="stat-queries-today">${stats.queriesToday.toLocaleString()}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Contributions</div>
          <div class="stat-value" id="stat-contributions">${stats.contributionsToday.toLocaleString()}</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Hit Rate</div>
          <div class="stat-value" id="stat-hit-rate">${stats.hitRate}%</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Avg Response</div>
          <div class="stat-value" id="stat-response-time">${Math.round(stats.avgResponseTime)}ms</div>
        </div>
        <div class="stat-card">
          <div class="stat-label">Connected Tools</div>
          <div class="stat-value" id="stat-connected-tools">${stats.connectedTools}</div>
        </div>
      </div>
      
      <!-- Main Content Grid -->
      <div class="dashboard-content">
        <!-- Connected Tools -->
        <div class="dashboard-section">
          <h3>Connected Tools</h3>
          <div id="connected-tools-list" class="tools-list">
            ${tools.length > 0 ? tools.map((tool: any) => `
              <div class="tool-item">
                <div class="tool-header">
                  <span class="tool-name">${tool.name}</span>
                  <span class="tool-status active">Active</span>
                </div>
                <div class="tool-stats">
                  <span>Last Active: ${new Date(tool.lastActivity).toLocaleTimeString()}</span>
                </div>
              </div>
            `).join('') : '<div class="no-tools">No tools connected yet</div>'}
          </div>
        </div>
        
        <!-- Live Activity Stream -->
        <div class="dashboard-section">
          <h3>Live Activity Stream</h3>
          <div id="memory-activity-stream" class="activity-stream">
            ${activity.map((item: any) => `
              <div class="activity-item">
                <span class="activity-time">${new Date(item.timestamp).toLocaleTimeString()}</span>
                <span class="activity-icon">${
                  item.type === 'query' ? '🔍' : 
                  item.type === 'contribution' ? '📝' : '📊'
                }</span>
                <span class="activity-text">${this.formatActivity(item)}</span>
              </div>
            `).join('')}
          </div>
        </div>
      </div>
      
      <!-- Actions removed: Export/Import are managed in Settings → Advanced (backup/restore) -->
    `;
    
    // No global exposure required
    
    // Start periodic updates
    this.startUpdates();
    
    // Apply styles
    this.applyStyles();
    
    return container;
  }
  
  private startUpdates() {
    // Update every 5 seconds
    this.updateInterval = setInterval(async () => {
      await this.updateStats();
      await this.updateConnectedTools();
    }, 5000);
  }
  
  public destroy() {
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
    
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    
    this.container = null;
    delete (window as any).memoryDashboard;
  }
  
  // Export/Import functionality removed
  
  private applyStyles() {
    // Check if styles already exist
    if (document.getElementById('memory-dashboard-styles')) return;
    
    const style = document.createElement('style');
    style.id = 'memory-dashboard-styles';
    style.textContent = `
      .memory-dashboard {
        padding: 20px;
        height: 100%;
        overflow-y: auto;
        background: #1e1e1e;
        color: #cccccc;
      }
      
      .dashboard-header {
        margin-bottom: 20px;
      }
      
      .dashboard-header h2 {
        margin: 0 0 8px 0;
        color: #ffffff;
      }
      
      .dashboard-subtitle {
        color: #999;
        font-size: 14px;
      }
      
      .dashboard-status {
        display: flex;
        gap: 20px;
        padding: 12px;
        background: #252526;
        border-radius: 4px;
        margin-bottom: 20px;
      }
      
      .status-item {
        display: flex;
        align-items: center;
        gap: 8px;
      }
      
      .status-indicator {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: #666;
      }
      
      .status-indicator.active {
        background: #4ec9b0;
        animation: pulse 2s infinite;
      }
      
      .status-indicator.inactive {
        background: #f48771;
      }
      
      @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.5; }
      }
      
      .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
        gap: 12px;
        margin-bottom: 20px;
      }
      
      .stat-card {
        background: #252526;
        padding: 16px;
        border-radius: 4px;
        border: 1px solid #333;
      }
      
      .stat-label {
        font-size: 12px;
        color: #999;
        margin-bottom: 4px;
      }
      
      .stat-value {
        font-size: 24px;
        font-weight: bold;
        color: #4ec9b0;
      }
      
      .dashboard-content {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 20px;
        margin-bottom: 20px;
      }
      
      .dashboard-section {
        background: #252526;
        padding: 16px;
        border-radius: 4px;
      }
      
      .dashboard-section h3 {
        margin: 0 0 12px 0;
        color: #ffffff;
        font-size: 16px;
      }
      
      .tools-list {
        max-height: 300px;
        overflow-y: auto;
      }
      
      .tool-item {
        padding: 10px 12px;
        margin-bottom: 6px;
        background: #1e1e1e;
        border-radius: 4px;
        border: 1px solid #333;
        transition: background-color 0.15s;
      }

      .tool-item:hover {
        background: #252526;
      }
      
      .tool-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 4px;
      }
      
      .tool-name {
        font-weight: 500;
        color: #ffffff;
        font-size: 14px;
      }
      
      .tool-status {
        font-size: 11px;
        padding: 2px 6px;
        border-radius: 3px;
        background: #333;
        text-transform: uppercase;
        letter-spacing: 0.5px;
      }
      
      .tool-status.active {
        background: #2d5a2d;
        color: #4ec9b0;
      }
      
      .tool-stats {
        font-size: 11px;
        color: #8b8b8b;
      }
      
      .activity-stream {
        max-height: 300px;
        overflow-y: auto;
        font-size: 12px;
      }
      
      .activity-item {
        padding: 8px;
        margin-bottom: 4px;
        background: #1e1e1e;
        border-radius: 3px;
        display: flex;
        gap: 8px;
        align-items: center;
      }
      
      .activity-time {
        color: #666;
        min-width: 70px;
      }
      
      .activity-icon {
        font-size: 14px;
      }
      
      .activity-text {
        color: #ccc;
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }
      
      .integration-guide {
        background: #252526;
        padding: 16px;
        border-radius: 4px;
        margin-bottom: 20px;
      }
      
      .integration-guide h3 {
        margin: 0 0 12px 0;
        color: #ffffff;
        font-size: 16px;
      }
      
      .integration-step {
        margin-bottom: 12px;
      }
      
      .integration-step code {
        display: block;
        margin-top: 4px;
        padding: 8px;
        background: #1e1e1e;
        border-radius: 3px;
        font-size: 11px;
        color: #ce9178;
        overflow-x: auto;
      }
      
      .dashboard-actions {
        display: flex;
        gap: 12px;
      }
      
      .btn {
        padding: 8px 16px;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 14px;
      }
      
      .btn-primary {
        background: #007acc;
        color: white;
      }
      
      .btn-primary:hover {
        background: #005a9e;
      }
      
      .btn-secondary {
        background: #333;
        color: #ccc;
      }
      
      .btn-secondary:hover {
        background: #444;
      }
      
      .no-tools {
        padding: 20px;
        text-align: center;
        color: #666;
      }
    `;
    
    document.head.appendChild(style);
  }
}

// Export for use in renderer
export async function createMemoryDashboard(): Promise<HTMLElement> {
  const dashboard = new MemoryDashboard();
  return await dashboard.create();
}
