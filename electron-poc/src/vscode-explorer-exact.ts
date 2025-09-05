/**
 * Exact VS Code File Explorer Implementation
 * Matches VS Code's precise HTML structure and CSS classes
 */

import { FileNode } from './types/window';
import { getFileIcon, getFolderIcon, createIconElement } from './file-icons';
import { GitDecorationProvider, GitDecoration } from './git-decoration-provider';

// VS Code's TreeItemCollapsibleState
enum TreeItemCollapsibleState {
  None = 0,
  Collapsed = 1, 
  Expanded = 2
}

interface TreeNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  collapsibleState: TreeItemCollapsibleState;
  children?: TreeNode[];
  parent?: TreeNode;
  depth: number;
}

export class VSCodeExplorerExact {
  private container: HTMLElement;
  private treeData: Map<string, TreeNode> = new Map();
  private expandedNodes: Set<string> = new Set();
  private selectedNode: TreeNode | null = null;
  private onFileSelectCallback: ((path: string) => void) | null = null;
  private rootNodes: TreeNode[] = [];
  private draggedNode: TreeNode | null = null;
  private dropTarget: HTMLElement | null = null;
  private gitDecorationProvider: GitDecorationProvider | null = null;
  private rootPath: string = '';

  constructor(container: HTMLElement) {
    this.container = container;
    // Don't initialize automatically - wait for initialize() to be called with a path
  }

  // Public method to initialize/reinitialize with a specific path
  public async initialize(rootPath?: string) {
    if (rootPath) {
      this.rootPath = rootPath;
      console.log('[VSCodeExplorer] Setting root path to:', rootPath);
    }
    
    // Setup container and styles if not already done
    if (!this.container.querySelector('.explorer-folders-view')) {
      this.setupContainer();
      this.attachStyles();
    }
    
    // If no root path set or empty string, show welcome message
    if (!this.rootPath || this.rootPath.trim() === '') {
      console.log('[VSCodeExplorer] No root path set, showing welcome message');
      this.showWelcomeMessage();
      return;
    }
    
    // Re-initialize Git decorations with new path
    await this.initializeGitDecorations();
    
    // Reload the directory tree
    await this.loadRootDirectory();
    this.render();
  }

  private async initializeInternal() {
    console.log('[VSCodeExplorer] Initializing exact VS Code implementation...');
    this.setupContainer();
    this.attachStyles();
    
    // Initialize Git decoration provider
    await this.initializeGitDecorations();
    
    await this.loadRootDirectory();
    this.render();
  }
  
  private async initializeGitDecorations() {
    try {
      console.log('[VSCodeExplorer] Initializing Git decorations...');
      this.gitDecorationProvider = new GitDecorationProvider(this.rootPath);
      
      // Listen for decoration changes
      this.gitDecorationProvider.on('decorationsChanged', (changedPaths: string[]) => {
        console.log('[VSCodeExplorer] Git decorations changed, updating in place...');
        this.updateGitDecorationsInPlace(); // Update decorations without re-rendering
      });
      
      // Initialize the provider
      await this.gitDecorationProvider.initialize();
      console.log('[VSCodeExplorer] Git decorations initialized');
    } catch (error) {
      console.error('[VSCodeExplorer] Failed to initialize Git decorations:', error);
    }
  }

  private setupContainer() {
    this.container.className = 'explorer-folders-view';
    this.container.innerHTML = `
      <div class="monaco-list" role="tree" tabindex="0">
        <div class="monaco-scrollable-element">
          <div class="monaco-list-rows" style="height: 100%;">
            <!-- Tree items will be rendered here -->
          </div>
        </div>
      </div>
    `;
  }

  private showWelcomeMessage() {
    const rowsContainer = this.container.querySelector('.monaco-list-rows');
    if (!rowsContainer) return;
    
    rowsContainer.innerHTML = `
      <div style="padding: 20px; text-align: center; color: #888;">
        <div style="margin-bottom: 20px;">No folder opened</div>
        <button onclick="window.openFolder()" style="
          padding: 8px 16px;
          background: #007ACC;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        ">Open Folder</button>
      </div>
    `;
  }

  private attachStyles() {
    if (document.getElementById('vscode-exact-styles')) return;
    
    const style = document.createElement('style');
    style.id = 'vscode-exact-styles';
    style.textContent = `
      /* Exact VS Code Explorer Styles */
      .explorer-folders-view {
        height: 100%;
        position: relative;
      }

      .monaco-list {
        height: 100%;
        width: 100%;
        white-space: nowrap;
        user-select: none;
        -webkit-user-select: none;
        position: relative;
        overflow: hidden;
      }

      .monaco-scrollable-element {
        height: 100%;
        overflow: auto;
      }

      .monaco-list-rows {
        position: relative;
      }

      .monaco-list-row {
        position: relative;
        box-sizing: border-box;
        overflow: hidden;
        width: 100%;
      }

      .monaco-list-row:hover {
        background-color: rgba(255, 255, 255, 0.04);
      }

      .monaco-list-row.selected {
        background-color: rgba(255, 255, 255, 0.08);
      }

      .monaco-list-row.focused {
        background-color: #094771;
        outline: 1px solid #007ACC;
        outline-offset: -1px;
      }

      .monaco-tl-row {
        display: flex;
        height: 22px;
        line-height: 22px;
        align-items: center;
        cursor: pointer;
      }

      .monaco-tl-indent {
        height: 100%;
        width: 8px;
        display: inline-block;
      }

      .monaco-tl-twistie {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 20px;
        height: 22px;
        flex-shrink: 0;
        cursor: pointer;
      }

      .monaco-tl-twistie.collapsible .codicon {
        transition: transform 0.1s;
      }

      .monaco-tl-twistie.collapsible {
        font-size: 11px;
        color: #cccccc;
        display: flex;
        align-items: center;
        justify-content: center;
      }
      
      .monaco-tl-twistie.collapsible.collapsed::before {
        content: '>'; /* Right-pointing angle bracket */
        display: block;
        transition: transform 0.1s;
      }

      .monaco-tl-twistie.collapsible:not(.collapsed)::before {
        content: '>'; /* Right-pointing angle bracket */
        display: block;
        transform: rotate(90deg);
        transition: transform 0.1s;
      }

      .monaco-tl-contents {
        flex: 1;
        overflow: hidden;
        padding-left: 3px;
      }

      .explorer-item {
        display: flex;
        align-items: center;
        height: 22px;
        line-height: 22px;
      }

      .monaco-icon-label {
        display: flex;
        align-items: center;
        overflow: hidden;
        text-overflow: ellipsis;
        flex: 1;
      }

      .monaco-icon-label-container {
        display: flex;
        align-items: center;
        overflow: hidden;
        text-overflow: ellipsis;
        flex: 1;
      }

      .monaco-icon-name-container {
        display: flex;
        align-items: center;
        flex: 1;
        overflow: hidden;
      }

      .label-name {
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: pre;
        margin-left: 4px;
      }

      .monaco-icon-label .codicon {
        flex-shrink: 0;
        font-size: 16px;
      }

      /* File and folder icons */
      .codicon-folder::before {
        content: '\\eaf7';
        color: #dcb67a;
      }

      .codicon-folder-opened::before {
        content: '\\eaf8';
        color: #dcb67a;
      }

      .codicon-file::before {
        content: '\\ea7b';
      }

      .codicon-file-code::before {
        content: '\\eb08';
      }

      .codicon-json::before {
        content: '\\eb8c';
        color: #cbcb41;
      }

      .codicon-markdown::before {
        content: '\\eb03';
        color: #519aba;
      }
    `;
    document.head.appendChild(style);
  }

  private async loadRootDirectory() {
    try {
      // Don't try to load if rootPath is empty
      if (!this.rootPath || this.rootPath.trim() === '') {
        console.warn('[VSCodeExplorer] Cannot load directory: rootPath is empty');
        this.rootNodes = [];
        return;
      }
      
      console.log('[VSCodeExplorer] Loading root directory:', this.rootPath);
      console.log('[VSCodeExplorer] rootPath type:', typeof this.rootPath);
      console.log('[VSCodeExplorer] rootPath value:', JSON.stringify(this.rootPath));
      const rootItems = await window.fileAPI.getFileTree(this.rootPath);
      
      if (!rootItems || rootItems.length === 0) {
        console.warn('[VSCodeExplorer] No items returned from fileAPI');
        return;
      }
      
      // Convert to TreeNodes
      this.rootNodes = rootItems.map(item => this.createTreeNode(item, 0));
      
      // Store in treeData map
      this.rootNodes.forEach(node => {
        this.treeData.set(node.path, node);
      });
      
      console.log('[VSCodeExplorer] Loaded', this.rootNodes.length, 'root items');
    } catch (error) {
      console.error('[VSCodeExplorer] Failed to load root directory:', error);
    }
  }

  private createTreeNode(item: FileNode, depth: number, parent?: TreeNode): TreeNode {
    return {
      name: item.name,
      path: item.path,
      type: item.type,
      collapsibleState: item.type === 'directory' 
        ? TreeItemCollapsibleState.Collapsed 
        : TreeItemCollapsibleState.None,
      children: undefined,
      parent: parent,
      depth: depth
    };
  }

  private async render() {
    const rowsContainer = this.container.querySelector('.monaco-list-rows');
    if (!rowsContainer) {
      console.error('[VSCodeExplorer] Rows container not found');
      return;
    }

    console.log('[VSCodeExplorer] Rendering tree...');
    
    // Save scroll position before render
    const scrollContainer = this.container.querySelector('.monaco-list') as HTMLElement;
    const scrollTop = scrollContainer?.scrollTop || 0;
    
    // Get flattened visible nodes
    const flatNodes = await this.getFlattenedNodes();
    
    if (flatNodes.length === 0) {
      rowsContainer.innerHTML = '<div class="monaco-list-row">No files to display</div>';
      return;
    }

    // Use document fragment for performance
    const fragment = document.createDocumentFragment();
    
    for (const node of flatNodes) {
      const element = this.createTreeElement(node);
      fragment.appendChild(element);
    }
    
    rowsContainer.innerHTML = '';
    rowsContainer.appendChild(fragment);
    
    // Attach event listeners
    this.attachEventListeners();
    
    // Restore scroll position after render
    if (scrollContainer) {
      // Use requestAnimationFrame to ensure DOM has updated
      requestAnimationFrame(() => {
        scrollContainer.scrollTop = scrollTop;
      });
    }
  }

  private createTreeElement(node: TreeNode): HTMLElement {
    // Create exact VS Code structure
    const row = document.createElement('div');
    row.className = 'monaco-list-row';
    row.setAttribute('data-path', node.path);
    row.setAttribute('data-type', node.type);
    row.setAttribute('role', 'treeitem');
    
    // Add Git status attribute for styling
    if (this.gitDecorationProvider) {
      const decoration = this.gitDecorationProvider.getDecoration(node.path);
      if (decoration && decoration.badge) {
        let status = '';
        switch (decoration.badge) {
          case 'M': status = 'modified'; break;
          case 'A': status = 'added'; break;
          case 'D': status = 'deleted'; break;
          case 'U': status = 'untracked'; break;
          case 'R': status = 'renamed'; break;
        }
        if (status) {
          row.setAttribute('data-git-status', status);
        }
      }
    }
    
    // Make draggable
    row.draggable = true;
    
    if (this.selectedNode?.path === node.path) {
      row.classList.add('selected');
    }
    
    const tlRow = document.createElement('div');
    tlRow.className = 'monaco-tl-row';
    
    // Add indentation
    for (let i = 0; i < node.depth; i++) {
      const indent = document.createElement('div');
      indent.className = 'monaco-tl-indent';
      tlRow.appendChild(indent);
    }
    
    // Add twistie (chevron for directories)
    const twistie = document.createElement('div');
    twistie.className = 'monaco-tl-twistie';
    
    if (node.type === 'directory') {
      twistie.classList.add('collapsible');
      if (node.collapsibleState === TreeItemCollapsibleState.Collapsed) {
        twistie.classList.add('collapsed');
      }
      // No need to add child element - CSS ::before handles the chevron
    }
    
    tlRow.appendChild(twistie);
    
    // Add contents
    const contents = document.createElement('div');
    contents.className = 'monaco-tl-contents';
    
    const explorerItem = document.createElement('div');
    explorerItem.className = 'explorer-item';
    
    const iconLabel = document.createElement('div');
    iconLabel.className = 'monaco-icon-label';
    
    const container = document.createElement('div');
    container.className = 'monaco-icon-label-container';
    
    // Add file/folder icon with color
    const icon = document.createElement('i');
    if (node.type === 'directory') {
      const folderIcon = getFolderIcon(node.name, node.collapsibleState === TreeItemCollapsibleState.Expanded);
      icon.className = `codicon codicon-${folderIcon.icon}`;
      icon.style.cssText = `color: ${folderIcon.color || '#dcb67a'};`;
    } else {
      const fileIcon = getFileIcon(node.name);
      icon.className = `codicon codicon-${fileIcon.icon}`;
      icon.style.cssText = `color: ${fileIcon.color || '#969696'};`;
    }
    container.appendChild(icon);
    
    const nameContainer = document.createElement('span');
    nameContainer.className = 'monaco-icon-name-container';
    
    const label = document.createElement('span');
    label.className = 'label-name';
    label.textContent = node.name;
    
    // Apply Git decoration if available
    let gitIndicator: HTMLElement | null = null;
    if (this.gitDecorationProvider) {
      const decoration = this.gitDecorationProvider.getDecoration(node.path);
      if (decoration) {
        // Apply color to the label
        if (decoration.color) {
          label.style.color = decoration.color;
        }
        
        // Add Git status badge
        if (decoration.badge) {
          gitIndicator = document.createElement('span');
          gitIndicator.className = 'git-indicator';
          gitIndicator.textContent = decoration.badge;
          
          // Determine the status class based on the badge
          if (decoration.badge === 'M') {
            gitIndicator.classList.add('modified');
          } else if (decoration.badge === 'A') {
            gitIndicator.classList.add('added');
          } else if (decoration.badge === 'D') {
            gitIndicator.classList.add('deleted');
          } else if (decoration.badge === 'U') {
            gitIndicator.classList.add('untracked');
          }
        }
        
        // Add tooltip if available
        if (decoration.tooltip) {
          row.title = decoration.tooltip;
        }
      }
    }
    
    // Build the structure
    nameContainer.appendChild(label);
    container.appendChild(nameContainer);
    
    // Add git indicator to the end of the container if it exists
    if (gitIndicator) {
      container.appendChild(gitIndicator);
    }
    
    iconLabel.appendChild(container);
    explorerItem.appendChild(iconLabel);
    contents.appendChild(explorerItem);
    tlRow.appendChild(contents);
    row.appendChild(tlRow);
    
    return row;
  }

  private getFileIconClass(filename: string): string {
    const iconMapping = getFileIcon(filename);
    return `codicon-${iconMapping.icon}`;
  }
  
  private getFileIconStyle(filename: string): string {
    const iconMapping = getFileIcon(filename);
    return `color: ${iconMapping.color || '#969696'};`;
  }

  private async getFlattenedNodes(): Promise<TreeNode[]> {
    const flat: TreeNode[] = [];
    
    const processNode = async (node: TreeNode) => {
      flat.push(node);
      
      // If expanded and directory, include children
      if (node.type === 'directory' && 
          node.collapsibleState === TreeItemCollapsibleState.Expanded) {
        if (!node.children) {
          // Load children if not loaded
          node.children = await this.loadDirectoryChildren(node);
        }
        
        for (const child of node.children || []) {
          await processNode(child);
        }
      }
    };
    
    // Process all root nodes
    for (const node of this.rootNodes) {
      await processNode(node);
    }
    
    return flat;
  }

  private async loadDirectoryChildren(directory: TreeNode): Promise<TreeNode[]> {
    try {
      console.log('[VSCodeExplorer] Loading children for:', directory.path);
      const children = await window.fileAPI.getDirectoryContents(directory.path);
      
      if (!children || children.length === 0) {
        return [];
      }
      
      // Convert to TreeNodes
      const treeNodes = children.map(child => 
        this.createTreeNode(child, directory.depth + 1, directory)
      );
      
      // Store in treeData map
      treeNodes.forEach(node => {
        this.treeData.set(node.path, node);
      });
      
      return treeNodes;
    } catch (error) {
      console.error('[VSCodeExplorer] Failed to load directory children:', error);
      return [];
    }
  }

  private attachEventListeners() {
    const listElement = this.container.querySelector('.monaco-list');
    if (!listElement) return;
    
    // Remove existing listeners
    const newList = listElement.cloneNode(true) as HTMLElement;
    listElement.parentNode?.replaceChild(newList, listElement);
    
    // Use event delegation with proper error handling
    const handleClick = (e: MouseEvent) => {
      try {
        e.preventDefault();
        e.stopPropagation();
        
        const target = e.target as HTMLElement;
        const row = target.closest('.monaco-list-row') as HTMLElement;
        if (!row) return;
        
        const path = row.getAttribute('data-path');
        if (!path) return;
        
        const node = this.treeData.get(path);
        if (!node) return;
        
        // Check if click was on twistie
        const twistie = target.closest('.monaco-tl-twistie');
        if (twistie && node.type === 'directory') {
          // Don't wait for promise, just fire and forget
          this.toggleExpanded(node).catch(err => {
            console.error('[VSCodeExplorer] Error toggling directory:', err);
          });
          return;
        }
        
        // Handle selection
        this.selectedNode = node;
        
        if (node.type === 'file') {
          console.log('[VSCodeExplorer] File selected:', node.path);
          if (this.onFileSelectCallback) {
            // Call the callback in a setTimeout to avoid any sync errors
            setTimeout(() => {
              try {
                console.log('[VSCodeExplorer] About to call onFileSelectCallback');
                this.onFileSelectCallback!(node.path);
                console.log('[VSCodeExplorer] onFileSelectCallback completed');
              } catch (err) {
                console.error('[VSCodeExplorer] Error in file select callback:', err);
                console.error('[VSCodeExplorer] Error type:', err?.constructor?.name);
                console.error('[VSCodeExplorer] Is it an Event?', err instanceof Event);
              }
            }, 0);
          }
        } else {
          // Directory clicked - toggle expansion
          this.toggleExpanded(node).catch(err => {
            console.error('[VSCodeExplorer] Error toggling directory:', err);
          });
        }
        
        // Update selection
        this.updateSelection();
      } catch (error) {
        console.error('[VSCodeExplorer] Error in click handler:', error);
        // Don't re-throw to prevent webpack overlay from catching it
      }
    };
    
    newList.addEventListener('click', handleClick);
    
    // Add drag and drop event delegation
    this.setupDragAndDrop(newList);
  }
  
  private setupDragAndDrop(container: HTMLElement) {
    // Drag start
    container.addEventListener('dragstart', (e: DragEvent) => {
      const row = (e.target as HTMLElement).closest('.monaco-list-row') as HTMLElement;
      if (!row) return;
      
      const path = row.getAttribute('data-path');
      if (!path) return;
      
      const node = this.treeData.get(path);
      if (!node) return;
      
      this.handleDragStart(e, node);
    });
    
    // Drag over
    container.addEventListener('dragover', (e: DragEvent) => {
      e.preventDefault(); // Required to allow drop
      
      const row = (e.target as HTMLElement).closest('.monaco-list-row') as HTMLElement;
      if (!row) return;
      
      const path = row.getAttribute('data-path');
      if (!path) return;
      
      const node = this.treeData.get(path);
      if (!node) return;
      
      this.handleDragOver(e, node);
    });
    
    // Drag enter
    container.addEventListener('dragenter', (e: DragEvent) => {
      const row = (e.target as HTMLElement).closest('.monaco-list-row') as HTMLElement;
      if (!row) return;
      
      const path = row.getAttribute('data-path');
      if (!path) return;
      
      const node = this.treeData.get(path);
      if (!node) return;
      
      this.handleDragEnter(e, node);
    });
    
    // Drag leave
    container.addEventListener('dragleave', (e: DragEvent) => {
      this.handleDragLeave(e);
    });
    
    // Drop
    container.addEventListener('drop', (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      
      const row = (e.target as HTMLElement).closest('.monaco-list-row') as HTMLElement;
      if (!row) return;
      
      const path = row.getAttribute('data-path');
      if (!path) return;
      
      const node = this.treeData.get(path);
      if (!node) return;
      
      this.handleDrop(e, node);
    });
    
    // Drag end
    container.addEventListener('dragend', (e: DragEvent) => {
      this.handleDragEnd(e);
    });
  }

  private async toggleExpanded(node: TreeNode) {
    console.log('[VSCodeExplorer] Toggling:', node.name);
    
    // Save current scroll position before any changes
    const scrollContainer = this.container.querySelector('.monaco-list') as HTMLElement;
    const scrollTop = scrollContainer?.scrollTop || 0;
    
    if (node.collapsibleState === TreeItemCollapsibleState.Collapsed) {
      node.collapsibleState = TreeItemCollapsibleState.Expanded;
      this.expandedNodes.add(node.path);
      
      // Lazy load children if not loaded yet
      if (!node.children) {
        node.children = await this.loadDirectoryChildren(node);
      }
    } else {
      node.collapsibleState = TreeItemCollapsibleState.Collapsed;
      this.expandedNodes.delete(node.path);
    }
    
    await this.render();
    
    // Restore scroll position after render
    if (scrollContainer) {
      scrollContainer.scrollTop = scrollTop;
    }
  }

  private updateSelection() {
    const rows = this.container.querySelectorAll('.monaco-list-row');
    rows.forEach(row => {
      const path = row.getAttribute('data-path');
      if (path === this.selectedNode?.path) {
        row.classList.add('selected', 'focused');
      } else {
        row.classList.remove('selected', 'focused');
      }
    });
  }

  // Public API
  public onFileSelect(callback: (path: string) => void) {
    this.onFileSelectCallback = callback;
  }

  public async refresh() {
    console.log('[VSCodeExplorer] Refreshing...');
    this.treeData.clear();
    this.expandedNodes.clear();
    
    // Refresh Git decorations
    if (this.gitDecorationProvider) {
      await this.gitDecorationProvider.refreshStatus();
    }
    
    await this.loadRootDirectory();
    await this.render();
  }
  
  public async refreshGitStatus() {
    // Lightweight refresh that only updates Git decorations
    // without reloading the entire tree structure
    console.log('[VSCodeExplorer] Refreshing Git status only...');
    
    if (this.gitDecorationProvider) {
      await this.gitDecorationProvider.refreshStatus();
      // Update Git decorations in-place without re-rendering the entire tree
      await this.updateGitDecorationsInPlace();
    }
  }
  
  private async updateGitDecorationsInPlace() {
    // Update existing DOM elements with new Git status without rebuilding the tree
    const rowsContainer = this.container.querySelector('.monaco-list-rows');
    if (!rowsContainer) return;
    
    const rows = rowsContainer.querySelectorAll('.monaco-list-row');
    rows.forEach((row: Element) => {
      const path = row.getAttribute('data-path');
      if (!path) return;
      
      const decoration = this.gitDecorationProvider?.getDecoration(path);
      
      // Update Git status attribute
      if (decoration?.badge) {
        let status = '';
        switch (decoration.badge) {
          case 'M': status = 'modified'; break;
          case 'A': status = 'added'; break;
          case 'D': status = 'deleted'; break;
          case 'U': status = 'untracked'; break;
          case 'R': status = 'renamed'; break;
        }
        if (status) {
          row.setAttribute('data-git-status', status);
        }
      } else {
        row.removeAttribute('data-git-status');
      }
      
      // Update label color
      const label = row.querySelector('.label-name') as HTMLElement;
      if (label && decoration?.color) {
        label.style.color = decoration.color;
      } else if (label) {
        label.style.color = '';
      }
      
      // Update or add Git indicator badge
      let indicator = row.querySelector('.git-indicator') as HTMLElement;
      if (decoration?.badge) {
        if (!indicator) {
          // Create new indicator
          indicator = document.createElement('span');
          indicator.className = 'git-indicator';
          const container = row.querySelector('.monaco-icon-label-container');
          if (container) {
            container.appendChild(indicator);
          }
        }
        indicator.textContent = decoration.badge;
        indicator.className = 'git-indicator ' + (status || '');
      } else if (indicator) {
        // Remove indicator if no longer needed
        indicator.remove();
      }
    });
  }

  public async collapseAll() {
    console.log('[VSCodeExplorer] Collapsing all...');
    // Clear all expanded nodes
    this.expandedNodes.clear();
    
    // Reset all nodes to collapsed state
    this.treeData.forEach(node => {
      if (node.type === 'directory') {
        node.collapsibleState = TreeItemCollapsibleState.Collapsed;
        // Also reset children if loaded
        if (node.children) {
          node.children = undefined; // Clear children to force reload when expanded
        }
      }
    });
    
    // Re-render the tree
    await this.render();
  }

  public getCurrentPath(): string {
    return this.rootPath;
  }

  public async createFile(fileName: string) {
    console.log('[VSCodeExplorer] Create file:', fileName);
    
    // Check if we have a valid root path
    if (!this.rootPath || this.rootPath.trim() === '') {
      console.error('[VSCodeExplorer] Cannot create file: no folder is open');
      alert('Please open a folder first');
      return;
    }
    
    // Get the current directory (use selected directory or root)
    let targetDir = this.rootPath;
    if (this.selectedNode && this.selectedNode.type === 'directory') {
      targetDir = this.selectedNode.path;
    } else if (this.selectedNode && this.selectedNode.parent) {
      targetDir = this.selectedNode.parent.path;
    }
    
    console.log('[VSCodeExplorer] Target directory:', targetDir);
    console.log('[VSCodeExplorer] Full path will be:', targetDir + '/' + fileName);
    
    try {
      // Create the file through IPC
      const result = await window.fileAPI.createFile(targetDir, fileName);
      console.log('[VSCodeExplorer] IPC result:', result);
      console.log('[VSCodeExplorer] File created successfully, refreshing...');
      
      // Give Git a moment to detect the new file
      setTimeout(async () => {
        await this.refresh();
        console.log('[VSCodeExplorer] Refresh completed');
      }, 500);
    } catch (error) {
      console.error('[VSCodeExplorer] Failed to create file:', error);
      alert('Failed to create file: ' + (error as any).message);
    }
  }

  public async createFolder(folderName: string) {
    console.log('[VSCodeExplorer] Create folder:', folderName);
    
    // Check if we have a valid root path
    if (!this.rootPath || this.rootPath.trim() === '') {
      console.error('[VSCodeExplorer] Cannot create folder: no folder is open');
      alert('Please open a folder first');
      return;
    }
    
    // Get the current directory (use selected directory or root)
    let targetDir = this.rootPath;
    if (this.selectedNode && this.selectedNode.type === 'directory') {
      targetDir = this.selectedNode.path;
    } else if (this.selectedNode && this.selectedNode.parent) {
      targetDir = this.selectedNode.parent.path;
    }
    
    console.log('[VSCodeExplorer] Target directory:', targetDir);
    console.log('[VSCodeExplorer] Full path will be:', targetDir + '/' + folderName);
    
    try {
      // Create the folder through IPC
      const result = await window.fileAPI.createFolder(targetDir, folderName);
      console.log('[VSCodeExplorer] IPC result:', result);
      console.log('[VSCodeExplorer] Folder created successfully, refreshing...');
      
      // Give Git a moment to detect the new folder
      setTimeout(async () => {
        await this.refresh();
        console.log('[VSCodeExplorer] Refresh completed');
      }, 500);
    } catch (error) {
      console.error('[VSCodeExplorer] Failed to create folder:', error);
      alert('Failed to create folder: ' + (error as any).message);
    }
  }

  public destroy() {
    // Clean up Git decoration provider
    if (this.gitDecorationProvider) {
      this.gitDecorationProvider.dispose();
      this.gitDecorationProvider = null;
    }
    
    this.container.innerHTML = '';
  }
  
  // Drag and Drop Handlers
  private handleDragStart(e: DragEvent, node: TreeNode) {
    console.log('[VSCodeExplorer] Drag start:', node.name);
    this.draggedNode = node;
    
    // Set drag effect
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', node.path);
    }
    
    // Add dragging class to the row element
    const row = (e.target as HTMLElement).closest('.monaco-list-row') as HTMLElement;
    if (row) {
      row.classList.add('dragging');
      row.style.opacity = '0.5';
    }
  }
  
  private handleDragOver(e: DragEvent, node: TreeNode) {
    e.preventDefault(); // Allow drop
    
    if (!this.draggedNode) return;
    if (this.draggedNode === node) return; // Can't drop on itself
    
    // Only allow dropping on directories
    if (node.type === 'directory') {
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = 'move';
      }
    }
  }
  
  private handleDragEnter(e: DragEvent, node: TreeNode) {
    if (!this.draggedNode) return;
    if (this.draggedNode === node) return; // Can't drop on itself
    
    const row = (e.target as HTMLElement).closest('.monaco-list-row') as HTMLElement;
    if (!row) return;
    
    // Clear previous drop target
    if (this.dropTarget && this.dropTarget !== row) {
      this.dropTarget.classList.remove('drop-target');
      this.dropTarget.style.background = '';
      this.dropTarget.style.border = '';
    }
    
    // Only highlight directories as drop targets
    if (node.type === 'directory') {
      row.classList.add('drop-target');
      row.style.background = 'rgba(0, 122, 204, 0.2)';
      row.style.border = '1px solid #007acc';
      this.dropTarget = row;
    }
  }
  
  private handleDragLeave(e: DragEvent) {
    // Only clear if we're leaving the row entirely
    const relatedTarget = e.relatedTarget as HTMLElement;
    const row = (e.target as HTMLElement).closest('.monaco-list-row') as HTMLElement;
    
    if (row && relatedTarget && !row.contains(relatedTarget)) {
      row.classList.remove('drop-target');
      row.style.background = '';
      row.style.border = '';
    }
  }
  
  private async handleDrop(e: DragEvent, targetNode: TreeNode) {
    e.preventDefault();
    e.stopPropagation();
    
    console.log('[VSCodeExplorer] Drop on:', targetNode.name);
    console.log('[VSCodeExplorer] Dragged node:', this.draggedNode?.name);
    
    if (!this.draggedNode) {
      console.log('[VSCodeExplorer] No dragged node, aborting drop');
      return;
    }
    
    if (this.draggedNode === targetNode) {
      console.log('[VSCodeExplorer] Cannot drop on itself');
      return;
    }
    
    // Clear drop target styles
    const row = (e.target as HTMLElement).closest('.monaco-list-row') as HTMLElement;
    if (row) {
      row.classList.remove('drop-target');
      row.style.background = '';
      row.style.border = '';
    }
    
    // Only allow dropping on directories
    if (targetNode.type === 'directory') {
      try {
        console.log('[VSCodeExplorer] Moving', this.draggedNode.path, 'to', targetNode.path);
        
        // Move the file/folder
        await this.moveItem(this.draggedNode.path, targetNode.path);
        
        console.log('[VSCodeExplorer] Move completed, refreshing tree...');
        
        // Refresh the tree
        await this.refresh();
        
        console.log('[VSCodeExplorer] Tree refreshed');
      } catch (error) {
        console.error('[VSCodeExplorer] Failed to move item:', error);
        alert('Failed to move: ' + (error as any).message);
      }
    } else {
      console.log('[VSCodeExplorer] Target is not a directory, cannot drop');
    }
    
    this.draggedNode = null;
  }
  
  private handleDragEnd(e: DragEvent) {
    console.log('[VSCodeExplorer] Drag end');
    
    // Clear dragging styles
    const draggingElements = this.container.querySelectorAll('.dragging');
    draggingElements.forEach(el => {
      el.classList.remove('dragging');
      (el as HTMLElement).style.opacity = '';
    });
    
    // Clean up any drop targets
    const dropTargets = this.container.querySelectorAll('.drop-target');
    dropTargets.forEach(target => {
      target.classList.remove('drop-target');
      (target as HTMLElement).style.background = '';
      (target as HTMLElement).style.border = '';
    });
    
    this.draggedNode = null;
    this.dropTarget = null;
  }
  
  private async moveItem(sourcePath: string, targetDir: string) {
    console.log('[VSCodeExplorer] Moving', sourcePath, 'to', targetDir);
    
    // Extract the item name from the source path
    const itemName = sourcePath.split('/').pop();
    if (!itemName) throw new Error('Invalid source path');
    
    const newPath = targetDir + '/' + itemName;
    
    // Check if target already exists
    if (await window.fileAPI.fileExists(newPath)) {
      throw new Error(`Item '${itemName}' already exists in the target directory`);
    }
    
    // Move the item using file system operations
    await window.fileAPI.moveFile(sourcePath, newPath);
    
    console.log('[VSCodeExplorer] Item moved successfully');
  }
}