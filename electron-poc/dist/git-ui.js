"use strict";
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
exports.GitUI = void 0;
class GitUI {
    constructor(container) {
        this.refreshInterval = null;
        this.currentStatus = null;
        this.container = container;
        this.init();
    }
    init() {
        return __awaiter(this, void 0, void 0, function* () {
            // Set up auto-refresh
            yield this.refresh();
            this.refreshInterval = setInterval(() => this.refresh(), 2000);
        });
    }
    refresh() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                this.currentStatus = yield window.gitAPI.getStatus();
                this.render();
            }
            catch (error) {
                console.error('Failed to refresh git status:', error);
                // Show error state in UI
                this.container.innerHTML = `
        <div class="git-empty">
          <p style="color: #f48771;">Git status unavailable</p>
          <p style="color: #6b6b6b; font-size: 11px;">Check if this is a Git repository</p>
        </div>
      `;
            }
        });
    }
    render() {
        if (!this.currentStatus) {
            this.container.innerHTML = `
        <div class="git-empty">
          <p>Not a git repository</p>
        </div>
      `;
            return;
        }
        const { files, branch, ahead, behind } = this.currentStatus;
        // Group files by status
        const staged = [];
        const changes = [];
        const untracked = [];
        files.forEach(file => {
            if (file.index !== ' ' && file.index !== '?') {
                staged.push(file);
            }
            else if (file.working === '?') {
                untracked.push(file);
            }
            else if (file.working !== ' ') {
                changes.push(file);
            }
        });
        this.container.innerHTML = `
      <div class="git-container">
        <!-- Header with branch info -->
        <div class="git-header">
          <div class="git-branch">
            <svg class="branch-icon" width="16" height="16" viewBox="0 0 16 16">
              <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 113 2.122V6A2.5 2.5 0 0110 8.5H6a1 1 0 00-1 1v1.128a2.251 2.251 0 11-1.5 0V5.372a2.25 2.25 0 111.5 0v1.836A2.492 2.492 0 016 7h4a1 1 0 001-1v-.628A2.25 2.25 0 019.5 3.25zm-6 0a.75.75 0 101.5 0 .75.75 0 00-1.5 0zm8.25-.75a.75.75 0 100 1.5.75.75 0 000-1.5zM4.25 12a.75.75 0 100 1.5.75.75 0 000-1.5z"/>
            </svg>
            <span class="branch-name">${branch}</span>
            ${ahead > 0 ? `<span class="sync-badge ahead">↑${ahead}</span>` : ''}
            ${behind > 0 ? `<span class="sync-badge behind">↓${behind}</span>` : ''}
          </div>
          <div class="git-actions">
            <button class="git-action-btn" title="Refresh" onclick="window.gitUI?.refresh?.()">
              <svg width="16" height="16" viewBox="0 0 16 16">
                <path fill="currentColor" d="M5.563 2.516A6.001 6.001 0 008 14A6 6 0 003.405 4.41a.5.5 0 00-.845.535A5 5 0 107.5 2.959a.5.5 0 10.002-1 6 6 0 013.061.584l-.011-.026.002-.003.026.011V.5a.5.5 0 00-1 0V2l.001.03a.294.294 0 00.075.204.296.296 0 00.213.089L11.5 2.5a.5.5 0 000-1H9.908A7.001 7.001 0 002 8a7 7 0 1012.135-4.734.5.5 0 00-.634.774z"/>
              </svg>
            </button>
            <button class="git-action-btn" title="Commit" onclick="window.gitUI?.showCommitDialog?.()">
              <svg width="16" height="16" viewBox="0 0 16 16">
                <path fill="currentColor" d="M11.93 8.5a4.002 4.002 0 01-7.86 0H.5a.5.5 0 010-1h3.57a4.002 4.002 0 017.86 0h3.57a.5.5 0 110 1h-3.57zM8 11a3 3 0 100-6 3 3 0 000 6z"/>
              </svg>
            </button>
            <button class="git-action-btn" title="Pull" onclick="window.gitUI?.pull?.()">
              <svg width="16" height="16" viewBox="0 0 16 16">
                <path fill="currentColor" d="M8 1a.5.5 0 01.5.5v9.793l2.146-2.147a.5.5 0 01.708.708l-3 3a.5.5 0 01-.708 0l-3-3a.5.5 0 11.708-.708L7.5 11.293V1.5A.5.5 0 018 1zm-4 13.5a.5.5 0 010-1h8a.5.5 0 010 1H4z"/>
              </svg>
            </button>
            <button class="git-action-btn" title="Push" onclick="window.gitUI?.push?.()">
              <svg width="16" height="16" viewBox="0 0 16 16">
                <path fill="currentColor" d="M8 14.5a.5.5 0 01-.5-.5V4.207L5.354 6.354a.5.5 0 11-.708-.708l3-3a.5.5 0 01.708 0l3 3a.5.5 0 01-.708.708L8.5 4.207V14a.5.5 0 01-.5.5z"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- Message input for quick commit -->
        <div class="commit-message-box" id="commit-message-box" style="display: none;">
          <input type="text" class="commit-input" id="commit-input" placeholder="Commit message..." />
          <div class="commit-actions">
            <button class="commit-btn commit-confirm" onclick="window.gitUI?.doCommit?.()">Commit</button>
            <button class="commit-btn commit-cancel" onclick="window.gitUI?.hideCommitDialog?.()">Cancel</button>
          </div>
        </div>

        <!-- File lists -->
        <div class="git-files">
          ${staged.length > 0 ? `
            <div class="file-section">
              <div class="section-header">
                <span class="section-title">STAGED CHANGES</span>
                <span class="file-count">${staged.length}</span>
              </div>
              <div class="file-list">
                ${staged.map(file => this.renderFile(file, 'staged')).join('')}
              </div>
            </div>
          ` : ''}

          ${changes.length > 0 ? `
            <div class="file-section">
              <div class="section-header">
                <span class="section-title">CHANGES</span>
                <span class="file-count">${changes.length}</span>
              </div>
              <div class="file-list">
                ${changes.map(file => this.renderFile(file, 'changes')).join('')}
              </div>
            </div>
          ` : ''}

          ${untracked.length > 0 ? `
            <div class="file-section">
              <div class="section-header">
                <span class="section-title">UNTRACKED</span>
                <span class="file-count">${untracked.length}</span>
              </div>
              <div class="file-list">
                ${untracked.map(file => this.renderFile(file, 'untracked')).join('')}
              </div>
            </div>
          ` : ''}

          ${files.length === 0 ? `
            <div class="git-empty">
              <p>No changes</p>
            </div>
          ` : ''}
        </div>
      </div>
    `;
    }
    renderFile(file, section) {
        const statusIcon = this.getStatusIcon(file);
        const fileName = file.path.split('/').pop();
        const filePath = file.path.includes('/') ? file.path.substring(0, file.path.lastIndexOf('/')) : '';
        return `
      <div class="git-file" data-file="${file.path}" data-section="${section}">
        <div class="file-info">
          <span class="file-status ${statusIcon.class}">${statusIcon.icon}</span>
          <span class="file-name">${fileName}</span>
          ${filePath ? `<span class="file-path">${filePath}</span>` : ''}
        </div>
        <div class="file-actions">
          ${section === 'staged' ? `
            <button class="file-action" title="Unstage" onclick="window.gitUI?.unstageFile?.('${file.path}')">
              <svg width="16" height="16" viewBox="0 0 16 16">
                <path fill="currentColor" d="M8 4a.5.5 0 01.5.5v7a.5.5 0 01-1 0v-7A.5.5 0 018 4z"/>
                <path fill="currentColor" d="M3.5 8a.5.5 0 01.5-.5h7a.5.5 0 010 1H4a.5.5 0 01-.5-.5z"/>
              </svg>
            </button>
          ` : section === 'changes' ? `
            <button class="file-action" title="Stage" onclick="window.gitUI?.stageFile?.('${file.path}')">
              <svg width="16" height="16" viewBox="0 0 16 16">
                <path fill="currentColor" d="M8 4a.5.5 0 01.5.5v3h3a.5.5 0 010 1h-3v3a.5.5 0 01-1 0v-3h-3a.5.5 0 010-1h3v-3A.5.5 0 018 4z"/>
              </svg>
            </button>
            <button class="file-action" title="Discard" onclick="window.gitUI?.discardFile?.('${file.path}')">
              <svg width="16" height="16" viewBox="0 0 16 16">
                <path fill="currentColor" d="M2.146 2.854a.5.5 0 11.708-.708L8 7.293l5.146-5.147a.5.5 0 01.708.708L8.707 8l5.147 5.146a.5.5 0 01-.708.708L8 8.707l-5.146 5.147a.5.5 0 01-.708-.708L7.293 8 2.146 2.854z"/>
              </svg>
            </button>
          ` : section === 'untracked' ? `
            <button class="file-action" title="Stage" onclick="window.gitUI?.stageFile?.('${file.path}')">
              <svg width="16" height="16" viewBox="0 0 16 16">
                <path fill="currentColor" d="M8 4a.5.5 0 01.5.5v3h3a.5.5 0 010 1h-3v3a.5.5 0 01-1 0v-3h-3a.5.5 0 010-1h3v-3A.5.5 0 018 4z"/>
              </svg>
            </button>
          ` : ''}
        </div>
      </div>
    `;
    }
    getStatusIcon(file) {
        // Check staged status first
        if (file.index === 'A')
            return { icon: 'A', class: 'status-added' };
        if (file.index === 'M')
            return { icon: 'M', class: 'status-modified' };
        if (file.index === 'D')
            return { icon: 'D', class: 'status-deleted' };
        if (file.index === 'R')
            return { icon: 'R', class: 'status-renamed' };
        // Check working tree status
        if (file.working === '?')
            return { icon: 'U', class: 'status-untracked' };
        if (file.working === 'M')
            return { icon: 'M', class: 'status-modified' };
        if (file.working === 'D')
            return { icon: 'D', class: 'status-deleted' };
        return { icon: '?', class: 'status-unknown' };
    }
    stageFile(path) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                yield window.gitAPI.stage([path]);
                yield this.refresh();
            }
            catch (error) {
                console.error('Failed to stage file:', error);
            }
        });
    }
    unstageFile(path) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                yield window.gitAPI.unstage([path]);
                yield this.refresh();
            }
            catch (error) {
                console.error('Failed to unstage file:', error);
            }
        });
    }
    discardFile(path) {
        return __awaiter(this, void 0, void 0, function* () {
            if (confirm(`Are you sure you want to discard changes to ${path}?`)) {
                try {
                    yield window.gitAPI.discard([path]);
                    yield this.refresh();
                }
                catch (error) {
                    console.error('Failed to discard file:', error);
                }
            }
        });
    }
    showCommitDialog() {
        const box = document.getElementById('commit-message-box');
        const input = document.getElementById('commit-input');
        if (box && input) {
            box.style.display = 'block';
            input.focus();
        }
    }
    hideCommitDialog() {
        const box = document.getElementById('commit-message-box');
        const input = document.getElementById('commit-input');
        if (box && input) {
            box.style.display = 'none';
            input.value = '';
        }
    }
    doCommit() {
        return __awaiter(this, void 0, void 0, function* () {
            const input = document.getElementById('commit-input');
            if (input && input.value.trim()) {
                try {
                    yield window.gitAPI.commit(input.value.trim());
                    this.hideCommitDialog();
                    yield this.refresh();
                }
                catch (error) {
                    console.error('Failed to commit:', error);
                    alert(`Commit failed: ${error}`);
                }
            }
        });
    }
    pull() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                yield window.gitAPI.pull();
                yield this.refresh();
            }
            catch (error) {
                console.error('Failed to pull:', error);
                alert(`Pull failed: ${error}`);
            }
        });
    }
    push() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                yield window.gitAPI.push();
                yield this.refresh();
            }
            catch (error) {
                console.error('Failed to push:', error);
                alert(`Push failed: ${error}`);
            }
        });
    }
    destroy() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
        }
    }
}
exports.GitUI = GitUI;
//# sourceMappingURL=git-ui.js.map