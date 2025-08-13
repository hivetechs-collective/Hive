/**
 * VS Code Style Source Control View
 * Exact implementation matching VS Code's SCM panel
 */

import { GitStatus, GitFileStatus } from './types/git';
import { GitDecorationProvider } from './git-decoration-provider';
import { GitGraphView } from './git-graph';

interface ResourceGroup {
  id: string;
  label: string;
  resources: GitFileStatus[];
  hideWhenEmpty?: boolean;
}

export class VSCodeSCMView {
  private container: HTMLElement;
  private gitStatus: GitStatus | null = null;
  private refreshInterval: NodeJS.Timeout | null = null;
  private selectedFiles: Set<string> = new Set();
  private commitMessage: string = '';
  private gitDecorationProvider: GitDecorationProvider | null = null;
  private gitGraphView: GitGraphView | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
    this.initialize();
  }

  private async initialize() {
    // Initialize Git decoration provider for the parent hive repo
    this.gitDecorationProvider = new GitDecorationProvider('/Users/veronelazio/Developer/Private/hive');
    await this.gitDecorationProvider.initialize();
    
    // Start auto-refresh
    await this.refresh();
    this.refreshInterval = setInterval(() => this.refresh(), 2000);
    
    // Set up global reference
    (window as any).scmView = this;
  }

  public async refresh() {
    try {
      this.gitStatus = await window.gitAPI.getStatus();
      this.render();
    } catch (error) {
      console.error('[SCM] Failed to refresh:', error);
    }
  }

  private render() {
    if (!this.gitStatus || !this.gitStatus.isRepo) {
      this.container.innerHTML = `
        <div class="scm-view-welcome">
          <div class="welcome-icon">
            <svg width="48" height="48" viewBox="0 0 24 24">
              <path fill="currentColor" d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
            </svg>
          </div>
          <h3>No source control provider registered.</h3>
          <p>Source control providers can be registered by extensions.</p>
        </div>
      `;
      return;
    }

    // Group resources
    const groups = this.groupResources();
    
    // Check if SCM view already exists - if so, just update the content
    const existingScmView = this.container.querySelector('.scm-view');
    if (existingScmView) {
      // Update only the file groups, not the entire view
      const contentElement = this.container.querySelector('.scm-view-content');
      if (contentElement) {
        contentElement.innerHTML = groups.map(group => this.renderResourceGroup(group)).join('');
        this.attachEventListeners();
        return; // Don't recreate the entire view
      }
    }
    
    // First render - create the entire structure
    this.container.innerHTML = `
      <div class="scm-view">
        <!-- Header -->
        <div class="scm-view-header">
          <div class="scm-provider-container">
            <div class="scm-provider">
              <span class="codicon codicon-source-control"></span>
              <span class="scm-provider-label">Git</span>
              <span class="scm-provider-path">${this.getRepoName()}</span>
            </div>
          </div>
        </div>

        <!-- Commit Input -->
        <div class="scm-input-container">
          <div class="scm-editor">
            <textarea 
              class="scm-input" 
              placeholder="Message (Ctrl+Enter to commit)"
              value="${this.commitMessage}"
              oninput="window.scmView?.updateCommitMessage(this.value)"
              onkeydown="window.scmView?.handleCommitKeydown(event)"
            >${this.commitMessage}</textarea>
            <div class="scm-input-counter ${this.commitMessage.length > 50 ? 'warn' : ''}">
              ${this.commitMessage.length}
            </div>
          </div>
          <div class="scm-input-actions">
            <button class="scm-action-button" title="Commit" onclick="window.scmView?.commit()">
              <span class="codicon codicon-check"></span>
            </button>
            <button class="scm-action-button" title="Commit and Push" onclick="window.scmView?.commitAndPush()">
              <span class="codicon codicon-cloud-upload"></span>
            </button>
            <button class="scm-action-button" title="More Actions">
              <span class="codicon codicon-ellipsis"></span>
            </button>
          </div>
        </div>

        <!-- Resource Groups -->
        <div class="scm-view-content">
          ${groups.map(group => this.renderResourceGroup(group)).join('')}
        </div>
        
        <!-- Git Graph Section -->
        <div id="git-graph-container" style="border-top: 1px solid var(--vscode-sideBarSectionHeader-border, #1e1e1e);"></div>

        <!-- Status Bar -->
        <div class="scm-status-bar">
          <div class="scm-status-branch">
            <span class="codicon codicon-git-branch"></span>
            <span>${this.gitStatus.branch}</span>
            ${this.gitStatus.ahead > 0 ? `<span class="badge">↑${this.gitStatus.ahead}</span>` : ''}
            ${this.gitStatus.behind > 0 ? `<span class="badge">↓${this.gitStatus.behind}</span>` : ''}
          </div>
          <div class="scm-status-actions">
            <button class="icon-button" title="Synchronize Changes" onclick="window.scmView?.sync()">
              <span class="codicon codicon-sync"></span>
            </button>
            <button class="icon-button" title="Refresh" onclick="window.scmView?.refresh()">
              <span class="codicon codicon-refresh"></span>
            </button>
          </div>
        </div>
      </div>
    `;

    this.attachEventListeners();
    
    // Initialize Git Graph view if not already done
    setTimeout(() => {
      const graphContainer = document.getElementById('git-graph-container');
      console.log('[SCM] Git graph container found:', !!graphContainer);
      console.log('[SCM] Git graph view exists:', !!this.gitGraphView);
      if (graphContainer && !this.gitGraphView) {
        console.log('[SCM] Creating new GitGraphView...');
        try {
          this.gitGraphView = new GitGraphView(graphContainer);
          console.log('[SCM] GitGraphView created successfully');
        } catch (error) {
          console.error('[SCM] Failed to create GitGraphView:', error);
        }
      } else if (graphContainer && this.gitGraphView) {
        console.log('[SCM] Git graph already exists, refreshing...');
        this.gitGraphView.refresh();
      }
    }, 100); // Small delay to ensure DOM is ready
  }

  private groupResources(): ResourceGroup[] {
    if (!this.gitStatus) return [];

    const groups: ResourceGroup[] = [];
    const { files } = this.gitStatus;

    // Staged Changes
    const staged = files.filter(f => f.index !== ' ' && f.index !== '?');
    if (staged.length > 0) {
      groups.push({
        id: 'staged',
        label: 'Staged Changes',
        resources: staged
      });
    }

    // Changes
    const changes = files.filter(f => 
      f.working !== ' ' && f.working !== '?' && !(f.index !== ' ' && f.index !== '?')
    );
    if (changes.length > 0) {
      groups.push({
        id: 'changes',
        label: 'Changes',
        resources: changes
      });
    }

    // Untracked
    const untracked = files.filter(f => f.working === '?');
    if (untracked.length > 0) {
      groups.push({
        id: 'untracked',
        label: 'Untracked',
        resources: untracked
      });
    }

    return groups;
  }

  private renderResourceGroup(group: ResourceGroup): string {
    const isExpanded = true; // For now, always expanded
    
    return `
      <div class="scm-resource-group" data-group="${group.id}">
        <div class="scm-resource-group-header">
          <div class="scm-resource-group-title">
            <span class="codicon codicon-chevron-${isExpanded ? 'down' : 'right'}"></span>
            <span class="label">${group.label}</span>
            <span class="count-badge">${group.resources.length}</span>
          </div>
          <div class="scm-resource-group-actions">
            ${group.id === 'changes' ? `
              <button class="icon-button" title="Stage All Changes" onclick="window.scmView?.stageAll()">
                <span class="codicon codicon-add"></span>
              </button>
            ` : group.id === 'staged' ? `
              <button class="icon-button" title="Unstage All" onclick="window.scmView?.unstageAll()">
                <span class="codicon codicon-remove"></span>
              </button>
            ` : ''}
            <button class="icon-button" title="Discard All Changes" onclick="window.scmView?.discardAll('${group.id}')">
              <span class="codicon codicon-discard"></span>
            </button>
          </div>
        </div>
        ${isExpanded ? `
          <div class="scm-resource-group-content">
            ${group.resources.map(resource => this.renderResource(resource, group.id)).join('')}
          </div>
        ` : ''}
      </div>
    `;
  }

  private renderResource(file: GitFileStatus, groupId: string): string {
    const fileName = file.path.split('/').pop() || file.path;
    const folderPath = file.path.includes('/') ? 
      file.path.substring(0, file.path.lastIndexOf('/')) : '';
    
    const statusIcon = this.getStatusIcon(file);
    const isSelected = this.selectedFiles.has(file.path);
    
    // Escape the path for use in onclick handlers
    const escapedPath = file.path.replace(/'/g, "\\'").replace(/"/g, '&quot;');

    return `
      <div class="scm-resource-item ${isSelected ? 'selected' : ''}" 
           data-path="${file.path}" 
           data-group="${groupId}">
        <div class="scm-resource-item-content" onclick="window.scmView?.openFile('${escapedPath}')" style="cursor: pointer; flex: 1;" title="${file.working === 'M' || file.index === 'M' || file.working === 'D' || file.index === 'D' ? 'Click to view diff' : 'Click to open file'}">
          <div class="scm-resource-decoration" title="${statusIcon.tooltip}">
            <span class="${statusIcon.className}">${statusIcon.letter}</span>
          </div>
          <div class="scm-resource-name">
            <span class="codicon codicon-file"></span>
            <span class="label">${fileName}</span>
            ${folderPath ? `<span class="description">${folderPath}</span>` : ''}
            ${file.working === 'M' || file.index === 'M' || file.working === 'D' || file.index === 'D' ? 
              '<span style="margin-left: 6px; opacity: 0.5; font-size: 10px;">(diff)</span>' : ''}
          </div>
        </div>
        <div class="scm-resource-actions">
          ${groupId === 'changes' || groupId === 'untracked' ? `
            <button class="icon-button" title="Stage Changes" onclick="event.stopPropagation(); window.scmView?.stageFile('${escapedPath}')">
              <span class="codicon codicon-add"></span>
            </button>
          ` : groupId === 'staged' ? `
            <button class="icon-button" title="Unstage Changes" onclick="event.stopPropagation(); window.scmView?.unstageFile('${escapedPath}')">
              <span class="codicon codicon-remove"></span>
            </button>
          ` : ''}
        </div>
      </div>
    `;
  }

  private getStatusIcon(file: GitFileStatus): { letter: string; className: string; tooltip: string } {
    if (file.index === 'A') return { letter: 'A', className: 'added', tooltip: 'Added' };
    if (file.index === 'M') return { letter: 'M', className: 'modified', tooltip: 'Modified (Staged)' };
    if (file.index === 'D') return { letter: 'D', className: 'deleted', tooltip: 'Deleted (Staged)' };
    if (file.index === 'R') return { letter: 'R', className: 'renamed', tooltip: 'Renamed' };
    if (file.working === 'M') return { letter: 'M', className: 'modified', tooltip: 'Modified' };
    if (file.working === 'D') return { letter: 'D', className: 'deleted', tooltip: 'Deleted' };
    if (file.working === '?') return { letter: 'U', className: 'untracked', tooltip: 'Untracked' };
    return { letter: '?', className: 'unknown', tooltip: 'Unknown' };
  }

  private getRepoName(): string {
    const path = '/Users/veronelazio/Developer/Private/hive/electron-poc';
    return path.split('/').pop() || 'Repository';
  }

  private attachEventListeners() {
    // Add context menu support
    const resourceItems = this.container.querySelectorAll('.scm-resource-item');
    resourceItems.forEach(item => {
      item.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        this.showContextMenu(e as MouseEvent, item as HTMLElement);
      });
      
      item.addEventListener('dblclick', () => {
        const path = item.getAttribute('data-path');
        if (path) this.openDiff(path);
      });
    });
  }

  private showContextMenu(event: MouseEvent, element: HTMLElement) {
    // TODO: Implement context menu
    console.log('Context menu for', element.getAttribute('data-path'));
  }

  // Public methods for event handlers
  public updateCommitMessage(message: string) {
    this.commitMessage = message;
    const counter = this.container.querySelector('.scm-input-counter');
    if (counter) {
      counter.textContent = String(message.length);
      counter.classList.toggle('warn', message.length > 50);
    }
  }

  public handleCommitKeydown(event: KeyboardEvent) {
    if (event.ctrlKey && event.key === 'Enter') {
      this.commit();
    }
  }

  public toggleFileSelection(path: string) {
    if (this.selectedFiles.has(path)) {
      this.selectedFiles.delete(path);
    } else {
      this.selectedFiles.add(path);
    }
  }

  public async stageFile(path: string) {
    try {
      console.log('[SCM] Staging file:', path);
      await window.gitAPI.stage([path]);
      await this.refresh();
      console.log('[SCM] File staged successfully');
    } catch (error) {
      console.error('[SCM] Failed to stage:', error);
      alert(`Failed to stage file: ${error}`);
    }
  }

  public async unstageFile(path: string) {
    try {
      console.log('[SCM] Unstaging file:', path);
      await window.gitAPI.unstage([path]);
      await this.refresh();
      console.log('[SCM] File unstaged successfully');
    } catch (error) {
      console.error('[SCM] Failed to unstage:', error);
      alert(`Failed to unstage file: ${error}`);
    }
  }

  public async discardFile(path: string) {
    if (confirm(`Discard changes to ${path}?`)) {
      try {
        await window.gitAPI.discard([path]);
        await this.refresh();
      } catch (error) {
        console.error('Failed to discard:', error);
      }
    }
  }

  public async stageAll() {
    try {
      const changes = this.gitStatus?.files.filter(f => 
        f.working !== ' ' && !(f.index !== ' ' && f.index !== '?')
      ) || [];
      await window.gitAPI.stage(changes.map(f => f.path));
      await this.refresh();
    } catch (error) {
      console.error('Failed to stage all:', error);
    }
  }

  public async unstageAll() {
    try {
      const staged = this.gitStatus?.files.filter(f => 
        f.index !== ' ' && f.index !== '?'
      ) || [];
      await window.gitAPI.unstage(staged.map(f => f.path));
      await this.refresh();
    } catch (error) {
      console.error('Failed to unstage all:', error);
    }
  }

  public async discardAll(groupId: string) {
    if (!confirm('Discard all changes?')) return;
    
    try {
      let files: string[] = [];
      if (groupId === 'changes') {
        files = this.gitStatus?.files
          .filter(f => f.working !== ' ' && f.working !== '?')
          .map(f => f.path) || [];
      }
      if (files.length > 0) {
        await window.gitAPI.discard(files);
        await this.refresh();
      }
    } catch (error) {
      console.error('Failed to discard all:', error);
    }
  }

  public async commit() {
    if (!this.commitMessage.trim()) {
      alert('Please enter a commit message');
      return;
    }
    
    try {
      await window.gitAPI.commit(this.commitMessage);
      this.commitMessage = '';
      await this.refresh();
    } catch (error) {
      console.error('Failed to commit:', error);
      alert(`Commit failed: ${error}`);
    }
  }

  public async commitAndPush() {
    await this.commit();
    await this.push();
  }

  public async push() {
    try {
      await window.gitAPI.push();
      await this.refresh();
    } catch (error) {
      console.error('Failed to push:', error);
      alert(`Push failed: ${error}`);
    }
  }

  public async sync() {
    try {
      await window.gitAPI.pull();
      await window.gitAPI.push();
      await this.refresh();
    } catch (error) {
      console.error('Failed to sync:', error);
    }
  }

  public async openFile(path: string) {
    try {
      // Convert relative path to absolute path for electron-poc
      const fullPath = path.startsWith('/') ? path : `/Users/veronelazio/Developer/Private/hive/${path}`;
      
      console.log('[SCM] Opening file:', fullPath);
      
      // Check if file has changes (modified, staged, etc.)
      const fileStatus = this.gitStatus?.files.find(f => 
        f.path === path || `/Users/veronelazio/Developer/Private/hive/${f.path}` === fullPath
      );
      
      if (fileStatus && (fileStatus.working === 'M' || fileStatus.index === 'M' || 
                         fileStatus.working === 'D' || fileStatus.index === 'D' ||
                         fileStatus.working === 'A' || fileStatus.index === 'A')) {
        // File has changes - show diff view
        console.log('[SCM] File has changes, showing diff view');
        await this.showDiffView(path, fileStatus);
      } else {
        // No changes or untracked file - open normally
        const content = await window.fileAPI.readFile(fullPath);
        
        // Open file in editor tabs
        if (window.editorTabs) {
          window.editorTabs.openFile(fullPath, content);
          console.log('[SCM] File opened in editor tabs');
        } else {
          console.error('[SCM] Editor tabs not available');
        }
      }
    } catch (error) {
      console.error('[SCM] Failed to open file:', error);
    }
  }
  
  private async showDiffView(path: string, fileStatus: any) {
    try {
      const fullPath = path.startsWith('/') ? path : `/Users/veronelazio/Developer/Private/hive/${path}`;
      
      // Get the diff for this file
      let diff: string;
      if (fileStatus.index !== ' ' && fileStatus.index !== '?') {
        // File is staged - get staged diff
        diff = await window.gitAPI.getStagedDiff(path);
      } else {
        // File is not staged - get working tree diff
        diff = await window.gitAPI.getDiff(path);
      }
      
      // Import and create diff viewer
      const { DiffViewer } = await import('./diff-viewer');
      const diffViewer = new DiffViewer();
      
      // Show the diff
      await diffViewer.showDiff(fullPath, diff);
      
      // Open diff viewer in a new tab
      if (window.editorTabs) {
        const container = diffViewer.getContainer();
        window.editorTabs.openDiffTab(fullPath + ' (diff)', container);
      }
      
    } catch (error) {
      console.error('[SCM] Failed to show diff view:', error);
      // Fallback to normal file opening
      const fullPath = path.startsWith('/') ? path : `/Users/veronelazio/Developer/Private/hive/${path}`;
      const content = await window.fileAPI.readFile(fullPath);
      
      if (window.editorTabs) {
        window.editorTabs.openFile(fullPath, content);
      }
    }
  }

  public openDiff(path: string) {
    // TODO: Open diff view
    console.log('Open diff for', path);
  }

  public destroy() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
    }
    if (this.gitDecorationProvider) {
      this.gitDecorationProvider.dispose();
    }
    if (this.gitGraphView) {
      this.gitGraphView.destroy();
    }
  }
}