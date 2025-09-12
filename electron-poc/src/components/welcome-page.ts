export class WelcomePage {
  private container: HTMLElement;
  private recentItems: Array<{path: string, name: string, type: 'file' | 'folder', lastOpened?: Date}> = [];

  constructor(container: HTMLElement) {
    this.container = container;
    this.loadRecentItems();
  }

  private async loadRecentItems() {
    // Load recent folders from database
    try {
      if (window.databaseAPI) {
        // Get recent folders from database
        const recentFoldersJson = await window.databaseAPI.getSetting('recent.folders');
        if (recentFoldersJson) {
          const recentFolders = JSON.parse(recentFoldersJson);
          this.recentItems = recentFolders
            .filter((item: any) => item.path && item.name)
            .slice(0, 10) // Limit to 10 most recent
            .map((item: any) => ({
              path: item.path,
              name: item.name || item.path.split('/').pop(),
              type: 'folder' as const,
              lastOpened: item.lastOpened ? new Date(item.lastOpened) : new Date()
            }));
        } else {
          this.recentItems = [];
        }
      } else {
        // Fallback to empty if database not available
        this.recentItems = [];
      }
    } catch (error) {
      console.error('Failed to load recent folders:', error);
      this.recentItems = [];
    }
  }

  async render() {
    // Add styles
    const existingStyle = document.getElementById('welcome-page-styles');
    if (!existingStyle) {
      const style = document.createElement('style');
      style.id = 'welcome-page-styles';
      style.textContent = this.getStyles();
      document.head.appendChild(style);
    }

    // Determine layout based on recent items
    const hasRecentItems = this.recentItems.length > 0;
    const recentColumnWidth = hasRecentItems ? '60%' : '33.33%';

    this.container.innerHTML = `
      <div class="welcome-page">
        <div class="welcome-header">
          <button class="settings-btn" id="welcome-settings-btn" title="Settings">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="3"/>
              <path d="M12 1v6m0 6v6m4.22-13.22l4.24 4.24M1.54 1.54l4.24 4.24M20.46 20.46l-4.24-4.24M1.54 20.46l4.24-4.24M21 12h-6m-6 0H3"/>
            </svg>
          </button>
        </div>

        <div class="welcome-columns">
          <div class="welcome-column start-column" style="width: ${hasRecentItems ? '20%' : '33.33%'}">
            <h2 class="column-title">START</h2>
            <div class="column-content">
                <button class="action-item" id="new-project-btn">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 5v14m7-7H5"/>
                </svg>
                <span>New Project</span>
              </button>
              <button class="action-item" id="open-folder-btn">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
                </svg>
                <span>Open Folder</span>
              </button>
              <button class="action-item" id="getting-started-btn">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M9 19V6l12-3v13M9 12l12-3"/>
                </svg>
                <span>Getting Started</span>
              </button>
            </div>
          </div>

          <div class="welcome-column recent-column" style="width: ${recentColumnWidth}">
            <h2 class="column-title">RECENT</h2>
            <div class="column-content recent-list">
              ${this.recentItems.length > 0 ? this.recentItems.map(item => `
                <button class="recent-item" data-path="${item.path}">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
                  </svg>
                  <div class="recent-item-info">
                    <span class="recent-item-name">${item.name}</span>
                    <span class="recent-item-path">${item.path}</span>
                  </div>
                </button>
              `).join('') : `
                <div class="empty-state">
                  <p>No recent folders</p>
                  <p class="empty-hint">Open a folder to get started</p>
                </div>
              `}
              ${this.recentItems.length > 10 ? `
                <button class="show-all-btn">Show all ${this.recentItems.length} items...</button>
              ` : ''}
            </div>
          </div>

          <div class="welcome-column learn-column" style="width: ${hasRecentItems ? '20%' : '33.33%'}">
            <h2 class="column-title">LEARN</h2>
            <div class="column-content">
              <button class="action-item" id="shortcuts-btn">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="3" y="3" width="7" height="7"/>
                  <rect x="14" y="3" width="7" height="7"/>
                  <rect x="14" y="14" width="7" height="7"/>
                  <rect x="3" y="14" width="7" height="7"/>
                </svg>
                <span>Keyboard Shortcuts</span>
              </button>
              <button class="action-item" id="workflows-btn">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="16 18 22 12 16 6"/>
                  <polyline points="8 6 2 12 8 18"/>
                </svg>
                <span>AI Workflows</span>
              </button>
              <button class="action-item whats-new-btn" id="whats-new-btn">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                  <circle cx="12" cy="12" r="3"/>
                </svg>
                <span>What's New</span>
                <span class="badge">v1.8.326</span>
              </button>
            </div>
          </div>
        </div>

        <div class="welcome-footer">
          <label class="checkbox-container">
            <input type="checkbox" id="show-on-startup" checked>
            <span class="checkbox-label">Show on startup</span>
          </label>
        </div>
      </div>
    `;

    this.attachEventListeners();
    await this.loadPreferences();
  }

  private getStyles(): string {
    return `
      .welcome-page {
        display: flex;
        flex-direction: column;
        height: 100%;
        background: #1e1e1e;
        color: #cccccc;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        user-select: none;
      }

      .welcome-header {
        display: flex;
        justify-content: flex-end;
        align-items: center;
        padding: 15px 20px;
        border-bottom: 1px solid #2d2d30;
      }


      .settings-btn {
        background: none;
        border: none;
        color: #858585;
        cursor: pointer;
        padding: 8px;
        border-radius: 4px;
        transition: all 0.2s;
      }

      .settings-btn:hover {
        background: #2d2d30;
        color: #cccccc;
      }

      .welcome-columns {
        display: flex;
        flex: 1;
        overflow: hidden;
      }

      .welcome-column {
        display: flex;
        flex-direction: column;
        padding: 20px;
        border-right: 1px solid #2d2d30;
      }

      .welcome-column:last-child {
        border-right: none;
      }

      .column-title {
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 1px;
        color: #858585;
        margin: 0 0 20px 0;
        text-transform: uppercase;
      }

      .column-content {
        flex: 1;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        gap: 8px;
      }

      .action-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 12px;
        background: none;
        border: none;
        color: #cccccc;
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.2s;
        text-align: left;
        font-size: 13px;
      }

      .action-item:hover {
        background: #2d2d30;
      }

      .recent-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 12px;
        background: none;
        border: none;
        color: #cccccc;
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.2s;
        text-align: left;
        width: 100%;
      }

      .recent-item:hover {
        background: #2d2d30;
      }

      .recent-item-info {
        display: flex;
        flex-direction: column;
        flex: 1;
        min-width: 0;
      }

      .recent-item-name {
        font-size: 13px;
        font-weight: 500;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .recent-item-path {
        font-size: 11px;
        color: #858585;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        flex: 1;
        color: #858585;
        text-align: center;
        padding: 40px 20px;
      }

      .empty-state p {
        margin: 5px 0;
      }

      .empty-hint {
        font-size: 12px;
        opacity: 0.7;
      }

      .show-all-btn {
        margin-top: 10px;
        padding: 6px 12px;
        background: none;
        border: 1px solid #2d2d30;
        color: #858585;
        cursor: pointer;
        border-radius: 4px;
        font-size: 12px;
        transition: all 0.2s;
      }

      .show-all-btn:hover {
        background: #2d2d30;
        color: #cccccc;
      }

      .whats-new-btn {
        position: relative;
      }

      .badge {
        position: absolute;
        top: 4px;
        right: 4px;
        background: #FFC107;
        color: #1e1e1e;
        font-size: 9px;
        padding: 2px 4px;
        border-radius: 2px;
        font-weight: 600;
      }

      .welcome-footer {
        display: flex;
        justify-content: center;
        align-items: center;
        padding: 15px 30px;
        border-top: 1px solid #2d2d30;
        background: #252526;
      }

      .checkbox-container {
        display: flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        font-size: 13px;
        color: #cccccc;
      }

      .checkbox-container input[type="checkbox"] {
        width: 16px;
        height: 16px;
        cursor: pointer;
      }

      /* Responsive adjustments */
      @media (max-width: 800px) {
        .welcome-columns {
          flex-direction: column;
        }
        
        .welcome-column {
          width: 100% !important;
          border-right: none;
          border-bottom: 1px solid #2d2d30;
        }
      }

      /* Scrollbar styling */
      .column-content::-webkit-scrollbar {
        width: 10px;
      }

      .column-content::-webkit-scrollbar-track {
        background: transparent;
      }

      .column-content::-webkit-scrollbar-thumb {
        background: #424242;
        border-radius: 5px;
      }

      .column-content::-webkit-scrollbar-thumb:hover {
        background: #4a4a4d;
      }
    `;
  }

  private attachEventListeners() {
    // Settings button
    const settingsBtn = document.getElementById('welcome-settings-btn');
    settingsBtn?.addEventListener('click', () => {
      const event = new CustomEvent('showSettings');
      window.dispatchEvent(event);
    });

    // Start column actions
    document.getElementById('new-project-btn')?.addEventListener('click', () => {
      this.createNewProject();
    });

    document.getElementById('open-folder-btn')?.addEventListener('click', () => {
      this.openFolder();
    });

    document.getElementById('getting-started-btn')?.addEventListener('click', () => {
      this.showGettingStarted();
    });

    // Recent folders
    document.querySelectorAll('.recent-item').forEach(item => {
      item.addEventListener('click', (e) => {
        const target = e.currentTarget as HTMLElement;
        const path = target.dataset.path;
        if (path) {
          this.openRecentFolder(path);
        }
      });
    });

    // Learn column actions
    document.getElementById('shortcuts-btn')?.addEventListener('click', () => {
      this.showShortcuts();
    });

    document.getElementById('workflows-btn')?.addEventListener('click', () => {
      this.showWorkflows();
    });

    document.getElementById('whats-new-btn')?.addEventListener('click', () => {
      this.showWhatsNew();
    });

    // Footer checkbox only

    // Show on startup checkbox
    const checkbox = document.getElementById('show-on-startup') as HTMLInputElement;
    checkbox?.addEventListener('change', async (e) => {
      const target = e.target as HTMLInputElement;
      await this.savePreference(target.checked);
    });
  }

  private async loadPreferences() {
    const checkbox = this.container.querySelector('#show-on-startup') as HTMLInputElement;
    if (!checkbox) return;
    
    try {
      if (window.databaseAPI) {
        const value = await window.databaseAPI.getSetting('welcome.showOnStartup');
        const showOnStartup = !value || value !== '0';
        checkbox.checked = showOnStartup;
      }
    } catch (error) {
      console.error('Failed to load welcome preferences:', error);
      checkbox.checked = true;
    }
  }

  private async savePreference(showOnStartup: boolean): Promise<void> {
    try {
      if (window.databaseAPI) {
        await window.databaseAPI.setSetting(
          'welcome.showOnStartup',
          showOnStartup ? '1' : '0'
        );
        console.log('Welcome preference saved:', showOnStartup);
      }
    } catch (error) {
      console.error('Failed to save welcome preference:', error);
    }
  }

  private createNewProject() {
    console.log('Creating new project...');
    // Open folder dialog to create a new project
    const event = new CustomEvent('showExplorerWithDialog');
    window.dispatchEvent(event);
  }

  private showGettingStarted() {
    console.log('Showing getting started guide...');
    // Show internal documentation
    const event = new CustomEvent('showDocumentation', { detail: { section: 'getting-started' } });
    window.dispatchEvent(event);
  }

  private async openFolder() {
    console.log('Open folder...');
    // Show Explorer pane with folder dialog
    const event = new CustomEvent('showExplorerWithDialog');
    window.dispatchEvent(event);
  }

  private openRecentFolder(path: string) {
    console.log(`Opening folder: ${path}`);
    // Dispatch event to open the folder in Explorer pane
    const event = new CustomEvent('openFolderInExplorer', { detail: { path } });
    window.dispatchEvent(event);
  }

  private showShortcuts() {
    console.log('Showing keyboard shortcuts...');
    // Show internal documentation
    const event = new CustomEvent('showDocumentation', { detail: { section: 'shortcuts' } });
    window.dispatchEvent(event);
  }

  private showWorkflows() {
    console.log('Showing AI workflows...');
    // Show internal documentation
    const event = new CustomEvent('showDocumentation', { detail: { section: 'ai-workflows' } });
    window.dispatchEvent(event);
  }

  private showWhatsNew() {
    console.log('Showing what\'s new...');
    // Show internal documentation
    const event = new CustomEvent('showDocumentation', { detail: { section: 'whats-new' } });
    window.dispatchEvent(event);
  }
}