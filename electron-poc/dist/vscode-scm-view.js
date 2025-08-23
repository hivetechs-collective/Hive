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
            // Start auto-refresh
            yield this.refresh();
            this.refreshInterval = setInterval(() => this.refresh(), 2000);
            // Set up global reference
            window.scmView = this;
        });
    }
    refresh() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                this.gitStatus = yield window.gitAPI.getStatus();
            }
            catch (error) {
                console.error('[SCM] Failed to refresh:', error);
                // Set gitStatus to null to show welcome message
                this.gitStatus = null;
            }
            // Always render, even if there was an error
            this.render();
        });
    }
    render() {
        var _a, _b, _c, _d;
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
              <button class="scm-toolbar-button" title="Pull${((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.behind) ? ` (${this.gitStatus.behind} behind)` : ''}" onclick="window.scmView?.pull()">
                <span class="codicon codicon-cloud-download"></span>
              </button>
              <button class="scm-toolbar-button" title="Push${((_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.ahead) ? ` (${this.gitStatus.ahead} ahead)` : ''}" onclick="window.scmView?.push()">
                <span class="codicon codicon-cloud-upload"></span>
              </button>
              <button class="scm-toolbar-button" title="Sync Changes${((_c = this.gitStatus) === null || _c === void 0 ? void 0 : _c.ahead) || ((_d = this.gitStatus) === null || _d === void 0 ? void 0 : _d.behind) ? ` (${this.gitStatus.ahead || 0}↑ ${this.gitStatus.behind || 0}↓)` : ''}" onclick="window.scmView?.sync()">
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
        const changes = files.filter(f => f.working !== ' ' && f.working !== '?' && !(f.index !== ' ' && f.index !== '?'));
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
          <div class="scm-resource-group-content">
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
    stageAll() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const changes = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files.filter(f => f.working !== ' ' && !(f.index !== ' ' && f.index !== '?'))) || [];
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
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            if (!confirm('Discard all changes?'))
                return;
            try {
                let files = [];
                if (groupId === 'changes') {
                    files = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.files.filter(f => f.working !== ' ' && f.working !== '?').map(f => f.path)) || [];
                }
                if (files.length > 0) {
                    yield window.gitAPI.discard(files);
                    yield this.refresh();
                }
            }
            catch (error) {
                console.error('Failed to discard all:', error);
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
                yield this.refresh();
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
        var _a, _b, _c, _d, _e, _f, _g, _h;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Push button clicked');
            // Check if gitAPI is available
            if (!window.gitAPI || !window.gitAPI.push) {
                alert('Git API not available!');
                console.error('[SCM] window.gitAPI:', window.gitAPI);
                return;
            }
            const branch = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.branch) || 'current branch';
            const aheadCount = ((_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.ahead) || 0;
            console.log('[SCM] Current status - branch:', branch, 'ahead:', aheadCount, 'hasUpstream:', (_c = this.gitStatus) === null || _c === void 0 ? void 0 : _c.hasUpstream);
            // For branches without upstream, we still want to push if there are local commits
            if (aheadCount === 0 && ((_d = this.gitStatus) === null || _d === void 0 ? void 0 : _d.hasUpstream)) {
                notification_1.notifications.show({
                    title: 'Nothing to push',
                    message: 'Your branch is up to date with remote',
                    type: 'info',
                    duration: 3000
                });
                return;
            }
            // Show loading notification
            const pushMessage = !((_e = this.gitStatus) === null || _e === void 0 ? void 0 : _e.hasUpstream) ?
                `Publishing branch ${branch} to remote...` :
                `Pushing ${aheadCount} commit(s) to remote...`;
            console.log('[SCM] Showing push notification:', pushMessage);
            const notificationId = notification_1.notifications.show({
                title: !((_f = this.gitStatus) === null || _f === void 0 ? void 0 : _f.hasUpstream) ? 'Publishing Branch' : 'Git Push',
                message: pushMessage,
                type: 'loading',
                duration: 0 // Persistent until updated
            });
            try {
                console.log('[SCM] About to call gitAPI.push()');
                // Show progress with regular updates
                let progressInterval = setInterval(() => {
                    const currentMessage = document.querySelector('.notification-message');
                    if (currentMessage && currentMessage.textContent) {
                        const dots = (currentMessage.textContent.match(/\./g) || []).length;
                        if (dots < 3) {
                            currentMessage.textContent += '.';
                        }
                        else {
                            currentMessage.textContent = currentMessage.textContent.replace(/\.+$/, '');
                        }
                    }
                }, 500);
                // Add timeout for push operation - 30 seconds
                const pushPromise = window.gitAPI.push();
                const timeoutPromise = new Promise((_, reject) => setTimeout(() => reject(new Error('Push operation timed out after 30 seconds')), 30000));
                try {
                    yield Promise.race([pushPromise, timeoutPromise]);
                    clearInterval(progressInterval);
                    console.log('[SCM] Push completed, refreshing status...');
                    yield this.refresh();
                    console.log('[SCM] Status refreshed, new ahead count:', (_g = this.gitStatus) === null || _g === void 0 ? void 0 : _g.ahead);
                    // Update to success notification
                    const successMessage = !((_h = this.gitStatus) === null || _h === void 0 ? void 0 : _h.hasUpstream) ?
                        `Successfully published ${branch} to remote` :
                        `Successfully pushed ${aheadCount} commit(s) to ${branch}`;
                    notification_1.notifications.update(notificationId, {
                        title: 'Push Successful',
                        message: successMessage,
                        type: 'success',
                        duration: 3000
                    });
                }
                catch (innerError) {
                    clearInterval(progressInterval);
                    throw innerError;
                }
            }
            catch (error) {
                console.error('[SCM] Push failed:', error);
                notification_1.notifications.update(notificationId, {
                    title: 'Push Failed',
                    message: (error === null || error === void 0 ? void 0 : error.message) || 'An error occurred while pushing',
                    type: 'error',
                    duration: 5000
                });
            }
        });
    }
    pull() {
        var _a, _b;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Pull button clicked');
            // Check if there's anything to pull
            if (((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.behind) === 0) {
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
                message: `Pulling ${((_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.behind) || ''} commit(s) from remote...`,
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
        var _a, _b, _c;
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[SCM] Sync button clicked');
            const branch = ((_a = this.gitStatus) === null || _a === void 0 ? void 0 : _a.branch) || 'current branch';
            const ahead = ((_b = this.gitStatus) === null || _b === void 0 ? void 0 : _b.ahead) || 0;
            const behind = ((_c = this.gitStatus) === null || _c === void 0 ? void 0 : _c.behind) || 0;
            console.log('[SCM] Sync status - branch:', branch, 'ahead:', ahead, 'behind:', behind);
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