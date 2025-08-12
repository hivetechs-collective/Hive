/**
 * Exact VS Code File Explorer Implementation
 * Matches VS Code's precise HTML structure and CSS classes
 */

import { FileNode } from './types/window';

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

  constructor(container: HTMLElement) {
    this.container = container;
    this.initialize();
  }

  private async initialize() {
    console.log('[VSCodeExplorer] Initializing exact VS Code implementation...');
    this.setupContainer();
    this.attachStyles();
    await this.loadRootDirectory();
    this.render();
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
      console.log('[VSCodeExplorer] Loading root directory...');
      const rootItems = await window.fileAPI.getFileTree();
      
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
  }

  private createTreeElement(node: TreeNode): HTMLElement {
    // Create exact VS Code structure
    const row = document.createElement('div');
    row.className = 'monaco-list-row';
    row.setAttribute('data-path', node.path);
    row.setAttribute('role', 'treeitem');
    
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
    
    // Add file/folder icon
    const icon = document.createElement('i');
    if (node.type === 'directory') {
      icon.className = node.collapsibleState === TreeItemCollapsibleState.Expanded 
        ? 'codicon codicon-folder-opened'
        : 'codicon codicon-folder';
    } else {
      icon.className = `codicon ${this.getFileIconClass(node.name)}`;
    }
    container.appendChild(icon);
    
    const nameContainer = document.createElement('span');
    nameContainer.className = 'monaco-icon-name-container';
    
    const label = document.createElement('span');
    label.className = 'label-name';
    label.textContent = node.name;
    
    nameContainer.appendChild(label);
    container.appendChild(nameContainer);
    iconLabel.appendChild(container);
    explorerItem.appendChild(iconLabel);
    contents.appendChild(explorerItem);
    tlRow.appendChild(contents);
    row.appendChild(tlRow);
    
    return row;
  }

  private getFileIconClass(filename: string): string {
    const ext = filename.split('.').pop()?.toLowerCase();
    const iconMap: Record<string, string> = {
      'ts': 'codicon-file-code',
      'tsx': 'codicon-file-code',
      'js': 'codicon-file-code',
      'jsx': 'codicon-file-code',
      'json': 'codicon-json',
      'md': 'codicon-markdown',
      'css': 'codicon-file-code',
      'html': 'codicon-file-code',
      'rs': 'codicon-file-code',
      'toml': 'codicon-file-code',
      'yaml': 'codicon-file-code',
      'yml': 'codicon-file-code'
    };
    
    return iconMap[ext || ''] || 'codicon-file';
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
  }

  private async toggleExpanded(node: TreeNode) {
    console.log('[VSCodeExplorer] Toggling:', node.name);
    
    if (node.collapsibleState === TreeItemCollapsibleState.Collapsed) {
      node.collapsibleState = TreeItemCollapsibleState.Expanded;
      this.expandedNodes.add(node.path);
    } else {
      node.collapsibleState = TreeItemCollapsibleState.Collapsed;
      this.expandedNodes.delete(node.path);
    }
    
    await this.render();
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
    await this.loadRootDirectory();
    await this.render();
  }

  public async collapseAll() {
    console.log('[VSCodeExplorer] Collapsing all...');
    this.treeData.forEach(node => {
      if (node.type === 'directory') {
        node.collapsibleState = TreeItemCollapsibleState.Collapsed;
      }
    });
    this.expandedNodes.clear();
    await this.render();
  }

  public async createFile(fileName: string) {
    console.log('[VSCodeExplorer] Create file:', fileName);
    
    // Get the current directory (use selected directory or root)
    let targetDir = '/Users/veronelazio/Developer/Private/hive/electron-poc';
    if (this.selectedNode && this.selectedNode.type === 'directory') {
      targetDir = this.selectedNode.path;
    } else if (this.selectedNode && this.selectedNode.parent) {
      targetDir = this.selectedNode.parent.path;
    }
    
    try {
      // Create the file through IPC
      await window.fileAPI.createFile(targetDir, fileName);
      console.log('[VSCodeExplorer] File created successfully');
      await this.refresh();
    } catch (error) {
      console.error('[VSCodeExplorer] Failed to create file:', error);
    }
  }

  public async createFolder(folderName: string) {
    console.log('[VSCodeExplorer] Create folder:', folderName);
    
    // Get the current directory (use selected directory or root)
    let targetDir = '/Users/veronelazio/Developer/Private/hive/electron-poc';
    if (this.selectedNode && this.selectedNode.type === 'directory') {
      targetDir = this.selectedNode.path;
    } else if (this.selectedNode && this.selectedNode.parent) {
      targetDir = this.selectedNode.parent.path;
    }
    
    try {
      // Create the folder through IPC
      await window.fileAPI.createFolder(targetDir, folderName);
      console.log('[VSCodeExplorer] Folder created successfully');
      await this.refresh();
    } catch (error) {
      console.error('[VSCodeExplorer] Failed to create folder:', error);
    }
  }

  public destroy() {
    this.container.innerHTML = '';
  }
}