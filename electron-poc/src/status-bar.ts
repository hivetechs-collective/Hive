/**
 * VS Code-style Status Bar
 * Shows Git branch, file info, and other status indicators
 */

import { GitStatus } from './types/git';

export class StatusBar {
  private container: HTMLElement;
  private gitStatus: GitStatus | null = null;
  private updateInterval: NodeJS.Timeout | null = null;
  private currentFile: string | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
    this.initialize();
  }

  private initialize() {
    this.container.className = 'status-bar';
    this.render();
    this.startPeriodicUpdate();
  }

  private startPeriodicUpdate() {
    // Update Git status every 5 seconds
    this.updateInterval = setInterval(() => {
      this.updateGitStatus();
    }, 5000);
    
    // Initial update
    this.updateGitStatus();
  }

  private async updateGitStatus() {
    try {
      this.gitStatus = await window.gitAPI.getStatus();
      this.render();
    } catch (error) {
      console.warn('Status bar: Failed to get Git status:', error);
      this.gitStatus = null;
      this.render();
    }
  }

  private render() {
    const leftItems = this.renderLeftItems();
    const rightItems = this.renderRightItems();

    this.container.innerHTML = `
      <div class="status-bar-left">
        ${leftItems}
      </div>
      <div class="status-bar-right">
        ${rightItems}
      </div>
    `;
  }

  private renderLeftItems(): string {
    const items: string[] = [];

    // Git branch info
    if (this.gitStatus && this.gitStatus.isRepo) {
      const { branch, ahead, behind, files } = this.gitStatus;
      const changedFiles = files.filter(f => f.index !== ' ' || f.working !== ' ').length;
      
      let gitInfo = `
        <div class="status-item git-branch" title="Git branch: ${branch}">
          <svg width="16" height="16" viewBox="0 0 16 16" class="status-icon">
            <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 113 2.122V6A2.5 2.5 0 0110 8.5H6a1 1 0 00-1 1v1.128a2.251 2.251 0 11-1.5 0V5.372a2.25 2.25 0 111.5 0v1.836A2.492 2.492 0 016 7h4a1 1 0 001-1v-.628A2.25 2.25 0 019.5 3.25zm-6 0a.75.75 0 101.5 0 .75.75 0 00-1.5 0zm8.25-.75a.75.75 0 100 1.5.75.75 0 000-1.5zM4.25 12a.75.75 0 100 1.5.75.75 0 000-1.5z"/>
          </svg>
          <span>${branch}</span>
      `;

      // Add sync indicators
      if (ahead > 0) {
        gitInfo += `<span class="sync-indicator ahead" title="${ahead} commits ahead">↑${ahead}</span>`;
      }
      if (behind > 0) {
        gitInfo += `<span class="sync-indicator behind" title="${behind} commits behind">↓${behind}</span>`;
      }

      // Add changed files indicator
      if (changedFiles > 0) {
        gitInfo += `<span class="changes-indicator" title="${changedFiles} changed files">${changedFiles}</span>`;
      }

      gitInfo += '</div>';
      items.push(gitInfo);
    }

    // Current file info
    if (this.currentFile) {
      const fileName = this.currentFile.split('/').pop() || this.currentFile;
      items.push(`
        <div class="status-item current-file" title="Current file: ${this.currentFile}">
          <svg width="16" height="16" viewBox="0 0 16 16" class="status-icon">
            <path fill="currentColor" d="M4 0h8a2 2 0 012 2v12a2 2 0 01-2 2H4a2 2 0 01-2-2V2a2 2 0 012-2zM2 3v10a1 1 0 001 1h10a1 1 0 001-1V3H2z"/>
          </svg>
          <span>${fileName}</span>
        </div>
      `);
    }

    return items.join('');
  }

  private renderRightItems(): string {
    const items: string[] = [];

    // Git status summary
    if (this.gitStatus && this.gitStatus.isRepo && this.gitStatus.files.length > 0) {
      const staged = this.gitStatus.files.filter(f => f.index !== ' ' && f.index !== '?').length;
      const modified = this.gitStatus.files.filter(f => f.working === 'M').length;
      const untracked = this.gitStatus.files.filter(f => f.working === '?').length;

      if (staged > 0) {
        items.push(`
          <div class="status-item git-staged" title="${staged} staged files">
            <svg width="16" height="16" viewBox="0 0 16 16" class="status-icon">
              <path fill="currentColor" d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.75.75 0 011.06-1.06L6 10.94l6.72-6.72a.75.75 0 011.06 0z"/>
            </svg>
            <span>${staged}</span>
          </div>
        `);
      }

      if (modified > 0) {
        items.push(`
          <div class="status-item git-modified" title="${modified} modified files">
            <svg width="16" height="16" viewBox="0 0 16 16" class="status-icon">
              <path fill="currentColor" d="M11.013 1.427a1.75 1.75 0 012.474 0l1.086 1.086a1.75 1.75 0 010 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 01-.927-.928l.929-3.25a1.75 1.75 0 01.445-.758l8.61-8.61zm1.414 1.06a.25.25 0 00-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 000-.354l-1.086-1.086zM11.189 6.25L9.75 4.81l-6.286 6.287a.25.25 0 00-.064.108l-.558 1.953 1.953-.558a.249.249 0 00.108-.064l6.286-6.286z"/>
            </svg>
            <span>${modified}</span>
          </div>
        `);
      }

      if (untracked > 0) {
        items.push(`
          <div class="status-item git-untracked" title="${untracked} untracked files">
            <svg width="16" height="16" viewBox="0 0 16 16" class="status-icon">
              <path fill="currentColor" d="M7.75 2a.75.75 0 01.75.75V8h5.25a.75.75 0 010 1.5H8.5v5.25a.75.75 0 01-1.5 0V9.5H1.75a.75.75 0 010-1.5H7V2.75A.75.75 0 017.75 2z"/>
            </svg>
            <span>${untracked}</span>
          </div>
        `);
      }
    }

    // Feedback message
    items.push(`
      <div class="status-item ready" title="Ready">
        <span>Ready</span>
      </div>
    `);

    return items.join('');
  }

  public setCurrentFile(filePath: string | null) {
    this.currentFile = filePath;
    this.render();
  }
  
  public setGitInfo(info: { branch: string; ahead: number; behind: number }) {
    if (!this.gitStatus) {
      this.gitStatus = {
        isRepo: true,
        files: [],
        branch: info.branch,
        ahead: info.ahead,
        behind: info.behind
      };
    } else {
      this.gitStatus.branch = info.branch;
      this.gitStatus.ahead = info.ahead;
      this.gitStatus.behind = info.behind;
    }
    this.render();
  }
  
  public setWorkspaceInfo(info: { name: string; path: string }) {
    // Store workspace info and re-render
    this.render();
  }

  public destroy() {
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
    this.container.innerHTML = '';
  }
}