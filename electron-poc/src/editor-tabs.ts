/**
 * High-Performance Editor Tabs with Monaco Integration
 * Uses web workers for syntax highlighting to avoid blocking main thread
 */

import * as monaco from 'monaco-editor';

// Configure Monaco to use web workers for better performance
(self as any).MonacoEnvironment = {
  getWorkerUrl: function (_moduleId: string, label: string) {
    if (label === 'json') {
      return './json.worker.bundle.js';
    }
    if (label === 'css' || label === 'scss' || label === 'less') {
      return './css.worker.bundle.js';
    }
    if (label === 'html' || label === 'handlebars' || label === 'razor') {
      return './html.worker.bundle.js';
    }
    if (label === 'typescript' || label === 'javascript') {
      return './ts.worker.bundle.js';
    }
    return './editor.worker.bundle.js';
  }
};

export interface EditorTab {
  id: string;
  path: string;
  name: string;
  content?: string;
  isDirty: boolean;
  language?: string;
}

export class EditorTabs {
  private container: HTMLElement;
  private tabs: EditorTab[] = [];
  private activeTabId: string | null = null;
  private editors = new Map<string, monaco.editor.IStandaloneCodeEditor>();
  private models = new Map<string, monaco.editor.ITextModel>();
  private tabsContainer: HTMLElement;
  private editorsContainer: HTMLElement;
  private diffEditor: monaco.editor.IStandaloneDiffEditor | null = null;
  private saveCallbacks: ((path: string, content: string) => void)[] = [];

  constructor(container: HTMLElement) {
    this.container = container;
    this.init();
  }

  private init() {
    // Create tabs bar
    this.tabsContainer = document.createElement('div');
    this.tabsContainer.className = 'editor-tabs-bar';
    
    // Create editors container
    this.editorsContainer = document.createElement('div');
    this.editorsContainer.className = 'editors-container';
    
    this.container.appendChild(this.tabsContainer);
    this.container.appendChild(this.editorsContainer);
    
    // Set up keyboard shortcuts
    this.setupKeyboardShortcuts();
  }

  /**
   * Open a file in a new tab or focus existing tab
   */
  async openFile(filePath: string): Promise<void> {
    try {
      console.log('[EditorTabs] Opening file:', filePath);
      
      // Check if already open
      const existingTab = this.tabs.find(t => t.path === filePath);
      if (existingTab) {
        console.log('[EditorTabs] File already open, activating tab');
        this.activateTab(existingTab.id);
        return;
      }

      // Load file content
      console.log('[EditorTabs] Loading file content...');
      console.log('[EditorTabs] window.fileAPI:', window.fileAPI);
      
      if (!window.fileAPI) {
        throw new Error('window.fileAPI is not defined');
      }
      
      if (!window.fileAPI.readFile) {
        throw new Error('window.fileAPI.readFile is not a function');
      }
      
      const content = await window.fileAPI.readFile(filePath);
      const name = filePath.split('/').pop() || 'untitled';
      const language = this.detectLanguage(name);

      console.log('[EditorTabs] Creating new tab for:', name);
      // Create new tab
      const tab: EditorTab = {
        id: `tab-${Date.now()}`,
        path: filePath,
        name,
        content,
        isDirty: false,
        language
      };

      this.tabs.push(tab);
      
      console.log('[EditorTabs] Creating editor...');
      this.createEditor(tab);
      
      console.log('[EditorTabs] Rendering tabs...');
      this.renderTabs();
      
      console.log('[EditorTabs] Activating tab...');
      this.activateTab(tab.id);

      // Watch file for external changes
      console.log('[EditorTabs] Setting up file watch...');
      try {
        await window.fileAPI.watchFile(filePath);
        // Only set up the handler once globally, not per file
        if (!this.fileWatchHandlerSet) {
          window.fileAPI.onFileChanged(this.handleExternalFileChange.bind(this));
          this.fileWatchHandlerSet = true;
        }
      } catch (err) {
        console.error('[EditorTabs] Error setting up file watch:', err);
      }
      
      console.log('[EditorTabs] File opened successfully');
    } catch (error) {
      console.error('[EditorTabs] Error opening file:', error);
      console.error('[EditorTabs] Error stack:', error instanceof Error ? error.stack : 'No stack');
      // Don't re-throw, just log the error
    }
  }
  
  private fileWatchHandlerSet = false;

  /**
   * Open Git diff view for a file
   */
  async openDiff(filePath: string): Promise<void> {
    const currentContent = await window.fileAPI.readFile(filePath);
    const gitDiff = await window.gitAPI.getDiff(filePath);
    
    // Create diff editor if not exists
    if (!this.diffEditor) {
      const diffContainer = document.createElement('div');
      diffContainer.className = 'diff-editor';
      diffContainer.style.height = '100%';
      this.editorsContainer.appendChild(diffContainer);

      this.diffEditor = monaco.editor.createDiffEditor(diffContainer, {
        automaticLayout: true,
        readOnly: false,
        renderSideBySide: true,
        scrollBeyondLastLine: false,
        minimap: { enabled: false }
      });
    }

    // Set original and modified models
    const originalModel = monaco.editor.createModel(
      this.reconstructOriginalFromDiff(currentContent, gitDiff),
      this.detectLanguage(filePath)
    );
    
    const modifiedModel = monaco.editor.createModel(
      currentContent,
      this.detectLanguage(filePath)
    );

    this.diffEditor.setModel({
      original: originalModel,
      modified: modifiedModel
    });
  }

  private reconstructOriginalFromDiff(current: string, diff: string): string {
    // Simple reconstruction - in production, use a proper diff parser
    return current; // Placeholder
  }

  /**
   * Create Monaco editor for a tab
   */
  private createEditor(tab: EditorTab): void {
    console.log('[EditorTabs] Creating editor container for tab:', tab.id);
    
    const editorContainer = document.createElement('div');
    editorContainer.className = 'editor-container';
    editorContainer.id = `editor-${tab.id}`;
    editorContainer.style.display = 'none';
    editorContainer.style.height = '100%';
    this.editorsContainer.appendChild(editorContainer);

    console.log('[EditorTabs] Creating Monaco model...');
    // Create or reuse model for better performance
    let model = this.models.get(tab.path);
    if (!model) {
      model = monaco.editor.createModel(
        tab.content || '',
        tab.language
      );
      this.models.set(tab.path, model);
    }

    console.log('[EditorTabs] Creating Monaco editor instance...');
    
    // Create editor with performance optimizations
    let editor;
    try {
      editor = monaco.editor.create(editorContainer, {
        model,
        theme: 'vs-dark',
        automaticLayout: true,
        scrollBeyondLastLine: false,
        renderWhitespace: 'selection',
        minimap: {
          enabled: true,
          maxColumn: 120
        },
        fontSize: 13,
        fontFamily: 'Menlo, Monaco, "Courier New", monospace',
        lineNumbers: 'on',
        glyphMargin: true,
        folding: true,
        lineDecorationsWidth: 0,
        lineNumbersMinChars: 3,
        renderLineHighlight: 'line',
        scrollbar: {
          useShadows: false,
          vertical: 'visible',
          horizontal: 'visible'
        },
        suggestOnTriggerCharacters: true,
        quickSuggestions: true,
        wordWrap: 'off'
      });

      // Track changes for dirty state
      editor.onDidChangeModelContent(() => {
        if (!tab.isDirty) {
          tab.isDirty = true;
          this.renderTabs();
        }
      });

      // Save on Ctrl+S
      editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
        this.saveTab(tab.id);
      });

      this.editors.set(tab.id, editor);
      console.log('[EditorTabs] Editor created successfully for tab:', tab.id);
    } catch (error) {
      console.error('[EditorTabs] Error creating editor:', error);
      console.error('[EditorTabs] Error details:', error instanceof Error ? error.message : 'Unknown error');
      // Create a fallback text area if Monaco fails
      const fallbackContainer = document.getElementById(`editor-${tab.id}`);
      if (fallbackContainer) {
        fallbackContainer.innerHTML = `
          <textarea style="width: 100%; height: 100%; background: #1e1e1e; color: #fff; border: none; padding: 10px; font-family: monospace;">${tab.content || ''}</textarea>
        `;
      }
    }
  }

  /**
   * Activate a tab
   */
  private activateTab(tabId: string): void {
    // Hide all editors
    this.editorsContainer.querySelectorAll('.editor-container').forEach(el => {
      (el as HTMLElement).style.display = 'none';
    });

    // Show active editor
    const editorEl = document.getElementById(`editor-${tabId}`);
    if (editorEl) {
      editorEl.style.display = 'block';
    }

    // Update active tab
    this.activeTabId = tabId;
    this.renderTabs();

    // Focus editor
    const editor = this.editors.get(tabId);
    if (editor) {
      editor.focus();
    }
  }

  /**
   * Close a tab
   */
  private async closeTab(tabId: string): Promise<void> {
    const tab = this.tabs.find(t => t.id === tabId);
    if (!tab) return;

    // Check if dirty
    if (tab.isDirty) {
      const save = confirm(`Save changes to ${tab.name}?`);
      if (save) {
        await this.saveTab(tabId);
      }
    }

    // Clean up
    const editor = this.editors.get(tabId);
    if (editor) {
      editor.dispose();
      this.editors.delete(tabId);
    }

    // Remove tab
    this.tabs = this.tabs.filter(t => t.id !== tabId);
    
    // Unwatch file
    window.fileAPI.unwatchFile(tab.path);

    // Remove editor element
    const editorEl = document.getElementById(`editor-${tabId}`);
    if (editorEl) {
      editorEl.remove();
    }

    // Activate another tab if needed
    if (this.activeTabId === tabId && this.tabs.length > 0) {
      this.activateTab(this.tabs[0].id);
    } else {
      this.renderTabs();
    }
  }

  /**
   * Save a tab
   */
  private async saveTab(tabId: string): Promise<void> {
    const tab = this.tabs.find(t => t.id === tabId);
    if (!tab || !tab.isDirty) return;

    const editor = this.editors.get(tabId);
    if (!editor) return;

    const content = editor.getValue();
    
    try {
      await window.fileAPI.writeFile(tab.path, content);
      tab.isDirty = false;
      tab.content = content;
      this.renderTabs();
      
      // Notify callbacks
      this.saveCallbacks.forEach(cb => cb(tab.path, content));
    } catch (error) {
      console.error('Failed to save file:', error);
      alert(`Failed to save ${tab.name}`);
    }
  }

  /**
   * Render tabs UI
   */
  private renderTabs(): void {
    this.tabsContainer.innerHTML = '';
    
    this.tabs.forEach(tab => {
      const tabEl = document.createElement('div');
      tabEl.className = `editor-tab ${this.activeTabId === tab.id ? 'active' : ''}`;
      tabEl.dataset.tabId = tab.id;
      
      // Tab content
      const nameEl = document.createElement('span');
      nameEl.className = 'tab-name';
      nameEl.textContent = tab.name;
      if (tab.isDirty) {
        nameEl.textContent = '● ' + nameEl.textContent;
      }
      
      // Close button
      const closeBtn = document.createElement('button');
      closeBtn.className = 'tab-close';
      closeBtn.innerHTML = '×';
      closeBtn.onclick = (e) => {
        e.stopPropagation();
        this.closeTab(tab.id);
      };
      
      tabEl.appendChild(nameEl);
      tabEl.appendChild(closeBtn);
      
      // Click to activate
      tabEl.onclick = () => this.activateTab(tab.id);
      
      this.tabsContainer.appendChild(tabEl);
    });
  }

  /**
   * Handle external file changes
   */
  private handleExternalFileChange(filePath: string): void {
    const tab = this.tabs.find(t => t.path === filePath);
    if (tab && !tab.isDirty) {
      // Reload file if not dirty
      window.fileAPI.readFile(filePath).then(content => {
        const model = this.models.get(filePath);
        if (model) {
          model.setValue(content);
        }
      });
    }
  }

  /**
   * Detect language from file extension
   */
  private detectLanguage(filename: string): string {
    const ext = filename.split('.').pop()?.toLowerCase();
    const languageMap: Record<string, string> = {
      'ts': 'typescript',
      'tsx': 'typescript',
      'js': 'javascript',
      'jsx': 'javascript',
      'json': 'json',
      'html': 'html',
      'css': 'css',
      'scss': 'scss',
      'less': 'less',
      'rs': 'rust',
      'py': 'python',
      'go': 'go',
      'java': 'java',
      'cpp': 'cpp',
      'c': 'c',
      'cs': 'csharp',
      'php': 'php',
      'rb': 'ruby',
      'swift': 'swift',
      'kt': 'kotlin',
      'md': 'markdown',
      'yaml': 'yaml',
      'yml': 'yaml',
      'toml': 'toml',
      'xml': 'xml',
      'sh': 'shell',
      'bash': 'shell',
      'zsh': 'shell',
      'fish': 'shell'
    };
    
    return languageMap[ext || ''] || 'plaintext';
  }

  /**
   * Set up keyboard shortcuts
   */
  private setupKeyboardShortcuts(): void {
    document.addEventListener('keydown', (e) => {
      // Ctrl+W to close tab
      if ((e.ctrlKey || e.metaKey) && e.key === 'w') {
        e.preventDefault();
        if (this.activeTabId) {
          this.closeTab(this.activeTabId);
        }
      }
      
      // Ctrl+Tab to switch tabs
      if ((e.ctrlKey || e.metaKey) && e.key === 'Tab') {
        e.preventDefault();
        const currentIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
        const nextIndex = (currentIndex + 1) % this.tabs.length;
        if (this.tabs[nextIndex]) {
          this.activateTab(this.tabs[nextIndex].id);
        }
      }
    });
  }

  /**
   * Register save callback
   */
  public onSave(callback: (path: string, content: string) => void): void {
    this.saveCallbacks.push(callback);
  }

  /**
   * Get active editor
   */
  public getActiveEditor(): monaco.editor.IStandaloneCodeEditor | null {
    if (!this.activeTabId) return null;
    return this.editors.get(this.activeTabId) || null;
  }

  /**
   * Clean up resources
   */
  public destroy(): void {
    this.editors.forEach(editor => editor.dispose());
    this.models.forEach(model => model.dispose());
    if (this.diffEditor) {
      this.diffEditor.dispose();
    }
    this.editors.clear();
    this.models.clear();
    this.tabs = [];
  }
}