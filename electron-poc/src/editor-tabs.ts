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
    // Create tabs wrapper with navigation
    const tabsWrapper = document.createElement('div');
    tabsWrapper.className = 'editor-tabs-wrapper';
    
    // Create left navigation button
    const leftNav = document.createElement('button');
    leftNav.className = 'tab-nav-button tab-nav-left';
    leftNav.innerHTML = '<svg width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M10 3L5 8l5 5V3z"/></svg>';
    leftNav.title = 'Show previous tabs (Alt+Left)';
    leftNav.onclick = () => this.scrollTabs('left');
    
    // Create tabs container
    this.tabsContainer = document.createElement('div');
    this.tabsContainer.className = 'editor-tabs-bar';
    
    // Add scroll listener to update navigation buttons
    this.tabsContainer.addEventListener('scroll', () => {
      this.updateNavigationButtons();
    });
    
    // Create right navigation button
    const rightNav = document.createElement('button');
    rightNav.className = 'tab-nav-button tab-nav-right';
    rightNav.innerHTML = '<svg width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M6 3v10l5-5-5-5z"/></svg>';
    rightNav.title = 'Show next tabs (Alt+Right)';
    rightNav.onclick = () => this.scrollTabs('right');
    
    // Create tab actions (new tab button, etc.)
    const tabActions = document.createElement('div');
    tabActions.className = 'tab-actions';
    tabActions.innerHTML = `
      <button class="tab-action-button" title="Split Editor">
        <svg width="16" height="16" viewBox="0 0 16 16">
          <path fill="currentColor" d="M2 2v12h12V2H2zm7 11H3V3h6v10zm4 0h-3V3h3v10z"/>
        </svg>
      </button>
    `;
    
    // Assemble the tab bar
    tabsWrapper.appendChild(leftNav);
    tabsWrapper.appendChild(this.tabsContainer);
    tabsWrapper.appendChild(rightNav);
    tabsWrapper.appendChild(tabActions);
    
    // Create editors container
    this.editorsContainer = document.createElement('div');
    this.editorsContainer.className = 'editors-container';
    
    this.container.appendChild(tabsWrapper);
    this.container.appendChild(this.editorsContainer);
    
    // Set up keyboard shortcuts
    this.setupKeyboardShortcuts();
    
    // Update navigation button visibility
    this.updateNavigationButtons();
  }
  
  private scrollTabs(direction: 'left' | 'right') {
    const scrollAmount = 200;
    if (direction === 'left') {
      this.tabsContainer.scrollLeft -= scrollAmount;
    } else {
      this.tabsContainer.scrollLeft += scrollAmount;
    }
    this.updateNavigationButtons();
  }
  
  private updateNavigationButtons() {
    const wrapper = this.container.querySelector('.editor-tabs-wrapper');
    if (!wrapper) return;
    
    const leftBtn = wrapper.querySelector('.tab-nav-left') as HTMLButtonElement;
    const rightBtn = wrapper.querySelector('.tab-nav-right') as HTMLButtonElement;
    
    if (leftBtn) {
      leftBtn.style.display = this.tabsContainer.scrollLeft > 0 ? 'flex' : 'none';
    }
    
    if (rightBtn) {
      const hasOverflow = this.tabsContainer.scrollWidth > this.tabsContainer.clientWidth;
      const canScrollRight = this.tabsContainer.scrollLeft < 
        (this.tabsContainer.scrollWidth - this.tabsContainer.clientWidth);
      rightBtn.style.display = hasOverflow && canScrollRight ? 'flex' : 'none';
    }
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
    
    // Update navigation button visibility after rendering tabs
    this.updateNavigationButtons();
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
      
      // Ctrl+Tab to switch tabs forward
      if ((e.ctrlKey || e.metaKey) && e.key === 'Tab' && !e.shiftKey) {
        e.preventDefault();
        const currentIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
        const nextIndex = (currentIndex + 1) % this.tabs.length;
        if (this.tabs[nextIndex]) {
          this.activateTab(this.tabs[nextIndex].id);
        }
      }
      
      // Ctrl+Shift+Tab to switch tabs backward
      if ((e.ctrlKey || e.metaKey) && e.key === 'Tab' && e.shiftKey) {
        e.preventDefault();
        const currentIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
        const prevIndex = currentIndex > 0 ? currentIndex - 1 : this.tabs.length - 1;
        if (this.tabs[prevIndex]) {
          this.activateTab(this.tabs[prevIndex].id);
        }
      }
      
      // Alt+Left Arrow to go to previous tab
      if (e.altKey && e.key === 'ArrowLeft') {
        e.preventDefault();
        const currentIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
        if (currentIndex > 0) {
          this.activateTab(this.tabs[currentIndex - 1].id);
        } else if (this.tabs.length > 0) {
          // Wrap around to last tab
          this.activateTab(this.tabs[this.tabs.length - 1].id);
        }
      }
      
      // Alt+Right Arrow to go to next tab
      if (e.altKey && e.key === 'ArrowRight') {
        e.preventDefault();
        const currentIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
        if (currentIndex < this.tabs.length - 1) {
          this.activateTab(this.tabs[currentIndex + 1].id);
        } else if (this.tabs.length > 0) {
          // Wrap around to first tab
          this.activateTab(this.tabs[0].id);
        }
      }
      
      // Ctrl+PageUp to go to previous tab (VS Code style)
      if ((e.ctrlKey || e.metaKey) && e.key === 'PageUp') {
        e.preventDefault();
        const currentIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
        if (currentIndex > 0) {
          this.activateTab(this.tabs[currentIndex - 1].id);
        }
      }
      
      // Ctrl+PageDown to go to next tab (VS Code style)
      if ((e.ctrlKey || e.metaKey) && e.key === 'PageDown') {
        e.preventDefault();
        const currentIndex = this.tabs.findIndex(t => t.id === this.activeTabId);
        if (currentIndex < this.tabs.length - 1) {
          this.activateTab(this.tabs[currentIndex + 1].id);
        }
      }
      
      // Ctrl+1-9 to jump to specific tab (like browsers and VS Code)
      if ((e.ctrlKey || e.metaKey) && e.key >= '1' && e.key <= '9') {
        e.preventDefault();
        const tabIndex = parseInt(e.key) - 1;
        if (tabIndex < this.tabs.length) {
          this.activateTab(this.tabs[tabIndex].id);
        }
      }
      
      // F1 to show keyboard shortcuts help
      if (e.key === 'F1') {
        e.preventDefault();
        this.showKeyboardShortcuts();
      }
    });
  }
  
  /**
   * Show keyboard shortcuts in a modal or alert
   */
  private showKeyboardShortcuts(): void {
    const shortcuts = `
    Editor Tab Keyboard Shortcuts:
    
    Navigation:
    • Alt + ← / →           Navigate between tabs
    • Ctrl + Tab            Next tab
    • Ctrl + Shift + Tab    Previous tab
    • Ctrl + PageUp/Down    Previous/Next tab
    • Ctrl + 1-9            Jump to tab by number
    
    File Operations:
    • Ctrl + S              Save current file
    • Ctrl + W              Close current tab
    
    Help:
    • F1                    Show this help
    `;
    
    // Create a simple modal to show shortcuts
    const modal = document.createElement('div');
    modal.style.cssText = `
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background: #252526;
      border: 1px solid #007acc;
      border-radius: 4px;
      padding: 20px;
      z-index: 10000;
      color: #cccccc;
      font-family: monospace;
      white-space: pre-wrap;
      max-width: 500px;
      box-shadow: 0 4px 16px rgba(0,0,0,0.5);
    `;
    modal.textContent = shortcuts;
    
    // Add close button
    const closeBtn = document.createElement('button');
    closeBtn.textContent = '✕ Close (ESC)';
    closeBtn.style.cssText = `
      display: block;
      margin-top: 15px;
      padding: 8px 16px;
      background: #007acc;
      color: white;
      border: none;
      border-radius: 3px;
      cursor: pointer;
      font-family: sans-serif;
    `;
    closeBtn.onclick = () => modal.remove();
    modal.appendChild(closeBtn);
    
    // Add backdrop
    const backdrop = document.createElement('div');
    backdrop.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0,0,0,0.5);
      z-index: 9999;
    `;
    backdrop.onclick = () => {
      modal.remove();
      backdrop.remove();
    };
    
    // Close on ESC key
    const closeOnEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        modal.remove();
        backdrop.remove();
        document.removeEventListener('keydown', closeOnEsc);
      }
    };
    document.addEventListener('keydown', closeOnEsc);
    
    document.body.appendChild(backdrop);
    document.body.appendChild(modal);
    closeBtn.focus();
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