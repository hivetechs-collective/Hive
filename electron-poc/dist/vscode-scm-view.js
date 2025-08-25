"use strict";
/// <reference path="./types/window.d.ts" />
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.VSCodeSCMView = void 0;
const git_graph_1 = require("./git-graph");
const notification_1 = require("./notification");
const git_error_handler_1 = require("./git-error-handler");
const git_push_strategy_1 = require("./git-push-strategy");
const git_push_dialog_1 = require("./git-push-dialog");
const git_push_executor_1 = require("./git-push-executor");
const git_consensus_advisor_1 = require("./git-consensus-advisor");
class VSCodeSCMView {
    constructor(container) {
        this.gitStatus = null;
        this.refreshInterval = null;
        this.selectedFiles = new Set();
        this.commitMessage = '';
        this.gitDecorationProvider = null;
        this.gitGraphView = null;
        this.pendingOperations = new Set(); // Track files being processed
        this.container = container;
        this.initialize();
    }
    initialize() {
        return __awaiter(this, void 0, void 0, function* () {
            // Don't initialize Git decoration provider without a folder
            // It will be initialized when a folder is opened
            // Initial refresh only - no auto-refresh interval
            yield this.refresh();
            // Auto-refresh removed - will refresh on Git operations and file saves only
            // Global reference is now set in renderer.ts when creating the instance
        });
    }
    refresh() {
        var _a, _b, _c, _d, _e, _f, _g;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Refresh button clicked!');
            try {
                // Check if a folder is open first
                const currentFolder = window.currentOpenedFolder;
                if (!currentFolder) {
                    console.log('[SCM] No folder open, showing welcome view');
                    this.gitStatus = null;
                    this.render();
                    return;
                }
                // Fetch from remote to get latest ahead/behind counts
                console.log('[SCM] Fetching from remote to get latest status...');
                try {
                    yield window.gitAPI.fetch();
                    console.log('[SCM] Fetch completed successfully');
                }
                catch (fetchError) {
                    console.log('[SCM] Fetch failed (might be offline):', fetchError);
                    // Continue anyway - we can still show local status
                }
                console.log('[SCM] Getting git status for:', currentFolder);
                this.gitStatus = yield window.gitAPI.getStatus();
                console.log('[SCM] Got git status with', ((_b = (_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files) === null || _b === void 0 ? void 0 : _b.length) || 0, 'files');
                console.log('[SCM] Branch:', (_c = this.gitStatus) === null || _c === void 0 ? void 0 : _c.branch, 'Ahead:', (_d = this.gitStatus) === null || _d === void 0 ? void 0 : _d.ahead, 'Behind:', (_e = this.gitStatus) === null || _e === void 0 ? void 0 : _e.behind);
                // Check for untracked files
                const untracked = ((_g = (_f = this.gitStatus) === null || _f === void 0 ? void 0 : _f.files) === null || _g === void 0 ? void 0 : _g.filter(f => (f.working_dir === '?' || f.working === '?') && f.index === '?')) || [];
                console.log('[SCM] Untracked files:', untracked.map(f => f.path));
            }
            catch (error) {
                console.error('[SCM] Failed to refresh:', error);
                // Set gitStatus to null to show welcome message
                this.gitStatus = null;
            }
            // Always render, even if there was an error
            this.render();
            console.log('[SCM] Render complete');
        });
    }
    render() {
        var _a, _b, _c, _d, _e, _f, _g;
        console.log('[SCM] Rendering with status - ahead:', (_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.ahead, 'behind:', (_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.behind, 'branch:', (_c = this.gitStatus) === null || _c === void 0 ? void 0 : _c.branch);
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
      <div class="scm-view" style="
        display: flex;
        flex-direction: column;
        height: calc(100vh - 22px);
        overflow: hidden;
      ">
        <!-- Branch Status Bar - Always visible at top -->
        <div class="scm-status-bar" style="
          flex-shrink: 0;
          border-bottom: 1px solid var(--vscode-sideBarSectionHeader-border, #1e1e1e);
        ">
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

        <!-- Header with VS Code-style toolbar -->
        <div class="scm-view-header" style="flex-shrink: 0;">
          <div class="scm-provider-container">
            <div class="scm-toolbar">
              <button class="scm-toolbar-button" title="Refresh" onclick="window.scmView?.refresh()">
                <span class="codicon codicon-refresh"></span>
              </button>
              <button class="scm-toolbar-button" title="Commit" onclick="window.scmView?.commit()">
                <span class="codicon codicon-check"></span>
              </button>
              <div class="scm-toolbar-separator"></div>
              <button class="scm-toolbar-button" title="Pull${((_d = this.gitStatus) === null || _d === void 0 ? void 0 : _d.behind) ? ` (${this.gitStatus.behind} behind)` : ''}" onclick="window.scmView?.pull()">
                <span class="codicon codicon-cloud-download"></span>
              </button>
              <button class="scm-toolbar-button" title="Push${((_e = this.gitStatus) === null || _e === void 0 ? void 0 : _e.ahead) ? ` (${this.gitStatus.ahead} ahead)` : ''}" onclick="window.scmView?.push()">
                <span class="codicon codicon-cloud-upload"></span>
              </button>
              <button class="scm-toolbar-button" title="Sync Changes${((_f = this.gitStatus) === null || _f === void 0 ? void 0 : _f.ahead) || ((_g = this.gitStatus) === null || _g === void 0 ? void 0 : _g.behind) ? ` (${this.gitStatus.ahead || 0}↑ ${this.gitStatus.behind || 0}↓)` : ''}" onclick="window.scmView?.sync()">
                <span class="codicon codicon-sync"></span>
              </button>
            </div>
          </div>
        </div>

        <!-- Commit Input -->
        <div class="scm-input-container" style="flex-shrink: 0;">
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

        <!-- All scrollable content including resource groups and commits -->
        <div style="
          flex: 1;
          overflow-y: auto;
          overflow-x: hidden;
          min-height: 0;
        ">
          <!-- Resource Groups -->
          <div class="scm-view-content">
            ${groups.map(group => this.renderResourceGroup(group)).join('')}
          </div>
          
          <!-- Git Graph Container for commits -->
          <div id="git-graph-container" style="
            margin-top: 10px;
            max-height: 200px;
            overflow-y: auto;
            overflow-x: hidden;
            padding-bottom: 5px;
          "></div>
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
            console.log('[SCM] Window gitGraph exists:', !!window.gitGraph);
            if (graphContainer && !this.gitGraphView) {
                console.log('[SCM] Creating new GitGraphView...');
                try {
                    this.gitGraphView = new git_graph_1.GitGraphView(graphContainer);
                    window.gitGraph = this.gitGraphView; // Ensure global reference
                    console.log('[SCM] GitGraphView created successfully');
                    // Immediately refresh to load commits
                    setTimeout(() => {
                        var _a;
                        console.log('[SCM] Calling refresh on Git graph...');
                        (_a = this.gitGraphView) === null || _a === void 0 ? void 0 : _a.refresh();
                    }, 2000); // Increased delay from 100ms to 2s to not block UI
                }
                catch (error) {
                    console.error('[SCM] Failed to create GitGraphView:', error);
                }
            }
            else if (graphContainer && this.gitGraphView) {
                console.log('[SCM] Git graph already exists, refreshing...');
                this.gitGraphView.refresh();
            }
        }, 1000); // Increased delay to ensure Git is fully ready
    }
    groupResources() {
        if (!this.gitStatus)
            return [];
        const groups = [];
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
    renderResourceGroup(group) {
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
          <div class="scm-resource-group-content" style="
            max-height: 200px;
            overflow-y: auto;
            overflow-x: hidden;
          ">
            ${group.resources.map(resource => this.renderResource(resource, group.id)).join('')}
          </div>
        ` : ''}
      </div>
    `;
    }
    renderResource(file, groupId) {
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
    getStatusIcon(file) {
        if (file.index === 'A')
            return { letter: 'A', className: 'added', tooltip: 'Added' };
        if (file.index === 'M')
            return { letter: 'M', className: 'modified', tooltip: 'Modified (Staged)' };
        if (file.index === 'D')
            return { letter: 'D', className: 'deleted', tooltip: 'Deleted (Staged)' };
        if (file.index === 'R')
            return { letter: 'R', className: 'renamed', tooltip: 'Renamed' };
        if (file.working === 'M')
            return { letter: 'M', className: 'modified', tooltip: 'Modified' };
        if (file.working === 'D')
            return { letter: 'D', className: 'deleted', tooltip: 'Deleted' };
        if (file.working === '?')
            return { letter: 'U', className: 'untracked', tooltip: 'Untracked' };
        return { letter: '?', className: 'unknown', tooltip: 'Unknown' };
    }
    getRepoName() {
        // Use the actual repo path from git status, or return empty if no repo
        if (this.gitStatus && this.gitStatus.repoPath) {
            return this.gitStatus.repoPath.split('/').pop() || '';
        }
        return '';
    }
    attachEventListeners() {
        // Add context menu support
        const resourceItems = this.container.querySelectorAll('.scm-resource-item');
        resourceItems.forEach(item => {
            item.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                this.showContextMenu(e, item);
            });
            item.addEventListener('dblclick', () => {
                const path = item.getAttribute('data-path');
                if (path)
                    this.openDiff(path);
            });
        });
    }
    showContextMenu(event, element) {
        // TODO: Implement context menu
        console.log('Context menu for', element.getAttribute('data-path'));
    }
    // Public methods for event handlers
    updateCommitMessage(message) {
        this.commitMessage = message;
        const counter = this.container.querySelector('.scm-input-counter');
        if (counter) {
            counter.textContent = String(message.length);
            counter.classList.toggle('warn', message.length > 50);
        }
    }
    handleCommitKeydown(event) {
        if (event.ctrlKey && event.key === 'Enter') {
            this.commit();
        }
    }
    toggleFileSelection(path) {
        if (this.selectedFiles.has(path)) {
            this.selectedFiles.delete(path);
        }
        else {
            this.selectedFiles.add(path);
        }
    }
    stageFile(path) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            // Prevent double-clicks
            if (this.pendingOperations.has(path)) {
                console.log('[SCM] Operation already pending for:', path);
                return;
            }
            try {
                console.log('[SCM] Staging file:', path);
                this.pendingOperations.add(path);
                // Optimistically update UI immediately
                const fileStatus = (_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files.find(f => f.path === path);
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
            }
            catch (error) {
                console.error('[SCM] Failed to stage:', error);
                this.pendingOperations.delete(path);
                alert(`Failed to stage file: ${error}`);
            }
        });
    }
    unstageFile(path) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            // Prevent double-clicks
            if (this.pendingOperations.has(path)) {
                console.log('[SCM] Operation already pending for:', path);
                return;
            }
            try {
                console.log('[SCM] Unstaging file:', path);
                this.pendingOperations.add(path);
                // Optimistically update UI immediately
                const fileStatus = (_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files.find(f => f.path === path);
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
            }
            catch (error) {
                console.error('[SCM] Failed to unstage:', error);
                this.pendingOperations.delete(path);
                alert(`Failed to unstage file: ${error}`);
            }
        });
    }
    discardFile(path) {
        return __awaiter(this, void 0, void 0, function* () {
            if (confirm(`Discard changes to ${path}?`)) {
                try {
                    yield window.gitAPI.discard([path]);
                    yield this.refresh();
                }
                catch (error) {
                    console.error('Failed to discard:', error);
                }
            }
        });
    }
    deleteUntrackedFile(path) {
        return __awaiter(this, void 0, void 0, function* () {
            const fileName = path.split('/').pop() || path;
            if (confirm(`Delete untracked file "${fileName}"? This cannot be undone.`)) {
                try {
                    // Use git clean to remove the untracked file
                    yield window.gitAPI.clean([path]);
                    yield this.refresh();
                    // Also refresh File Explorer
                    if (window.fileExplorer) {
                        yield window.fileExplorer.refreshGitStatus();
                    }
                    notification_1.notifications.show({
                        title: 'File Deleted',
                        message: `Deleted untracked file: ${fileName}`,
                        type: 'info',
                        duration: 3000
                    });
                }
                catch (error) {
                    console.error('Failed to delete untracked file:', error);
                    notification_1.notifications.show({
                        title: 'Delete Failed',
                        message: `Failed to delete ${fileName}: ${error}`,
                        type: 'error',
                        duration: 5000
                    });
                }
            }
        });
    }
    stageAll() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Only stage tracked files with changes (not untracked files)
                const changes = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files.filter(f => {
                    const working = f.working_dir || f.working || ' ';
                    const hasWorkingChanges = working !== ' ' && working !== '?';
                    const isNotStaged = f.index === ' ';
                    const isTracked = working !== '?'; // Not untracked
                    return hasWorkingChanges && isNotStaged && isTracked;
                })) || [];
                yield window.gitAPI.stage(changes.map(f => f.path));
                yield this.refresh();
            }
            catch (error) {
                console.error('Failed to stage all:', error);
            }
        });
    }
    unstageAll() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const staged = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files.filter(f => f.index !== ' ' && f.index !== '?')) || [];
                yield window.gitAPI.unstage(staged.map(f => f.path));
                yield this.refresh();
            }
            catch (error) {
                console.error('Failed to unstage all:', error);
            }
        });
    }
    discardAll(groupId) {
        var _a, _b, _c, _d;
        return __awaiter(this, void 0, void 0, function* () {
            if (groupId === 'changes') {
                // For the Changes group - only modified/deleted files
                const files = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files.filter(f => ((f.working_dir === 'M' || f.working_dir === 'D') || (f.working === 'M' || f.working === 'D')) && f.index === ' ').map(f => f.path)) || [];
                if (files.length === 0)
                    return;
                const confirmMessage = `Discard ${files.length} change(s)?`;
                if (!confirm(confirmMessage))
                    return;
                try {
                    yield window.gitAPI.discard(files);
                }
                catch (error) {
                    console.error('Failed to discard changes:', error);
                    notification_1.notifications.show({
                        title: 'Discard Failed',
                        message: `Failed to discard changes: ${error}`,
                        type: 'error',
                        duration: 5000
                    });
                }
            }
            else if (groupId === 'untracked') {
                // For the Untracked group - delete untracked files
                const files = ((_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.files.filter(f => (f.working_dir === '?' || f.working === '?') && f.index === '?').map(f => f.path)) || [];
                if (files.length === 0)
                    return;
                const confirmMessage = `Delete ${files.length} untracked file(s)? This cannot be undone.`;
                if (!confirm(confirmMessage))
                    return;
                try {
                    yield window.gitAPI.clean(files);
                }
                catch (error) {
                    console.error('Failed to delete untracked files:', error);
                    notification_1.notifications.show({
                        title: 'Delete Failed',
                        message: `Failed to delete untracked files: ${error}`,
                        type: 'error',
                        duration: 5000
                    });
                }
            }
            else if (groupId === 'staged') {
                const confirmMessage = 'Discard all staged changes? This will unstage and discard the changes.';
                if (!confirm(confirmMessage))
                    return;
                try {
                    // For staged files: first unstage, then discard
                    const files = ((_c = this.gitStatus) === null || _c === void 0 ? void 0 : _c.files.filter(f => f.index !== ' ' && f.index !== '?').map(f => f.path)) || [];
                    if (files.length > 0) {
                        // First unstage all
                        yield window.gitAPI.unstage(files);
                        // Then discard the changes (but only for modified files, not new files)
                        const modifiedFiles = ((_d = this.gitStatus) === null || _d === void 0 ? void 0 : _d.files.filter(f => f.index === 'M' || f.working_dir === 'M' || f.working === 'M').map(f => f.path)) || [];
                        if (modifiedFiles.length > 0) {
                            yield window.gitAPI.discard(modifiedFiles);
                        }
                    }
                }
                catch (error) {
                    console.error('Failed to discard staged:', error);
                    notification_1.notifications.show({
                        title: 'Discard Failed',
                        message: `Failed to discard staged changes: ${error}`,
                        type: 'error',
                        duration: 5000
                    });
                }
            }
            // Refresh the view after operations
            yield this.refresh();
            // Also refresh File Explorer to update Git decorations
            if (window.fileExplorer) {
                yield window.fileExplorer.refreshGitStatus();
            }
        });
    }
    commit() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.commitMessage.trim()) {
                notification_1.notifications.show({
                    title: 'Commit Failed',
                    message: 'Please enter a commit message',
                    type: 'warning',
                    duration: 3000
                });
                return;
            }
            // Check if there are staged files
            const stagedFiles = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files.filter(f => f.index !== ' ' && f.index !== '?')) || [];
            if (stagedFiles.length === 0) {
                notification_1.notifications.show({
                    title: 'No Staged Files',
                    message: 'Please stage files before committing',
                    type: 'warning',
                    duration: 3000
                });
                return;
            }
            const notificationId = notification_1.notifications.show({
                title: 'Committing',
                message: `Committing ${stagedFiles.length} file(s)...`,
                type: 'loading',
                duration: 0
            });
            try {
                yield window.gitAPI.commit(this.commitMessage);
                const message = this.commitMessage;
                this.commitMessage = '';
                // Update the commit message input
                const input = this.container.querySelector('.scm-input');
                if (input) {
                    input.value = '';
                }
                // Simply recreate the entire panel to ensure fresh state
                console.log('[SCM] Commit successful, recreating panel...');
                // Small delay to ensure Git has updated
                yield new Promise(resolve => setTimeout(resolve, 500));
                // Recreate the entire panel with fresh data
                yield this.recreatePanel();
                notification_1.notifications.update(notificationId, {
                    title: 'Commit Successful',
                    message: `Committed: "${message.substring(0, 50)}${message.length > 50 ? '...' : ''}"`,
                    type: 'success',
                    duration: 3000
                });
            }
            catch (error) {
                console.error('Failed to commit:', error);
                notification_1.notifications.update(notificationId, {
                    title: 'Commit Failed',
                    message: (error === null || error === void 0 ? void 0 : error.message) || 'An error occurred while committing',
                    type: 'error',
                    duration: 5000
                });
            }
        });
    }
    commitAndPush() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.commit();
            yield this.push();
        });
    }
    push() {
        var _a, _b, _c, _d;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Smart push button clicked');
            // Check if gitAPI is available
            if (!window.gitAPI) {
                alert('Git API not available!');
                console.error('[SCM] window.gitAPI:', window.gitAPI);
                return;
            }
            const branch = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.branch) || 'current branch';
            const aheadCount = ((_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.ahead) || 0;
            console.log('[SCM] Current status - branch:', branch, 'ahead:', aheadCount, 'hasUpstream:', (_c = this.gitStatus) === null || _c === void 0 ? void 0 : _c.hasUpstream);
            // REMOVED: Don't block the dialog when nothing to push
            // Users might want to use custom commands even with 0 commits ahead
            // Example: pushing current branch to overwrite a different branch
            // Special handling when there's nothing to push normally
            let customModeMessage = '';
            if (aheadCount === 0 && ((_d = this.gitStatus) === null || _d === void 0 ? void 0 : _d.hasUpstream)) {
                console.log('[SCM] Branch is up to date, opening Smart Push dialog for custom operations');
                customModeMessage = '⚠️ Your branch is up to date. Showing custom push options only.';
            }
            try {
                // Get repository stats
                console.log('[SCM] Getting repository stats...');
                const stats = yield window.gitAPI.getRepoStats();
                console.log('[SCM] Repository stats:', JSON.stringify(stats, null, 2));
                console.log('[SCM] Push size from stats:', stats.pushSize, 'Push size MB:', stats.pushSizeMB);
                // Analyze and get strategy recommendations
                const analysis = git_push_strategy_1.GitPushStrategyAnalyzer.analyzeRepository(stats, this.gitStatus);
                console.log('[SCM] Analysis result:', analysis);
                // Override analysis if nothing to push
                if (aheadCount === 0) {
                    // Keep the recommendation but update the reasoning
                    analysis.reasoning = ['No commits to push', 'Use Custom Command for special operations like cross-branch pushing'];
                    analysis.commitCount = 0;
                    console.log('[SCM] Overriding analysis for 0 commits ahead scenario');
                }
                // Get intelligent recommendation (simulated AI analysis for safety)
                // This uses the same logic the consensus AI would use, but doesn't interfere
                // with the actual consensus engine or AI Helpers
                console.log('[SCM] Getting intelligent strategy recommendation...');
                const aiAdvice = yield git_consensus_advisor_1.GitConsensusAdvisor.getStrategyAdvice(stats, this.gitStatus);
                if (aiAdvice) {
                    console.log('[SCM] Intelligent recommendation:', aiAdvice);
                }
                // Get available strategies
                let strategies = git_push_strategy_1.GitPushStrategyAnalyzer.getPushStrategies(analysis, this.gitStatus);
                // If we have AI advice, enhance the recommendation message
                if (aiAdvice) {
                    // Find the strategy that matches the AI recommendation
                    const recommendedIndex = strategies.findIndex(s => s.label.toLowerCase().includes(aiAdvice.recommendedStrategy.toLowerCase().split(' ')[0]));
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
                }
                else if (aiAdvice) {
                    explanation = `🤖 Intelligent Analysis: ${aiAdvice.reasoning}`;
                }
                else {
                    explanation = `Repository: ${analysis.totalSize} | ${analysis.commitCount} commits | ${analysis.recommendation}`;
                }
                // Show strategy selection dialog
                const selectedStrategy = yield git_push_dialog_1.GitPushDialog.show(analysis, strategies, explanation);
                if (!selectedStrategy) {
                    console.log('[SCM] User cancelled strategy selection');
                    return;
                }
                console.log('[SCM] User selected strategy:', selectedStrategy);
                // Execute the selected strategy  
                const result = yield git_push_executor_1.GitPushExecutor.execute(selectedStrategy, analysis, window.gitAPI, this.gitStatus);
                console.log('[SCM] Execution result:', result);
                // Show success notification
                if (result.success) {
                    notification_1.notifications.show({
                        title: '✅ Push Successful',
                        message: result.message,
                        type: 'info',
                        duration: 5000
                    });
                }
                else {
                    notification_1.notifications.show({
                        title: '⚠️ Push Partially Successful',
                        message: result.message,
                        type: 'warning',
                        duration: 0 // Keep visible until dismissed
                    });
                }
                // Simply recreate the entire panel after push
                console.log('[SCM] Push completed, recreating panel...');
                // Small delay to ensure Git and remote are updated
                yield new Promise(resolve => setTimeout(resolve, 1000));
                // Recreate the entire panel with fresh data
                yield this.recreatePanel();
            }
            catch (error) {
                console.error('[SCM] Smart push failed:', error);
                // Parse the error to get structured information
                const errorInfo = git_error_handler_1.GitErrorHandler.parseError(error);
                // Show error notification
                notification_1.notifications.show({
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
        });
    }
    pull() {
        var _a, _b, _c;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Pull button clicked');
            // Check if branch has upstream
            if (!((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.hasUpstream)) {
                notification_1.notifications.show({
                    title: 'No upstream branch',
                    message: 'This branch has no upstream branch set. Push first to create it.',
                    type: 'warning',
                    duration: 4000
                });
                return;
            }
            // Check if there's anything to pull
            if (((_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.behind) === 0) {
                notification_1.notifications.show({
                    title: 'Already up to date',
                    message: 'No changes to pull from remote',
                    type: 'info',
                    duration: 3000
                });
                return;
            }
            const notificationId = notification_1.notifications.show({
                title: 'Git Pull',
                message: `Pulling ${((_c = this.gitStatus) === null || _c === void 0 ? void 0 : _c.behind) || ''} commit(s) from remote...`,
                type: 'loading',
                duration: 0
            });
            try {
                yield window.gitAPI.pull();
                yield this.refresh();
                notification_1.notifications.update(notificationId, {
                    title: 'Pull Successful',
                    message: 'Successfully pulled changes from remote',
                    type: 'success',
                    duration: 3000
                });
            }
            catch (error) {
                console.error('Failed to pull:', error);
                notification_1.notifications.update(notificationId, {
                    title: 'Pull Failed',
                    message: (error === null || error === void 0 ? void 0 : error.message) || 'An error occurred while pulling',
                    type: 'error',
                    duration: 5000
                });
            }
        });
    }
    sync() {
        var _a, _b, _c, _d, _e;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Sync button clicked');
            const branch = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.branch) || 'current branch';
            const ahead = ((_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.ahead) || 0;
            const behind = ((_c = this.gitStatus) === null || _c === void 0 ? void 0 : _c.behind) || 0;
            console.log('[SCM] Sync status - branch:', branch, 'ahead:', ahead, 'behind:', behind, 'hasUpstream:', (_d = this.gitStatus) === null || _d === void 0 ? void 0 : _d.hasUpstream);
            // Check if branch has upstream
            if (!((_e = this.gitStatus) === null || _e === void 0 ? void 0 : _e.hasUpstream)) {
                // If we have commits to push, do push to create upstream
                if (ahead > 0) {
                    console.log('[SCM] No upstream, but have commits to push - will push to create upstream');
                    yield this.push();
                    return;
                }
                else {
                    notification_1.notifications.show({
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
                notification_1.notifications.show({
                    title: 'Already up to date',
                    message: 'Your branch is synchronized with remote',
                    type: 'info',
                    duration: 3000
                });
                return;
            }
            const syncMessage = `Pulling ${behind} and pushing ${ahead} commit(s)...`;
            const notificationId = notification_1.notifications.show({
                title: 'Git Sync',
                message: syncMessage,
                type: 'loading',
                duration: 0
            });
            try {
                // Use new sync API if available
                if (window.gitAPI.sync) {
                    console.log('[SCM] Using new sync API');
                    yield window.gitAPI.sync();
                }
                else {
                    // Fallback to sequential pull/push
                    console.log('[SCM] Using fallback pull+push');
                    // Pull first if behind
                    if (behind > 0) {
                        console.log('[SCM] Pulling changes...');
                        yield window.gitAPI.pull();
                    }
                    // Then push if ahead
                    if (ahead > 0) {
                        console.log('[SCM] Pushing changes...');
                        yield window.gitAPI.push();
                    }
                }
                console.log('[SCM] Sync complete, refreshing...');
                yield this.refresh();
                notification_1.notifications.update(notificationId, {
                    title: 'Sync Complete',
                    message: `Successfully synchronized ${branch} with remote`,
                    type: 'success',
                    duration: 3000
                });
            }
            catch (error) {
                console.error('[SCM] Sync failed:', error);
                notification_1.notifications.update(notificationId, {
                    title: 'Sync Failed',
                    message: (error === null || error === void 0 ? void 0 : error.message) || 'An error occurred during sync',
                    type: 'error',
                    duration: 5000
                });
            }
        });
    }
    pullAndPush() {
        var _a, _b;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Pull and push requested from behind badge click');
            // First fetch to ensure we have latest
            try {
                yield window.gitAPI.fetch();
                console.log('[SCM] Fetch completed');
            }
            catch (error) {
                console.log('[SCM] Fetch failed, continuing anyway:', error);
            }
            // Refresh to get latest status
            yield this.refresh();
            // If still behind, pull first
            if ((((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.behind) || 0) > 0) {
                try {
                    const notificationId = notification_1.notifications.show({
                        title: 'Syncing with remote',
                        message: `Pulling ${this.gitStatus.behind} commits from remote...`,
                        type: 'info',
                        duration: 0
                    });
                    yield window.gitAPI.pull();
                    notification_1.notifications.update(notificationId, {
                        title: 'Pull completed',
                        message: `Successfully pulled ${this.gitStatus.behind} commits`,
                        type: 'success',
                        duration: 3000
                    });
                    // Refresh status after pull
                    yield this.refresh();
                }
                catch (error) {
                    console.error('[SCM] Pull failed:', error);
                    notification_1.notifications.show({
                        title: 'Pull failed',
                        message: (error === null || error === void 0 ? void 0 : error.message) || 'Failed to pull from remote',
                        type: 'error',
                        duration: 5000
                    });
                    return;
                }
            }
            // Now check if we have anything to push
            if ((((_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.ahead) || 0) > 0) {
                // Open push dialog
                yield this.push();
            }
            else {
                notification_1.notifications.show({
                    title: 'Up to date',
                    message: 'Your branch is now synchronized with remote',
                    type: 'success',
                    duration: 3000
                });
            }
        });
    }
    recreatePanel() {
        var _a, _b;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Recreating entire Source Control panel...');
            // Clear git graph view reference before clearing DOM
            if (this.gitGraphView) {
                this.gitGraphView.destroy();
                this.gitGraphView = null;
            }
            // Clear and recreate the entire panel
            const container = this.container;
            container.innerHTML = '';
            // Get fresh status
            this.gitStatus = yield window.gitAPI.getStatus();
            console.log('[SCM] Fresh status for recreated panel - ahead:', (_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.ahead, 'behind:', (_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.behind);
            // Render fresh
            this.render();
            console.log('[SCM] Panel recreated successfully');
        });
    }
    showBranchSwitcher() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Opening branch switcher');
            try {
                // Get list of branches
                const branchData = yield window.gitAPI.getBranches();
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
                const currentBranch = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.branch) || '';
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
                window.branchSwitcherModal = modal;
                window.branchSwitcherBackdrop = backdrop;
                // Close on backdrop click
                backdrop.onclick = () => this.closeBranchSwitcher();
            }
            catch (error) {
                console.error('[SCM] Failed to get branches:', error);
                alert('Failed to get branch list');
            }
        });
    }
    closeBranchSwitcher() {
        const modal = window.branchSwitcherModal;
        const backdrop = window.branchSwitcherBackdrop;
        if (modal)
            modal.remove();
        if (backdrop)
            backdrop.remove();
        delete window.branchSwitcherModal;
        delete window.branchSwitcherBackdrop;
    }
    switchToBranch(branchName) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Switching to branch:', branchName);
            try {
                // Close the modal first
                this.closeBranchSwitcher();
                // Show progress notification
                const notificationId = notification_1.notifications.show({
                    title: 'Switching Branch',
                    message: `Switching to branch: ${branchName}...`,
                    type: 'info',
                    duration: 0
                });
                // Switch branch
                yield window.gitAPI.switchBranch(branchName);
                // Update notification
                notification_1.notifications.update(notificationId, {
                    title: 'Branch Switched',
                    message: `Successfully switched to branch: ${branchName}`,
                    type: 'success',
                    duration: 3000
                });
                // Refresh the Git panel
                yield this.refresh();
                // Also refresh file explorer to update decorations
                if (window.fileExplorer) {
                    yield window.fileExplorer.refreshGitStatus();
                }
            }
            catch (error) {
                console.error('[SCM] Failed to switch branch:', error);
                notification_1.notifications.show({
                    title: 'Branch Switch Failed',
                    message: (error === null || error === void 0 ? void 0 : error.message) || 'Failed to switch branch',
                    type: 'error',
                    duration: 5000
                });
            }
        });
    }
    showMoreActions() {
        // TODO: Implement dropdown menu with additional Git actions
        console.log('More actions menu - to be implemented');
    }
    openFile(path) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Convert relative path to absolute path for electron-poc
                const fullPath = path.startsWith('/') ? path : `/Users/veronelazio/Developer/Private/hive/${path}`;
                console.log('[SCM] Opening file:', fullPath);
                // Check if file has changes (modified, staged, etc.)
                const fileStatus = (_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files.find(f => f.path === path || `/Users/veronelazio/Developer/Private/hive/${f.path}` === fullPath);
                // Check if this is a submodule (directories like dioxus-fork or src/hive_ui)
                const isSubmodule = path === 'dioxus-fork' || path === 'src/hive_ui';
                if (isSubmodule) {
                    // For submodules, show actual changes
                    console.log('[SCM] Submodule detected:', path);
                    if (window.editorTabs) {
                        try {
                            // Get submodule status and diff
                            const submoduleStatus = yield window.gitAPI.getSubmoduleStatus(fullPath);
                            const submoduleDiff = yield window.gitAPI.getSubmoduleDiff(fullPath);
                            let statusHtml = '';
                            let diffHtml = '';
                            // Parse and format the status
                            if (submoduleStatus) {
                                const lines = submoduleStatus.split('\n').filter((l) => l.trim());
                                statusHtml = lines.map((line) => {
                                    // Color code the status lines
                                    if (line.includes('modified:')) {
                                        return `<div style="color: #e2c08d;">📝 ${line}</div>`;
                                    }
                                    else if (line.includes('new file:')) {
                                        return `<div style="color: #73c991;">➕ ${line}</div>`;
                                    }
                                    else if (line.includes('deleted:')) {
                                        return `<div style="color: #f48771;">➖ ${line}</div>`;
                                    }
                                    else if (line.includes('Your branch')) {
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
                                    }
                                    else if (line.startsWith('-') && !line.startsWith('---')) {
                                        diffHtml += `<span style="color: #f48771;">${this.escapeHtml(line)}</span>\n`;
                                    }
                                    else if (line.startsWith('@@')) {
                                        diffHtml += `<span style="color: #007acc; font-weight: bold;">${this.escapeHtml(line)}</span>\n`;
                                    }
                                    else if (line.startsWith('diff --git')) {
                                        diffHtml += `<span style="color: #e2c08d; font-weight: bold;">${this.escapeHtml(line)}</span>\n`;
                                    }
                                    else {
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
                        }
                        catch (error) {
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
                }
                else if (fileStatus && (fileStatus.working === 'M' || fileStatus.index === 'M' ||
                    fileStatus.working === 'D' || fileStatus.index === 'D' ||
                    fileStatus.working === 'A' || fileStatus.index === 'A')) {
                    // File has changes - show diff view
                    console.log('[SCM] File has changes, showing diff view');
                    yield this.showDiffView(path, fileStatus);
                }
                else {
                    // No changes or untracked file - open normally
                    const content = yield window.fileAPI.readFile(fullPath);
                    // Open file in editor tabs
                    if (window.editorTabs) {
                        window.editorTabs.openFile(fullPath, content);
                        console.log('[SCM] File opened in editor tabs');
                    }
                    else {
                        console.error('[SCM] Editor tabs not available');
                    }
                }
            }
            catch (error) {
                console.error('[SCM] Failed to open file:', error);
            }
        });
    }
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    showDiffView(path, fileStatus) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const fullPath = path.startsWith('/') ? path : `/Users/veronelazio/Developer/Private/hive/${path}`;
                // Get the diff for this file
                let diff;
                if (fileStatus.index !== ' ' && fileStatus.index !== '?') {
                    // File is staged - get staged diff
                    diff = yield window.gitAPI.getStagedDiff(path);
                }
                else {
                    // File is not staged - get working tree diff
                    diff = yield window.gitAPI.getDiff(path);
                }
                // Import and create diff viewer
                const { DiffViewer } = yield Promise.resolve().then(() => __importStar(require('./diff-viewer')));
                const diffViewer = new DiffViewer();
                // Show the diff
                yield diffViewer.showDiff(fullPath, diff);
                // Open diff viewer in a new tab
                if (window.editorTabs) {
                    const container = diffViewer.getContainer();
                    window.editorTabs.openDiffTab(fullPath + ' (diff)', container);
                }
            }
            catch (error) {
                console.error('[SCM] Failed to show diff view:', error);
                // Fallback to normal file opening
                const fullPath = path.startsWith('/') ? path : `/Users/veronelazio/Developer/Private/hive/${path}`;
                const content = yield window.fileAPI.readFile(fullPath);
                if (window.editorTabs) {
                    window.editorTabs.openFile(fullPath, content);
                }
            }
        });
    }
    openDiff(path) {
        // TODO: Open diff view
        console.log('Open diff for', path);
    }
    attachWelcomeStyles() {
        if (document.getElementById('scm-welcome-styles'))
            return;
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
    showErrorActions(errorInfo, notificationId) {
        var _a;
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
        (_a = errorInfo.actions) === null || _a === void 0 ? void 0 : _a.forEach(action => {
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
            button.addEventListener('click', () => __awaiter(this, void 0, void 0, function* () {
                try {
                    yield action.action();
                    // Close notification if action succeeds
                    if (notificationId) {
                        notification_1.notifications.hide(notificationId);
                    }
                }
                catch (error) {
                    console.error('Error executing action:', error);
                }
            }));
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
    pushWithChunks() {
        return __awaiter(this, void 0, void 0, function* () {
            const notificationId = notification_1.notifications.show({
                title: 'Chunked Push',
                message: 'This feature needs to be implemented through the main process...',
                type: 'info',
                duration: 5000
            });
            // TODO: Implement through IPC to main process
            // For now, show the solutions dialog
            git_error_handler_1.GitErrorHandler.showSizeLimitSolutions();
        });
    }
    /**
     * Analyze repository for size issues
     */
    analyzeRepository() {
        return __awaiter(this, void 0, void 0, function* () {
            const notificationId = notification_1.notifications.show({
                title: 'Repository Analysis',
                message: 'This feature needs to be implemented through the main process...',
                type: 'info',
                duration: 5000
            });
            // TODO: Implement through IPC to main process
            // For now, show the solutions dialog
            git_error_handler_1.GitErrorHandler.showSizeLimitSolutions();
        });
    }
    destroy() {
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
exports.VSCodeSCMView = VSCodeSCMView;
//# sourceMappingURL=vscode-scm-view.js.map