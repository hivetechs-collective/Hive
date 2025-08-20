/**
 * Advanced Options Component
 * Configuration panel for CLI tools installation and management preferences
 */

export interface AdvancedSettings {
  installation: {
    autoInstallRecommended: boolean;
    useLocalNpm: boolean;
    usePipx: boolean;
    customInstallPath?: string;
  };
  updates: {
    autoCheck: boolean;
    autoInstall: boolean;
    checkInterval: number; // hours
    includePrerelease: boolean;
  };
  integration: {
    memoryServiceAutoConnect: boolean;
    shareUsageAnalytics: boolean;
    enableExperimentalFeatures: boolean;
  };
}

export class AdvancedOptions {
  private container: HTMLElement;
  private settings: AdvancedSettings;

  constructor(container: HTMLElement) {
    this.container = container;
    this.settings = this.loadSettings();
    this.render();
  }

  private loadSettings(): AdvancedSettings {
    // Load from localStorage or use defaults
    const saved = localStorage.getItem('cliToolsAdvancedSettings');
    if (saved) {
      return JSON.parse(saved);
    }

    return {
      installation: {
        autoInstallRecommended: false,
        useLocalNpm: false,
        usePipx: true,
        customInstallPath: undefined
      },
      updates: {
        autoCheck: true,
        autoInstall: false,
        checkInterval: 24,
        includePrerelease: false
      },
      integration: {
        memoryServiceAutoConnect: true,
        shareUsageAnalytics: false,
        enableExperimentalFeatures: false
      }
    };
  }

  private saveSettings() {
    localStorage.setItem('cliToolsAdvancedSettings', JSON.stringify(this.settings));
    // Notify main process of settings change
    window.api.updateCliToolsSettings(this.settings);
  }

  private render() {
    this.container.innerHTML = `
      <div class="advanced-options">
        <div class="settings-section">
          <h3>Installation Options</h3>
          
          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" 
                id="auto-install-recommended"
                ${this.settings.installation.autoInstallRecommended ? 'checked' : ''}>
              <span>Auto-install recommended tools on first launch</span>
            </label>
            <p class="setting-description">Automatically installs Claude Code and Gemini CLI</p>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" 
                id="use-local-npm"
                ${this.settings.installation.useLocalNpm ? 'checked' : ''}>
              <span>Use local npm installations (no sudo required)</span>
            </label>
            <p class="setting-description">Installs npm packages to ~/.hive/tools/node_modules</p>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" 
                id="use-pipx"
                ${this.settings.installation.usePipx ? 'checked' : ''}>
              <span>Use pipx for Python tools (recommended)</span>
            </label>
            <p class="setting-description">Isolates Python tools in virtual environments</p>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <span>Custom installation directory:</span>
            </label>
            <div class="input-group">
              <input type="text" 
                id="custom-install-path"
                class="setting-input"
                placeholder="~/.hive/tools"
                value="${this.settings.installation.customInstallPath || ''}">
              <button class="browse-btn" id="browse-install-path">Browse</button>
            </div>
          </div>
        </div>

        <div class="settings-section">
          <h3>Update Settings</h3>
          
          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" 
                id="auto-check-updates"
                ${this.settings.updates.autoCheck ? 'checked' : ''}>
              <span>Check for updates on startup</span>
            </label>
            <p class="setting-description">Automatically checks for CLI tool updates when app starts</p>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" 
                id="auto-install-updates"
                ${this.settings.updates.autoInstall ? 'checked' : ''}
                ${!this.settings.updates.autoCheck ? 'disabled' : ''}>
              <span>Auto-install updates</span>
            </label>
            <p class="setting-description">Automatically installs updates when available</p>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <span>Update check interval:</span>
            </label>
            <select id="update-interval" class="setting-select">
              <option value="6" ${this.settings.updates.checkInterval === 6 ? 'selected' : ''}>Every 6 hours</option>
              <option value="12" ${this.settings.updates.checkInterval === 12 ? 'selected' : ''}>Every 12 hours</option>
              <option value="24" ${this.settings.updates.checkInterval === 24 ? 'selected' : ''}>Daily</option>
              <option value="168" ${this.settings.updates.checkInterval === 168 ? 'selected' : ''}>Weekly</option>
            </select>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" 
                id="include-prerelease"
                ${this.settings.updates.includePrerelease ? 'checked' : ''}>
              <span>Include pre-release versions</span>
            </label>
            <p class="setting-description">Install beta and preview releases (may be unstable)</p>
          </div>
        </div>

        <div class="settings-section">
          <h3>Integration Settings</h3>
          
          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" 
                id="memory-auto-connect"
                ${this.settings.integration.memoryServiceAutoConnect ? 'checked' : ''}>
              <span>Auto-connect tools to Memory Service</span>
            </label>
            <p class="setting-description">Automatically configures Memory Service integration for supported tools</p>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" 
                id="share-analytics"
                ${this.settings.integration.shareUsageAnalytics ? 'checked' : ''}>
              <span>Share anonymous usage analytics</span>
            </label>
            <p class="setting-description">Help improve CLI tools by sharing anonymous usage data</p>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" 
                id="experimental-features"
                ${this.settings.integration.enableExperimentalFeatures ? 'checked' : ''}>
              <span>Enable experimental features</span>
            </label>
            <p class="setting-description">Try new features before they're officially released</p>
          </div>
        </div>

        <div class="settings-actions">
          <button class="btn secondary" id="reset-settings">Reset to Defaults</button>
          <button class="btn primary" id="apply-settings">Apply Changes</button>
        </div>

        <div class="settings-info">
          <h4>Quick Tips</h4>
          <ul>
            <li><strong>Gemini CLI</strong> offers 1000 free requests per day - perfect for trying out agentic coding!</li>
            <li><strong>Local npm</strong> installation avoids permission issues on restricted systems</li>
            <li><strong>pipx</strong> prevents Python package conflicts by isolating each tool</li>
            <li><strong>Memory Service</strong> integration allows tools to share context and learn from your usage</li>
          </ul>
        </div>
      </div>
    `;

    this.attachEventListeners();
  }

  private attachEventListeners() {
    // Installation options
    document.getElementById('auto-install-recommended')?.addEventListener('change', (e) => {
      this.settings.installation.autoInstallRecommended = (e.target as HTMLInputElement).checked;
    });

    document.getElementById('use-local-npm')?.addEventListener('change', (e) => {
      this.settings.installation.useLocalNpm = (e.target as HTMLInputElement).checked;
    });

    document.getElementById('use-pipx')?.addEventListener('change', (e) => {
      this.settings.installation.usePipx = (e.target as HTMLInputElement).checked;
    });

    document.getElementById('custom-install-path')?.addEventListener('change', (e) => {
      const value = (e.target as HTMLInputElement).value;
      this.settings.installation.customInstallPath = value || undefined;
    });

    document.getElementById('browse-install-path')?.addEventListener('click', async () => {
      const path = await window.api.selectDirectory();
      if (path) {
        this.settings.installation.customInstallPath = path;
        (document.getElementById('custom-install-path') as HTMLInputElement).value = path;
      }
    });

    // Update settings
    document.getElementById('auto-check-updates')?.addEventListener('change', (e) => {
      this.settings.updates.autoCheck = (e.target as HTMLInputElement).checked;
      const autoInstall = document.getElementById('auto-install-updates') as HTMLInputElement;
      if (!this.settings.updates.autoCheck) {
        autoInstall.disabled = true;
        autoInstall.checked = false;
        this.settings.updates.autoInstall = false;
      } else {
        autoInstall.disabled = false;
      }
    });

    document.getElementById('auto-install-updates')?.addEventListener('change', (e) => {
      this.settings.updates.autoInstall = (e.target as HTMLInputElement).checked;
    });

    document.getElementById('update-interval')?.addEventListener('change', (e) => {
      this.settings.updates.checkInterval = parseInt((e.target as HTMLSelectElement).value);
    });

    document.getElementById('include-prerelease')?.addEventListener('change', (e) => {
      this.settings.updates.includePrerelease = (e.target as HTMLInputElement).checked;
    });

    // Integration settings
    document.getElementById('memory-auto-connect')?.addEventListener('change', (e) => {
      this.settings.integration.memoryServiceAutoConnect = (e.target as HTMLInputElement).checked;
    });

    document.getElementById('share-analytics')?.addEventListener('change', (e) => {
      this.settings.integration.shareUsageAnalytics = (e.target as HTMLInputElement).checked;
    });

    document.getElementById('experimental-features')?.addEventListener('change', (e) => {
      this.settings.integration.enableExperimentalFeatures = (e.target as HTMLInputElement).checked;
    });

    // Action buttons
    document.getElementById('reset-settings')?.addEventListener('click', () => {
      if (confirm('Reset all advanced settings to defaults?')) {
        this.settings = {
          installation: {
            autoInstallRecommended: false,
            useLocalNpm: false,
            usePipx: true,
            customInstallPath: undefined
          },
          updates: {
            autoCheck: true,
            autoInstall: false,
            checkInterval: 24,
            includePrerelease: false
          },
          integration: {
            memoryServiceAutoConnect: true,
            shareUsageAnalytics: false,
            enableExperimentalFeatures: false
          }
        };
        this.render();
        this.showNotification('Settings reset to defaults');
      }
    });

    document.getElementById('apply-settings')?.addEventListener('click', () => {
      this.saveSettings();
      this.showNotification('Settings saved successfully');
    });
  }

  private showNotification(message: string) {
    // Create a temporary notification
    const notification = document.createElement('div');
    notification.className = 'notification success';
    notification.textContent = message;
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: #10b981;
      color: white;
      padding: 12px 20px;
      border-radius: 8px;
      box-shadow: 0 4px 6px rgba(0,0,0,0.1);
      z-index: 10000;
      animation: slideIn 0.3s ease;
    `;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
      notification.style.animation = 'slideOut 0.3s ease';
      setTimeout(() => notification.remove(), 300);
    }, 3000);
  }

  public getSettings(): AdvancedSettings {
    return this.settings;
  }
}

export default AdvancedOptions;