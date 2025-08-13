/**
 * VS Code Style Source Control View
 * Exact implementation matching VS Code's SCM panel
 */

import { GitStatus, GitFileStatus } from './types/git';
import { GitDecorationProvider } from './git-decoration-provider';
import { GitGraphView } from './git-graph';
import { notifications } from './notification';

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
  private pendingOperations = new Set<string>(); // Track files being processed

  constructor(container: HTMLElement) {
    this.container = container;
    this.initialize();
  }

  private async initialize() {
    // Don't initialize Git decoration provider without a folder
    // It will be initialized when a folder is opened
    
    // Start auto-refresh
    await this.refresh();
    this.refreshInterval = setInterval(() => this.refresh(), 2000);
    
    // Set up global reference
    (window as any).scmView = this;
  }

  public async refresh() {
    try {
      this.gitStatus = await window.gitAPI.getStatus();
    } catch (error) {
      console.error('[SCM] Failed to refresh:', error);
      // Set gitStatus to null to show welcome message
      this.gitStatus = null;
    }
    // Always render, even if there was an error
    this.render();
  }

  private render() {
    if (!this.gitStatus || !this.gitStatus.isRepo) {
      // VS Code-style welcome message for Source Control
      this.container.innerHTML = `
        <div class="scm-view">
          <div class="scm-view-header">
            <div class="scm-provider-container">
              <div class="scm-provider">
                <span class="codicon codicon-source-control"></span>
                <span class="scm-provider-label">Source Control</span>
              </div>
            </div>
          </div>
          
          <div class="scm-welcome-view">
            <div class="scm-welcome-content">
              <div class="scm-welcome-icon">
                <span class="codicon codicon-source-control" style="font-size: 48px; opacity: 0.5;"></span>
              </div>
              <p class="scm-welcome-message">
                In order to use Git features, you can open a folder containing a Git repository or clone from a URL.
              </p>
              <button class="scm-welcome-button primary" onclick="window.openFolder()">
                Open Folder
              </button>
              <button class="scm-welcome-button" onclick="window.cloneRepository()">
                Clone Repository
              </button>
              <p class="scm-welcome-docs">
                To learn more about how to use Git and source control 
                <a href="https://code.visualstudio.com/docs/editor/versioncontrol" target="_blank" class="scm-welcome-link">read our docs</a>.
              </p>
            </div>
          </div>
        </div>
      `;
      
      // Add styles for the welcome view
      this.attachWelcomeStyles();
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
        <!-- Header with VS Code-style toolbar -->
        <div class="scm-view-header">
          <div class="scm-provider-container">
            <div class="scm-toolbar">
              <button class="scm-toolbar-button" title="Refresh" onclick="window.scmView?.refresh()">
                <span class="codicon codicon-refresh"></span>
              </button>
              <button class="scm-toolbar-button" title="Commit" onclick="window.scmView?.commit()">
                <span class="codicon codicon-check"></span>
              </button>
              <div class="scm-toolbar-separator"></div>
              <button class="scm-toolbar-button" title="Pull..." onclick="window.scmView?.pull()">
                <span class="codicon codicon-cloud-download"></span>
              </button>
              <button class="scm-toolbar-button" title="Push" onclick="window.scmView?.push()">
                <span class="codicon codicon-cloud-upload"></span>
              </button>
              <button class="scm-toolbar-button" title="Sync Changes" onclick="window.scmView?.sync()">
                <span class="codicon codicon-sync"></span>
              </button>
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
        </div>

        <!-- Resource Groups -->
        <div class="scm-view-content">
          ${groups.map(group => this.renderResourceGroup(group)).join('')}
        </div>
        
        <!-- Git Graph Section -->
        <div id="git-graph-container" style="
          border-top: 1px solid var(--vscode-sideBarSectionHeader-border, #1e1e1e);
          height: 65vh;
          min-height: 400px;
          max-height: 800px;
          overflow: hidden;
        "></div>

        <!-- Status Bar -->
        <div class="scm-status-bar">
          <div class="scm-status-branch">
            <span class="codicon codicon-git-branch"></span>
            <span>${this.gitStatus.branch}</span>
            ${this.gitStatus.ahead > 0 ? `<span class="badge">↑${this.gitStatus.ahead}</span>` : ''}
            ${this.gitStatus.behind > 0 ? `<span class="badge">↓${this.gitStatus.behind}</span>` : ''}
          </div>
          <div class="scm-status-actions">
            <!-- Removed redundant sync and refresh buttons -->
          </div>
        </div>
      </div>
    `;

    this.attachEventListeners();
    
    // Initialize Git Graph view if not already done
    // Use a longer delay to ensure Git status is fully loaded
    setTimeout(() => {
      const graphContainer = document.getElementById('git-graph-container');
      console.log('[SCM] Git graph container found:', !!graphContainer);
      console.log('[SCM] Git graph view exists:', !!this.gitGraphView);
      if (graphContainer && !this.gitGraphView) {
        console.log('[SCM] Creating new GitGraphView...');
        try {
          this.gitGraphView = new GitGraphView(graphContainer);
          console.log('[SCM] GitGraphView created successfully');
          // Immediately refresh to load commits
          this.gitGraphView.refresh();
        } catch (error) {
          console.error('[SCM] Failed to create GitGraphView:', error);
        }
      } else if (graphContainer && this.gitGraphView) {
        console.log('[SCM] Git graph already exists, refreshing...');
        this.gitGraphView.refresh();
      }
    }, 500); // Increased delay to ensure Git is fully ready
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
    // Use the actual repo path from git status, or return empty if no repo
    if (this.gitStatus && this.gitStatus.repoPath) {
      return this.gitStatus.repoPath.split('/').pop() || '';
    }
    return '';
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
    // Prevent double-clicks
    if (this.pendingOperations.has(path)) {
      console.log('[SCM] Operation already pending for:', path);
      return;
    }
    
    try {
      console.log('[SCM] Staging file:', path);
      this.pendingOperations.add(path);
      
      // Optimistically update UI immediately
      const fileStatus = this.gitStatus?.files.find(f => f.path === path);
      if (fileStatus) {
        // Move from working to index immediately in UI
        if (fileStatus.working !== ' ') {
          fileStatus.index = fileStatus.working;
          fileStatus.working = ' ';
        }
        this.render(); // Re-render immediately
      }
      
      // Perform actual git operation in background
      window.gitAPI.stage([path]).then(() => {
        console.log('[SCM] File staged successfully');
        this.pendingOperations.delete(path);
        // Refresh to get real status
        this.refresh();
      }).catch(error => {
        console.error('[SCM] Failed to stage:', error);
        this.pendingOperations.delete(path);
        alert(`Failed to stage file: ${error}`);
        // Revert optimistic update on error
        this.refresh();
      });
      
    } catch (error) {
      console.error('[SCM] Failed to stage:', error);
      this.pendingOperations.delete(path);
      alert(`Failed to stage file: ${error}`);
    }
  }

  public async unstageFile(path: string) {
    // Prevent double-clicks
    if (this.pendingOperations.has(path)) {
      console.log('[SCM] Operation already pending for:', path);
      return;
    }
    
    try {
      console.log('[SCM] Unstaging file:', path);
      this.pendingOperations.add(path);
      
      // Optimistically update UI immediately
      const fileStatus = this.gitStatus?.files.find(f => f.path === path);
      if (fileStatus) {
        // Move from index to working immediately in UI
        if (fileStatus.index !== ' ') {
          fileStatus.working = fileStatus.index === 'A' ? '?' : 'M';
          fileStatus.index = ' ';
        }
        this.render(); // Re-render immediately
      }
      
      // Perform actual git operation in background
      window.gitAPI.unstage([path]).then(() => {
        console.log('[SCM] File unstaged successfully');
        this.pendingOperations.delete(path);
        // Refresh to get real status
        this.refresh();
      }).catch(error => {
        console.error('[SCM] Failed to unstage:', error);
        this.pendingOperations.delete(path);
        alert(`Failed to unstage file: ${error}`);
        // Revert optimistic update on error
        this.refresh();
      });
      
    } catch (error) {
      console.error('[SCM] Failed to unstage:', error);
      this.pendingOperations.delete(path);
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
      notifications.show({
        title: 'Commit Failed',
        message: 'Please enter a commit message',
        type: 'warning',
        duration: 3000
      });
      return;
    }
    
    // Check if there are staged files
    const stagedFiles = this.gitStatus?.files.filter(f => f.index !== ' ' && f.index !== '?') || [];
    if (stagedFiles.length === 0) {
      notifications.show({
        title: 'No Staged Files',
        message: 'Please stage files before committing',
        type: 'warning',
        duration: 3000
      });
      return;
    }
    
    const notificationId = notifications.show({
      title: 'Committing',
      message: `Committing ${stagedFiles.length} file(s)...`,
      type: 'loading',
      duration: 0
    });
    
    try {
      await window.gitAPI.commit(this.commitMessage);
      const message = this.commitMessage;
      this.commitMessage = '';
      // Update the commit message input
      const input = this.container.querySelector('.scm-input') as HTMLTextAreaElement;
      if (input) {
        input.value = '';
      }
      await this.refresh();
      
      notifications.update(notificationId, {
        title: 'Commit Successful',
        message: `Committed: "${message.substring(0, 50)}${message.length > 50 ? '...' : ''}"`,
        type: 'success',
        duration: 3000
      });
    } catch (error: any) {
      console.error('Failed to commit:', error);
      notifications.update(notificationId, {
        title: 'Commit Failed',
        message: error?.message || 'An error occurred while committing',
        type: 'error',
        duration: 5000
      });
    }
  }

  public async commitAndPush() {
    await this.commit();
    await this.push();
  }

  public async push() {
    const branch = this.gitStatus?.branch || 'current branch';
    
    // Show loading notification
    const notificationId = notifications.show({
      title: 'Git Push',
      message: `Pushing ${branch} to remote...`,
      type: 'loading',
      duration: 0 // Persistent until updated
    });

    try {
      console.log('[SCM] Pushing to remote...');
      await window.gitAPI.push();
      await this.refresh();
      console.log('[SCM] Push successful');
      
      // Update to success notification
      notifications.update(notificationId, {
        title: 'Push Successful',
        message: `Successfully pushed ${branch} to remote`,
        type: 'success',
        duration: 3000
      });
    } catch (error: any) {
      console.error('Failed to push:', error);
      
      // Check if it's an upstream branch error
      if (error?.message?.includes('no upstream branch')) {
        notifications.update(notificationId, {
          title: 'Setting Upstream',
          message: `Setting upstream branch for ${branch}...`,
          type: 'info',
          duration: 5000
        });
        // The git-manager will handle setting upstream automatically
      } else {
        notifications.update(notificationId, {
          title: 'Push Failed',
          message: error?.message || 'An error occurred while pushing',
          type: 'error',
          duration: 5000
        });
      }
    }
  }

  public async pull() {
    const notificationId = notifications.show({
      title: 'Git Pull',
      message: 'Pulling from remote...',
      type: 'loading',
      duration: 0
    });

    try {
      await window.gitAPI.pull();
      await this.refresh();
      
      notifications.update(notificationId, {
        title: 'Pull Successful',
        message: 'Successfully pulled changes from remote',
        type: 'success',
        duration: 3000
      });
    } catch (error: any) {
      console.error('Failed to pull:', error);
      notifications.update(notificationId, {
        title: 'Pull Failed', 
        message: error?.message || 'An error occurred while pulling',
        type: 'error',
        duration: 5000
      });
    }
  }

  public async sync() {
    const notificationId = notifications.show({
      title: 'Git Sync',
      message: 'Synchronizing with remote...',
      type: 'loading',
      duration: 0
    });

    try {
      await window.gitAPI.pull();
      await window.gitAPI.push();
      await this.refresh();
      
      notifications.update(notificationId, {
        title: 'Sync Complete',
        message: 'Successfully synchronized with remote',
        type: 'success',
        duration: 3000
      });
    } catch (error: any) {
      console.error('Failed to sync:', error);
      notifications.update(notificationId, {
        title: 'Sync Failed',
        message: error?.message || 'An error occurred during sync',
        type: 'error',
        duration: 5000
      });
    }
  }
  
  public showMoreActions() {
    // TODO: Implement dropdown menu with additional Git actions
    console.log('More actions menu - to be implemented');
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

  private attachWelcomeStyles() {
    if (document.getElementById('scm-welcome-styles')) return;
    
    const style = document.createElement('style');
    style.id = 'scm-welcome-styles';
    style.textContent = `
      .scm-welcome-view {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: calc(100% - 35px);
        padding: 20px;
      }
      
      .scm-welcome-content {
        text-align: center;
        max-width: 400px;
      }
      
      .scm-welcome-icon {
        margin-bottom: 20px;
        color: var(--vscode-foreground);
      }
      
      .scm-welcome-message {
        font-size: 14px;
        color: var(--vscode-foreground);
        margin-bottom: 20px;
        line-height: 1.5;
      }
      
      .scm-welcome-button {
        display: block;
        width: 100%;
        padding: 6px 14px;
        margin: 8px 0;
        background: var(--vscode-button-secondaryBackground, #3a3d41);
        color: var(--vscode-button-secondaryForeground, #cccccc);
        border: 1px solid var(--vscode-button-border, transparent);
        border-radius: 2px;
        cursor: pointer;
        font-size: 13px;
        text-align: center;
      }
      
      .scm-welcome-button.primary {
        background: var(--vscode-button-background, #0e639c);
        color: var(--vscode-button-foreground, #ffffff);
      }
      
      .scm-welcome-button:hover {
        background: var(--vscode-button-hoverBackground, #1177bb);
      }
      
      .scm-welcome-docs {
        font-size: 13px;
        color: var(--vscode-descriptionForeground);
        line-height: 1.5;
      }
      
      .scm-welcome-link {
        color: var(--vscode-textLink-foreground);
        text-decoration: none;
      }
      
      .scm-welcome-link:hover {
        text-decoration: underline;
      }
    `;
    document.head.appendChild(style);
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