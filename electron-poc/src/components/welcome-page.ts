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
        // Try structured recent_folders table first
        try {
          const api: any = window.databaseAPI as any;
          if (api.getRecentFolders) {
            const rows = await api.getRecentFolders();
            if (Array.isArray(rows)) {
              this.recentItems = rows.map((row: any) => ({
                path: row.folder_path,
                name: (row.folder_path || '').split('/').pop() || row.folder_path,
                type: 'folder' as const,
                lastOpened: row.last_opened ? new Date(row.last_opened) : undefined
              }));
              return;
            }
          }
        } catch (e) {
          console.warn('[Welcome] Structured recent folders failed, falling back:', e);
        }
        // Get recent folders from database
        const recentFoldersJson = await window.databaseAPI.getSetting('recent.folders');
        if (recentFoldersJson) {
          const recentFolders = JSON.parse(recentFoldersJson);
          this.recentItems = recentFolders
            .filter((item: any) => item.path && (item.name || item.path))
            .slice(0, 20) // Limit to 20 most recent (per architecture spec)
            .map((item: any) => ({
              path: item.path,
              name: item.name || item.path.split('/').pop(),
              type: 'folder' as const,
              lastOpened: item.lastOpened ? new Date(item.lastOpened) : undefined
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
    const recentColumnWidth = hasRecentItems ? '70%' : '33.33%';
    const sideColumnWidth = hasRecentItems ? '15%' : '33.33%';

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
          <div class="welcome-column start-column" style="width: ${sideColumnWidth}">
            <h2 class="column-title">START</h2>
            <div class="column-content">
                <button class="action-item" id="new-project-btn">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 5v14m7-7H5"/>
                </svg>
                <span>New Project</span>
              </button>
              <button class="action-item" id="clone-repo-btn">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M3 7h18M3 12h18M3 17h18"/>
                </svg>
                <span>Clone Repository</span>
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
              <div class="dropzone" id="welcome-dropzone">
                <div class="dropzone-icon">📁</div>
                <div class="dropzone-text">Drop a folder to open</div>
              </div>
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
              ${this.recentItems.length > 20 ? `
                <button class="show-all-btn">Show all ${this.recentItems.length} items...</button>
              ` : ''}
            </div>
          </div>

          <div class="welcome-column learn-column" style="width: ${sideColumnWidth}">
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
                <span class="badge" id="whats-new-badge" style="display:none"></span>
              </button>
            </div>
          </div>
        </div>

        <div class="welcome-footer">
          <div class="footer-left">
            <label class="checkbox-container">
              <input type="checkbox" id="show-on-startup" checked>
              <span class="checkbox-label">Show on startup</span>
            </label>
          </div>
          <div class="footer-right">
            ${hasRecentItems ? '<button class="footer-btn" id="restore-session-btn">Restore Session</button>' : ''}
            <button class="footer-btn" id="open-recent-btn">Open Recent ▾</button>
            <button class="footer-btn" id="show-all-recents-btn">Show All…</button>
            <button class="footer-btn" id="clear-recents-btn">Clear</button>
          </div>
        </div>

        <!-- Popover for Open Recent -->
        <div class="recent-popover" id="recent-popover" style="display:none"></div>
      </div>
    `;

    this.attachEventListeners();
    this.enableDragAndDrop();
    await this.loadPreferences();
    await this.updateWhatsNewBadge();
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
        justify-content: space-between;
        align-items: center;
        padding: 15px 30px;
        border-top: 1px solid #2d2d30;
        background: #252526;
      }
      .footer-right {
        display: flex;
        gap: 8px;
      }
      .footer-btn {
        background: #2d2d30;
        border: 1px solid #3a3a3a;
        color: #ccc;
        padding: 6px 10px;
        border-radius: 4px;
        cursor: pointer;
        font-size: 12px;
      }
      .footer-btn:hover {
        background: #38383b;
      }

      .recent-popover {
        position: absolute;
        bottom: 58px;
        right: 30px;
        background: #1f1f1f;
        border: 1px solid #2d2d30;
        border-radius: 6px;
        min-width: 360px;
        max-width: 520px;
        box-shadow: 0 10px 30px rgba(0,0,0,0.35);
        z-index: 1000;
      }
      .recent-popover ul {
        list-style: none;
        margin: 0;
        padding: 6px 0;
        max-height: 300px;
        overflow: auto;
      }
      .recent-popover li {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 12px;
        cursor: pointer;
      }
      .recent-popover li:hover {
        background: #2d2d30;
      }
      .recent-popover .item-main {
        display: flex;
        flex-direction: column;
        flex: 1;
        min-width: 0;
      }
      .recent-popover .item-name {
        font-size: 13px;
        overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
      }
      .recent-popover .item-path {
        font-size: 11px; color: #858585;
        overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
      }
      .recent-popover .remove-btn {
        color: #999; border: none; background: transparent; cursor: pointer;
      }

      /* Drag-and-drop dropzone */
      .dropzone {
        margin-top: 12px;
        padding: 14px;
        border: 1px dashed #3a3a3a;
        border-radius: 6px;
        text-align: center;
        color: #9aa0a6;
        background: rgba(255,255,255,0.02);
        transition: border-color 0.15s ease, background 0.15s ease, color 0.15s ease;
        user-select: none;
      }
      .dropzone:hover {
        border-color: #4a4a4d;
        color: #bcbcbc;
      }
      .dropzone.dragover {
        border-color: #FFC107;
        background: rgba(255,193,7,0.08);
        color: #ffd777;
      }
      .dropzone-icon {
        font-size: 18px;
        margin-bottom: 6px;
      }
      .dropzone-text {
        font-size: 12px;
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

    document.getElementById('clone-repo-btn')?.addEventListener('click', () => {
      (window as any).cloneRepository?.();
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
      // Mark current version as seen to clear badge
      (async () => {
        try {
          const version = await (window as any).electronAPI?.getVersion?.();
          if (version && (window as any).databaseAPI?.setSetting) {
            await (window as any).databaseAPI.setSetting('welcome.lastSeenVersion', version);
          }
          await this.updateWhatsNewBadge();
        } catch {}
      })();
    });

    // Footer checkbox only

    // Show on startup checkbox
    const checkbox = document.getElementById('show-on-startup') as HTMLInputElement;
    checkbox?.addEventListener('change', async (e) => {
      const target = e.target as HTMLInputElement;
      await this.savePreference(target.checked);
    });

    // Open Recent popover
    const openRecentBtn = document.getElementById('open-recent-btn');
    const popover = document.getElementById('recent-popover');
    openRecentBtn?.addEventListener('click', (e) => {
      e.stopPropagation();
      if (!popover) return;
      if (popover.style.display === 'none' || !popover.style.display) {
        this.renderRecentPopover();
        popover.style.display = 'block';
      } else {
        popover.style.display = 'none';
      }
    });
    // Hide popover when clicking outside
    document.addEventListener('click', (e) => {
      if (!popover) return;
      const target = e.target as HTMLElement;
      if (popover.style.display === 'block' && !popover.contains(target) && target.id !== 'open-recent-btn') {
        popover.style.display = 'none';
      }
    });

    // Show All recents modal
    document.getElementById('show-all-recents-btn')?.addEventListener('click', () => {
      this.showAllRecentsModal();
    });

    // Clear recents
    document.getElementById('clear-recents-btn')?.addEventListener('click', async () => {
      try {
        if ((window as any).databaseAPI?.clearRecentFolders) {
          await (window as any).databaseAPI.clearRecentFolders();
        }
        this.recentItems = [];
        this.render();
      } catch (err) {
        console.error('Failed to clear recent folders:', err);
      }
    });

    // Restore last session (most recent folder)
    document.getElementById('restore-session-btn')?.addEventListener('click', () => {
      if (this.recentItems.length > 0) {
        const p = this.recentItems[0].path;
        this.openRecentFolder(p);
      }
    });
  }

  private renderRecentPopover() {
    const pop = document.getElementById('recent-popover');
    if (!pop) return;
    const items = this.recentItems.slice(0, 10);
    if (items.length === 0) {
      pop.innerHTML = '<div style="padding:10px;color:#858585">No recent folders</div>';
      return;
    }
    const list = items.map(item => `
      <li data-path="${item.path}">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        </svg>
        <div class="item-main">
          <div class="item-name">${item.name}</div>
          <div class="item-path">${item.path}</div>
        </div>
        <button class="remove-btn" title="Remove" data-remove="${item.path}">✕</button>
      </li>
    `).join('');
    pop.innerHTML = `<ul>${list}</ul>`;
    // Wire clicks
    pop.querySelectorAll('li').forEach(li => {
      li.addEventListener('click', (e) => {
        const path = (e.currentTarget as HTMLElement).dataset.path;
        if (!path) return;
        this.openRecentFolder(path);
        pop.style.display = 'none';
      });
    });
    pop.querySelectorAll('button[data-remove]').forEach(btn => {
      btn.addEventListener('click', async (e) => {
        e.stopPropagation();
        const path = (e.currentTarget as HTMLElement).getAttribute('data-remove');
        if (!path) return;
        try {
          if ((window as any).databaseAPI?.removeRecentFolder) {
            await (window as any).databaseAPI.removeRecentFolder(path);
          }
          this.recentItems = this.recentItems.filter(i => i.path !== path);
          this.renderRecentPopover();
        } catch (err) {
          console.error('Failed to remove recent folder:', err);
        }
      });
    });
  }

  private showAllRecentsModal() {
    const overlay = document.createElement('div');
    overlay.style.position = 'fixed';
    overlay.style.inset = '0';
    overlay.style.background = 'rgba(0,0,0,0.5)';
    overlay.style.zIndex = '2000';

    const modal = document.createElement('div');
    modal.style.width = '700px';
    modal.style.maxHeight = '70vh';
    modal.style.overflow = 'auto';
    modal.style.background = '#1f1f1f';
    modal.style.border = '1px solid #2d2d30';
    modal.style.borderRadius = '8px';
    modal.style.margin = '10vh auto';
    modal.style.padding = '16px';

    const header = document.createElement('div');
    header.style.display = 'flex';
    header.style.justifyContent = 'space-between';
    header.style.alignItems = 'center';
    header.innerHTML = `
      <div style="font-weight:600">All Recent Folders</div>
      <input id="recent-search" placeholder="Search…" style="background:#2a2a2e;border:1px solid #3a3a3a;color:#ccc;border-radius:4px;padding:6px 8px;width: 260px;" />
    `;

    const list = document.createElement('div');
    const renderList = (q: string = '') => {
      const ql = q.toLowerCase();
      const items = this.recentItems.filter(i => i.name.toLowerCase().includes(ql) || i.path.toLowerCase().includes(ql));
      list.innerHTML = items.map(i => `
        <div class="recent-row" data-path="${i.path}" style="display:flex;align-items:center;gap:10px;padding:8px;border-bottom:1px solid #2d2d30;cursor:pointer">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>
          <div style="flex:1; min-width:0">
            <div style="font-size:13px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${i.name}</div>
            <div style="font-size:11px;color:#858585;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">${i.path}</div>
          </div>
          <button class="footer-btn" data-open="${i.path}">Open</button>
          <button class="footer-btn" data-terminal="${i.path}">Terminal</button>
          <button class="footer-btn" data-remove="${i.path}">Remove</button>
        </div>
      `).join('');
      list.querySelectorAll('button[data-open]').forEach(btn => btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const p = (e.currentTarget as HTMLElement).getAttribute('data-open');
        if (p) this.openRecentFolder(p);
        document.body.removeChild(overlay);
      }));
      list.querySelectorAll('button[data-terminal]').forEach(btn => btn.addEventListener('click', async (e) => {
        e.stopPropagation();
        const p = (e.currentTarget as HTMLElement).getAttribute('data-terminal');
        if (!p) return;
        try {
          const id = `term-${Date.now()}`;
          await (window as any).terminalAPI?.createTerminalProcess?.({ terminalId: id, cwd: p, command: undefined });
        } catch (err) { console.error('Failed to open terminal:', err); }
      }));

      list.querySelectorAll('button[data-remove]').forEach(btn => btn.addEventListener('click', async (e) => {
        e.stopPropagation();
        const p = (e.currentTarget as HTMLElement).getAttribute('data-remove');
        if (!p) return;
        try {
          if ((window as any).databaseAPI?.removeRecentFolder) {
            await (window as any).databaseAPI.removeRecentFolder(p);
          }
          this.recentItems = this.recentItems.filter(i => i.path !== p);
          renderList((document.getElementById('recent-search') as HTMLInputElement)?.value || '');
        } catch (err) { console.error(err); }
      }));
      list.querySelectorAll('.recent-row').forEach(row => row.addEventListener('click', (e) => {
        const p = (e.currentTarget as HTMLElement).getAttribute('data-path');
        if (p) this.openRecentFolder(p);
        document.body.removeChild(overlay);
      }));
    };

    modal.appendChild(header);
    modal.appendChild(list);
    overlay.appendChild(modal);
    document.body.appendChild(overlay);

    (document.getElementById('recent-search') as HTMLInputElement)?.addEventListener('input', (e) => {
      renderList((e.target as HTMLInputElement).value);
    });

    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) document.body.removeChild(overlay);
    });

    renderList('');
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

  private async updateWhatsNewBadge() {
    try {
      const version = await (window as any).electronAPI?.getVersion?.();
      if (!version) return;
      const lastSeen = await window.databaseAPI.getSetting('welcome.lastSeenVersion');
      const badge = document.getElementById('whats-new-badge') as HTMLElement;
      if (!badge) return;
      if (lastSeen !== version) {
        badge.textContent = `v${version}`;
        badge.style.display = 'inline-block';
      } else {
        badge.style.display = 'none';
      }
    } catch (e) {
      // On error, hide badge
      const badge = document.getElementById('whats-new-badge') as HTMLElement;
      if (badge) badge.style.display = 'none';
    }
  }

  private createNewProject() {
    console.log('Creating new project...');
    // Guided flow with minimal prompts + optional template scaffold
    (async () => {
      try {
        const name = await (window as any).electronAPI.showInputDialog('New Project', 'Enter project name:');
        if (!name) return;
        const template = await (window as any).electronAPI.showInputDialog('Project Template', 'Enter template (node|python|rust|empty):');
        const tpl = (template || 'empty').trim().toLowerCase();
        const destSel = await (window as any).electronAPI.showOpenDialog({
          properties: ['openDirectory', 'createDirectory'],
          title: 'Select location for project'
        });
        if (destSel.canceled || destSel.filePaths.length === 0) return;
        const parent = destSel.filePaths[0];
        const projectPath = `${parent}/${name}`;
        
        const exists = await (window as any).fileAPI.fileExists(projectPath);
        if (!exists) {
          await (window as any).fileAPI.createFolder(parent, name);
        }
        
        // Scaffold minimal template
        try {
          if (tpl === 'node') {
            await (window as any).fileAPI.writeFile(`${projectPath}/package.json`, JSON.stringify({ name, version: '0.1.0', scripts: { start: 'node index.js' } }, null, 2));
            await (window as any).fileAPI.writeFile(`${projectPath}/index.js`, "console.log('Hello from Node project');\n");
            await (window as any).fileAPI.writeFile(`${projectPath}/.gitignore`, "node_modules\n.DS_Store\n");
          } else if (tpl === 'python') {
            await (window as any).fileAPI.writeFile(`${projectPath}/main.py`, "print('Hello from Python project')\n");
            await (window as any).fileAPI.writeFile(`${projectPath}/.gitignore`, "__pycache__/\n.DS_Store\n");
          } else if (tpl === 'rust') {
            // Minimal Cargo setup
            await (window as any).fileAPI.createFolder(projectPath, 'src');
            await (window as any).fileAPI.writeFile(`${projectPath}/Cargo.toml`, `[package]\nname = "${name}"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n`);
            await (window as any).fileAPI.writeFile(`${projectPath}/src/main.rs`, "fn main() {\n    println!(\"Hello from Rust project\");\n}\n");
            await (window as any).fileAPI.writeFile(`${projectPath}/.gitignore`, "target/\n.DS_Store\n");
          } else {
            await (window as any).fileAPI.writeFile(`${projectPath}/README.md`, `# ${name}\n\nCreated with Hive Consensus IDE.`);
          }
        } catch (e) {
          console.warn('[Welcome] Template scaffold failed:', e);
        }

        try { await (window as any).gitAPI.initRepo(projectPath); } catch {}
        
        this.openRecentFolder(projectPath);
      } catch (e) {
        console.error('New project flow failed:', e);
      }
    })();
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
    // Lightweight in-app shortcuts modal
    const overlay = document.createElement('div');
    overlay.style.position = 'fixed';
    overlay.style.inset = '0';
    overlay.style.background = 'rgba(0,0,0,0.5)';
    overlay.style.zIndex = '2000';

    const modal = document.createElement('div');
    modal.style.width = '560px';
    modal.style.background = '#1f1f1f';
    modal.style.border = '1px solid #2d2d30';
    modal.style.borderRadius = '8px';
    modal.style.margin = '15vh auto';
    modal.style.padding = '16px';
    modal.innerHTML = `
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px">
        <div style="font-weight:600">Keyboard Shortcuts</div>
        <button class="footer-btn" id="shortcuts-open-docs">Open Full Cheatsheet</button>
      </div>
      <div style="display:grid;grid-template-columns: 1fr auto;gap:6px 16px">
        <div>Open Folder</div><div>Cmd/Ctrl+O</div>
        <div>Show Welcome</div><div>Cmd/Ctrl+Shift+W</div>
        <div>Go to File</div><div>Cmd/Ctrl+P</div>
        <div>Go to Line</div><div>Cmd/Ctrl+G</div>
        <div>Toggle Terminal</div><div>Cmd/Ctrl+`</div>
        <div>Save</div><div>Cmd/Ctrl+S</div>
      </div>
    `;

    overlay.appendChild(modal);
    document.body.appendChild(overlay);

    overlay.addEventListener('click', (e) => { if (e.target === overlay) document.body.removeChild(overlay); });
    document.getElementById('shortcuts-open-docs')?.addEventListener('click', () => {
      const event = new CustomEvent('showDocumentation', { detail: { section: 'shortcuts' } });
      window.dispatchEvent(event);
      document.body.removeChild(overlay);
    });
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
  
  // Enable drag-and-drop to open a folder quickly
  private enableDragAndDrop() {
    const root = this.container.querySelector('.welcome-page') as HTMLElement;
    const dropzone = this.container.querySelector('#welcome-dropzone') as HTMLElement;
    if (!root || !dropzone) return;

    const stop = (e: Event) => { e.preventDefault(); e.stopPropagation(); };
    ['dragenter','dragover','dragleave','drop'].forEach(type => root.addEventListener(type, stop));

    // Visual highlight
    const add = () => dropzone.classList.add('dragover');
    const remove = () => dropzone.classList.remove('dragover');
    ['dragenter','dragover'].forEach(t => root.addEventListener(t, add));
    ['dragleave','drop'].forEach(t => root.addEventListener(t, remove));

    const handleDrop = (e: DragEvent) => {
      if (!e.dataTransfer) return;
      const file = e.dataTransfer.files?.[0];
      if (!file) return;
      const path = (file as any).path as string;
      if (path) this.openRecentFolder(path);
    };
    dropzone.addEventListener('drop', handleDrop);
    root.addEventListener('drop', handleDrop);
  }
}
