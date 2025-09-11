/**
 * Welcome Page Component
 * Displays when the app launches to orient new users
 */

export class WelcomePage {
  private container: HTMLElement | null = null;

  async mount(container: HTMLElement): Promise<void> {
    this.container = container;
    this.render();
    await this.loadPreferences();
    this.attachEventListeners();
  }

  unmount(): void {
    if (this.container) {
      this.container.innerHTML = '';
    }
    this.container = null;
  }

  private render(): void {
    if (!this.container) return;

    // Add styles if not already present
    if (!document.getElementById('welcome-page-styles')) {
      const style = document.createElement('style');
      style.id = 'welcome-page-styles';
      style.textContent = this.getStyles();
      document.head.appendChild(style);
    }

    this.container.innerHTML = `
      <div class="welcome-page">
        <div class="welcome-content">
          <div class="welcome-header">
            <h1 class="welcome-title">Hive Consensus</h1>
            <p class="welcome-subtitle">Enterprise AI Development Platform</p>
          </div>

          <!-- What's New Section -->
          <div class="whats-new-section">
            <div class="whats-new-header">
              <h2>Release Notes</h2>
              <span class="version-badge">v1.8.288</span>
            </div>
            <div class="whats-new-content">
              <div class="changelog-item">
                <span class="changelog-badge new">NEW</span>
                <span class="changelog-text">Integrated documentation viewer with VS Code styling</span>
              </div>
              <div class="changelog-item">
                <span class="changelog-badge improved">IMPROVED</span>
                <span class="changelog-text">Panel navigation with dedicated sidebar buttons</span>
              </div>
              <div class="changelog-item">
                <span class="changelog-badge fixed">FIXED</span>
                <span class="changelog-text">Settings and documentation panel visibility issues</span>
              </div>
              <div class="changelog-item">
                <span class="changelog-badge new">NEW</span>
                <span class="changelog-text">Welcome page with quick access to all features</span>
              </div>
            </div>
          </div>

          <div class="welcome-features">
            <div class="feature-card">
              <div class="feature-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                </svg>
              </div>
              <h3>Consensus Engine</h3>
              <p>4-stage AI pipeline with multiple models for optimal code generation.</p>
            </div>

            <div class="feature-card">
              <div class="feature-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 11H3v2h6v-2zm0-4H3v2h6V7zm0 8H3v2h6v-2zm12-8h-6v2h6V7zm0 4h-6v2h6v-2zm0 4h-6v2h6v-2z"/>
                </svg>
              </div>
              <h3>Memory System</h3>
              <p>Persistent context across all AI CLI tools via unified database.</p>
            </div>

            <div class="feature-card">
              <div class="feature-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
                  <line x1="9" y1="9" x2="15" y2="9"/>
                  <line x1="9" y1="12" x2="15" y2="12"/>
                  <line x1="9" y1="15" x2="11" y2="15"/>
                </svg>
              </div>
              <h3>CLI Integration</h3>
              <p>Seamless integration with Claude Code, Gemini, Grok, and more.</p>
            </div>
          </div>

          <div class="welcome-quickstart">
            <h2>Quick Start Guide</h2>
            
            <div class="quickstart-steps">
              <div class="quickstart-step">
                <div class="step-number">1</div>
                <div class="step-content">
                  <h4>Open a Project Folder</h4>
                  <p>Use <kbd>Cmd/Ctrl + O</kbd> or click "Open Folder" to select your project directory.</p>
                </div>
              </div>

              <div class="quickstart-step">
                <div class="step-number">2</div>
                <div class="step-content">
                  <h4>Launch an AI Tool</h4>
                  <p>Click on any AI tool in the sidebar to launch it with automatic memory integration.</p>
                </div>
              </div>

              <div class="quickstart-step">
                <div class="step-number">3</div>
                <div class="step-content">
                  <h4>Use the Consensus Engine</h4>
                  <p>Type your prompt in the Consensus panel to get refined AI responses through our 4-stage pipeline.</p>
                </div>
              </div>
            </div>
          </div>

          <div class="welcome-memory">
            <h2>Memory System Integration</h2>
            <div class="memory-box">
              <p>Tell your AI CLI tools to read your memory:</p>
              <div class="memory-command">
                <code>Read ~/.MEMORY.md to understand how to access my memory and context system through the database at ~/.hive-ai.db</code>
                <button class="copy-btn" data-copy-text="Read ~/.MEMORY.md to understand how to access my memory and context system through the database at ~/.hive-ai.db">
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M4 2v10h8V4.414L9.586 2H4zm7 10H5V3h3v3h3v6z"/>
                    <path d="M2 4v10h8v-1H3V4h1V3H2z"/>
                  </svg>
                  Copy
                </button>
              </div>
            </div>
          </div>

          <div class="welcome-footer">
            <div class="startup-preference">
              <label class="checkbox-label">
                <input type="checkbox" id="show-on-startup" checked>
                <span>Show welcome page on startup</span>
              </label>
            </div>
            <div class="welcome-actions">
              <button class="action-btn primary" id="open-documentation">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M14 1H2a1 1 0 00-1 1v8a1 1 0 001 1h5.5l3 3 3-3H14a1 1 0 001-1V2a1 1 0 00-1-1zM8 9.5a.5.5 0 110-1 .5.5 0 010 1zm1-2.5a1 1 0 01-2 0V4a1 1 0 012 0v3z"/>
                </svg>
                View Documentation
              </button>
              <button class="action-btn" id="close-welcome">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M4.646 4.646a.5.5 0 01.708 0L8 7.293l2.646-2.647a.5.5 0 01.708.708L8.707 8l2.647 2.646a.5.5 0 01-.708.708L8 8.707l-2.646 2.647a.5.5 0 01-.708-.708L7.293 8 4.646 5.354a.5.5 0 010-.708z"/>
                </svg>
                Close Welcome
              </button>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  private async loadPreferences(): Promise<void> {
    if (!this.container) return;
    
    const checkbox = this.container.querySelector('#show-on-startup') as HTMLInputElement;
    if (!checkbox) return;
    
    try {
      // Try to load preference from database
      if (window.databaseAPI) {
        const result = await window.databaseAPI.query(
          'SELECT value FROM settings WHERE key = ?',
          ['welcome.showOnStartup']
        );
        
        // Default to true if no preference is saved
        const showOnStartup = result.length === 0 || result[0]?.value !== '0';
        checkbox.checked = showOnStartup;
      }
    } catch (error) {
      console.error('Failed to load welcome preferences:', error);
      // Default to checked on error
      checkbox.checked = true;
    }
  }

  private async savePreference(showOnStartup: boolean): Promise<void> {
    try {
      if (window.databaseAPI) {
        // Save to database
        await window.databaseAPI.execute(
          'INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)',
          ['welcome.showOnStartup', showOnStartup ? '1' : '0']
        );
        
        // Show toast notification
        this.showToast(showOnStartup 
          ? 'Welcome page will show on startup' 
          : 'Welcome page disabled on startup. Access via Help menu anytime.'
        );
      }
    } catch (error) {
      console.error('Failed to save welcome preference:', error);
      this.showToast('Failed to save preference', 'error');
    }
  }

  private showToast(message: string, type: 'info' | 'error' = 'info'): void {
    // Create toast element
    const toast = document.createElement('div');
    toast.className = `toast-notification ${type}`;
    toast.textContent = message;
    toast.style.cssText = `
      position: fixed;
      bottom: 20px;
      right: 20px;
      padding: 12px 20px;
      background: ${type === 'error' ? '#f44336' : '#007acc'};
      color: white;
      border-radius: 4px;
      font-size: 14px;
      z-index: 10000;
      animation: slideIn 0.3s ease-out;
    `;
    
    document.body.appendChild(toast);
    
    // Remove after 3 seconds
    setTimeout(() => {
      toast.style.animation = 'slideOut 0.3s ease-out';
      setTimeout(() => toast.remove(), 300);
    }, 3000);
  }

  private attachEventListeners(): void {
    if (!this.container) return;

    // Show on startup checkbox
    const checkbox = this.container.querySelector('#show-on-startup') as HTMLInputElement;
    if (checkbox) {
      checkbox.addEventListener('change', async (e) => {
        const target = e.target as HTMLInputElement;
        await this.savePreference(target.checked);
      });
    }

    // Documentation button
    const docBtn = this.container.querySelector('#open-documentation');
    if (docBtn) {
      docBtn.addEventListener('click', () => {
        // Emit event to open documentation
        window.dispatchEvent(new CustomEvent('open-documentation', { 
          detail: { section: 'getting-started' } 
        }));
      });
    }

    // Close welcome button
    const closeBtn = this.container.querySelector('#close-welcome');
    if (closeBtn) {
      closeBtn.addEventListener('click', () => {
        // Emit event to close welcome page
        window.dispatchEvent(new CustomEvent('close-welcome'));
      });
    }

    // Copy button
    const copyBtn = this.container.querySelector('.copy-btn');
    if (copyBtn) {
      copyBtn.addEventListener('click', (e) => {
        const btn = e.currentTarget as HTMLElement;
        const textToCopy = btn.dataset.copyText;
        if (textToCopy) {
          navigator.clipboard.writeText(textToCopy).then(() => {
            const originalHTML = btn.innerHTML;
            btn.innerHTML = `
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M13.854 3.646a.5.5 0 010 .708l-7 7a.5.5 0 01-.708 0l-3.5-3.5a.5.5 0 11.708-.708L6.5 10.293l6.646-6.647a.5.5 0 01.708 0z"/>
              </svg>
              Copied!
            `;
            btn.classList.add('copied');
            setTimeout(() => {
              btn.innerHTML = originalHTML;
              btn.classList.remove('copied');
            }, 2000);
          });
        }
      });
    }
  }

  private getStyles(): string {
    return `
      .welcome-page {
        display: flex;
        justify-content: center;
        align-items: flex-start;
        padding: 40px;
        height: 100%;
        overflow-y: auto;
        background: #1e1e1e;
        color: #cccccc;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'SF Pro Text', 'SF Pro Display', system-ui, sans-serif;
      }

      .welcome-content {
        max-width: 1200px;
        width: 100%;
      }

      .welcome-header {
        text-align: center;
        margin-bottom: 48px;
      }

      .welcome-title {
        font-size: 36px;
        font-weight: 400;
        color: #e1e1e1;
        margin: 0 0 8px 0;
        letter-spacing: -0.5px;
      }

      .welcome-subtitle {
        font-size: 14px;
        color: #8b8b8b;
        margin: 0;
        text-transform: uppercase;
        letter-spacing: 1px;
        font-weight: 500;
      }

      .whats-new-section {
        background: #252526;
        border: 1px solid #3c3c3c;
        border-left: 3px solid #007acc;
        border-radius: 4px;
        padding: 20px;
        margin-bottom: 32px;
        position: relative;
      }

      .whats-new-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 20px;
      }

      .whats-new-header h2 {
        font-size: 20px;
        font-weight: 500;
        color: #e1e1e1;
        margin: 0;
        display: flex;
        align-items: center;
      }

      .version-badge {
        background: transparent;
        color: #8b8b8b;
        padding: 4px 8px;
        border: 1px solid #3c3c3c;
        border-radius: 3px;
        font-size: 12px;
        font-weight: 500;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
      }

      .whats-new-content {
        display: flex;
        flex-direction: column;
        gap: 12px;
      }

      .changelog-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 8px 0;
        border-bottom: 1px solid #3c3c3c33;
      }

      .changelog-item:last-child {
        border-bottom: none;
      }

      .changelog-badge {
        padding: 3px 8px;
        border-radius: 4px;
        font-size: 10px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        min-width: 60px;
        text-align: center;
      }

      .changelog-badge.new {
        background: transparent;
        color: #4ec9b0;
        border: 1px solid #16825d;
      }

      .changelog-badge.improved {
        background: transparent;
        color: #75beff;
        border: 1px solid #004880;
      }

      .changelog-badge.fixed {
        background: transparent;
        color: #ce9178;
        border: 1px solid #8f3b00;
      }

      .changelog-text {
        color: #cccccc;
        font-size: 13px;
        flex: 1;
      }

      .welcome-features {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 24px;
        margin-bottom: 48px;
      }

      .feature-card {
        background: #252526;
        border: 1px solid #3c3c3c;
        border-radius: 4px;
        padding: 20px;
        transition: border-color 0.15s;
      }

      .feature-card:hover {
        border-color: #464647;
      }

      .feature-icon {
        width: 40px;
        height: 40px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: transparent;
        border: 1px solid #3c3c3c;
        border-radius: 4px;
        margin-bottom: 16px;
      }

      .feature-icon svg {
        color: #007acc;
      }

      .feature-card h3 {
        font-size: 18px;
        font-weight: 500;
        color: #e1e1e1;
        margin: 0 0 8px 0;
      }

      .feature-card p {
        font-size: 14px;
        line-height: 1.6;
        color: #969696;
        margin: 0;
      }

      .welcome-quickstart {
        background: #252526;
        border: 1px solid #3c3c3c;
        border-radius: 8px;
        padding: 32px;
        margin-bottom: 32px;
      }

      .welcome-quickstart h2 {
        font-size: 24px;
        font-weight: 400;
        color: #e1e1e1;
        margin: 0 0 24px 0;
      }

      .quickstart-steps {
        display: flex;
        flex-direction: column;
        gap: 20px;
      }

      .quickstart-step {
        display: flex;
        align-items: flex-start;
        gap: 16px;
      }

      .step-number {
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #007acc;
        color: white;
        border-radius: 50%;
        font-weight: bold;
        flex-shrink: 0;
      }

      .step-content h4 {
        font-size: 16px;
        font-weight: 500;
        color: #e1e1e1;
        margin: 0 0 4px 0;
      }

      .step-content p {
        font-size: 14px;
        color: #969696;
        margin: 0;
      }

      .step-content kbd {
        display: inline-block;
        padding: 2px 6px;
        background: #3c3c3c;
        border: 1px solid #464647;
        border-radius: 3px;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
        font-size: 12px;
        color: #e1e1e1;
      }

      .welcome-memory {
        background: #252526;
        border: 1px solid #007acc;
        border-radius: 8px;
        padding: 32px;
        margin-bottom: 32px;
      }

      .welcome-memory h2 {
        font-size: 24px;
        font-weight: 400;
        color: #e1e1e1;
        margin: 0 0 16px 0;
      }

      .memory-box p {
        font-size: 14px;
        color: #969696;
        margin: 0 0 16px 0;
      }

      .memory-command {
        display: flex;
        align-items: center;
        gap: 12px;
        background: #1a1a1a;
        border: 1px solid #3c3c3c;
        border-radius: 6px;
        padding: 16px;
      }

      .memory-command code {
        flex: 1;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
        font-size: 13px;
        color: #4ec9b0;
        word-break: break-all;
      }

      .copy-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 6px 12px;
        background: #007acc;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 13px;
        white-space: nowrap;
        transition: background 0.2s;
      }

      .copy-btn:hover {
        background: #1177bb;
      }

      .copy-btn.copied {
        background: #4ec9b0;
      }

      .welcome-footer {
        display: flex;
        flex-direction: column;
        gap: 20px;
        align-items: center;
      }

      .startup-preference {
        padding: 12px 0;
        border-top: 1px solid #3c3c3c;
        width: 100%;
        display: flex;
        justify-content: center;
      }

      .checkbox-label {
        display: flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        color: #969696;
        font-size: 13px;
        user-select: none;
      }

      .checkbox-label:hover {
        color: #cccccc;
      }

      .checkbox-label input[type="checkbox"] {
        width: 16px;
        height: 16px;
        cursor: pointer;
        accent-color: #007acc;
      }

      .welcome-actions {
        display: flex;
        justify-content: center;
        gap: 16px;
      }

      .action-btn {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 20px;
        background: #252526;
        color: #cccccc;
        border: 1px solid #3c3c3c;
        border-radius: 6px;
        cursor: pointer;
        font-size: 14px;
        transition: all 0.2s;
      }

      .action-btn:hover {
        background: #2a2d2e;
        border-color: #464647;
      }

      .action-btn.primary {
        background: #007acc;
        color: white;
        border-color: #007acc;
      }

      .action-btn.primary:hover {
        background: #1177bb;
        border-color: #1177bb;
      }

      /* Scrollbar styling */
      .welcome-page::-webkit-scrollbar {
        width: 14px;
      }

      .welcome-page::-webkit-scrollbar-track {
        background: #1e1e1e;
        border-left: 1px solid #3c3c3c;
      }

      .welcome-page::-webkit-scrollbar-thumb {
        background: #424242;
        border-radius: 7px;
        border: 3px solid #1e1e1e;
      }

      .welcome-page::-webkit-scrollbar-thumb:hover {
        background: #4a4a4a;
      }
    `;
  }
}

// Export singleton instance
export const welcomePage = new WelcomePage();