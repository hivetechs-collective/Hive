/**
 * Welcome Page Component
 * Displays when the app launches to orient new users
 */

export class WelcomePage {
  private container: HTMLElement | null = null;

  mount(container: HTMLElement): void {
    this.container = container;
    this.render();
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
            <h1 class="welcome-title">Welcome to Hive Consensus</h1>
            <p class="welcome-subtitle">Your Enterprise AI Development Platform</p>
          </div>

          <div class="welcome-features">
            <div class="feature-card">
              <div class="feature-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                </svg>
              </div>
              <h3>4-Stage Consensus Engine</h3>
              <p>Experience the power of multiple AI models working together to provide optimal code generation and refinement.</p>
            </div>

            <div class="feature-card">
              <div class="feature-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 11H3v2h6v-2zm0-4H3v2h6V7zm0 8H3v2h6v-2zm12-8h-6v2h6V7zm0 4h-6v2h6v-2zm0 4h-6v2h6v-2z"/>
                </svg>
              </div>
              <h3>Unified Memory System</h3>
              <p>Your conversations and context are preserved across all AI CLI tools through our innovative memory database.</p>
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
              <h3>AI CLI Tool Integration</h3>
              <p>Launch and manage Claude Code, Gemini, Grok, and other AI tools with automatic memory context injection.</p>
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

          <div class="welcome-actions">
            <button class="action-btn primary" id="open-documentation">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M14 1H2a1 1 0 00-1 1v8a1 1 0 001 1h5.5l3 3 3-3H14a1 1 0 001-1V2a1 1 0 00-1-1zM8 9.5a.5.5 0 110-1 .5.5 0 010 1zm1-2.5a1 1 0 01-2 0V4a1 1 0 012 0v3z"/>
              </svg>
              View Documentation
            </button>
            <button class="action-btn" id="open-folder-welcome">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M1.5 2a.5.5 0 00-.5.5v3a.5.5 0 00.146.354l5.5 5.5a.5.5 0 00.708 0l3-3a.5.5 0 000-.708l-5.5-5.5A.5.5 0 004.5 2h-3z"/>
                <path d="M5.5 3a.5.5 0 11-1 0 .5.5 0 011 0z"/>
                <path d="M6.854 11.854l5-5A.5.5 0 0011.5 6H7.707L5.854 7.854a.5.5 0 000 .708l1 1a.5.5 0 00.708 0z"/>
              </svg>
              Open Project Folder
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
    `;
  }

  private attachEventListeners(): void {
    if (!this.container) return;

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

    // Open folder button
    const folderBtn = this.container.querySelector('#open-folder-welcome');
    if (folderBtn) {
      folderBtn.addEventListener('click', () => {
        // Emit event to open folder dialog
        window.dispatchEvent(new CustomEvent('open-folder-dialog'));
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
        font-size: 48px;
        font-weight: 300;
        color: #e1e1e1;
        margin: 0 0 8px 0;
        letter-spacing: -1px;
      }

      .welcome-subtitle {
        font-size: 20px;
        color: #969696;
        margin: 0;
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
        border-radius: 8px;
        padding: 24px;
        transition: border-color 0.2s, transform 0.2s;
      }

      .feature-card:hover {
        border-color: #007acc;
        transform: translateY(-2px);
      }

      .feature-icon {
        width: 48px;
        height: 48px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #007acc;
        border-radius: 8px;
        margin-bottom: 16px;
      }

      .feature-icon svg {
        color: white;
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