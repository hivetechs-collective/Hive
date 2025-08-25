/// <reference path="./types/window.d.ts" />

/**
 * VS Code Style Source Control View
 * Exact implementation matching VS Code's SCM panel
 */

import { GitStatus, GitFileStatus } from './types/git';
import { GitDecorationProvider } from './git-decoration-provider';
import { GitGraphView } from './git-graph';
import { notifications } from './notification';
import { GitErrorHandler, GitErrorOptions } from './git-error-handler';
import { GitPushStrategyAnalyzer } from './git-push-strategy';
import { GitPushDialog } from './git-push-dialog';
import { GitPushExecutor } from './git-push-executor';
import { GitConsensusAdvisor } from './git-consensus-advisor';

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
    
    // Initial refresh only - no auto-refresh interval
    await this.refresh();
    // Auto-refresh removed - will refresh on Git operations and file saves only
    
    // Global reference is now set in renderer.ts when creating the instance
  }

  public async refresh() {
    console.log('[SCM] Refresh button clicked!');
    try {
      // Check if a folder is open first
      const currentFolder = (window as any).currentOpenedFolder;
      if (!currentFolder) {
        console.log('[SCM] No folder open, showing welcome view');
        this.gitStatus = null;
        this.render();
        return;
      }
      
      // Fetch from remote to get latest ahead/behind counts
      console.log('[SCM] Fetching from remote to get latest status...');
      try {
        await window.gitAPI.fetch();
        console.log('[SCM] Fetch completed successfully');
      } catch (fetchError) {
        console.log('[SCM] Fetch failed (might be offline):', fetchError);
        // Continue anyway - we can still show local status
      }
      
      console.log('[SCM] Getting git status for:', currentFolder);
      this.gitStatus = await window.gitAPI.getStatus();
      console.log('[SCM] Got git status with', this.gitStatus?.files?.length || 0, 'files');
      console.log('[SCM] Branch:', this.gitStatus?.branch, 'Ahead:', this.gitStatus?.ahead, 'Behind:', this.gitStatus?.behind);
      
      // Check for untracked files
      const untracked = this.gitStatus?.files?.filter(f => (f.working_dir === '?' || f.working === '?') && f.index === '?') || [];
      console.log('[SCM] Untracked files:', untracked.map(f => f.path));
    } catch (error) {
      console.error('[SCM] Failed to refresh:', error);
      // Set gitStatus to null to show welcome message
      this.gitStatus = null;
    }
    // Always render, even if there was an error
    this.render();
    console.log('[SCM] Render complete');
  }

  private render() {
    console.log('[SCM] Rendering with status - ahead:', this.gitStatus?.ahead, 'behind:', this.gitStatus?.behind, 'branch:', this.gitStatus?.branch);
    
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
              <button class="scm-toolbar-button" title="Pull${this.gitStatus?.behind ? ` (${this.gitStatus.behind} behind)` : ''}" onclick="window.scmView?.pull()">
                <span class="codicon codicon-cloud-download"></span>
              </button>
              <button class="scm-toolbar-button" title="Push${this.gitStatus?.ahead ? ` (${this.gitStatus.ahead} ahead)` : ''}" onclick="window.scmView?.push()">
                <span class="codicon codicon-cloud-upload"></span>
              </button>
              <button class="scm-toolbar-button" title="Sync Changes${this.gitStatus?.ahead || this.gitStatus?.behind ? ` (${this.gitStatus.ahead || 0}↑ ${this.gitStatus.behind || 0}↓)` : ''}" onclick="window.scmView?.sync()">
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
        <div class="scm-view-content" style="
          flex: 1 1 auto;
          overflow-y: auto;
          overflow-x: hidden;
          min-height: 0;
          max-height: calc(100vh - 400px);
        ">
          ${groups.map(group => this.renderResourceGroup(group)).join('')}
        </div>
        
        <!-- Git Graph Section -->
        <div id="git-graph-container" style="
          border-top: 1px solid var(--vscode-sideBarSectionHeader-border, #1e1e1e);
          height: 300px;
          flex-shrink: 0;
          overflow: hidden;
        "></div>

        <!-- Status Bar -->
        <div class="scm-status-bar">
          <div class="scm-status-branch" style="position: relative;">
            <span class="codicon codicon-git-branch"></span>
            <span class="branch-switcher" style="cursor: pointer; text-decoration: underline;" onclick="window.scmView?.showBranchSwitcher?.()">${this.gitStatus.branch}</span>
            <span class="badge" style="background: #007acc; color: white; padding: 2px 6px; border-radius: 10px; margin-left: 8px; font-size: 11px; cursor: ${(this.gitStatus.ahead || 0) > 0 ? 'pointer' : 'default'};" 
                  onclick="${(this.gitStatus.ahead || 0) > 0 ? 'window.scmView?.push()' : ''}"
                  title="${(this.gitStatus.ahead || 0) > 0 ? 'Click to push' : 'Nothing to push'}">↑${this.gitStatus.ahead || 0}</span>
            <span class="badge" style="background: #f48771; color: white; padding: 2px 6px; border-radius: 10px; margin-left: 4px; font-size: 11px; cursor: ${(this.gitStatus.behind || 0) > 0 ? 'pointer' : 'default'};" 
                  onclick="${(this.gitStatus.behind || 0) > 0 ? 'window.scmView?.pullAndPush()' : ''}"
                  title="${(this.gitStatus.behind || 0) > 0 ? 'Click to sync (pull then push)' : 'Up to date'}">↓${this.gitStatus.behind || 0}</span>
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
      console.log('[SCM] Window gitGraph exists:', !!(window as any).gitGraph);
      
      if (graphContainer && !this.gitGraphView) {
        console.log('[SCM] Creating new GitGraphView...');
        try {
          this.gitGraphView = new GitGraphView(graphContainer);
          (window as any).gitGraph = this.gitGraphView; // Ensure global reference
          console.log('[SCM] GitGraphView created successfully');
          // Immediately refresh to load commits
          setTimeout(() => {
            console.log('[SCM] Calling refresh on Git graph...');
            this.gitGraphView?.refresh();
          }, 2000);  // Increased delay from 100ms to 2s to not block UI
        } catch (error) {
          console.error('[SCM] Failed to create GitGraphView:', error);
        }
      } else if (graphContainer && this.gitGraphView) {
        console.log('[SCM] Git graph already exists, refreshing...');
        this.gitGraphView.refresh();
      }
    }, 1000); // Increased delay to ensure Git is fully ready
  }

  private groupResources(): ResourceGroup[] {
    if (!this.gitStatus) return [];

    const groups: ResourceGroup[] = [];
    const { files } = this.gitStatus;
    
    // Debug logging
    console.log('[SCM] Grouping files:', files.map(f => ({
      path: f.path,
      index: f.index || ' ',
      working: f.working_dir || f.working || ' ',
      indexChar: f.index ? f.index.charCodeAt(0) : 32,
      workingChar: (f.working_dir || f.working) ? (f.working_dir || f.working).charCodeAt(0) : 32
    })));

    // Staged Changes (Index Group in VS Code)
    const staged = files.filter(f => f.index !== ' ' && f.index !== '?');
    if (staged.length > 0) {
      console.log('[SCM] Staged files:', staged.map(f => f.path));
      groups.push({
        id: 'staged',
        label: 'Staged Changes',
        resources: staged
      });
    }

    // Changes (Working Tree Group in VS Code - modified/deleted files)
    const changes = files.filter(f => {
      // Has working tree changes but not untracked
      // Note: simple-git uses 'working_dir' not 'working'
      const working = f.working_dir || f.working || ' ';
      const hasWorkingChanges = working !== ' ' && working !== '?';
      // Not staged
      const notStaged = f.index === ' ';
      const isInChanges = hasWorkingChanges && notStaged;
      console.log(`[SCM] File ${f.path}: working='${working}' index='${f.index}' -> inChanges=${isInChanges}`);
      return isInChanges;
    });
    if (changes.length > 0) {
      console.log('[SCM] Changes files:', changes.map(f => f.path));
      groups.push({
        id: 'changes',
        label: 'Changes',
        resources: changes
      });
    }

    // Untracked (Untracked Group in VS Code - new files)
    const untracked = files.filter(f => {
      // Note: simple-git uses 'working_dir' not 'working'
      const working = f.working_dir || f.working || ' ';
      const isUntracked = working === '?' && f.index === '?';
      console.log(`[SCM] File ${f.path}: working='${working}' index='${f.index}' -> untracked=${isUntracked}`);
      return isUntracked;
    });
    if (untracked.length > 0) {
      console.log('[SCM] Untracked files:', untracked.map(f => f.path));
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
    
    // Debug log
    console.log(`[SCM] Rendering file ${file.path} in group ${groupId}, working: '${file.working}', index: '${file.index}'`);
    
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
          ${groupId === 'changes' ? `
            <button class="icon-button" title="Discard Changes" onclick="event.stopPropagation(); window.scmView?.discardFile('${escapedPath}')">
              <span class="codicon codicon-discard"></span>
            </button>
            <button class="icon-button" title="Stage Changes" onclick="event.stopPropagation(); window.scmView?.stageFile('${escapedPath}')">
              <span class="codicon codicon-add"></span>
            </button>
          ` : groupId === 'untracked' ? `
            <button class="icon-button" title="Delete File" onclick="event.stopPropagation(); window.scmView?.deleteUntrackedFile('${escapedPath}')" style="color: #f48771;">
              <span class="codicon codicon-trash"></span>
            </button>
            <button class="icon-button" title="Stage File" onclick="event.stopPropagation(); window.scmView?.stageFile('${escapedPath}')">
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

  public async deleteUntrackedFile(path: string) {
    const fileName = path.split('/').pop() || path;
    if (confirm(`Delete untracked file "${fileName}"? This cannot be undone.`)) {
      try {
        // Use git clean to remove the untracked file
        await window.gitAPI.clean([path]);
        await this.refresh();
        
        // Also refresh File Explorer
        if (window.fileExplorer) {
          await window.fileExplorer.refreshGitStatus();
        }
        
        notifications.show({
          title: 'File Deleted',
          message: `Deleted untracked file: ${fileName}`,
          type: 'info',
          duration: 3000
        });
      } catch (error) {
        console.error('Failed to delete untracked file:', error);
        notifications.show({
          title: 'Delete Failed',
          message: `Failed to delete ${fileName}: ${error}`,
          type: 'error',
          duration: 5000
        });
      }
    }
  }

  public async stageAll() {
    try {
      // Only stage tracked files with changes (not untracked files)
      const changes = this.gitStatus?.files.filter(f => {
        const working = f.working_dir || f.working || ' ';
        const hasWorkingChanges = working !== ' ' && working !== '?';
        const isNotStaged = f.index === ' ';
        const isTracked = working !== '?'; // Not untracked
        return hasWorkingChanges && isNotStaged && isTracked;
      }) || [];
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
    if (groupId === 'changes') {
      // For the Changes group - only modified/deleted files
      const files = this.gitStatus?.files
        .filter(f => ((f.working_dir === 'M' || f.working_dir === 'D') || (f.working === 'M' || f.working === 'D')) && f.index === ' ')
        .map(f => f.path) || [];
      
      if (files.length === 0) return;
      
      const confirmMessage = `Discard ${files.length} change(s)?`;
      if (!confirm(confirmMessage)) return;
      
      try {
        await window.gitAPI.discard(files);
      } catch (error) {
        console.error('Failed to discard changes:', error);
        notifications.show({
          title: 'Discard Failed',
          message: `Failed to discard changes: ${error}`,
          type: 'error',
          duration: 5000
        });
      }
    } else if (groupId === 'untracked') {
      // For the Untracked group - delete untracked files
      const files = this.gitStatus?.files
        .filter(f => (f.working_dir === '?' || f.working === '?') && f.index === '?')
        .map(f => f.path) || [];
      
      if (files.length === 0) return;
      
      const confirmMessage = `Delete ${files.length} untracked file(s)? This cannot be undone.`;
      if (!confirm(confirmMessage)) return;
      
      try {
        await window.gitAPI.clean(files);
      } catch (error) {
        console.error('Failed to delete untracked files:', error);
        notifications.show({
          title: 'Delete Failed',
          message: `Failed to delete untracked files: ${error}`,
          type: 'error',
          duration: 5000
        });
      }
    } else if (groupId === 'staged') {
      const confirmMessage = 'Discard all staged changes? This will unstage and discard the changes.';
      if (!confirm(confirmMessage)) return;
      
      try {
        // For staged files: first unstage, then discard
        const files = this.gitStatus?.files
          .filter(f => f.index !== ' ' && f.index !== '?')
          .map(f => f.path) || [];
        
        if (files.length > 0) {
          // First unstage all
          await window.gitAPI.unstage(files);
          // Then discard the changes (but only for modified files, not new files)
          const modifiedFiles = this.gitStatus?.files
            .filter(f => f.index === 'M' || f.working_dir === 'M' || f.working === 'M')
            .map(f => f.path) || [];
          if (modifiedFiles.length > 0) {
            await window.gitAPI.discard(modifiedFiles);
          }
        }
      } catch (error) {
        console.error('Failed to discard staged:', error);
        notifications.show({
          title: 'Discard Failed',
          message: `Failed to discard staged changes: ${error}`,
          type: 'error',
          duration: 5000
        });
      }
    }
    
    // Refresh the view after operations
    await this.refresh();
    
    // Also refresh File Explorer to update Git decorations
    if (window.fileExplorer) {
      await window.fileExplorer.refreshGitStatus();
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
      
      // Simply recreate the entire panel to ensure fresh state
      console.log('[SCM] Commit successful, recreating panel...');
      
      // Small delay to ensure Git has updated
      await new Promise(resolve => setTimeout(resolve, 500));
      
      // Recreate the entire panel with fresh data
      await this.recreatePanel();
      
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
    console.log('[SCM] Smart push button clicked');
    
    // Check if gitAPI is available
    if (!window.gitAPI) {
      alert('Git API not available!');
      console.error('[SCM] window.gitAPI:', window.gitAPI);
      return;
    }
    
    const branch = this.gitStatus?.branch || 'current branch';
    const aheadCount = this.gitStatus?.ahead || 0;
    
    console.log('[SCM] Current status - branch:', branch, 'ahead:', aheadCount, 'hasUpstream:', this.gitStatus?.hasUpstream);
    
    // REMOVED: Don't block the dialog when nothing to push
    // Users might want to use custom commands even with 0 commits ahead
    // Example: pushing current branch to overwrite a different branch
    
    // Special handling when there's nothing to push normally
    let customModeMessage = '';
    if (aheadCount === 0 && this.gitStatus?.hasUpstream) {
      console.log('[SCM] Branch is up to date, opening Smart Push dialog for custom operations');
      customModeMessage = '⚠️ Your branch is up to date. Showing custom push options only.';
    }
    
    try {
      // Get repository stats
      console.log('[SCM] Getting repository stats...');
      const stats = await (window.gitAPI as any).getRepoStats();
      console.log('[SCM] Repository stats:', JSON.stringify(stats, null, 2));
      console.log('[SCM] Push size from stats:', stats.pushSize, 'Push size MB:', stats.pushSizeMB);
      
      // Analyze and get strategy recommendations
      const analysis = GitPushStrategyAnalyzer.analyzeRepository(stats, this.gitStatus!);
      console.log('[SCM] Analysis result:', analysis);
      
      // Override analysis if nothing to push
      if (aheadCount === 0) {
        analysis.recommendation = 'No commits to push. Use Custom Command for special operations.';
        analysis.commitCount = 0;
        console.log('[SCM] Overriding analysis for 0 commits ahead scenario');
      }
      
      // Get intelligent recommendation (simulated AI analysis for safety)
      // This uses the same logic the consensus AI would use, but doesn't interfere
      // with the actual consensus engine or AI Helpers
      console.log('[SCM] Getting intelligent strategy recommendation...');
      
      const aiAdvice = await GitConsensusAdvisor.getStrategyAdvice(
        stats, 
        this.gitStatus
      );
      
      if (aiAdvice) {
        console.log('[SCM] Intelligent recommendation:', aiAdvice);
      }
      
      // Get available strategies
      let strategies = GitPushStrategyAnalyzer.getPushStrategies(analysis, this.gitStatus);
      
      // If we have AI advice, enhance the recommendation message
      if (aiAdvice) {
        // Find the strategy that matches the AI recommendation
        const recommendedIndex = strategies.findIndex(s => 
          s.label.toLowerCase().includes(aiAdvice.recommendedStrategy.toLowerCase().split(' ')[0])
        );
        
        if (recommendedIndex >= 0) {
          // Clear all other recommendations
          strategies.forEach(s => s.recommended = false);
          
          // Mark the AI-recommended strategy
          strategies[recommendedIndex].recommended = true;
          
          // Add AI reasoning to the description
          const originalDesc = strategies[recommendedIndex].description;
          strategies[recommendedIndex].description = `${originalDesc}\n\n🤖 AI Analysis: ${aiAdvice.reasoning}`;
          
          // Add risks if provided
          if (aiAdvice.risks && aiAdvice.risks.length > 0) {
            strategies[recommendedIndex].cons = [
              ...(strategies[recommendedIndex].cons || []),
              ...aiAdvice.risks
            ];
          }
        }
      }
      
      console.log('[SCM] Available strategies:', strategies);
      
      // Create explanation text
      let explanation = '';
      if (aheadCount === 0) {
        explanation = customModeMessage || '⚠️ No commits to push. Use Custom Command to push to a different branch or force operations.';
      } else if (aiAdvice) {
        explanation = `🤖 Intelligent Analysis: ${aiAdvice.reasoning}`;
      } else {
        explanation = `Repository: ${analysis.totalSize} | ${analysis.commitCount} commits | ${analysis.recommendation}`;
      }
      
      // Show strategy selection dialog
      const selectedStrategy = await GitPushDialog.show(analysis, strategies, explanation);
      
      if (!selectedStrategy) {
        console.log('[SCM] User cancelled strategy selection');
        return;
      }
      
      console.log('[SCM] User selected strategy:', selectedStrategy);
      
      // Execute the selected strategy  
      const result = await GitPushExecutor.execute(
        selectedStrategy, 
        analysis,
        window.gitAPI,
        this.gitStatus
      );
      
      console.log('[SCM] Execution result:', result);
      
      // Show success notification
      if (result.success) {
        notifications.show({
          title: '✅ Push Successful',
          message: result.message,
          type: 'info',
          duration: 5000
        });
      } else {
        notifications.show({
          title: '⚠️ Push Partially Successful',
          message: result.message,
          type: 'warning',
          duration: 0 // Keep visible until dismissed
        });
      }
      
      // Simply recreate the entire panel after push
      console.log('[SCM] Push completed, recreating panel...');
      
      // Small delay to ensure Git and remote are updated
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Recreate the entire panel with fresh data
      await this.recreatePanel();
      
    } catch (error: any) {
      console.error('[SCM] Smart push failed:', error);
      
      // Parse the error to get structured information
      const errorInfo = GitErrorHandler.parseError(error);
      
      // Show error notification
      notifications.show({
        title: errorInfo.title,
        message: errorInfo.message,
        type: 'error',
        duration: errorInfo.type === 'size-limit' ? 0 : 5000
      });
      
      // Show action buttons for errors with actions
      if (errorInfo.actions && errorInfo.actions.length > 0) {
        this.showErrorActions(errorInfo, '');
      }
    }
  }

  public async pull() {
    console.log('[SCM] Pull button clicked');
    
    // Check if branch has upstream
    if (!this.gitStatus?.hasUpstream) {
      notifications.show({
        title: 'No upstream branch',
        message: 'This branch has no upstream branch set. Push first to create it.',
        type: 'warning',
        duration: 4000
      });
      return;
    }
    
    // Check if there's anything to pull
    if (this.gitStatus?.behind === 0) {
      notifications.show({
        title: 'Already up to date',
        message: 'No changes to pull from remote',
        type: 'info',
        duration: 3000
      });
      return;
    }
    
    const notificationId = notifications.show({
      title: 'Git Pull',
      message: `Pulling ${this.gitStatus?.behind || ''} commit(s) from remote...`,
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
    console.log('[SCM] Sync button clicked');
    const branch = this.gitStatus?.branch || 'current branch';
    const ahead = this.gitStatus?.ahead || 0;
    const behind = this.gitStatus?.behind || 0;
    
    console.log('[SCM] Sync status - branch:', branch, 'ahead:', ahead, 'behind:', behind, 'hasUpstream:', this.gitStatus?.hasUpstream);
    
    // Check if branch has upstream
    if (!this.gitStatus?.hasUpstream) {
      // If we have commits to push, do push to create upstream
      if (ahead > 0) {
        console.log('[SCM] No upstream, but have commits to push - will push to create upstream');
        await this.push();
        return;
      } else {
        notifications.show({
          title: 'No upstream branch',
          message: 'This branch has no upstream branch set. Make a commit and push to create it.',
          type: 'warning',
          duration: 4000
        });
        return;
      }
    }
    
    // If nothing to sync
    if (ahead === 0 && behind === 0) {
      notifications.show({
        title: 'Already up to date',
        message: 'Your branch is synchronized with remote',
        type: 'info',
        duration: 3000
      });
      return;
    }
    
    const syncMessage = `Pulling ${behind} and pushing ${ahead} commit(s)...`;
    const notificationId = notifications.show({
      title: 'Git Sync',
      message: syncMessage,
      type: 'loading',
      duration: 0
    });

    try {
      // Use new sync API if available
      if (window.gitAPI.sync) {
        console.log('[SCM] Using new sync API');
        await window.gitAPI.sync();
      } else {
        // Fallback to sequential pull/push
        console.log('[SCM] Using fallback pull+push');
        // Pull first if behind
        if (behind > 0) {
          console.log('[SCM] Pulling changes...');
          await window.gitAPI.pull();
        }
        
        // Then push if ahead
        if (ahead > 0) {
          console.log('[SCM] Pushing changes...');
          await window.gitAPI.push();
        }
      }
      
      console.log('[SCM] Sync complete, refreshing...');
      await this.refresh();
      
      notifications.update(notificationId, {
        title: 'Sync Complete',
        message: `Successfully synchronized ${branch} with remote`,
        type: 'success',
        duration: 3000
      });
    } catch (error: any) {
      console.error('[SCM] Sync failed:', error);
      notifications.update(notificationId, {
        title: 'Sync Failed',
        message: error?.message || 'An error occurred during sync',
        type: 'error',
        duration: 5000
      });
    }
  }
  
  public async pullAndPush() {
    console.log('[SCM] Pull and push requested from behind badge click');
    
    // First fetch to ensure we have latest
    try {
      await window.gitAPI.fetch();
      console.log('[SCM] Fetch completed');
    } catch (error) {
      console.log('[SCM] Fetch failed, continuing anyway:', error);
    }
    
    // Refresh to get latest status
    await this.refresh();
    
    // If still behind, pull first
    if ((this.gitStatus?.behind || 0) > 0) {
      try {
        const notificationId = notifications.show({
          title: 'Syncing with remote',
          message: `Pulling ${this.gitStatus.behind} commits from remote...`,
          type: 'info',
          duration: 0
        });
        
        await window.gitAPI.pull();
        
        notifications.update(notificationId, {
          title: 'Pull completed',
          message: `Successfully pulled ${this.gitStatus.behind} commits`,
          type: 'success',
          duration: 3000
        });
        
        // Refresh status after pull
        await this.refresh();
      } catch (error: any) {
        console.error('[SCM] Pull failed:', error);
        notifications.show({
          title: 'Pull failed',
          message: error?.message || 'Failed to pull from remote',
          type: 'error',
          duration: 5000
        });
        return;
      }
    }
    
    // Now check if we have anything to push
    if ((this.gitStatus?.ahead || 0) > 0) {
      // Open push dialog
      await this.push();
    } else {
      notifications.show({
        title: 'Up to date',
        message: 'Your branch is now synchronized with remote',
        type: 'success',
        duration: 3000
      });
    }
  }
  
  public async recreatePanel() {
    console.log('[SCM] Recreating entire Source Control panel...');
    
    // Clear and recreate the entire panel
    const container = this.container;
    container.innerHTML = '';
    
    // Get fresh status
    this.gitStatus = await window.gitAPI.getStatus();
    console.log('[SCM] Fresh status for recreated panel - ahead:', this.gitStatus?.ahead, 'behind:', this.gitStatus?.behind);
    
    // Render fresh
    this.render();
    console.log('[SCM] Panel recreated successfully');
  }
  
  public async showBranchSwitcher() {
    console.log('[SCM] Opening branch switcher');
    
    try {
      // Get list of branches
      const branchData = await window.gitAPI.getBranches();
      console.log('[SCM] Available branches:', branchData);
      
      // Handle both array format and object format
      const branches = Array.isArray(branchData) ? branchData : (branchData.all || []);
      
      if (!branches || branches.length === 0) {
        alert('No branches available');
        return;
      }
      
      // Create modal dialog for branch selection
      const modal = document.createElement('div');
      modal.className = 'git-branch-modal';
      modal.style.cssText = `
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        background: var(--vscode-dropdown-background, #252526);
        border: 1px solid var(--vscode-dropdown-border, #454545);
        border-radius: 4px;
        padding: 16px;
        z-index: 10000;
        min-width: 300px;
        max-width: 500px;
        max-height: 400px;
        overflow-y: auto;
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
      `;
      
      // Add backdrop
      const backdrop = document.createElement('div');
      backdrop.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        z-index: 9999;
      `;
      
      // Build branch list HTML
      const currentBranch = this.gitStatus?.branch || '';
      let branchListHtml = '<h3 style="margin: 0 0 12px 0; color: var(--vscode-foreground);">Switch Branch</h3>';
      branchListHtml += '<div style="display: flex; flex-direction: column; gap: 4px;">';
      
      for (const branch of branches) {
        // Handle both string and object format
        const branchName = typeof branch === 'string' ? branch : branch.name;
        const isCurrentBranch = (typeof branch === 'object' && branch.current) || 
                               branchName === currentBranch || 
                               branchName === `* ${currentBranch}`;
        const cleanBranchName = branchName.replace('* ', '').trim();
        
        branchListHtml += `
          <button 
            class="branch-item" 
            style="
              padding: 8px 12px;
              text-align: left;
              background: ${isCurrentBranch ? 'var(--vscode-list-activeSelectionBackground, #094771)' : 'transparent'};
              color: var(--vscode-foreground);
              border: 1px solid ${isCurrentBranch ? 'var(--vscode-list-activeSelectionBorder, #007acc)' : 'transparent'};
              border-radius: 4px;
              cursor: ${isCurrentBranch ? 'default' : 'pointer'};
              font-family: var(--vscode-font-family);
              font-size: 13px;
              ${!isCurrentBranch ? 'hover: background: var(--vscode-list-hoverBackground, #2a2d2e);' : ''}
            "
            ${isCurrentBranch ? 'disabled' : `onclick="window.scmView?.switchToBranch?.('${cleanBranchName}')"`}
          >
            ${isCurrentBranch ? '✓ ' : ''}${cleanBranchName}
            ${isCurrentBranch ? ' (current)' : ''}
          </button>
        `;
      }
      
      branchListHtml += '</div>';
      branchListHtml += `
        <div style="margin-top: 16px; text-align: right;">
          <button 
            style="
              padding: 6px 14px;
              background: var(--vscode-button-secondaryBackground, #3a3d41);
              color: var(--vscode-button-secondaryForeground);
              border: 1px solid var(--vscode-button-border, transparent);
              border-radius: 4px;
              cursor: pointer;
              font-size: 13px;
            "
            onclick="window.scmView?.closeBranchSwitcher?.()"
          >
            Cancel
          </button>
        </div>
      `;
      
      modal.innerHTML = branchListHtml;
      
      // Add to DOM
      document.body.appendChild(backdrop);
      document.body.appendChild(modal);
      
      // Store references for cleanup
      (window as any).branchSwitcherModal = modal;
      (window as any).branchSwitcherBackdrop = backdrop;
      
      // Close on backdrop click
      backdrop.onclick = () => this.closeBranchSwitcher();
      
    } catch (error) {
      console.error('[SCM] Failed to get branches:', error);
      alert('Failed to get branch list');
    }
  }
  
  public closeBranchSwitcher() {
    const modal = (window as any).branchSwitcherModal;
    const backdrop = (window as any).branchSwitcherBackdrop;
    
    if (modal) modal.remove();
    if (backdrop) backdrop.remove();
    
    delete (window as any).branchSwitcherModal;
    delete (window as any).branchSwitcherBackdrop;
  }
  
  public async switchToBranch(branchName: string) {
    console.log('[SCM] Switching to branch:', branchName);
    
    try {
      // Close the modal first
      this.closeBranchSwitcher();
      
      // Show progress notification
      const notificationId = notifications.show({
        title: 'Switching Branch',
        message: `Switching to branch: ${branchName}...`,
        type: 'info',
        duration: 0
      });
      
      // Switch branch
      await window.gitAPI.switchBranch(branchName);
      
      // Update notification
      notifications.update(notificationId, {
        title: 'Branch Switched',
        message: `Successfully switched to branch: ${branchName}`,
        type: 'success',
        duration: 3000
      });
      
      // Refresh the Git panel
      await this.refresh();
      
      // Also refresh file explorer to update decorations
      if (window.fileExplorer) {
        await window.fileExplorer.refreshGitStatus();
      }
      
    } catch (error: any) {
      console.error('[SCM] Failed to switch branch:', error);
      notifications.show({
        title: 'Branch Switch Failed',
        message: error?.message || 'Failed to switch branch',
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
      
      // Check if this is a submodule (directories like dioxus-fork or src/hive_ui)
      const isSubmodule = path === 'dioxus-fork' || path === 'src/hive_ui';
      
      if (isSubmodule) {
        // For submodules, show actual changes
        console.log('[SCM] Submodule detected:', path);
        if (window.editorTabs) {
          try {
            // Get submodule status and diff
            const submoduleStatus = await (window.gitAPI as any).getSubmoduleStatus(fullPath);
            const submoduleDiff = await (window.gitAPI as any).getSubmoduleDiff(fullPath);
            
            let statusHtml = '';
            let diffHtml = '';
            
            // Parse and format the status
            if (submoduleStatus) {
              const lines = submoduleStatus.split('\n').filter((l: string) => l.trim());
              statusHtml = lines.map((line: string) => {
                // Color code the status lines
                if (line.includes('modified:')) {
                  return `<div style="color: #e2c08d;">📝 ${line}</div>`;
                } else if (line.includes('new file:')) {
                  return `<div style="color: #73c991;">➕ ${line}</div>`;
                } else if (line.includes('deleted:')) {
                  return `<div style="color: #f48771;">➖ ${line}</div>`;
                } else if (line.includes('Your branch')) {
                  return `<div style="color: #007acc; font-weight: bold;">${line}</div>`;
                }
                return `<div>${line}</div>`;
              }).join('');
            }
            
            // Format the diff with syntax highlighting
            if (submoduleDiff) {
              const diffLines = submoduleDiff.split('\n');
              diffHtml = `<pre style="background: #1e1e1e; padding: 15px; border-radius: 4px; overflow-x: auto;">`;
              
              for (const line of diffLines) {
                if (line.startsWith('+') && !line.startsWith('+++')) {
                  diffHtml += `<span style="color: #73c991;">${this.escapeHtml(line)}</span>\n`;
                } else if (line.startsWith('-') && !line.startsWith('---')) {
                  diffHtml += `<span style="color: #f48771;">${this.escapeHtml(line)}</span>\n`;
                } else if (line.startsWith('@@')) {
                  diffHtml += `<span style="color: #007acc; font-weight: bold;">${this.escapeHtml(line)}</span>\n`;
                } else if (line.startsWith('diff --git')) {
                  diffHtml += `<span style="color: #e2c08d; font-weight: bold;">${this.escapeHtml(line)}</span>\n`;
                } else {
                  diffHtml += `<span>${this.escapeHtml(line)}</span>\n`;
                }
              }
              diffHtml += `</pre>`;
            }
            
            const submoduleInfo = `
              <div style="padding: 20px; font-family: var(--vscode-font-family);">
                <h2 style="margin-bottom: 20px;">Submodule: ${path}</h2>
                
                <div style="margin-bottom: 30px;">
                  <h3 style="color: #007acc; margin-bottom: 10px;">Status</h3>
                  <div style="background: #252526; padding: 15px; border-radius: 4px;">
                    ${statusHtml || '<div style="color: #888;">No changes in submodule</div>'}
                  </div>
                </div>
                
                ${diffHtml ? `
                <div>
                  <h3 style="color: #007acc; margin-bottom: 10px;">Changes</h3>
                  ${diffHtml}
                </div>
                ` : ''}
                
                <div style="margin-top: 20px; padding-top: 20px; border-top: 1px solid #3c3c3c;">
                  <p style="color: #888; font-size: 12px;">
                    Submodule path: ${fullPath}
                  </p>
                </div>
              </div>
            `;
            
            const container = document.createElement('div');
            container.innerHTML = submoduleInfo;
            window.editorTabs.openCustomTab(`submodule-${path}`, `Submodule: ${path}`, container);
            console.log('[SCM] Submodule changes opened in editor tabs');
            
          } catch (error) {
            console.error('[SCM] Failed to get submodule info:', error);
            // Fallback to basic info if we can't get the status
            const fallbackInfo = `
              <div style="padding: 20px; font-family: var(--vscode-font-family);">
                <h2>Submodule: ${path}</h2>
                <p style="color: #f48771;">
                  Failed to retrieve submodule status: ${error}
                </p>
                <p style="margin-top: 20px; color: var(--vscode-descriptionForeground);">
                  This is a Git submodule. To see changes manually:
                </p>
                <ol style="margin-top: 10px; line-height: 1.8;">
                  <li>Navigate to: <code>cd ${fullPath}</code></li>
                  <li>Check status: <code>git status</code></li>
                  <li>View diff: <code>git diff</code></li>
                </ol>
              </div>
            `;
            const container = document.createElement('div');
            container.innerHTML = fallbackInfo;
            window.editorTabs.openCustomTab(`submodule-${path}`, `Submodule: ${path}`, container);
          }
        }
      } else if (fileStatus && (fileStatus.working === 'M' || fileStatus.index === 'M' || 
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
  
  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
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

  /**
   * Show error actions dialog
   */
  private showErrorActions(errorInfo: GitErrorOptions, notificationId?: string) {
    // Create action buttons container
    const actionsContainer = document.createElement('div');
    actionsContainer.className = 'error-actions-container';
    actionsContainer.style.cssText = `
      display: flex;
      gap: 8px;
      margin-top: 12px;
      flex-wrap: wrap;
    `;
    
    // Add each action as a button
    errorInfo.actions?.forEach(action => {
      const button = document.createElement('button');
      button.className = action.primary ? 'action-button-primary' : 'action-button';
      button.textContent = action.label;
      button.style.cssText = `
        padding: 6px 12px;
        border-radius: 4px;
        border: none;
        cursor: pointer;
        font-size: 12px;
        background: ${action.primary ? 'var(--vscode-button-background, #0e639c)' : 'var(--vscode-button-secondaryBackground, #3a3d41)'};
        color: ${action.primary ? 'var(--vscode-button-foreground, #fff)' : 'var(--vscode-button-secondaryForeground, #ccc)'};
      `;
      
      button.addEventListener('click', async () => {
        try {
          await action.action();
          // Close notification if action succeeds
          if (notificationId) {
            notifications.hide(notificationId);
          }
        } catch (error) {
          console.error('Error executing action:', error);
        }
      });
      
      actionsContainer.appendChild(button);
    });
    
    // Append to the last notification
    setTimeout(() => {
      const lastNotification = document.querySelector('.notification-item:last-child .notification-content');
      if (lastNotification && !lastNotification.querySelector('.error-actions-container')) {
        lastNotification.appendChild(actionsContainer);
      }
    }, 100);
  }

  /**
   * Push using chunked strategy for large repositories
   */
  public async pushWithChunks() {
    const notificationId = notifications.show({
      title: 'Chunked Push',
      message: 'This feature needs to be implemented through the main process...',
      type: 'info',
      duration: 5000
    });
    
    // TODO: Implement through IPC to main process
    // For now, show the solutions dialog
    GitErrorHandler.showSizeLimitSolutions();
  }
  
  /**
   * Analyze repository for size issues
   */
  public async analyzeRepository() {
    const notificationId = notifications.show({
      title: 'Repository Analysis',
      message: 'This feature needs to be implemented through the main process...',
      type: 'info',
      duration: 5000
    });
    
    // TODO: Implement through IPC to main process
    // For now, show the solutions dialog
    GitErrorHandler.showSizeLimitSolutions();
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