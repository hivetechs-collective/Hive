/**
 * CLI Tool Card Component
 * Individual card for displaying and managing a single CLI tool
 */

import { CliTool, ToolStatus } from './CliToolsTab';

export class CliToolCard {
  private tool: CliTool;
  private status: ToolStatus;
  private container: HTMLElement;
  private onAction: (action: string, toolId: string) => void;

  constructor(
    container: HTMLElement,
    tool: CliTool,
    status: ToolStatus,
    onAction: (action: string, toolId: string) => void
  ) {
    this.container = container;
    this.tool = tool;
    this.status = status;
    this.onAction = onAction;
    this.render();
  }

  public updateStatus(status: ToolStatus) {
    this.status = status;
    this.render();
  }

  private render() {
    const isRecommended = this.tool.id === 'claude' || this.tool.id === 'gemini';
    const isFree = this.tool.id === 'gemini';
    const isNew = this.tool.id === 'qwen'; // Qwen Code released July 2025

    this.container.innerHTML = `
      <div class="cli-tool-card ${this.getCardClass()}" data-tool-id="${this.tool.id}">
        <div class="tool-header">
          <div class="tool-icon-wrapper">
            <span class="tool-icon">${this.tool.icon}</span>
            ${this.renderStatusDot()}
          </div>
          
          <div class="tool-info">
            <h3 class="tool-name">${this.tool.name}</h3>
            <div class="tool-badges">
              ${isRecommended ? '<span class="badge recommended" title="Recommended tool">★ Recommended</span>' : ''}
              ${isFree ? '<span class="badge free" title="1000 requests/day FREE!">FREE</span>' : ''}
              ${isNew ? '<span class="badge new" title="Released July 2025">NEW</span>' : ''}
              ${this.status.updateAvailable ? '<span class="badge update">Update Available</span>' : ''}
            </div>
          </div>
        </div>

        <div class="tool-body">
          ${this.renderBody()}
        </div>

        <div class="tool-footer">
          ${this.renderActions()}
        </div>

        ${this.renderProgressOverlay()}
      </div>
    `;

    this.attachEventListeners();
  }

  private getCardClass(): string {
    const classes = [];
    
    if (this.status.installed) classes.push('installed');
    if (this.status.installProgress !== undefined) classes.push('installing');
    if (this.status.error) classes.push('error');
    if (this.status.updateAvailable) classes.push('update-available');
    
    return classes.join(' ');
  }

  private renderStatusDot(): string {
    let color = 'gray';
    let title = 'Not installed';
    
    if (this.status.installed) {
      color = 'green';
      title = 'Installed';
      if (this.status.updateAvailable) {
        color = 'amber';
        title = 'Update available';
      }
    } else if (this.status.installProgress !== undefined) {
      color = 'blue';
      title = 'Installing...';
    } else if (this.status.error) {
      color = 'red';
      title = 'Error';
    }
    
    return `<span class="status-dot status-${color}" title="${title}"></span>`;
  }

  private renderBody(): string {
    if (this.status.error) {
      return this.renderError();
    }

    if (this.status.installed) {
      return this.renderInstalledInfo();
    }

    return this.renderDescription();
  }

  private renderDescription(): string {
    return `
      <div class="tool-description">
        <p>${this.tool.description}</p>
        ${this.tool.package ? `
          <div class="tool-package">
            <code>${this.tool.package}</code>
          </div>
        ` : ''}
      </div>
    `;
  }

  private renderInstalledInfo(): string {
    return `
      <div class="tool-installed-info">
        <div class="info-row">
          <span class="info-label">Version:</span>
          <span class="info-value">${this.status.version || 'Unknown'}</span>
        </div>
        
        ${this.tool.memoryServiceIntegration ? `
          <div class="info-row">
            <span class="info-label">Memory Service:</span>
            <span class="info-value ${this.status.authenticated ? 'connected' : 'disconnected'}">
              ${this.status.authenticated ? '✅ Connected' : '⚠️ Not connected'}
            </span>
          </div>
        ` : ''}
        
        ${this.status.lastChecked ? `
          <div class="info-row">
            <span class="info-label">Last updated:</span>
            <span class="info-value">${this.formatTime(this.status.lastChecked)}</span>
          </div>
        ` : ''}
      </div>
    `;
  }

  private renderError(): string {
    return `
      <div class="tool-error">
        <div class="error-icon">⚠️</div>
        <div class="error-details">
          <p class="error-message">${this.status.error}</p>
          ${this.getErrorHint()}
        </div>
      </div>
    `;
  }

  private getErrorHint(): string {
    const error = this.status.error?.toLowerCase() || '';
    
    if (error.includes('permission')) {
      return '<p class="error-hint">Try using local installation or run with elevated permissions</p>';
    }
    if (error.includes('python')) {
      return '<p class="error-hint">Python 3.8+ is required. Install from python.org</p>';
    }
    if (error.includes('node')) {
      return '<p class="error-hint">Node.js 18+ is required. Install from nodejs.org</p>';
    }
    if (error.includes('network')) {
      return '<p class="error-hint">Check your internet connection and try again</p>';
    }
    
    return '';
  }

  private renderActions(): string {
    if (this.status.installProgress !== undefined) {
      return `
        <button class="tool-btn cancel" data-action="cancel">
          Cancel Installation
        </button>
      `;
    }

    if (this.status.error) {
      return `
        <button class="tool-btn primary" data-action="retry">
          Retry Installation
        </button>
        <button class="tool-btn secondary" data-action="logs">
          View Logs
        </button>
      `;
    }

    if (this.status.installed) {
      const buttons = [];
      
      if (this.status.updateAvailable) {
        buttons.push(`
          <button class="tool-btn primary update-btn" data-action="update">
            Update Now
          </button>
        `);
      }
      
      if (!this.status.authenticated && this.tool.memoryServiceIntegration) {
        buttons.push(`
          <button class="tool-btn secondary" data-action="configure">
            Connect to Memory
          </button>
        `);
      }
      
      buttons.push(`
        <div class="tool-btn-group">
          <button class="tool-btn icon-btn" data-action="check" title="Check for updates">
            🔄
          </button>
          <button class="tool-btn icon-btn" data-action="settings" title="Settings">
            ⚙️
          </button>
          <button class="tool-btn icon-btn danger" data-action="uninstall" title="Uninstall">
            🗑️
          </button>
        </div>
      `);
      
      return buttons.join('');
    }

    // Not installed
    return `
      <button class="tool-btn primary install-btn" data-action="install">
        Install
      </button>
      <button class="tool-btn secondary" data-action="learn">
        Learn More
      </button>
    `;
  }

  private renderProgressOverlay(): string {
    if (this.status.installProgress === undefined) {
      return '';
    }

    return `
      <div class="progress-overlay">
        <div class="progress-content">
          <div class="progress-spinner"></div>
          <div class="progress-info">
            <p class="progress-title">Installing ${this.tool.name}</p>
            <div class="progress-bar">
              <div class="progress-fill" style="width: ${this.status.installProgress}%"></div>
            </div>
            <p class="progress-message">${this.status.installMessage || 'Preparing installation...'}</p>
            <p class="progress-percent">${this.status.installProgress}%</p>
          </div>
        </div>
      </div>
    `;
  }

  private formatTime(date: Date): string {
    const now = new Date();
    const diff = now.getTime() - new Date(date).getTime();
    const minutes = Math.floor(diff / (1000 * 60));
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes} minute${minutes !== 1 ? 's' : ''} ago`;
    if (hours < 24) return `${hours} hour${hours !== 1 ? 's' : ''} ago`;
    if (days < 30) return `${days} day${days !== 1 ? 's' : ''} ago`;
    
    return new Date(date).toLocaleDateString();
  }

  private attachEventListeners() {
    this.container.querySelectorAll('[data-action]').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.preventDefault();
        const action = (e.currentTarget as HTMLElement).dataset.action;
        if (action) {
          this.handleAction(action);
        }
      });
    });
  }

  private handleAction(action: string) {
    switch (action) {
      case 'install':
        this.showInstallConfirmation();
        break;
      case 'uninstall':
        this.showUninstallConfirmation();
        break;
      case 'update':
        this.onAction('update', this.tool.id);
        break;
      case 'configure':
        this.showConfigurationDialog();
        break;
      case 'check':
        this.onAction('check', this.tool.id);
        break;
      case 'settings':
        this.showSettingsDialog();
        break;
      case 'learn':
        this.openDocumentation();
        break;
      case 'retry':
        this.status.error = undefined;
        this.onAction('install', this.tool.id);
        break;
      case 'cancel':
        this.onAction('cancel', this.tool.id);
        break;
      case 'logs':
        this.showLogs();
        break;
      default:
        this.onAction(action, this.tool.id);
    }
  }

  private showInstallConfirmation() {
    const isAider = this.tool.id === 'aider';
    const requiresPython = isAider;
    
    let message = `Install ${this.tool.name}?\n\n`;
    message += `This will run:\n${this.tool.installCommand}\n\n`;
    
    if (requiresPython) {
      message += '⚠️ Requires Python 3.8+ to be installed\n';
    }
    
    if (this.tool.memoryServiceIntegration) {
      message += '✅ Will automatically connect to Memory Service\n';
    }
    
    if (confirm(message)) {
      this.onAction('install', this.tool.id);
    }
  }

  private showUninstallConfirmation() {
    if (confirm(`Are you sure you want to uninstall ${this.tool.name}?\n\nThis will remove the tool but preserve your settings.`)) {
      this.onAction('uninstall', this.tool.id);
    }
  }

  private showConfigurationDialog() {
    // TODO: Show auth configuration dialog
    console.log('Configure:', this.tool.name);
  }

  private showSettingsDialog() {
    // TODO: Show tool-specific settings
    console.log('Settings for:', this.tool.name);
  }

  private openDocumentation() {
    const urls: Record<string, string> = {
      'claude': 'https://claude.ai/code',
      'gemini': 'https://ai.google.dev/gemini-cli',
      'qwen': 'https://github.com/QwenLM/qwen-code',
      'openai': 'https://platform.openai.com/docs/cli',
      'aider': 'https://aider.chat',
      'cline': 'https://github.com/cline/cli'
    };
    
    const url = urls[this.tool.id];
    if (url) {
      window.open(url, '_blank');
    }
  }

  private showLogs() {
    // TODO: Show installation logs
    console.log('Show logs for:', this.tool.name);
  }
}

export default CliToolCard;