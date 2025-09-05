# VS Code Git/Source Control Implementation Plan

## Overview

This document provides a comprehensive implementation plan to recreate VS Code's complete Git/Source Control functionality in the Electron app. Based on analysis of VS Code's source code, this plan covers all major components needed to match VS Code's Git UI exactly.

## Current State Analysis

### Existing Implementation
The current project has basic Git functionality:
- `git-manager.ts`: Basic Git operations wrapper around simple-git
- `git-ui.ts`: Simple file list with staging/unstaging
- `git.css`: Basic styling
- Limited to simple file operations

### VS Code Components to Implement

Based on analysis of VS Code's source code (`src/vs/workbench/contrib/scm/` and `extensions/git/src/`):

1. **SCM View Pane** (`scmViewPane.ts`)
2. **Repository Renderer** (`scmRepositoryRenderer.ts`) 
3. **Decoration Provider** (`decorationProvider.ts`)
4. **Quick Diff** (`quickDiffDecorator.ts`, `quickDiffWidget.ts`)
5. **Git Repository Management** (`repository.ts`)
6. **Activity Bar Integration** (`activity.ts`)

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    SCM View Container                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐│
│  │  Commit Input   │  │   Tool Bar      │  │  Repo Actions   ││
│  │     Widget      │  │                 │  │                 ││
│  └─────────────────┘  └─────────────────┘  └─────────────────┘│
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                Repository Tree                          │ │
│  │  ├─ MERGE CHANGES (2)                                  │ │
│  │  ├─ STAGED CHANGES (5)                                 │ │
│  │  │  ├─ [M] src/file1.ts                               │ │
│  │  │  └─ [A] src/new-file.ts                            │ │
│  │  └─ CHANGES (3)                                        │ │
│  │     ├─ [M] package.json                               │ │
│  │     └─ [D] old-file.ts                                │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Component Implementation Plan

### 1. Enhanced Git Manager

**File**: `src/git/enhanced-git-manager.ts`

```typescript
export interface GitResourceGroup {
  id: string;
  label: string;
  hideWhenEmpty?: boolean;
  resources: GitResource[];
}

export interface GitResource {
  resourceUri: string;
  decorations?: GitResourceDecorations;
  contextValue?: string;
  command?: Command;
}

export interface GitResourceDecorations {
  strikeThrough?: boolean;
  faded?: boolean;
  tooltip?: string;
  badge?: string;
  color?: string;
}

export class EnhancedGitManager extends GitManager {
  // Enhanced methods for VS Code-style operations
  getResourceGroups(): Promise<GitResourceGroup[]>;
  getResourceDecorations(uri: string): GitResourceDecorations;
  stageResources(resources: GitResource[]): Promise<void>;
  unstageResources(resources: GitResource[]): Promise<void>;
}
```

### 2. SCM View Pane

**File**: `src/scm/scm-view-pane.ts`

```typescript
export class SCMViewPane {
  private container: HTMLElement;
  private tree: SCMTree;
  private inputWidget: CommitInputWidget;
  private toolbar: SCMToolbar;
  
  constructor(container: HTMLElement) {
    this.container = container;
    this.initializeComponents();
  }
  
  private initializeComponents() {
    // Initialize all sub-components
    this.createLayout();
    this.setupEventHandlers();
  }
  
  private createLayout() {
    this.container.innerHTML = `
      <div class="scm-view-pane">
        <div class="scm-repository-container">
          <div class="scm-repository-header">
            <div class="scm-repository-provider">
              <div class="scm-provider-title">
                <span class="provider-name">Source Control</span>
                <div class="provider-actions"></div>
              </div>
            </div>
          </div>
          <div class="scm-editor">
            <!-- Commit input widget -->
          </div>
          <div class="scm-tree-container">
            <!-- Resource groups tree -->
          </div>
        </div>
      </div>
    `;
  }
}
```

### 3. Resource Tree Implementation

**File**: `src/scm/scm-tree.ts`

```typescript
export class SCMTree {
  private container: HTMLElement;
  private resourceGroups: Map<string, GitResourceGroup> = new Map();
  private treeMode: 'list' | 'tree' = 'list';
  
  renderResourceGroups(groups: GitResourceGroup[]) {
    this.container.innerHTML = '';
    
    groups.forEach(group => {
      if (group.hideWhenEmpty && group.resources.length === 0) return;
      
      const groupElement = this.createResourceGroup(group);
      this.container.appendChild(groupElement);
    });
  }
  
  private createResourceGroup(group: GitResourceGroup): HTMLElement {
    const element = document.createElement('div');
    element.className = 'scm-resource-group';
    element.innerHTML = `
      <div class="scm-resource-group-header">
        <div class="scm-resource-group-label">
          <span class="resource-group-title">${group.label.toUpperCase()}</span>
          <span class="resource-group-count">${group.resources.length}</span>
        </div>
        <div class="scm-resource-group-actions">
          ${this.createGroupActions(group)}
        </div>
      </div>
      <div class="scm-resource-list">
        ${group.resources.map(resource => this.createResourceElement(resource)).join('')}
      </div>
    `;
    return element;
  }
  
  private createResourceElement(resource: GitResource): string {
    const decorations = resource.decorations || {};
    const fileName = path.basename(resource.resourceUri);
    const relativePath = path.dirname(resource.resourceUri);
    
    return `
      <div class="scm-resource" data-uri="${resource.resourceUri}">
        <div class="scm-resource-checkbox">
          <input type="checkbox" class="resource-checkbox" />
        </div>
        <div class="scm-resource-icon">
          ${this.getResourceIcon(resource)}
        </div>
        <div class="scm-resource-name">
          <span class="resource-name-label">${fileName}</span>
          ${relativePath !== '.' ? `<span class="resource-path">${relativePath}</span>` : ''}
        </div>
        <div class="scm-resource-decoration">
          ${decorations.badge || ''}
        </div>
        <div class="scm-resource-actions">
          ${this.createResourceActions(resource)}
        </div>
      </div>
    `;
  }
}
```

### 4. Commit Input Widget

**File**: `src/scm/commit-input-widget.ts`

```typescript
export class CommitInputWidget {
  private container: HTMLElement;
  private textArea: HTMLTextAreaElement;
  private characterCount: HTMLElement;
  private commitButton: HTMLButtonElement;
  
  constructor(container: HTMLElement) {
    this.container = container;
    this.create();
  }
  
  private create() {
    this.container.innerHTML = `
      <div class="scm-editor">
        <div class="scm-input-widget">
          <textarea 
            class="scm-input-message" 
            placeholder="Message (press Ctrl+Enter to commit)"
            rows="1">
          </textarea>
          <div class="scm-input-actions">
            <div class="scm-input-validation">
              <span class="character-count">0</span>
            </div>
            <div class="scm-input-buttons">
              <button class="scm-commit-button" title="Commit">
                <svg class="codicon codicon-check" viewBox="0 0 16 16">
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M14.431 3.323l-8.47 10-.79-.036-3.35-4.77.818-.574 2.978 4.24 8.051-9.506.762.646z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    `;
    
    this.setupElements();
    this.setupEventHandlers();
  }
  
  private setupElements() {
    this.textArea = this.container.querySelector('.scm-input-message') as HTMLTextAreaElement;
    this.characterCount = this.container.querySelector('.character-count') as HTMLElement;
    this.commitButton = this.container.querySelector('.scm-commit-button') as HTMLButtonElement;
  }
  
  private setupEventHandlers() {
    this.textArea.addEventListener('input', () => {
      this.updateCharacterCount();
      this.autoResize();
    });
    
    this.textArea.addEventListener('keydown', (e) => {
      if (e.ctrlKey && e.key === 'Enter') {
        this.commit();
      }
    });
    
    this.commitButton.addEventListener('click', () => {
      this.commit();
    });
  }
}
```

### 5. File Decoration System

**File**: `src/git/decoration-provider.ts`

```typescript
export interface FileDecoration {
  badge?: string;
  color?: string;
  strikeThrough?: boolean;
  faded?: boolean;
  tooltip?: string;
}

export class GitDecorationProvider {
  private decorations: Map<string, FileDecoration> = new Map();
  private gitManager: EnhancedGitManager;
  
  constructor(gitManager: EnhancedGitManager) {
    this.gitManager = gitManager;
    this.setupWatchers();
  }
  
  getDecoration(uri: string): FileDecoration | undefined {
    return this.decorations.get(uri);
  }
  
  private async updateDecorations() {
    const status = await this.gitManager.getStatus();
    this.decorations.clear();
    
    status.files.forEach(file => {
      const decoration = this.createDecoration(file);
      this.decorations.set(file.path, decoration);
    });
    
    this.onDidChangeDecorations.fire();
  }
  
  private createDecoration(file: GitFileStatus): FileDecoration {
    const decoration: FileDecoration = {};
    
    // Set badge based on Git status
    if (file.index === 'A') decoration.badge = 'A';
    else if (file.index === 'M') decoration.badge = 'M';
    else if (file.index === 'D') decoration.badge = 'D';
    else if (file.working === 'M') decoration.badge = 'M';
    else if (file.working === '?') decoration.badge = 'U';
    
    // Set colors
    if (file.index !== ' ' && file.index !== '?') {
      decoration.color = 'charts.green'; // Staged
    } else if (file.working === '?') {
      decoration.color = 'charts.blue'; // Untracked
    } else {
      decoration.color = 'charts.yellow'; // Modified
    }
    
    return decoration;
  }
}
```

### 6. Quick Diff Implementation

**File**: `src/diff/quick-diff-decorator.ts`

```typescript
export class QuickDiffDecorator {
  private editor: any; // Monaco editor instance
  private decorations: string[] = [];
  
  constructor(editor: any) {
    this.editor = editor;
    this.setupProvider();
  }
  
  async updateDecorations(uri: string) {
    const diff = await this.gitManager.getDiff(uri);
    const decorations = this.parseDiffToDecorations(diff);
    
    this.decorations = this.editor.deltaDecorations(
      this.decorations,
      decorations
    );
  }
  
  private parseDiffToDecorations(diff: string): any[] {
    const decorations: any[] = [];
    const lines = diff.split('\n');
    let currentLine = 1;
    
    lines.forEach(line => {
      if (line.startsWith('@@')) {
        // Parse hunk header
        const match = line.match(/@@ -(\d+),?(\d*) \+(\d+),?(\d*) @@/);
        if (match) {
          currentLine = parseInt(match[3]);
        }
      } else if (line.startsWith('+') && !line.startsWith('+++')) {
        // Added line
        decorations.push({
          range: new monaco.Range(currentLine, 1, currentLine, 1),
          options: {
            isWholeLine: true,
            className: 'git-diff-added-line',
            glyphMarginClassName: 'git-glyph-margin-added'
          }
        });
        currentLine++;
      } else if (line.startsWith('-') && !line.startsWith('---')) {
        // Deleted line (show as decoration on next line)
        decorations.push({
          range: new monaco.Range(currentLine, 1, currentLine, 1),
          options: {
            isWholeLine: true,
            className: 'git-diff-deleted-line',
            glyphMarginClassName: 'git-glyph-margin-deleted'
          }
        });
      } else if (!line.startsWith('\\')) {
        // Unchanged line
        currentLine++;
      }
    });
    
    return decorations;
  }
}
```

### 7. Status Bar Integration

**File**: `src/status-bar/git-status-bar.ts`

```typescript
export class GitStatusBarIntegration {
  private statusBar: any; // Reference to existing status bar
  private branchItem: HTMLElement;
  private syncItem: HTMLElement;
  
  constructor(statusBar: any) {
    this.statusBar = statusBar;
    this.createItems();
  }
  
  private createItems() {
    // Branch indicator
    this.branchItem = this.createStatusItem({
      id: 'git.branch',
      text: '$(git-branch) main',
      tooltip: 'Git branch',
      command: 'git.checkout'
    });
    
    // Sync indicator
    this.syncItem = this.createStatusItem({
      id: 'git.sync',
      text: '$(sync)',
      tooltip: 'Synchronize changes',
      command: 'git.sync'
    });
  }
  
  updateBranch(branch: string, ahead?: number, behind?: number) {
    let text = `$(git-branch) ${branch}`;
    let tooltip = `Branch: ${branch}`;
    
    if (ahead || behind) {
      if (ahead) text += ` ↑${ahead}`;
      if (behind) text += ` ↓${behind}`;
      tooltip += `\nAhead: ${ahead || 0}, Behind: ${behind || 0}`;
    }
    
    this.branchItem.textContent = text;
    this.branchItem.setAttribute('title', tooltip);
  }
  
  updateSyncStatus(isRunning: boolean, hasChanges: boolean) {
    if (isRunning) {
      this.syncItem.innerHTML = '$(sync~spin)';
      this.syncItem.setAttribute('title', 'Synchronizing...');
    } else if (hasChanges) {
      this.syncItem.innerHTML = '$(cloud-upload)';
      this.syncItem.setAttribute('title', 'Push changes');
    } else {
      this.syncItem.innerHTML = '$(sync)';
      this.syncItem.setAttribute('title', 'Synchronize changes');
    }
  }
}
```

## CSS Styling (VS Code Theme)

**File**: `src/scm/scm.css`

```css
/* SCM View Pane */
.scm-view-pane {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--vscode-sideBar-background);
  color: var(--vscode-sideBar-foreground);
}

.scm-repository-container {
  flex: 1;
  display: flex;
  flex-direction: column;
}

/* Repository Header */
.scm-repository-header {
  padding: 12px 16px 8px;
  border-bottom: 1px solid var(--vscode-sideBar-border);
}

.scm-provider-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.provider-name {
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  color: var(--vscode-sideBarSectionHeader-foreground);
}

/* Commit Input Widget */
.scm-editor {
  padding: 8px 16px;
  border-bottom: 1px solid var(--vscode-sideBar-border);
}

.scm-input-widget {
  position: relative;
  border: 1px solid var(--vscode-input-border);
  border-radius: 3px;
  background: var(--vscode-input-background);
}

.scm-input-message {
  width: 100%;
  padding: 8px;
  border: none;
  background: transparent;
  color: var(--vscode-input-foreground);
  font-family: var(--vscode-font-family);
  font-size: var(--vscode-font-size);
  resize: none;
  outline: none;
  min-height: 36px;
  max-height: 200px;
}

.scm-input-message::placeholder {
  color: var(--vscode-input-placeholderForeground);
}

.scm-input-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  border-top: 1px solid var(--vscode-input-border);
  background: var(--vscode-input-background);
}

.character-count {
  font-size: 11px;
  color: var(--vscode-descriptionForeground);
}

.scm-commit-button {
  padding: 4px 8px;
  border: none;
  border-radius: 3px;
  background: var(--vscode-button-background);
  color: var(--vscode-button-foreground);
  cursor: pointer;
}

.scm-commit-button:hover {
  background: var(--vscode-button-hoverBackground);
}

/* Resource Groups */
.scm-tree-container {
  flex: 1;
  overflow-y: auto;
}

.scm-resource-group {
  margin-bottom: 8px;
}

.scm-resource-group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: var(--vscode-sideBarSectionHeader-background);
  border-top: 1px solid var(--vscode-sideBar-border);
  border-bottom: 1px solid var(--vscode-sideBar-border);
}

.scm-resource-group-label {
  display: flex;
  align-items: center;
  gap: 8px;
}

.resource-group-title {
  font-weight: 600;
  font-size: 11px;
  color: var(--vscode-sideBarSectionHeader-foreground);
}

.resource-group-count {
  background: var(--vscode-badge-background);
  color: var(--vscode-badge-foreground);
  border-radius: 10px;
  padding: 2px 6px;
  font-size: 10px;
  font-weight: 600;
}

/* Resource Items */
.scm-resource {
  display: flex;
  align-items: center;
  padding: 4px 16px;
  cursor: pointer;
  transition: background-color 0.1s;
}

.scm-resource:hover {
  background: var(--vscode-list-hoverBackground);
}

.scm-resource.selected {
  background: var(--vscode-list-activeSelectionBackground);
  color: var(--vscode-list-activeSelectionForeground);
}

.scm-resource-checkbox {
  margin-right: 8px;
}

.resource-checkbox {
  width: 16px;
  height: 16px;
}

.scm-resource-icon {
  margin-right: 8px;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.scm-resource-name {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.resource-name-label {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.resource-path {
  font-size: 11px;
  color: var(--vscode-descriptionForeground);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.scm-resource-decoration {
  margin-left: 8px;
  font-size: 11px;
  font-weight: 600;
  color: var(--vscode-gitDecoration-modifiedResourceForeground);
}

.scm-resource-actions {
  margin-left: 8px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.1s;
}

.scm-resource:hover .scm-resource-actions {
  opacity: 1;
}

/* Git Status Colors */
.status-modified { color: var(--vscode-gitDecoration-modifiedResourceForeground); }
.status-added { color: var(--vscode-gitDecoration-addedResourceForeground); }
.status-deleted { color: var(--vscode-gitDecoration-deletedResourceForeground); }
.status-untracked { color: var(--vscode-gitDecoration-untrackedResourceForeground); }
.status-ignored { color: var(--vscode-gitDecoration-ignoredResourceForeground); }
.status-conflicted { color: var(--vscode-gitDecoration-conflictingResourceForeground); }

/* Quick Diff Decorations */
.git-diff-added-line {
  background: var(--vscode-diffEditor-insertedTextBackground);
}

.git-diff-modified-line {
  background: var(--vscode-diffEditor-modifiedTextBackground);
}

.git-diff-deleted-line {
  background: var(--vscode-diffEditor-removedTextBackground);
}

.git-glyph-margin-added {
  background: var(--vscode-editorGutter-addedBackground);
  width: 3px;
}

.git-glyph-margin-modified {
  background: var(--vscode-editorGutter-modifiedBackground);
  width: 3px;
}

.git-glyph-margin-deleted {
  background: var(--vscode-editorGutter-deletedBackground);
  width: 3px;
}
```

## Event Handling and Data Flow

### Event System

```typescript
// Event emitter for Git changes
export class GitEventEmitter extends EventTarget {
  emitStatusChange(status: GitStatus) {
    this.dispatchEvent(new CustomEvent('status-change', { detail: status }));
  }
  
  emitRepositoryChange(repository: Repository) {
    this.dispatchEvent(new CustomEvent('repository-change', { detail: repository }));
  }
}

// Main coordinator
export class SCMCoordinator {
  private gitManager: EnhancedGitManager;
  private decorationProvider: GitDecorationProvider;
  private viewPane: SCMViewPane;
  private statusBarIntegration: GitStatusBarIntegration;
  private eventEmitter: GitEventEmitter;
  
  constructor() {
    this.setupEventHandlers();
    this.startWatching();
  }
  
  private setupEventHandlers() {
    this.eventEmitter.addEventListener('status-change', (e) => {
      this.updateAllComponents(e.detail);
    });
  }
  
  private async updateAllComponents(status: GitStatus) {
    // Update view pane
    this.viewPane.updateResourceGroups(status.resourceGroups);
    
    // Update decorations
    this.decorationProvider.updateDecorations();
    
    // Update status bar
    this.statusBarIntegration.updateBranch(
      status.branch, 
      status.ahead, 
      status.behind
    );
  }
}
```

## Integration with Existing Code

### 1. Replace Current Git UI

Update `src/index.ts` to use the new SCM implementation:

```typescript
// Replace the existing git UI initialization
const scmContainer = document.getElementById('git-container');
if (scmContainer) {
  const scmCoordinator = new SCMCoordinator();
  scmCoordinator.initialize(scmContainer);
}
```

### 2. Extend File Explorer

Integrate decorations with the existing file explorer:

```typescript
// In vs-file-explorer.ts or similar
export class EnhancedFileExplorer extends VSFileExplorer {
  private decorationProvider: GitDecorationProvider;
  
  protected renderFile(file: FileItem): string {
    const decoration = this.decorationProvider.getDecoration(file.path);
    const decorationClass = decoration ? `decoration-${decoration.badge?.toLowerCase()}` : '';
    
    return `
      <div class="file-item ${decorationClass}" data-path="${file.path}">
        ${super.renderFile(file)}
        ${decoration?.badge ? `<span class="file-decoration">${decoration.badge}</span>` : ''}
      </div>
    `;
  }
}
```

### 3. Monaco Editor Integration

Add quick diff decorations to Monaco:

```typescript
// In renderer-monaco.ts or similar
export class MonacoGitIntegration {
  private quickDiffDecorator: QuickDiffDecorator;
  
  constructor(editor: monaco.editor.IStandaloneCodeEditor) {
    this.quickDiffDecorator = new QuickDiffDecorator(editor);
    this.setupFileChangeListener();
  }
  
  private setupFileChangeListener() {
    // Listen for file changes and update decorations
    this.quickDiffDecorator.onDidChangeModel(() => {
      this.updateDecorations();
    });
  }
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Implement EnhancedGitManager
- [ ] Create basic SCMViewPane structure
- [ ] Set up event system

### Phase 2: Core UI (Week 2)
- [ ] Implement resource tree with grouping
- [ ] Add commit input widget
- [ ] Create toolbar and actions

### Phase 3: Decorations (Week 3)
- [ ] Implement decoration provider
- [ ] Add file explorer decorations
- [ ] Create status bar integration

### Phase 4: Diff Features (Week 4)
- [ ] Implement quick diff decorations
- [ ] Add diff editor (inline/side-by-side)
- [ ] Integrate with Monaco editor

### Phase 5: Polish (Week 5)
- [ ] Implement all VS Code keyboard shortcuts
- [ ] Add context menus
- [ ] Performance optimizations
- [ ] Testing and bug fixes

## Testing Strategy

1. **Unit Tests**: Test individual components
2. **Integration Tests**: Test Git operations with real repositories
3. **Visual Tests**: Screenshot comparison with VS Code
4. **Performance Tests**: Ensure responsive UI with large repositories

## Success Criteria

- [ ] Source Control sidebar matches VS Code exactly
- [ ] All Git operations work identically to VS Code
- [ ] File decorations appear in real-time
- [ ] Diff views show changes accurately
- [ ] Performance is comparable to VS Code
- [ ] Keyboard shortcuts match VS Code
- [ ] Context menus provide same functionality

This implementation plan provides a comprehensive roadmap to recreate VS Code's Git functionality with complete fidelity to the original implementation.