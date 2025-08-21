/**
 * VS Code-style Git Graph View
 * Shows commit history with visual branch graph
 */

export interface GitCommit {
    hash: string;
    abbrevHash: string;
    subject: string;
    body: string;
    authorName: string;
    authorEmail: string;
    authorDate: Date;
    committerName: string;
    committerEmail: string;
    committerDate: Date;
    refs: string[];
    parent: string[];
    files?: string[]; // Files changed in this commit
    stats?: { additions: number; deletions: number; }; // Change statistics
}

export class GitGraphView {
    private container: HTMLElement;
    private commits: GitCommit[] = [];
    private refreshInterval: NodeJS.Timeout | null = null;
    private expandedCommits: Set<string> = new Set();
    private selectedCommit: string | null = null;

    constructor(container: HTMLElement) {
        console.log('[GitGraph] Constructor called with container:', container);
        this.container = container;
        this.initialize();
    }

    private async initialize() {
        console.log('[GitGraph] Initializing...');
        this.attachStyles();
        console.log('[GitGraph] Styles attached');
        await this.loadCommits();
        console.log('[GitGraph] Commits loaded:', this.commits.length);
        this.render();
        console.log('[GitGraph] Rendered');
        
        // Disable auto-refresh for now to prevent too many calls
        // this.refreshInterval = setInterval(() => this.loadCommits(), 30000);
        
        // Set up global reference
        (window as any).gitGraph = this;
        console.log('[GitGraph] Global reference set');
    }

    private attachStyles() {
        if (document.getElementById('git-graph-styles')) return;
        
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

    private async loadCommits() {
        console.log('[GitGraph] loadCommits called');
        try {
            // Get commit log with graph information
            const log = await window.gitAPI.getLog({
                maxCount: 20,  // Reduced from 200 to prevent freezing
                graph: true
            }) as string;
            
            console.log('[GitGraph] Raw log received:', log ? log.substring(0, 500) : 'empty');
            console.log('[GitGraph] Log type:', typeof log);
            console.log('[GitGraph] Log length:', log ? log.length : 0);
            
            const parsedCommits = this.parseGitLog(log);
            console.log('[GitGraph] Parsed commits:', parsedCommits.length);
            if (parsedCommits.length > 0) {
                console.log('[GitGraph] First commit:', parsedCommits[0]);
                this.commits = parsedCommits;
                this.render();
            } else {
                console.error('[GitGraph] No commits parsed from log');
                // Don't clear existing commits if parsing fails
                if (this.commits.length === 0) {
                    this.render(); // Only render empty if we truly have no commits
                }
            }
        } catch (error) {
            console.error('[GitGraph] Failed to load commits:', error);
            // Don't clear existing commits on error
            if (this.commits.length === 0) {
                this.render();
            }
        }
    }

    private parseGitLog(log: string): GitCommit[] {
        const commits: GitCommit[] = [];
        
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
                    const commit: GitCommit = {
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
                } else {
                    console.log('[GitGraph] Not enough parts in line:', parts.length);
                }
            }
        }
        
        console.log('[GitGraph] Total commits parsed:', commits.length);
        return commits;
    }

    private render() {
        console.log('[GitGraph] Rendering with', this.commits.length, 'commits');
        
        if (this.commits.length === 0) {
            console.log('[GitGraph] Rendering empty state');
            this.renderEmpty();
            return;
        }
        
        console.log('[GitGraph] Rendering commits list');
        
        const commitsHtml = this.commits.map((commit, index) => {
            const isExpanded = this.expandedCommits.has(commit.hash);
            const isSelected = this.selectedCommit === commit.hash;
            
            // Ensure we have valid data before rendering
            const abbrevHash = commit.abbrevHash || commit.hash?.substring(0, 7) || 'unknown';
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

    private renderEmpty() {
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

    private renderRefs(refs: string[]): string {
        if (refs.length === 0) return '';
        
        return `
            <div class="git-graph-commit-refs">
                ${refs.map(ref => {
                    let className = 'branch';
                    if (ref.includes('HEAD')) className = 'head';
                    else if (ref.includes('tag:')) className = 'tag';
                    else if (ref.includes('origin/')) className = 'remote';
                    
                    return `<span class="git-graph-ref ${className}">${ref}</span>`;
                }).join('')}
            </div>
        `;
    }

    private renderCommitDetails(commit: GitCommit): string {
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

    private formatDate(date: Date): string {
        const now = new Date();
        const diff = now.getTime() - date.getTime();
        const seconds = Math.floor(diff / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);
        
        if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
        if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
        if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
        return 'just now';
    }

    private escapeHtml(text: string): string {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Public methods
    public toggleCommit(hash: string) {
        // Open commit details in main window instead of expanding inline
        this.selectedCommit = hash;
        const commit = this.commits.find(c => c.hash === hash);
        if (commit) {
            this.openCommitDetails(commit);
        }
        this.render();
    }
    
    private async openCommitDetails(commit: GitCommit) {
        console.log('[GitGraph] Opening commit details for:', commit.hash);
        
        // Get the commit details including changed files
        const commitInfo = await this.getCommitDetails(commit.hash);
        
        // Create a commit details view and open it in the editor
        if (window.editorTabs) {
            const container = await this.createCommitDetailsView(commit, commitInfo);
            const title = `Commit ${commit.abbrevHash}: ${commit.subject}`;
            
            // Open in a new tab
            window.editorTabs.openCustomTab(`commit-${commit.hash}`, title, container);
        }
    }
    
    private async getCommitDetails(hash: string): Promise<any> {
        try {
            // Get the list of files changed in this commit
            const result = await window.gitAPI.getCommitFiles(hash);
            return result;
        } catch (error) {
            console.error('[GitGraph] Failed to get commit details:', error);
            return { files: [] };
        }
    }
    
    private async createCommitDetailsView(commit: GitCommit, commitInfo: any): Promise<HTMLElement> {
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
            await this.addFileDiff(diffsContainer, commit.hash, fileObj);
        }
        
        return container;
    }
    
    private getStatusLabel(status: string): string {
        switch (status) {
            case 'A': return 'Added';
            case 'M': return 'Modified';
            case 'D': return 'Deleted';
            case 'R': return 'Renamed';
            case 'C': return 'Copied';
            default: return status;
        }
    }
    
    private async addFileDiff(container: HTMLElement, commitHash: string, file: any): Promise<void> {
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
            const diff = await window.gitAPI.getFileDiff(commitHash, file.path);
            
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
        } catch (error) {
            console.error('Failed to get diff for file:', file.path, error);
            const errorMsg = document.createElement('div');
            errorMsg.style.cssText = 'padding: 8px 16px; color: var(--vscode-errorForeground, #f48771);';
            errorMsg.textContent = 'Failed to load diff';
            fileSection.appendChild(errorMsg);
        }
        
        container.appendChild(fileSection);
    }
    
    private getStatusStyle(status: string): string {
        switch (status) {
            case 'A': return 'padding: 2px 6px; background: #28a745; color: white; border-radius: 3px; font-size: 11px;';
            case 'M': return 'padding: 2px 6px; background: #e2c08d; color: black; border-radius: 3px; font-size: 11px;';
            case 'D': return 'padding: 2px 6px; background: #dc3545; color: white; border-radius: 3px; font-size: 11px;';
            default: return 'padding: 2px 6px; background: #6c757d; color: white; border-radius: 3px; font-size: 11px;';
        }
    }
    
    private renderDiff(container: HTMLElement, diff: string): void {
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
            } else if (line.startsWith('diff --git') || line.startsWith('index ') || line.startsWith('new file') || line.startsWith('---') || line.startsWith('+++')) {
                // Skip or style diff headers minimally
                if (line.startsWith('+++') || line.startsWith('---')) {
                    return; // Skip these lines entirely
                }
                tr.style.cssText = 'color: #586069;';
                tr.innerHTML = `<td colspan="4" style="padding: 2px 8px; font-size: 11px;">${this.escapeHtml(line)}</td>`;
            } else if (line.startsWith('+') && inHunk) {
                lineNumNew++;
                tr.style.cssText = 'background: rgba(40, 167, 69, 0.15);';
                tr.innerHTML = `
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;"></td>
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;">${lineNumNew}</td>
                    <td style="width: 20px; text-align: center; color: #28a745; user-select: none;">+</td>
                    <td style="padding: 0 8px; white-space: pre;"><span style="color: #22863a;">${this.escapeHtml(line.substring(1))}</span></td>`;
            } else if (line.startsWith('-') && inHunk) {
                lineNumOld++;
                tr.style.cssText = 'background: rgba(220, 53, 69, 0.15);';
                tr.innerHTML = `
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;">${lineNumOld}</td>
                    <td style="width: 50px; text-align: right; padding: 0 8px; color: #959da5; user-select: none;"></td>
                    <td style="width: 20px; text-align: center; color: #dc3545; user-select: none;">-</td>
                    <td style="padding: 0 8px; white-space: pre;"><span style="color: #cb2431;">${this.escapeHtml(line.substring(1))}</span></td>`;
            } else if (inHunk && line.length > 0) {
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
    
    public async viewFileDiff(commitHash: string, filePath: string) {
        // Show diff for a specific file in the commit
        console.log('[GitGraph] Viewing diff for:', filePath, 'in commit:', commitHash);
        // TODO: Implement file diff viewer
    }

    public async refresh() {
        await this.loadCommits();
    }

    public openFullGraph() {
        // This could open a more detailed graph view in a new tab
        console.log('[GitGraph] Opening full graph view...');
        alert('Full Git Graph view would open here (like Git Graph extension)');
    }

    public destroy() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
        }
    }
}