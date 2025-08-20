"use strict";
/**
 * VS Code-style Diff Viewer
 * Shows file changes with syntax highlighting and line-by-line diffs
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
exports.DiffViewer = void 0;
class DiffViewer {
    constructor() {
        this.viewMode = 'inline';
        this.originalContent = '';
        this.modifiedContent = '';
        this.filePath = '';
        this.currentDiff = '';
        this.parsedLines = [];
        this.container = document.createElement('div');
        this.container.className = 'diff-viewer';
        this.attachStyles();
    }
    attachStyles() {
        if (document.getElementById('diff-viewer-styles'))
            return;
        const style = document.createElement('style');
        style.id = 'diff-viewer-styles';
        style.textContent = `
            .diff-viewer {
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                font-family: 'Consolas', 'Courier New', monospace;
                font-size: 13px;
                background: var(--vscode-editor-background, #1e1e1e);
                color: var(--vscode-editor-foreground, #d4d4d4);
            }
            
            .diff-header {
                display: flex;
                align-items: center;
                padding: 8px 16px;
                background: var(--vscode-editorGroupHeader-tabsBackground, #252526);
                border-bottom: 1px solid var(--vscode-editorGroupHeader-tabsBorder, #252526);
            }
            
            .diff-title {
                flex: 1;
                font-size: 13px;
                color: var(--vscode-foreground, #cccccc);
            }
            
            .diff-actions {
                display: flex;
                gap: 8px;
            }
            
            .diff-action-btn {
                padding: 4px 8px;
                background: transparent;
                border: 1px solid var(--vscode-button-border, #464647);
                color: var(--vscode-button-foreground, #cccccc);
                cursor: pointer;
                border-radius: 2px;
                font-size: 12px;
            }
            
            .diff-action-btn:hover {
                background: var(--vscode-button-hoverBackground, #2a2d2e);
            }
            
            .diff-action-btn.active {
                background: var(--vscode-button-background, #0e639c);
                color: var(--vscode-button-foreground, #ffffff);
            }
            
            .diff-content {
                flex: 1;
                overflow: auto;
                position: relative;
            }
            
            /* Inline diff view */
            .diff-inline {
                width: 100%;
            }
            
            .diff-line {
                display: flex;
                min-height: 18px;
                line-height: 18px;
                white-space: pre;
                position: relative;
            }
            
            .diff-line-number {
                width: 50px;
                padding: 0 8px;
                text-align: right;
                color: var(--vscode-editorLineNumber-foreground, #858585);
                background: var(--vscode-editorGutter-background, #1e1e1e);
                user-select: none;
                flex-shrink: 0;
            }
            
            .diff-line-content {
                flex: 1;
                padding: 0 16px;
                position: relative;
            }
            
            .diff-line-sign {
                position: absolute;
                left: 4px;
                width: 12px;
                text-align: center;
                font-weight: bold;
            }
            
            /* Addition lines */
            .diff-line.add {
                background: var(--vscode-diffEditor-insertedTextBackground, rgba(155, 185, 85, 0.2));
            }
            
            .diff-line.add .diff-line-content {
                background: var(--vscode-diffEditor-insertedTextBackground, rgba(155, 185, 85, 0.2));
            }
            
            .diff-line.add .diff-line-sign {
                color: var(--vscode-gitDecoration-addedResourceForeground, #81b88b);
            }
            
            /* Deletion lines */
            .diff-line.remove {
                background: var(--vscode-diffEditor-removedTextBackground, rgba(255, 0, 0, 0.2));
            }
            
            .diff-line.remove .diff-line-content {
                background: var(--vscode-diffEditor-removedTextBackground, rgba(255, 0, 0, 0.2));
            }
            
            .diff-line.remove .diff-line-sign {
                color: var(--vscode-gitDecoration-deletedResourceForeground, #c74e39);
            }
            
            /* Header lines */
            .diff-line.header {
                background: var(--vscode-diffEditor-diagonalFill, #3a3d41);
                color: var(--vscode-descriptionForeground, #969696);
                font-style: italic;
                padding: 4px 16px;
            }
            
            .diff-line.header .diff-line-content {
                background: transparent;
            }
            
            /* Side-by-side view */
            .diff-side-by-side {
                display: flex;
                width: 100%;
                height: 100%;
                gap: 1px;
                background: var(--vscode-editorWidget-border, #454545);
            }
            
            .diff-side {
                flex: 1;
                overflow: auto;
                background: var(--vscode-editor-background, #1e1e1e);
                display: flex;
                flex-direction: column;
            }
            
            .diff-side-title {
                padding: 8px 16px;
                background: var(--vscode-editorGroupHeader-tabsBackground, #252526);
                border-bottom: 1px solid var(--vscode-editorGroupHeader-tabsBorder, #252526);
                font-size: 12px;
                color: var(--vscode-descriptionForeground, #969696);
                flex-shrink: 0;
            }
            
            .diff-side-content {
                flex: 1;
                overflow: auto;
            }
            
            .diff-side .diff-line {
                display: flex;
                min-height: 18px;
                line-height: 18px;
            }
            
            .diff-side .diff-line-number {
                width: 50px;
                padding: 0 8px;
                text-align: right;
                color: var(--vscode-editorLineNumber-foreground, #858585);
                background: var(--vscode-editorGutter-background, #1e1e1e);
                user-select: none;
                flex-shrink: 0;
            }
            
            .diff-side .diff-line-content {
                flex: 1;
                padding: 0 16px;
                white-space: pre;
            }
            
            /* Gutter decorations */
            .diff-gutter-insert {
                border-left: 3px solid var(--vscode-diffEditor-insertedLineBackground, #9bb955);
            }
            
            .diff-gutter-delete {
                border-left: 3px solid var(--vscode-diffEditor-removedLineBackground, #ff0000);
            }
            
            /* Inline character changes */
            .diff-char-add {
                background: var(--vscode-diffEditor-insertedTextBackground, rgba(155, 185, 85, 0.4));
                border-radius: 2px;
            }
            
            .diff-char-remove {
                background: var(--vscode-diffEditor-removedTextBackground, rgba(255, 0, 0, 0.4));
                text-decoration: line-through;
                border-radius: 2px;
            }
        `;
        document.head.appendChild(style);
    }
    showDiff(filePath, diff) {
        return __awaiter(this, void 0, void 0, function* () {
            this.filePath = filePath;
            this.currentDiff = diff;
            this.parsedLines = this.parseDiff(diff);
            this.render();
        });
    }
    showFileDiff(filePath, originalContent, modifiedContent) {
        return __awaiter(this, void 0, void 0, function* () {
            this.filePath = filePath;
            this.originalContent = originalContent;
            this.modifiedContent = modifiedContent;
            // Generate diff from content
            this.currentDiff = this.generateDiff(originalContent, modifiedContent);
            this.parsedLines = this.parseDiff(this.currentDiff);
            this.render();
        });
    }
    parseDiff(diff) {
        const lines = [];
        const diffLines = diff.split('\n');
        let oldLineNum = 1;
        let newLineNum = 1;
        for (const line of diffLines) {
            if (line.startsWith('+++') || line.startsWith('---')) {
                // File headers
                lines.push({
                    type: 'header',
                    content: line
                });
            }
            else if (line.startsWith('@@')) {
                // Hunk header
                lines.push({
                    type: 'header',
                    content: line
                });
                // Parse line numbers from hunk header
                const match = line.match(/@@ -(\d+),?\d* \+(\d+),?\d* @@/);
                if (match) {
                    oldLineNum = parseInt(match[1]);
                    newLineNum = parseInt(match[2]);
                }
            }
            else if (line.startsWith('+')) {
                // Addition
                lines.push({
                    type: 'add',
                    content: line.substring(1),
                    newLineNumber: newLineNum++
                });
            }
            else if (line.startsWith('-')) {
                // Deletion
                lines.push({
                    type: 'remove',
                    content: line.substring(1),
                    oldLineNumber: oldLineNum++
                });
            }
            else if (line.startsWith(' ') || line === '') {
                // Context line
                lines.push({
                    type: 'normal',
                    content: line.substring(1),
                    oldLineNumber: oldLineNum++,
                    newLineNumber: newLineNum++
                });
            }
        }
        return lines;
    }
    generateDiff(original, modified) {
        // Simple line-by-line diff generation
        const originalLines = original.split('\n');
        const modifiedLines = modified.split('\n');
        let diff = `--- a/${this.filePath}\n+++ b/${this.filePath}\n`;
        diff += '@@ -1,' + originalLines.length + ' +1,' + modifiedLines.length + ' @@\n';
        // Simple diff algorithm (for demonstration - in production use a proper diff library)
        const maxLines = Math.max(originalLines.length, modifiedLines.length);
        for (let i = 0; i < maxLines; i++) {
            const origLine = originalLines[i];
            const modLine = modifiedLines[i];
            if (origLine === modLine) {
                if (origLine !== undefined) {
                    diff += ' ' + origLine + '\n';
                }
            }
            else {
                if (origLine !== undefined && modLine === undefined) {
                    diff += '-' + origLine + '\n';
                }
                else if (origLine === undefined && modLine !== undefined) {
                    diff += '+' + modLine + '\n';
                }
                else if (origLine !== modLine) {
                    diff += '-' + origLine + '\n';
                    diff += '+' + modLine + '\n';
                }
            }
        }
        return diff;
    }
    render() {
        this.container.innerHTML = `
            <div class="diff-header">
                <div class="diff-title">
                    <span style="margin-right: 8px;">📝</span>
                    ${this.filePath.split('/').pop()} (Working Tree)
                </div>
                <div class="diff-actions">
                    <button class="diff-action-btn ${this.viewMode === 'inline' ? 'active' : ''}" 
                            onclick="window.diffViewer?.setViewMode('inline')">
                        Inline
                    </button>
                    <button class="diff-action-btn ${this.viewMode === 'side-by-side' ? 'active' : ''}"
                            onclick="window.diffViewer?.setViewMode('side-by-side')">
                        Side by Side
                    </button>
                    <button class="diff-action-btn" onclick="window.diffViewer?.close()">
                        ✕ Close
                    </button>
                </div>
            </div>
            <div class="diff-content">
                ${this.viewMode === 'inline' ? this.renderInline(this.parsedLines) : this.renderSideBySide(this.parsedLines)}
            </div>
        `;
        // Set global reference
        window.diffViewer = this;
    }
    renderInline(lines) {
        let html = '<div class="diff-inline">';
        for (const line of lines) {
            if (line.type === 'header') {
                html += `<div class="diff-line header">
                    <div class="diff-line-content">${this.escapeHtml(line.content)}</div>
                </div>`;
            }
            else {
                const lineClass = line.type === 'add' ? 'add' : line.type === 'remove' ? 'remove' : '';
                const sign = line.type === 'add' ? '+' : line.type === 'remove' ? '-' : '';
                html += `<div class="diff-line ${lineClass}">`;
                // Old line number
                if (line.oldLineNumber) {
                    html += `<div class="diff-line-number">${line.oldLineNumber}</div>`;
                }
                else {
                    html += `<div class="diff-line-number"></div>`;
                }
                // New line number
                if (line.newLineNumber) {
                    html += `<div class="diff-line-number">${line.newLineNumber}</div>`;
                }
                else {
                    html += `<div class="diff-line-number"></div>`;
                }
                // Content with sign
                html += `<div class="diff-line-content">`;
                if (sign) {
                    html += `<span class="diff-line-sign">${sign}</span>`;
                }
                html += this.escapeHtml(line.content);
                html += `</div>`;
                html += `</div>`;
            }
        }
        html += '</div>';
        return html;
    }
    renderSideBySide(lines) {
        const leftLines = [];
        const rightLines = [];
        for (const line of lines) {
            if (line.type === 'header') {
                // Skip headers in side-by-side view
                continue;
            }
            else if (line.type === 'remove') {
                leftLines.push(`
                    <div class="diff-line remove">
                        <div class="diff-line-number">${line.oldLineNumber || ''}</div>
                        <div class="diff-line-content">${this.escapeHtml(line.content)}</div>
                    </div>
                `);
                rightLines.push(`<div class="diff-line" style="height: 18px; visibility: hidden;">
                    <div class="diff-line-number">&nbsp;</div>
                    <div class="diff-line-content">&nbsp;</div>
                </div>`);
            }
            else if (line.type === 'add') {
                leftLines.push(`<div class="diff-line" style="height: 18px; visibility: hidden;">
                    <div class="diff-line-number">&nbsp;</div>
                    <div class="diff-line-content">&nbsp;</div>
                </div>`);
                rightLines.push(`
                    <div class="diff-line add">
                        <div class="diff-line-number">${line.newLineNumber || ''}</div>
                        <div class="diff-line-content">${this.escapeHtml(line.content)}</div>
                    </div>
                `);
            }
            else {
                leftLines.push(`
                    <div class="diff-line">
                        <div class="diff-line-number">${line.oldLineNumber || ''}</div>
                        <div class="diff-line-content">${this.escapeHtml(line.content)}</div>
                    </div>
                `);
                rightLines.push(`
                    <div class="diff-line">
                        <div class="diff-line-number">${line.newLineNumber || ''}</div>
                        <div class="diff-line-content">${this.escapeHtml(line.content)}</div>
                    </div>
                `);
            }
        }
        return `
            <div class="diff-side-by-side">
                <div class="diff-side">
                    <div class="diff-side-title">Original (HEAD)</div>
                    <div class="diff-side-content">
                        ${leftLines.join('')}
                    </div>
                </div>
                <div class="diff-side">
                    <div class="diff-side-title">Modified (Working Tree)</div>
                    <div class="diff-side-content">
                        ${rightLines.join('')}
                    </div>
                </div>
            </div>
        `;
    }
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    setViewMode(mode) {
        this.viewMode = mode;
        // Re-render with the same data
        this.render();
    }
    close() {
        // Remove from DOM and notify editor tabs
        if (this.container.parentElement) {
            this.container.parentElement.removeChild(this.container);
        }
        // Clean up global reference
        if (window.diffViewer === this) {
            window.diffViewer = null;
        }
    }
    getContainer() {
        return this.container;
    }
}
exports.DiffViewer = DiffViewer;
// Export for global access
window.DiffViewer = DiffViewer;
//# sourceMappingURL=diff-viewer.js.map