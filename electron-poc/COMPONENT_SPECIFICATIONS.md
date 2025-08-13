# VS Code Git Components - Detailed Specifications

## Component 1: SCM View Pane Container

### HTML Structure (Exact VS Code Layout)

```html
<div class="pane-body" id="scm-view-pane">
  <div class="scm-view">
    <div class="scm-provider scm-provider-Git">
      <!-- Repository Header -->
      <div class="scm-provider-title">
        <div class="scm-provider-label">
          <span class="codicon codicon-source-control"></span>
          <span>Source Control</span>
        </div>
        <div class="scm-provider-actions">
          <div class="actions-container">
            <div class="action-item">
              <a class="action-label codicon codicon-refresh" title="Refresh" role="button"></a>
            </div>
            <div class="action-item">
              <a class="action-label codicon codicon-ellipsis" title="Views and More Actions..." role="button"></a>
            </div>
          </div>
        </div>
      </div>

      <!-- Repository Container -->
      <div class="scm-repository">
        <div class="scm-repository-container">
          
          <!-- Commit Input Area -->
          <div class="scm-editor">
            <div class="scm-input">
              <div class="monaco-inputbox idle">
                <div class="ibwrapper">
                  <textarea class="input empty" 
                           placeholder="Message (press Ctrl+Enter to commit)"
                           rows="1"
                           wrap="off"
                           autocorrect="off"
                           autocapitalize="off"
                           spellcheck="false"></textarea>
                </div>
              </div>
              <div class="scm-input-actions">
                <div class="scm-input-validation">
                  <div class="scm-input-counter">0</div>
                </div>
                <div class="scm-input-buttons">
                  <div class="monaco-button commit-button" title="Commit">
                    <span class="codicon codicon-check"></span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Repository Content -->
          <div class="scm-repository-content">
            <div class="scm-view-container">
              
              <!-- Resource Groups Tree -->
              <div class="scm-tree-container">
                <div class="monaco-list" role="tree">
                  <div class="monaco-scrollable-element">
                    <div class="monaco-list-rows">
                      
                      <!-- Resource Group: Changes -->
                      <div class="monaco-list-row" role="treeitem" aria-level="1">
                        <div class="scm-resource-group">
                          <div class="scm-resource-group-header">
                            <div class="scm-resource-group-expand codicon codicon-chevron-down"></div>
                            <div class="scm-resource-group-label">
                              <span class="scm-resource-group-name">Changes</span>
                              <div class="monaco-count-badge">3</div>
                            </div>
                            <div class="scm-resource-group-actions">
                              <div class="action-item">
                                <a class="action-label codicon codicon-add" title="Stage All Changes" role="button"></a>
                              </div>
                              <div class="action-item">
                                <a class="action-label codicon codicon-discard" title="Discard All Changes" role="button"></a>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>

                      <!-- Resource Items under Changes -->
                      <div class="monaco-list-row scm-resource-row" role="treeitem" aria-level="2">
                        <div class="scm-resource">
                          <div class="scm-resource-checkbox">
                            <input type="checkbox" class="monaco-checkbox">
                          </div>
                          <div class="scm-resource-icon">
                            <div class="monaco-icon-label file-icon package-json-file-icon"></div>
                          </div>
                          <div class="scm-resource-label">
                            <div class="monaco-icon-label">
                              <div class="monaco-icon-label-container">
                                <span class="monaco-icon-name-container">
                                  <span class="label-name">package.json</span>
                                  <span class="label-description">src</span>
                                </span>
                              </div>
                            </div>
                          </div>
                          <div class="scm-resource-decoration">
                            <div class="scm-resource-decoration-icon">M</div>
                          </div>
                          <div class="scm-resource-actions">
                            <div class="action-item">
                              <a class="action-label codicon codicon-add" title="Stage Changes" role="button"></a>
                            </div>
                            <div class="action-item">
                              <a class="action-label codicon codicon-discard" title="Discard Changes" role="button"></a>
                            </div>
                          </div>
                        </div>
                      </div>

                      <!-- Resource Group: Staged Changes -->
                      <div class="monaco-list-row" role="treeitem" aria-level="1">
                        <div class="scm-resource-group">
                          <div class="scm-resource-group-header">
                            <div class="scm-resource-group-expand codicon codicon-chevron-down"></div>
                            <div class="scm-resource-group-label">
                              <span class="scm-resource-group-name">Staged Changes</span>
                              <div class="monaco-count-badge">2</div>
                            </div>
                            <div class="scm-resource-group-actions">
                              <div class="action-item">
                                <a class="action-label codicon codicon-remove" title="Unstage All Changes" role="button"></a>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>

                      <!-- Resource Items under Staged Changes -->
                      <div class="monaco-list-row scm-resource-row" role="treeitem" aria-level="2">
                        <div class="scm-resource">
                          <div class="scm-resource-checkbox">
                            <input type="checkbox" class="monaco-checkbox" checked>
                          </div>
                          <div class="scm-resource-icon">
                            <div class="monaco-icon-label file-icon typescript-file-icon"></div>
                          </div>
                          <div class="scm-resource-label">
                            <div class="monaco-icon-label">
                              <div class="monaco-icon-label-container">
                                <span class="monaco-icon-name-container">
                                  <span class="label-name">git-manager.ts</span>
                                  <span class="label-description">src</span>
                                </span>
                              </div>
                            </div>
                          </div>
                          <div class="scm-resource-decoration">
                            <div class="scm-resource-decoration-icon">M</div>
                          </div>
                          <div class="scm-resource-actions">
                            <div class="action-item">
                              <a class="action-label codicon codicon-remove" title="Unstage Changes" role="button"></a>
                            </div>
                          </div>
                        </div>
                      </div>

                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
```

### CSS Classes (VS Code Theme Variables)

```css
/* Main SCM View */
.scm-view {
  height: 100%;
  background: var(--vscode-sideBar-background);
  color: var(--vscode-sideBar-foreground);
}

.scm-provider {
  display: flex;
  flex-direction: column;
  height: 100%;
}

/* Provider Title */
.scm-provider-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--vscode-sideBarSectionHeader-background);
  border-bottom: 1px solid var(--vscode-sideBarSectionHeader-border);
  font-size: 11px;
  font-weight: bold;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  color: var(--vscode-sideBarSectionHeader-foreground);
}

.scm-provider-label {
  display: flex;
  align-items: center;
  gap: 6px;
}

.scm-provider-actions .actions-container {
  display: flex;
  gap: 2px;
}

.action-item .action-label {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 3px;
  cursor: pointer;
  color: var(--vscode-icon-foreground);
  text-decoration: none;
}

.action-item .action-label:hover {
  background: var(--vscode-toolbar-hoverBackground);
}

/* Repository Container */
.scm-repository {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.scm-repository-container {
  flex: 1;
  display: flex;
  flex-direction: column;
}

/* Input Editor */
.scm-editor {
  padding: 8px 12px;
  border-bottom: 1px solid var(--vscode-sideBar-border);
}

.scm-input {
  border: 1px solid var(--vscode-input-border);
  border-radius: 2px;
  background: var(--vscode-input-background);
}

.monaco-inputbox {
  position: relative;
  display: flex;
  flex-direction: column;
}

.ibwrapper {
  position: relative;
}

.input {
  display: block;
  box-sizing: border-box;
  width: 100%;
  padding: 6px 8px;
  border: none;
  font-family: inherit;
  font-size: inherit;
  line-height: 18px;
  color: var(--vscode-input-foreground);
  background: transparent;
  resize: none;
  outline: none;
  overflow: hidden;
  min-height: 30px;
  max-height: 134px;
}

.input::placeholder {
  color: var(--vscode-input-placeholderForeground);
}

.input:focus {
  outline: 1px solid var(--vscode-focusBorder);
  outline-offset: -1px;
}

.scm-input-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  border-top: 1px solid var(--vscode-input-border);
  background: var(--vscode-input-background);
}

.scm-input-counter {
  font-size: 11px;
  color: var(--vscode-descriptionForeground);
}

.commit-button {
  padding: 4px 8px;
  border: 1px solid transparent;
  border-radius: 2px;
  background: var(--vscode-button-background);
  color: var(--vscode-button-foreground);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.commit-button:hover {
  background: var(--vscode-button-hoverBackground);
}

.commit-button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

/* Repository Content */
.scm-repository-content {
  flex: 1;
  overflow: hidden;
}

.scm-view-container {
  height: 100%;
  overflow: hidden;
}

.scm-tree-container {
  height: 100%;
  overflow: hidden;
}

/* Monaco List */
.monaco-list {
  position: relative;
  height: 100%;
  overflow: hidden;
}

.monaco-scrollable-element {
  position: relative;
  height: 100%;
  overflow: auto;
}

.monaco-list-rows {
  position: relative;
  overflow: hidden;
}

.monaco-list-row {
  position: relative;
  box-sizing: border-box;
  overflow: hidden;
  width: 100%;
}

.monaco-list-row:hover {
  background: var(--vscode-list-hoverBackground);
}

.monaco-list-row.selected {
  background: var(--vscode-list-activeSelectionBackground);
  color: var(--vscode-list-activeSelectionForeground);
}

.monaco-list-row.focused {
  outline: 1px solid var(--vscode-focusBorder);
  outline-offset: -1px;
}

/* Resource Groups */
.scm-resource-group {
  position: relative;
}

.scm-resource-group-header {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  background: var(--vscode-sideBarSectionHeader-background);
  border-top: 1px solid var(--vscode-sideBarSectionHeader-border);
  border-bottom: 1px solid var(--vscode-sideBarSectionHeader-border);
  font-size: 11px;
  font-weight: bold;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  color: var(--vscode-sideBarSectionHeader-foreground);
}

.scm-resource-group-expand {
  width: 16px;
  height: 16px;
  margin-right: 2px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.scm-resource-group-expand.collapsed::before {
  transform: rotate(-90deg);
}

.scm-resource-group-label {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
}

.scm-resource-group-name {
  flex: 1;
}

.monaco-count-badge {
  background: var(--vscode-badge-background);
  color: var(--vscode-badge-foreground);
  border-radius: 11px;
  padding: 2px 6px;
  font-size: 11px;
  font-weight: normal;
  line-height: 16px;
  min-width: 18px;
  text-align: center;
}

.scm-resource-group-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.2s;
}

.scm-resource-group:hover .scm-resource-group-actions {
  opacity: 1;
}

/* Resource Items */
.scm-resource-row {
  border-left: 2px solid transparent;
}

.scm-resource {
  display: flex;
  align-items: center;
  padding: 2px 8px 2px 24px; /* Left padding accounts for expand icon */
  cursor: pointer;
  min-height: 22px;
}

.scm-resource-checkbox {
  margin-right: 6px;
  display: flex;
  align-items: center;
}

.monaco-checkbox {
  width: 14px;
  height: 14px;
  cursor: pointer;
}

.scm-resource-icon {
  margin-right: 6px;
  display: flex;
  align-items: center;
}

.monaco-icon-label {
  display: flex;
  align-items: center;
  overflow: hidden;
}

.file-icon {
  width: 16px;
  height: 16px;
  background-size: 16px;
  background-repeat: no-repeat;
  background-position: center;
}

.scm-resource-label {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
}

.monaco-icon-label-container {
  flex: 1;
  overflow: hidden;
}

.monaco-icon-name-container {
  display: flex;
  overflow: hidden;
}

.label-name {
  font-size: 13px;
  color: var(--vscode-list-foreground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.label-description {
  font-size: 12px;
  color: var(--vscode-descriptionForeground);
  margin-left: 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.scm-resource-decoration {
  margin-left: 4px;
  display: flex;
  align-items: center;
}

.scm-resource-decoration-icon {
  font-size: 11px;
  font-weight: bold;
  color: var(--vscode-gitDecoration-modifiedResourceForeground);
  min-width: 12px;
  text-align: center;
}

/* Git Status Colors */
.scm-resource-decoration-icon.status-added { 
  color: var(--vscode-gitDecoration-addedResourceForeground); 
}
.scm-resource-decoration-icon.status-modified { 
  color: var(--vscode-gitDecoration-modifiedResourceForeground); 
}
.scm-resource-decoration-icon.status-deleted { 
  color: var(--vscode-gitDecoration-deletedResourceForeground); 
}
.scm-resource-decoration-icon.status-untracked { 
  color: var(--vscode-gitDecoration-untrackedResourceForeground); 
}
.scm-resource-decoration-icon.status-ignored { 
  color: var(--vscode-gitDecoration-ignoredResourceForeground); 
}
.scm-resource-decoration-icon.status-conflicted { 
  color: var(--vscode-gitDecoration-conflictingResourceForeground); 
}

.scm-resource-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.2s;
}

.scm-resource:hover .scm-resource-actions {
  opacity: 1;
}

/* Context Menu */
.context-menu {
  position: fixed;
  background: var(--vscode-menu-background);
  border: 1px solid var(--vscode-menu-border);
  border-radius: 3px;
  box-shadow: 0 2px 8px var(--vscode-widget-shadow);
  padding: 4px 0;
  z-index: 1000;
  font-size: 13px;
}

.context-menu-item {
  padding: 4px 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--vscode-menu-foreground);
}

.context-menu-item:hover {
  background: var(--vscode-menu-selectionBackground);
  color: var(--vscode-menu-selectionForeground);
}

.context-menu-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.context-menu-separator {
  height: 1px;
  background: var(--vscode-menu-separatorBackground);
  margin: 4px 0;
}

/* Keyboard focus */
.monaco-list-row.keyboard-focused {
  outline: 1px solid var(--vscode-focusBorder);
  outline-offset: -1px;
}

/* Loading states */
.scm-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  color: var(--vscode-descriptionForeground);
}

.scm-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 20px;
  color: var(--vscode-descriptionForeground);
  text-align: center;
}

.scm-empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.6;
}

.scm-empty-text {
  font-size: 14px;
  line-height: 1.4;
}
```

## Component 2: File Decoration System

### HTML Structure for Explorer Decorations

```html
<!-- File Explorer with Git Decorations -->
<div class="explorer-item">
  <div class="explorer-item-label">
    <div class="monaco-icon-label file-icon">
      <div class="monaco-icon-label-container">
        <span class="monaco-icon-name-container">
          <span class="label-name">package.json</span>
        </span>
        <!-- Git decoration badge -->
        <div class="decoration-badge decoration-modified">M</div>
      </div>
    </div>
  </div>
</div>

<!-- Folder with propagated decorations -->
<div class="explorer-item folder-item">
  <div class="explorer-item-label">
    <div class="monaco-icon-label folder-icon">
      <div class="monaco-icon-label-container">
        <span class="monaco-icon-name-container">
          <span class="label-name">src</span>
        </span>
        <!-- Folder decoration showing children have changes -->
        <div class="decoration-badge decoration-folder">3</div>
      </div>
    </div>
  </div>
</div>
```

### Decoration CSS

```css
/* File decorations */
.decoration-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-left: 6px;
  font-size: 11px;
  font-weight: bold;
  min-width: 16px;
  height: 16px;
  border-radius: 8px;
  text-align: center;
}

.decoration-modified {
  color: var(--vscode-gitDecoration-modifiedResourceForeground);
  background: var(--vscode-gitDecoration-modifiedResourceForeground);
  color: var(--vscode-badge-foreground);
}

.decoration-added {
  color: var(--vscode-gitDecoration-addedResourceForeground);
  background: var(--vscode-gitDecoration-addedResourceForeground);
  color: var(--vscode-badge-foreground);
}

.decoration-deleted {
  color: var(--vscode-gitDecoration-deletedResourceForeground);
  background: var(--vscode-gitDecoration-deletedResourceForeground);
  color: var(--vscode-badge-foreground);
}

.decoration-untracked {
  color: var(--vscode-gitDecoration-untrackedResourceForeground);
  background: var(--vscode-gitDecoration-untrackedResourceForeground);
  color: var(--vscode-badge-foreground);
}

.decoration-folder {
  background: var(--vscode-badge-background);
  color: var(--vscode-badge-foreground);
  font-size: 10px;
}

/* File name text decorations */
.label-name.git-modified {
  color: var(--vscode-gitDecoration-modifiedResourceForeground);
}

.label-name.git-added {
  color: var(--vscode-gitDecoration-addedResourceForeground);
}

.label-name.git-deleted {
  color: var(--vscode-gitDecoration-deletedResourceForeground);
  text-decoration: line-through;
}

.label-name.git-untracked {
  color: var(--vscode-gitDecoration-untrackedResourceForeground);
}

.label-name.git-ignored {
  color: var(--vscode-gitDecoration-ignoredResourceForeground);
  opacity: 0.6;
}
```

## Component 3: Quick Diff Gutter Decorations

### Monaco Editor Integration

```typescript
// Monaco editor decoration types
interface GitGutterDecoration {
  range: monaco.Range;
  options: {
    isWholeLine?: boolean;
    className?: string;
    glyphMarginClassName?: string;
    glyphMarginHoverMessage?: monaco.IMarkdownString;
    hoverMessage?: monaco.IMarkdownString;
  };
}

// Implementation
export class QuickDiffDecorator {
  private editor: monaco.editor.IStandaloneCodeEditor;
  private decorations: string[] = [];
  private currentDecorations: GitGutterDecoration[] = [];

  updateDecorations(uri: string, diff: string) {
    const newDecorations = this.parseDiffToDecorations(diff);
    
    this.decorations = this.editor.deltaDecorations(
      this.decorations,
      newDecorations
    );
  }

  private parseDiffToDecorations(diff: string): GitGutterDecoration[] {
    const decorations: GitGutterDecoration[] = [];
    const lines = diff.split('\n');
    let currentLine = 1;
    let hunks: DiffHunk[] = [];

    // Parse diff hunks
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      
      if (line.startsWith('@@')) {
        const match = line.match(/@@ -(\d+),?(\d*) \+(\d+),?(\d*) @@/);
        if (match) {
          currentLine = parseInt(match[3]);
          hunks.push({
            oldStart: parseInt(match[1]),
            oldCount: parseInt(match[2]) || 1,
            newStart: parseInt(match[3]),
            newCount: parseInt(match[4]) || 1,
            lines: []
          });
        }
      } else if (hunks.length > 0) {
        const currentHunk = hunks[hunks.length - 1];
        
        if (line.startsWith('+') && !line.startsWith('+++')) {
          // Added line
          currentHunk.lines.push({ type: 'added', content: line.substring(1), lineNumber: currentLine });
          decorations.push({
            range: new monaco.Range(currentLine, 1, currentLine, 1),
            options: {
              isWholeLine: true,
              className: 'git-diff-added-line',
              glyphMarginClassName: 'git-glyph-margin-added',
              glyphMarginHoverMessage: { value: 'Added line' }
            }
          });
          currentLine++;
        } else if (line.startsWith('-') && !line.startsWith('---')) {
          // Deleted line - show on the line where deletion occurred
          currentHunk.lines.push({ type: 'deleted', content: line.substring(1), lineNumber: currentLine });
          decorations.push({
            range: new monaco.Range(currentLine, 1, currentLine, 1),
            options: {
              isWholeLine: true,
              className: 'git-diff-deleted-line',
              glyphMarginClassName: 'git-glyph-margin-deleted',
              glyphMarginHoverMessage: { value: 'Deleted lines' }
            }
          });
        } else if (!line.startsWith('\\')) {
          // Context line
          currentLine++;
        }
      }
    }

    return decorations;
  }
}
```

### Gutter Decoration CSS

```css
/* Quick diff gutter decorations */
.git-glyph-margin-added {
  background: var(--vscode-editorGutter-addedBackground, #587c0c);
  width: 3px;
  left: 3px;
}

.git-glyph-margin-modified {
  background: var(--vscode-editorGutter-modifiedBackground, #0c7d9d);
  width: 3px;
  left: 3px;
}

.git-glyph-margin-deleted {
  background: var(--vscode-editorGutter-deletedBackground, #94151b);
  width: 3px;
  left: 3px;
  position: relative;
}

.git-glyph-margin-deleted::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 0;
  height: 0;
  border-left: 3px solid transparent;
  border-right: 3px solid transparent;
  border-top: 3px solid var(--vscode-editorGutter-deletedBackground, #94151b);
}

/* Line background highlighting */
.git-diff-added-line {
  background: var(--vscode-diffEditor-insertedTextBackground, rgba(155, 185, 85, 0.2));
}

.git-diff-modified-line {
  background: var(--vscode-diffEditor-modifiedTextBackground, rgba(155, 185, 85, 0.2));
}

.git-diff-deleted-line {
  background: var(--vscode-diffEditor-removedTextBackground, rgba(255, 0, 0, 0.2));
}

/* Hover widget for git changes */
.git-diff-hover-widget {
  background: var(--vscode-editorHoverWidget-background);
  border: 1px solid var(--vscode-editorHoverWidget-border);
  border-radius: 3px;
  box-shadow: 0 2px 8px var(--vscode-widget-shadow);
  padding: 8px 12px;
  font-family: var(--vscode-editor-font-family);
  font-size: var(--vscode-editor-font-size);
  color: var(--vscode-editorHoverWidget-foreground);
  max-width: 500px;
}

.git-diff-hover-content {
  white-space: pre;
  font-family: var(--vscode-editor-font-family);
}

.git-diff-hover-actions {
  margin-top: 8px;
  display: flex;
  gap: 8px;
}

.git-diff-hover-action {
  padding: 2px 8px;
  border: 1px solid var(--vscode-button-border);
  border-radius: 2px;
  background: var(--vscode-button-secondaryBackground);
  color: var(--vscode-button-secondaryForeground);
  cursor: pointer;
  font-size: 11px;
}

.git-diff-hover-action:hover {
  background: var(--vscode-button-secondaryHoverBackground);
}
```

## Component 4: Status Bar Git Integration

### HTML Structure

```html
<!-- Status Bar Git Items -->
<div class="statusbar-item" id="git-branch-status">
  <div class="statusbar-item-label">
    <span class="codicon codicon-source-control"></span>
    <span class="branch-name">main</span>
    <span class="sync-indicator">
      <span class="ahead-count">↑2</span>
      <span class="behind-count">↓1</span>
    </span>
  </div>
</div>

<div class="statusbar-item" id="git-sync-status">
  <div class="statusbar-item-label">
    <span class="codicon codicon-sync"></span>
  </div>
</div>
```

### Status Bar CSS

```css
/* Git status bar items */
.statusbar-item {
  display: flex;
  align-items: center;
  padding: 0 8px;
  height: 22px;
  cursor: pointer;
  font-size: 12px;
  background: transparent;
  color: var(--vscode-statusBar-foreground);
  border-right: 1px solid var(--vscode-statusBar-border);
}

.statusbar-item:hover {
  background: var(--vscode-statusBarItem-hoverBackground);
}

.statusbar-item-label {
  display: flex;
  align-items: center;
  gap: 4px;
}

.branch-name {
  font-weight: 500;
}

.sync-indicator {
  display: flex;
  gap: 2px;
  font-size: 11px;
}

.ahead-count {
  color: var(--vscode-gitDecoration-addedResourceForeground);
}

.behind-count {
  color: var(--vscode-gitDecoration-modifiedResourceForeground);
}

/* Sync animation */
.codicon-sync.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Status bar notifications */
.git-status-notification {
  background: var(--vscode-statusBarItem-warningBackground);
  color: var(--vscode-statusBarItem-warningForeground);
}

.git-status-error {
  background: var(--vscode-statusBarItem-errorBackground);
  color: var(--vscode-statusBarItem-errorForeground);
}
```

## Component 5: Event Handling and Data Flow

### JavaScript Implementation

```typescript
// Main SCM Controller
export class SCMController {
  private view: SCMViewPane;
  private gitManager: EnhancedGitManager;
  private decorationProvider: GitDecorationProvider;
  private quickDiff: QuickDiffDecorator;
  private statusBar: GitStatusBarIntegration;
  
  constructor() {
    this.setupEventHandlers();
    this.startPolling();
  }

  private setupEventHandlers() {
    // File system watchers
    this.gitManager.onRepositoryChange((repo) => {
      this.updateAllViews();
    });

    // User interactions
    document.addEventListener('click', this.handleClick.bind(this));
    document.addEventListener('contextmenu', this.handleContextMenu.bind(this));
    document.addEventListener('keydown', this.handleKeyDown.bind(this));

    // Commit input
    const commitInput = document.querySelector('.scm-input textarea');
    commitInput?.addEventListener('input', this.handleCommitInput.bind(this));
    commitInput?.addEventListener('keydown', this.handleCommitKeyDown.bind(this));
  }

  private async updateAllViews() {
    const status = await this.gitManager.getStatus();
    
    // Update SCM view
    this.view.updateResourceGroups(status.resourceGroups);
    
    // Update decorations
    this.decorationProvider.updateDecorations();
    
    // Update status bar
    this.statusBar.updateBranch(status.branch, status.ahead, status.behind);
    
    // Update quick diff if editor is open
    const activeEditor = this.getActiveEditor();
    if (activeEditor) {
      this.quickDiff.updateDecorations(activeEditor.getModel()?.uri.toString() || '');
    }
  }

  private handleClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    
    // Resource group expand/collapse
    if (target.classList.contains('scm-resource-group-expand')) {
      this.toggleResourceGroup(target);
      return;
    }
    
    // Resource selection
    if (target.closest('.scm-resource')) {
      this.selectResource(target.closest('.scm-resource')!);
      return;
    }
    
    // Action buttons
    if (target.classList.contains('action-label')) {
      this.handleAction(target);
      return;
    }
  }

  private handleContextMenu(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const resource = target.closest('.scm-resource');
    
    if (resource) {
      event.preventDefault();
      this.showContextMenu(event.clientX, event.clientY, resource);
    }
  }

  private showContextMenu(x: number, y: number, resource: HTMLElement) {
    const menu = document.createElement('div');
    menu.className = 'context-menu';
    menu.style.left = `${x}px`;
    menu.style.top = `${y}px`;
    
    const uri = resource.getAttribute('data-uri');
    const isStaged = resource.closest('.scm-resource-group[data-id="staged"]');
    
    menu.innerHTML = `
      <div class="context-menu-item" data-action="open">
        <span class="codicon codicon-go-to-file"></span>
        Open File
      </div>
      <div class="context-menu-item" data-action="openside">
        <span class="codicon codicon-split-horizontal"></span>
        Open to the Side
      </div>
      <div class="context-menu-separator"></div>
      ${isStaged ? `
        <div class="context-menu-item" data-action="unstage">
          <span class="codicon codicon-remove"></span>
          Unstage Changes
        </div>
      ` : `
        <div class="context-menu-item" data-action="stage">
          <span class="codicon codicon-add"></span>
          Stage Changes
        </div>
        <div class="context-menu-item" data-action="discard">
          <span class="codicon codicon-discard"></span>
          Discard Changes
        </div>
      `}
      <div class="context-menu-separator"></div>
      <div class="context-menu-item" data-action="history">
        <span class="codicon codicon-history"></span>
        View File History
      </div>
    `;
    
    document.body.appendChild(menu);
    
    // Handle menu clicks
    menu.addEventListener('click', async (e) => {
      const action = (e.target as HTMLElement).closest('[data-action]')?.getAttribute('data-action');
      if (action && uri) {
        await this.executeContextAction(action, uri);
      }
      document.body.removeChild(menu);
    });
    
    // Remove menu on outside click
    const removeMenu = () => {
      if (document.body.contains(menu)) {
        document.body.removeChild(menu);
      }
      document.removeEventListener('click', removeMenu);
    };
    setTimeout(() => document.addEventListener('click', removeMenu), 0);
  }

  private async executeContextAction(action: string, uri: string) {
    switch (action) {
      case 'open':
        await this.openFile(uri);
        break;
      case 'openside':
        await this.openFile(uri, true);
        break;
      case 'stage':
        await this.gitManager.stage([uri]);
        await this.updateAllViews();
        break;
      case 'unstage':
        await this.gitManager.unstage([uri]);
        await this.updateAllViews();
        break;
      case 'discard':
        if (confirm(`Are you sure you want to discard changes to ${uri}?`)) {
          await this.gitManager.discard([uri]);
          await this.updateAllViews();
        }
        break;
      case 'history':
        await this.showFileHistory(uri);
        break;
    }
  }
}
```

This comprehensive specification provides exact HTML structures, CSS classes, and JavaScript implementations that match VS Code's Git/Source Control implementation. Each component is designed to integrate seamlessly with the existing Electron app while providing the full functionality of VS Code's Git features.