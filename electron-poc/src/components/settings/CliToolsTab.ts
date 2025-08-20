/**
 * CLI Tools Tab Component
 * Main container for managing agentic coding CLI tools
 */

import { CliToolCard } from './CliToolCard';
import { AdvancedOptions } from './AdvancedOptions';

export interface CliTool {
  id: string;
  name: string;
  icon: string;
  description: string;
  package?: string;
  installCommand: string;
  memoryServiceIntegration: boolean;
}

export interface ToolStatus {
  installed: boolean;
  version?: string;
  lastChecked?: Date;
  updateAvailable?: boolean;
  authenticated?: boolean;
  installProgress?: number;
  installMessage?: string;
  error?: string;
}

export class CliToolsTab {
  private container: HTMLElement;
  private tools: Map<string, CliTool>;
  private statuses: Map<string, ToolStatus>;
  private selectedTools: Set<string> = new Set();
  private filterStatus: 'all' | 'installed' | 'available' | 'updates' = 'all';
  
  constructor(container: HTMLElement) {
    this.container = container;
    this.tools = new Map();
    this.statuses = new Map();
    this.initializeTools();
    this.loadStatuses();
    this.render();
  }

  private initializeTools() {
    const toolsData: CliTool[] = [
      {
        id: 'claude',
        name: 'Claude Code CLI',
        icon: '🤖',
        description: 'Terminal-native AI agent that understands entire codebases and makes coordinated changes',
        package: '@anthropic-ai/claude-code',
        installCommand: 'npm install -g @anthropic-ai/claude-code',
        memoryServiceIntegration: true
      },
      {
        id: 'gemini',
        name: 'Gemini CLI',
        icon: '✨',
        description: 'Google\'s free agentic assistant with 1M token context (1000 requests/day FREE!)',
        package: '@google/gemini-cli',
        installCommand: 'npm install -g @google/gemini-cli',
        memoryServiceIntegration: true
      },
      {
        id: 'qwen',
        name: 'Qwen Code',
        icon: '🐉',
        description: 'Alibaba\'s open-source agent optimized for Qwen3-Coder models',
        package: '@qwen-code/qwen-code',
        installCommand: 'npm install -g @qwen-code/qwen-code',
        memoryServiceIntegration: true
      },
      {
        id: 'openai',
        name: 'OpenAI Codex CLI',
        icon: '🌟',
        description: 'Smart terminal assistant with access to GPT-4.1 and o3 models',
        package: '@openai/codex-cli',
        installCommand: 'npm install -g @openai/codex-cli',
        memoryServiceIntegration: true
      },
      {
        id: 'aider',
        name: 'Aider',
        icon: '🔧',
        description: 'Git-integrated agentic editor with seamless version control',
        installCommand: 'pip install aider-chat',
        memoryServiceIntegration: false
      },
      {
        id: 'cline',
        name: 'Cline',
        icon: '💬',
        description: 'Lightweight conversational agent for multi-turn file editing',
        package: '@cline/cli',
        installCommand: 'npm install -g @cline/cli',
        memoryServiceIntegration: false
      }
    ];

    toolsData.forEach(tool => this.tools.set(tool.id, tool));
  }

  private async loadStatuses() {
    try {
      const statuses = await window.api.getCliToolsStatus();
      Object.entries(statuses).forEach(([toolId, status]) => {
        this.statuses.set(toolId, status as ToolStatus);
      });
    } catch (error) {
      console.error('Failed to load CLI tool statuses:', error);
    }
  }

  public render() {
    this.container.innerHTML = `
      <div class="cli-tools-container">
        <div class="cli-tools-header">
          <h2>AI CLI Tools Integration</h2>
          <p>Manage AI-powered command-line tools for agentic coding</p>
        </div>

        <div class="cli-tools-filters">
          <div class="filter-buttons">
            <button class="filter-btn ${this.filterStatus === 'all' ? 'active' : ''}" data-filter="all">
              All Tools (${this.tools.size})
            </button>
            <button class="filter-btn ${this.filterStatus === 'installed' ? 'active' : ''}" data-filter="installed">
              Installed (${this.getInstalledCount()})
            </button>
            <button class="filter-btn ${this.filterStatus === 'available' ? 'active' : ''}" data-filter="available">
              Available (${this.getAvailableCount()})
            </button>
            <button class="filter-btn ${this.filterStatus === 'updates' ? 'active' : ''}" data-filter="updates">
              Updates (${this.getUpdatesCount()})
            </button>
          </div>
          
          <div class="batch-actions" style="display: ${this.selectedTools.size > 0 ? 'flex' : 'none'}">
            <span>${this.selectedTools.size} selected</span>
            <button class="batch-btn" onclick="this.batchInstall()">Install Selected</button>
            <button class="batch-btn" onclick="this.clearSelection()">Clear</button>
          </div>
        </div>

        <div class="cli-tools-grid" id="cli-tools-grid">
          ${this.renderToolCards()}
        </div>

        <div class="cli-tools-advanced">
          <details>
            <summary>Advanced Options</summary>
            <div id="advanced-options-container"></div>
          </details>
        </div>
      </div>
    `;

    this.attachEventListeners();
    this.renderAdvancedOptions();
  }

  private renderToolCards(): string {
    const filteredTools = this.getFilteredTools();
    return Array.from(filteredTools)
      .map(([toolId, tool]) => {
        const status = this.statuses.get(toolId) || { installed: false };
        return this.renderToolCard(tool, status);
      })
      .join('');
  }

  private renderToolCard(tool: CliTool, status: ToolStatus): string {
    const isRecommended = tool.id === 'claude' || tool.id === 'gemini';
    const isFree = tool.id === 'gemini';
    
    return `
      <div class="cli-tool-card ${status.installed ? 'installed' : ''}" data-tool-id="${tool.id}">
        ${this.selectedTools.size > 0 ? `
          <div class="tool-checkbox">
            <input type="checkbox" 
              id="select-${tool.id}" 
              ${this.selectedTools.has(tool.id) ? 'checked' : ''}
              onchange="window.cliToolsTab.toggleSelection('${tool.id}')">
          </div>
        ` : ''}
        
        <div class="tool-header">
          <span class="tool-icon">${tool.icon}</span>
          <div class="tool-title">
            <h3>${tool.name}</h3>
            ${isRecommended ? '<span class="badge recommended">Recommended</span>' : ''}
            ${isFree ? '<span class="badge free">FREE</span>' : ''}
          </div>
          <div class="tool-status">
            ${this.renderStatus(status)}
          </div>
        </div>

        <div class="tool-body">
          ${status.installed ? `
            <div class="tool-info">
              <p><strong>Version:</strong> ${status.version || 'Unknown'}</p>
              ${tool.memoryServiceIntegration ? `
                <p><strong>Memory Service:</strong> 
                  ${status.authenticated ? '✅ Connected' : '⚠️ Not connected'}
                </p>
              ` : ''}
              ${status.lastChecked ? `
                <p><small>Last updated: ${this.formatTime(status.lastChecked)}</small></p>
              ` : ''}
            </div>
          ` : `
            <p class="tool-description">${tool.description}</p>
          `}

          ${status.installProgress !== undefined ? `
            <div class="install-progress">
              <div class="progress-bar">
                <div class="progress-fill" style="width: ${status.installProgress}%"></div>
              </div>
              <p class="progress-message">${status.installMessage || 'Installing...'}</p>
            </div>
          ` : ''}

          ${status.error ? `
            <div class="error-message">
              <span>⚠️</span> ${status.error}
            </div>
          ` : ''}
        </div>

        <div class="tool-actions">
          ${this.renderActions(tool, status)}
        </div>
      </div>
    `;
  }

  private renderStatus(status: ToolStatus): string {
    if (status.installProgress !== undefined) {
      return '<span class="status-indicator installing">◐ Installing...</span>';
    }
    if (status.error) {
      return '<span class="status-indicator error">● Error</span>';
    }
    if (status.installed) {
      if (status.updateAvailable) {
        return '<span class="status-indicator update">● Update Available</span>';
      }
      return '<span class="status-indicator installed">● Installed</span>';
    }
    return '<span class="status-indicator not-installed">○ Not Installed</span>';
  }

  private renderActions(tool: CliTool, status: ToolStatus): string {
    if (status.installProgress !== undefined) {
      return '<button class="tool-btn cancel" onclick="window.cliToolsTab.cancelInstall(\'' + tool.id + '\')">Cancel</button>';
    }

    if (status.error) {
      return `
        <button class="tool-btn primary" onclick="window.cliToolsTab.retryInstall('${tool.id}')">Retry</button>
        <button class="tool-btn" onclick="window.cliToolsTab.viewLogs('${tool.id}')">View Logs</button>
      `;
    }

    if (status.installed) {
      const buttons = [];
      if (status.updateAvailable) {
        buttons.push(`<button class="tool-btn primary" onclick="window.cliToolsTab.updateTool('${tool.id}')">Update</button>`);
      } else {
        buttons.push(`<button class="tool-btn" onclick="window.cliToolsTab.checkUpdate('${tool.id}')">Check Update</button>`);
      }
      
      if (!status.authenticated && tool.memoryServiceIntegration) {
        buttons.push(`<button class="tool-btn" onclick="window.cliToolsTab.configureTool('${tool.id}')">Configure</button>`);
      }
      
      buttons.push(`<button class="tool-btn danger" onclick="window.cliToolsTab.uninstallTool('${tool.id}')">Remove</button>`);
      
      return buttons.join('');
    }

    return `
      <button class="tool-btn primary" onclick="window.cliToolsTab.installTool('${tool.id}')">Install</button>
      <button class="tool-btn" onclick="window.cliToolsTab.learnMore('${tool.id}')">Learn More</button>
    `;
  }

  private renderAdvancedOptions() {
    const container = document.getElementById('advanced-options-container');
    if (container) {
      new AdvancedOptions(container);
    }
  }

  private getFilteredTools(): Map<string, CliTool> {
    if (this.filterStatus === 'all') {
      return this.tools;
    }

    const filtered = new Map<string, CliTool>();
    this.tools.forEach((tool, id) => {
      const status = this.statuses.get(id) || { installed: false };
      
      if (this.filterStatus === 'installed' && status.installed) {
        filtered.set(id, tool);
      } else if (this.filterStatus === 'available' && !status.installed) {
        filtered.set(id, tool);
      } else if (this.filterStatus === 'updates' && status.updateAvailable) {
        filtered.set(id, tool);
      }
    });
    
    return filtered;
  }

  private getInstalledCount(): number {
    return Array.from(this.statuses.values()).filter(s => s.installed).length;
  }

  private getAvailableCount(): number {
    return this.tools.size - this.getInstalledCount();
  }

  private getUpdatesCount(): number {
    return Array.from(this.statuses.values()).filter(s => s.updateAvailable).length;
  }

  private formatTime(date: Date): string {
    const now = new Date();
    const diff = now.getTime() - new Date(date).getTime();
    const hours = Math.floor(diff / (1000 * 60 * 60));
    
    if (hours < 1) return 'Just now';
    if (hours < 24) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    
    const days = Math.floor(hours / 24);
    return `${days} day${days > 1 ? 's' : ''} ago`;
  }

  private attachEventListeners() {
    // Filter buttons
    document.querySelectorAll('.filter-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const filter = (e.target as HTMLElement).dataset.filter as any;
        this.filterStatus = filter;
        this.render();
      });
    });

    // Listen for status updates from main process
    window.api.onCliToolProgress((event: any, data: any) => {
      this.handleProgressUpdate(data);
    });
  }

  private handleProgressUpdate(data: any) {
    const { toolId, status, progress, message, error } = data;
    
    const currentStatus = this.statuses.get(toolId) || { installed: false };
    
    if (status === 'installing') {
      currentStatus.installProgress = progress;
      currentStatus.installMessage = message;
    } else if (status === 'complete') {
      currentStatus.installed = true;
      currentStatus.installProgress = undefined;
      currentStatus.installMessage = undefined;
      currentStatus.version = data.version;
    } else if (status === 'error') {
      currentStatus.error = error;
      currentStatus.installProgress = undefined;
    }
    
    this.statuses.set(toolId, currentStatus);
    this.updateToolCard(toolId);
  }

  private updateToolCard(toolId: string) {
    const card = document.querySelector(`[data-tool-id="${toolId}"]`);
    if (card) {
      const tool = this.tools.get(toolId);
      const status = this.statuses.get(toolId);
      if (tool && status) {
        card.outerHTML = this.renderToolCard(tool, status);
      }
    }
  }

  // Public methods for button actions
  public async installTool(toolId: string) {
    try {
      await window.api.installCliTool(toolId);
    } catch (error) {
      console.error('Failed to install tool:', error);
    }
  }

  public async updateTool(toolId: string) {
    try {
      await window.api.updateCliTool(toolId);
    } catch (error) {
      console.error('Failed to update tool:', error);
    }
  }

  public async uninstallTool(toolId: string) {
    if (confirm(`Are you sure you want to remove ${this.tools.get(toolId)?.name}?`)) {
      try {
        await window.api.uninstallCliTool(toolId);
        const status = this.statuses.get(toolId);
        if (status) {
          status.installed = false;
          status.version = undefined;
          this.updateToolCard(toolId);
        }
      } catch (error) {
        console.error('Failed to uninstall tool:', error);
      }
    }
  }

  public async checkUpdate(toolId: string) {
    try {
      const hasUpdate = await window.api.checkCliToolUpdate(toolId);
      const status = this.statuses.get(toolId);
      if (status) {
        status.updateAvailable = hasUpdate;
        this.updateToolCard(toolId);
      }
    } catch (error) {
      console.error('Failed to check update:', error);
    }
  }

  public async configureTool(toolId: string) {
    // Open configuration dialog
    console.log('Configure tool:', toolId);
  }

  public learnMore(toolId: string) {
    const tool = this.tools.get(toolId);
    if (tool) {
      // Open documentation or website
      console.log('Learn more about:', tool.name);
    }
  }

  public toggleSelection(toolId: string) {
    if (this.selectedTools.has(toolId)) {
      this.selectedTools.delete(toolId);
    } else {
      this.selectedTools.add(toolId);
    }
    this.render();
  }

  public async batchInstall() {
    for (const toolId of this.selectedTools) {
      await this.installTool(toolId);
    }
    this.clearSelection();
  }

  public clearSelection() {
    this.selectedTools.clear();
    this.render();
  }

  public cancelInstall(toolId: string) {
    console.log('Cancel installation:', toolId);
  }

  public retryInstall(toolId: string) {
    const status = this.statuses.get(toolId);
    if (status) {
      status.error = undefined;
      this.updateToolCard(toolId);
    }
    this.installTool(toolId);
  }

  public viewLogs(toolId: string) {
    console.log('View logs for:', toolId);
  }
}

// Make it globally accessible
(window as any).cliToolsTab = null;