"use strict";
/**
 * VS Code-style Git Graph View
 * Shows commit history with visual branch graph
 */
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
exports.GitGraphView = void 0;
class GitGraphView {
    constructor(container) {
        this.commits = [];
        this.refreshInterval = null;
        this.expandedCommits = new Set();
        this.selectedCommit = null;
        console.log('[GitGraph] Constructor called with container:', container);
        this.container = container;
        this.initialize();
    }
    initialize() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[GitGraph] Initializing...');
            this.attachStyles();
            console.log('[GitGraph] Styles attached');
            yield this.loadCommits();
            console.log('[GitGraph] Commits loaded:', this.commits.length);
            this.render();
            console.log('[GitGraph] Rendered');
            // Disable auto-refresh for now to prevent too many calls
            // this.refreshInterval = setInterval(() => this.loadCommits(), 30000);
            // Set up global reference
            window.gitGraph = this;
            console.log('[GitGraph] Global reference set');
        });
    }
    attachStyles() {
        if (document.getElementById('git-graph-styles'))
            return;
        const style = document.createElement('style');
        style.id = 'git-graph-styles';
        style.textContent = `
            .git-graph {
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                background: var(--vscode-editor-background, #1e1e1e);
                color: var(--vscode-foreground, #cccccc);
                font-family: var(--vscode-font-family, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif);
                font-size: 13px;
            }
            
            .git-graph-header {
                display: flex;
                align-items: center;
                padding: 8px 16px;
                background: var(--vscode-sideBarSectionHeader-background, #252526);
                border-bottom: 1px solid var(--vscode-sideBarSectionHeader-border, #1e1e1e);
                min-height: 35px;
            }
            
            .git-graph-title {
                flex: 1;
                font-weight: 600;
                text-transform: uppercase;
                font-size: 11px;
                letter-spacing: 0.5px;
                display: flex;
                align-items: center;
                gap: 8px;
            }
            
            .git-graph-actions {
                display: flex;
                gap: 4px;
            }
            
            .git-graph-action-btn {
                background: transparent;
                border: none;
                color: var(--vscode-foreground, #cccccc);
                cursor: pointer;
                padding: 4px;
                border-radius: 3px;
                display: flex;
                align-items: center;
                justify-content: center;
                width: 22px;
                height: 22px;
            }
            
            .git-graph-action-btn:hover {
                background: var(--vscode-toolbar-hoverBackground, rgba(90, 93, 94, 0.31));
            }
            
            .git-graph-content {
                flex: 1;
                overflow-y: auto;
                overflow-x: hidden;
                padding: 0;
                min-height: 0;
            }
            
            .git-graph-content::-webkit-scrollbar {
                width: 10px;
            }
            
            .git-graph-content::-webkit-scrollbar-track {
                background: transparent;
            }
            
            .git-graph-content::-webkit-scrollbar-thumb {
                background: var(--vscode-scrollbarSlider-background, rgba(121, 121, 121, 0.4));
                border-radius: 5px;
            }
            
            .git-graph-content::-webkit-scrollbar-thumb:hover {
                background: var(--vscode-scrollbarSlider-hoverBackground, rgba(100, 100, 100, 0.7));
            }
            
            .git-graph-commits {
                display: flex;
                flex-direction: column;
            }
            
            .git-graph-commit {
                display: flex;
                align-items: center;
                padding: 4px 8px;
                cursor: pointer;
                position: relative;
                min-height: 24px;
                font-size: 12px;
            }
            
            .git-graph-commit:hover {
                background: var(--vscode-list-hoverBackground, #2a2d2e);
            }
            
            .git-graph-commit.selected {
                background: var(--vscode-list-activeSelectionBackground, #094771);
            }
            
            .git-graph-commit.expanded {
                background: var(--vscode-list-inactiveSelectionBackground, #37373d);
            }
            
            .git-graph-visual {
                width: 30px;
                height: 22px;
                margin-right: 8px;
                position: relative;
                flex-shrink: 0;
            }
            
            .git-graph-line {
                position: absolute;
                width: 1px;
                background: var(--vscode-gitDecoration-modifiedResourceForeground, #e2c08d);
                left: 15px;
                top: 0;
                bottom: 0;
            }
            
            .git-graph-node {
                position: absolute;
                width: 6px;
                height: 6px;
                border-radius: 50%;
                background: var(--vscode-gitDecoration-modifiedResourceForeground, #e2c08d);
                border: 1px solid var(--vscode-editor-background, #1e1e1e);
                left: 12px;
                top: 50%;
                transform: translateY(-50%);
                z-index: 1;
            }
            
            .git-graph-node.merge {
                background: var(--vscode-gitDecoration-modifiedResourceForeground, #e2c08d);
            }
            
            .git-graph-node.tag {
                background: var(--vscode-gitDecoration-addedResourceForeground, #81b88b);
                border-radius: 3px;
            }
            
            .git-graph-commit-info {
                flex: 1;
                display: flex;
                align-items: center;
                gap: 8px;
                min-width: 0;
            }
            
            .git-graph-commit-hash {
                font-family: "SF Mono", Monaco, "Courier New", monospace;
                font-size: 11px;
                color: var(--vscode-gitDecoration-modifiedResourceForeground, #e2c08d);
                flex-shrink: 0;
                opacity: 0.8;
            }
            
            .git-graph-commit-message {
                flex: 1;
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
                font-size: 12px;
            }
            
            .git-graph-commit-refs {
                display: flex;
                gap: 4px;
                flex-shrink: 0;
            }
            
            .git-graph-ref {
                padding: 2px 6px;
                border-radius: 3px;
                font-size: 11px;
                font-weight: 600;
            }
            
            .git-graph-ref.branch {
                background: var(--vscode-gitDecoration-untrackedResourceForeground, #73c991);
                color: var(--vscode-editor-background, #1e1e1e);
            }
            
            .git-graph-ref.tag {
                background: var(--vscode-gitDecoration-addedResourceForeground, #81b88b);
                color: var(--vscode-editor-background, #1e1e1e);
            }
            
            .git-graph-ref.head {
                background: var(--vscode-gitDecoration-modifiedResourceForeground, #e2c08d);
                color: var(--vscode-editor-background, #1e1e1e);
            }
            
            .git-graph-ref.remote {
                background: var(--vscode-gitDecoration-deletedResourceForeground, #c74e39);
                color: white;
            }
            
            .git-graph-commit-author {
                color: var(--vscode-descriptionForeground, #969696);
                font-size: 11px;
                flex-shrink: 0;
                opacity: 0.8;
            }
            
            .git-graph-commit-date {
                color: var(--vscode-descriptionForeground, #969696);
                font-size: 11px;
                flex-shrink: 0;
                opacity: 0.8;
            }
            
            .git-graph-commit-details {
                padding: 12px 12px 12px 132px;
                background: var(--vscode-editor-background, #1e1e1e);
                border-bottom: 1px solid var(--vscode-widget-border, #303031);
                font-size: 12px;
            }
            
            .git-graph-detail-row {
                display: flex;
                margin-bottom: 4px;
            }
            
            .git-graph-detail-label {
                width: 80px;
                color: var(--vscode-descriptionForeground, #969696);
                flex-shrink: 0;
            }
            
            .git-graph-detail-value {
                flex: 1;
                word-break: break-all;
            }
            
            .git-graph-commit-body {
                margin-top: 8px;
                padding-top: 8px;
                border-top: 1px solid var(--vscode-widget-border, #303031);
                white-space: pre-wrap;
                line-height: 1.4;
            }
            
            .git-graph-empty {
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                padding: 40px 20px;
                color: var(--vscode-descriptionForeground, #969696);
            }
            
            .git-graph-empty-icon {
                font-size: 48px;
                margin-bottom: 16px;
                opacity: 0.5;
            }
            
            .git-graph-loading {
                display: flex;
                align-items: center;
                justify-content: center;
                padding: 20px;
                color: var(--vscode-descriptionForeground, #969696);
            }
            
            @keyframes spin {
                0% { transform: rotate(0deg); }
                100% { transform: rotate(360deg); }
            }
            
            .git-graph-spinner {
                display: inline-block;
                width: 20px;
                height: 20px;
                border: 2px solid var(--vscode-widget-border, #303031);
                border-top: 2px solid var(--vscode-foreground, #cccccc);
                border-radius: 50%;
                animation: spin 1s linear infinite;
                margin-right: 10px;
            }
        `;
        document.head.appendChild(style);
    }
    loadCommits() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[GitGraph] loadCommits called');
            try {
                // Get commit log with graph information
                const log = yield window.gitAPI.getLog({
                    maxCount: 200,
                    graph: true
                });
                console.log('[GitGraph] Raw log received:', log ? log.substring(0, 500) : 'empty');
                console.log('[GitGraph] Log type:', typeof log);
                console.log('[GitGraph] Log length:', log ? log.length : 0);
                const parsedCommits = this.parseGitLog(log);
                console.log('[GitGraph] Parsed commits:', parsedCommits.length);
                if (parsedCommits.length > 0) {
                    console.log('[GitGraph] First commit:', parsedCommits[0]);
                    this.commits = parsedCommits;
                    this.render();
                }
                else {
                    console.error('[GitGraph] No commits parsed from log');
                    // Don't clear existing commits if parsing fails
                    if (this.commits.length === 0) {
                        this.render(); // Only render empty if we truly have no commits
                    }
                }
            }
            catch (error) {
                console.error('[GitGraph] Failed to load commits:', error);
                // Don't clear existing commits on error
                if (this.commits.length === 0) {
                    this.render();
                }
            }
        });
    }
    parseGitLog(log) {
        const commits = [];
        console.log('[GitGraph] Parsing log with length:', log.length);
        // Split by newlines to get individual commits
        const lines = log.split('\n').filter(line => line.trim());
        console.log('[GitGraph] Found', lines.length, 'lines');
        for (const line of lines) {
            // Handle lines with graph symbols (*, |, \, /) from --graph option
            const trimmedLine = line.replace(/^[\s\*\|\\\/]+/, '').trim();
            if (trimmedLine.includes('COMMIT_START') && trimmedLine.includes('COMMIT_END')) {
                console.log('[GitGraph] Processing line:', line.substring(0, 100));
                console.log('[GitGraph] Trimmed line:', trimmedLine.substring(0, 100));
                // Remove markers and split by |
                const cleanLine = trimmedLine.replace('COMMIT_START|', '').replace('|COMMIT_END', '');
                const parts = cleanLine.split('|');
                console.log('[GitGraph] Parts count:', parts.length, 'Parts:', parts.slice(0, 5));
                if (parts.length >= 5) {
                    const commit = {
                        hash: parts[0] || '',
                        abbrevHash: (parts[0] || '').substring(0, 7),
                        subject: parts[4] || 'No commit message',
                        body: '',
                        authorName: parts[1] || 'Unknown',
                        authorEmail: parts[2] || '',
                        authorDate: new Date(parts[3] || Date.now()),
                        committerName: parts[1] || 'Unknown',
                        committerEmail: parts[2] || '',
                        committerDate: new Date(parts[3] || Date.now()),
                        refs: [],
                        parent: []
                    };
                    console.log('[GitGraph] Parsed commit:', commit.abbrevHash, '-', commit.subject);
                    commits.push(commit);
                }
                else {
                    console.log('[GitGraph] Not enough parts in line:', parts.length);
                }
            }
        }
        console.log('[GitGraph] Total commits parsed:', commits.length);
        return commits;
    }
    render() {
        console.log('[GitGraph] Rendering with', this.commits.length, 'commits');
        if (this.commits.length === 0) {
            console.log('[GitGraph] Rendering empty state');
            this.renderEmpty();
            return;
        }
        console.log('[GitGraph] Rendering commits list');
        const commitsHtml = this.commits.map((commit, index) => {
            var _a;
            const isExpanded = this.expandedCommits.has(commit.hash);
            const isSelected = this.selectedCommit === commit.hash;
            // Ensure we have valid data before rendering
            const abbrevHash = commit.abbrevHash || ((_a = commit.hash) === null || _a === void 0 ? void 0 : _a.substring(0, 7)) || 'unknown';
            const subject = commit.subject || 'No commit message';
            const authorName = commit.authorName || 'Unknown';
            return `
                <div class="git-graph-commit ${isSelected ? 'selected' : ''}" 
                     data-hash="${commit.hash}"
                     onclick="window.gitGraph?.toggleCommit('${commit.hash}')">
                    <div class="git-graph-visual">
                        ${index < this.commits.length - 1 ? '<div class="git-graph-line"></div>' : ''}
                        <div class="git-graph-node"></div>
                    </div>
                    <div class="git-graph-commit-info">
                        <span class="git-graph-commit-hash">${abbrevHash}</span>
                        <span class="git-graph-commit-message" title="${this.escapeHtml(subject)}">${this.escapeHtml(subject)}</span>
                        <span class="git-graph-commit-author">${authorName}</span>
                        <span class="git-graph-commit-date">${this.formatDate(commit.authorDate)}</span>
                    </div>
                </div>
            `;
        }).join('');
        this.container.innerHTML = `
            <div class="git-graph">
                <div class="git-graph-header">
                    <div class="git-graph-title">
                        <span class="codicon codicon-git-commit"></span>
                        <span>COMMITS</span>
                    </div>
                    <div class="git-graph-actions">
                        <button class="git-graph-action-btn" title="Refresh" onclick="window.gitGraph?.refresh()">
                            <span class="codicon codicon-refresh"></span>
                        </button>
                        <button class="git-graph-action-btn" title="View Git Graph" onclick="window.gitGraph?.openFullGraph()">
                            <span class="codicon codicon-graph"></span>
                        </button>
                    </div>
                </div>
                <div class="git-graph-content">
                    <div class="git-graph-commits">
                        ${commitsHtml}
                    </div>
                </div>
            </div>
        `;
    }
    renderEmpty() {
        this.container.innerHTML = `
            <div class="git-graph">
                <div class="git-graph-header">
                    <div class="git-graph-title">
                        <span class="codicon codicon-git-commit"></span>
                        <span>COMMITS</span>
                    </div>
                    <div class="git-graph-actions">
                        <button class="git-graph-action-btn" title="Refresh" onclick="window.gitGraph?.refresh()">
                            <span class="codicon codicon-refresh"></span>
                        </button>
                    </div>
                </div>
                <div class="git-graph-content">
                    <div class="git-graph-empty">
                        <div class="git-graph-empty-icon">📊</div>
                        <div>No commits yet</div>
                        <div style="font-size: 12px; margin-top: 8px;">Make your first commit to see the history</div>
                    </div>
                </div>
            </div>
        `;
    }
    renderRefs(refs) {
        if (refs.length === 0)
            return '';
        return `
            <div class="git-graph-commit-refs">
                ${refs.map(ref => {
            let className = 'branch';
            if (ref.includes('HEAD'))
                className = 'head';
            else if (ref.includes('tag:'))
                className = 'tag';
            else if (ref.includes('origin/'))
                className = 'remote';
            return `<span class="git-graph-ref ${className}">${ref}</span>`;
        }).join('')}
            </div>
        `;
    }
    renderCommitDetails(commit) {
        return `
            <div class="git-graph-commit-details">
                <div class="git-graph-detail-row">
                    <span class="git-graph-detail-label">Commit:</span>
                    <span class="git-graph-detail-value">${commit.hash}</span>
                </div>
                <div class="git-graph-detail-row">
                    <span class="git-graph-detail-label">Author:</span>
                    <span class="git-graph-detail-value">${commit.authorName} &lt;${commit.authorEmail}&gt;</span>
                </div>
                <div class="git-graph-detail-row">
                    <span class="git-graph-detail-label">Date:</span>
                    <span class="git-graph-detail-value">${commit.authorDate.toLocaleString()}</span>
                </div>
                ${commit.parent.length > 0 ? `
                    <div class="git-graph-detail-row">
                        <span class="git-graph-detail-label">Parents:</span>
                        <span class="git-graph-detail-value">${commit.parent.join(', ')}</span>
                    </div>
                ` : ''}
                ${commit.body ? `
                    <div class="git-graph-commit-body">${this.escapeHtml(commit.body)}</div>
                ` : ''}
            </div>
        `;
    }
    formatDate(date) {
        const now = new Date();
        const diff = now.getTime() - date.getTime();
        const seconds = Math.floor(diff / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);
        if (days > 0)
            return `${days} day${days > 1 ? 's' : ''} ago`;
        if (hours > 0)
            return `${hours} hour${hours > 1 ? 's' : ''} ago`;
        if (minutes > 0)
            return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
        return 'just now';
    }
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    // Public methods
    toggleCommit(hash) {
        // Open commit details in main window instead of expanding inline
        this.selectedCommit = hash;
        const commit = this.commits.find(c => c.hash === hash);
        if (commit) {
            this.openCommitDetails(commit);
        }
        this.render();
    }
    openCommitDetails(commit) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[GitGraph] Opening commit details for:', commit.hash);
            // Get the commit details including changed files
            const commitInfo = yield this.getCommitDetails(commit.hash);
            // Create a commit details view and open it in the editor
            if (window.editorTabs) {
                const container = yield this.createCommitDetailsView(commit, commitInfo);
                const title = `Commit ${commit.abbrevHash}: ${commit.subject}`;
                // Open in a new tab
                window.editorTabs.openCustomTab(`commit-${commit.hash}`, title, container);
            }
        });
    }
    getCommitDetails(hash) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Get the list of files changed in this commit
                const result = yield window.gitAPI.getCommitFiles(hash);
                return result;
            }
            catch (error) {
                console.error('[GitGraph] Failed to get commit details:', error);
                return { files: [] };
            }
        });
    }
    createCommitDetailsView(commit, commitInfo) {
        return __awaiter(this, void 0, void 0, function* () {
            const files = Array.isArray(commitInfo) ? commitInfo : (commitInfo.files || []);
            // Create a container for the commit view
            const container = document.createElement('div');
            container.className = 'commit-details-container';
            container.style.cssText = 'height: 100%; overflow: auto; background: var(--vscode-editor-background);';
            // Add commit header
            const header = document.createElement('div');
            header.className = 'commit-header';
            header.style.cssText = 'padding: 16px; border-bottom: 1px solid var(--vscode-widget-border);';
            header.innerHTML = `
            <h2 style="margin: 0 0 8px 0; font-size: 18px;">Commit ${commit.abbrevHash}</h2>
            <div style="font-size: 14px; margin-bottom: 4px;">${commit.subject}</div>
            <div style="font-size: 12px; color: var(--vscode-descriptionForeground);">
                <span>Author: ${commit.authorName} &lt;${commit.authorEmail}&gt;</span><br>
                <span>Date: ${commit.authorDate.toLocaleString()}</span>
            </div>
        `;
            container.appendChild(header);
            // Add file diffs
            const diffsContainer = document.createElement('div');
            diffsContainer.className = 'commit-diffs';
            container.appendChild(diffsContainer);
            // Load diff for each file
            for (const file of files) {
                // Convert string to object if needed
                const fileObj = typeof file === 'string' ? { path: file, status: 'M' } : file;
                yield this.addFileDiff(diffsContainer, commit.hash, fileObj);
            }
            return container;
        });
    }
    getStatusLabel(status) {
        switch (status) {
            case 'A': return 'Added';
            case 'M': return 'Modified';
            case 'D': return 'Deleted';
            case 'R': return 'Renamed';
            case 'C': return 'Copied';
            default: return status;
        }
    }
    addFileDiff(container, commitHash, file) {
        return __awaiter(this, void 0, void 0, function* () {
            // Create file section
            const fileSection = document.createElement('div');
            fileSection.className = 'file-diff-section';
            fileSection.style.cssText = 'margin: 8px 0; border: 1px solid var(--vscode-panel-border, #2d2d30);';
            // File header - VS Code style
            const fileHeader = document.createElement('div');
            fileHeader.className = 'file-diff-header';
            fileHeader.style.cssText = `
            padding: 8px 16px;
            background: var(--vscode-editor-inactiveSelectionBackground, #3f3f46);
            border-bottom: 1px solid var(--vscode-panel-border, #2d2d30);
            font-size: 13px;
            display: flex;
            align-items: center;
            gap: 8px;
            cursor: pointer;
            user-select: none;
        `;
            // Add expand/collapse icon
            const expandIcon = document.createElement('span');
            expandIcon.className = 'codicon codicon-chevron-down';
            expandIcon.style.cssText = 'margin-right: 4px;';
            // Determine file status from the diff if not provided
            const fileStatus = file.status || 'M';
            const statusBadge = document.createElement('span');
            statusBadge.style.cssText = this.getStatusStyle(fileStatus);
            statusBadge.textContent = this.getStatusLabel(fileStatus);
            const fileName = document.createElement('span');
            fileName.style.cssText = 'flex: 1; font-family: var(--vscode-editor-font-family); color: var(--vscode-foreground);';
            fileName.textContent = file.path || file;
            fileHeader.appendChild(expandIcon);
            fileHeader.appendChild(statusBadge);
            fileHeader.appendChild(fileName);
            fileSection.appendChild(fileHeader);
            // Get the diff for this file
            try {
                const diff = yield window.gitAPI.getFileDiff(commitHash, file.path);
                if (diff) {
                    // Create diff viewer
                    const diffViewer = document.createElement('div');
                    diffViewer.className = 'file-diff-content';
                    diffViewer.style.cssText = `
                    max-height: 500px;
                    overflow: auto;
                    background: var(--vscode-editor-background, #1e1e1e);
                    border-top: 1px solid var(--vscode-panel-border, #2d2d30);
                `;
                    // Parse and render the diff
                    this.renderDiff(diffViewer, diff);
                    fileSection.appendChild(diffViewer);
                    // Toggle collapse/expand
                    let isExpanded = true;
                    fileHeader.addEventListener('click', () => {
                        isExpanded = !isExpanded;
                        diffViewer.style.display = isExpanded ? 'block' : 'none';
                        expandIcon.className = isExpanded ? 'codicon codicon-chevron-down' : 'codicon codicon-chevron-right';
                    });
                }
            }
            catch (error) {
                console.error('Failed to get diff for file:', file.path, error);
                const errorMsg = document.createElement('div');
                errorMsg.style.cssText = 'padding: 8px 16px; color: var(--vscode-errorForeground, #f48771);';
                errorMsg.textContent = 'Failed to load diff';
                fileSection.appendChild(errorMsg);
            }
            container.appendChild(fileSection);
        });
    }
    getStatusStyle(status) {
        switch (status) {
            case 'A': return 'padding: 2px 6px; background: #28a745; color: white; border-radius: 3px; font-size: 11px;';
            case 'M': return 'padding: 2px 6px; background: #e2c08d; color: black; border-radius: 3px; font-size: 11px;';
            case 'D': return 'padding: 2px 6px; background: #dc3545; color: white; border-radius: 3px; font-size: 11px;';
            default: return 'padding: 2px 6px; background: #6c757d; color: white; border-radius: 3px; font-size: 11px;';
        }
    }
    renderDiff(container, diff) {
        const lines = diff.split('\n');
        const table = document.createElement('table');
        table.style.cssText = 'width: 100%; border-collapse: collapse; font-family: "SF Mono", Monaco, Consolas, "Courier New", monospace; font-size: 12px;';
        let lineNumOld = 0;
        let lineNumNew = 0;
        let inHunk = false;
        lines.forEach(line => {
            const tr = document.createElement('tr');
            if (line.startsWith('@@')) {
                // Parse line numbers from hunk header
                const match = line.match(/@@ -(\d+),?\d* \+(\d+),?\d* @@/);
                if (match) {
                    lineNumOld = parseInt(match[1]) - 1;
                    lineNumNew = parseInt(match[2]) - 1;
                    inHunk = true;
                }
                tr.style.cssText = 'background: rgba(0, 123, 255, 0.05); color: #0366d6;';
                tr.innerHTML = `<td colspan="4" style="padding: 4px 8px; font-style: italic; color: #586069;">${this.escapeHtml(line)}</td>`;
            }
            else if (line.startsWith('diff --git') || line.startsWith('index ') || line.startsWith('new file') || line.startsWith('---') || line.startsWith('+++')) {
                // Skip or style diff headers minimally
                if (line.startsWith('+++') || line.startsWith('---')) {
                    return; // Skip these lines entirely
                }
                tr.style.cssText = 'color: #586069;';
                tr.innerHTML = `<td colspan="4" style="padding: 2px 8px; font-size: 11px;">${this.escapeHtml(line)}</td>`;
            }
            else if (line.startsWith('+') && inHunk) {
                lineNumNew++;
                tr.style.cssText = 'background: rgba(40, 167, 69, 0.15);';
                tr.innerHTML = `
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;"></td>
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;">${lineNumNew}</td>
                    <td style="width: 20px; text-align: center; color: #28a745; user-select: none;">+</td>
                    <td style="padding: 0 8px; white-space: pre;"><span style="color: #22863a;">${this.escapeHtml(line.substring(1))}</span></td>`;
            }
            else if (line.startsWith('-') && inHunk) {
                lineNumOld++;
                tr.style.cssText = 'background: rgba(220, 53, 69, 0.15);';
                tr.innerHTML = `
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;">${lineNumOld}</td>
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;"></td>
                    <td style="width: 20px; text-align: center; color: #dc3545; user-select: none;">-</td>
                    <td style="padding: 0 8px; white-space: pre;"><span style="color: #cb2431;">${this.escapeHtml(line.substring(1))}</span></td>`;
            }
            else if (inHunk && line.length > 0) {
                // Context line
                lineNumOld++;
                lineNumNew++;
                tr.style.cssText = '';
                tr.innerHTML = `
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;">${lineNumOld}</td>
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;">${lineNumNew}</td>
                    <td style="width: 20px; text-align: center; color: #959da5; user-select: none;"></td>
                    <td style="padding: 0 8px; white-space: pre; color: #24292e;">${this.escapeHtml(line)}</td>`;
            }
            if (tr.innerHTML) {
                table.appendChild(tr);
            }
        });
        container.appendChild(table);
    }
    viewFileDiff(commitHash, filePath) {
        return __awaiter(this, void 0, void 0, function* () {
            // Show diff for a specific file in the commit
            console.log('[GitGraph] Viewing diff for:', filePath, 'in commit:', commitHash);
            // TODO: Implement file diff viewer
        });
    }
    refresh() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.loadCommits();
        });
    }
    openFullGraph() {
        // This could open a more detailed graph view in a new tab
        console.log('[GitGraph] Opening full graph view...');
        alert('Full Git Graph view would open here (like Git Graph extension)');
    }
    destroy() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
        }
    }
}
exports.GitGraphView = GitGraphView;
//# sourceMappingURL=git-graph.js.map