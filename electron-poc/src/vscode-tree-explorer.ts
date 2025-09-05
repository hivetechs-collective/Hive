/**
 * VS Code-style File Explorer using their actual patterns
 * Based on VS Code's AsyncDataTree implementation
 */

// TreeItemCollapsibleState enum (from VS Code)
enum TreeItemCollapsibleState {
  None = 0,
  Collapsed = 1,
  Expanded = 2
}

// Tree node interface
interface TreeNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  collapsibleState: TreeItemCollapsibleState;
  children?: TreeNode[];
  parent?: TreeNode;
  level: number;
}

// Tree data provider interface (VS Code pattern)
interface ITreeDataProvider {
  getChildren(element?: TreeNode): Promise<TreeNode[]>;
  getTreeItem(element: TreeNode): TreeNode;
  getParent(element: TreeNode): TreeNode | undefined;
}

/**
 * VS Code-style File Explorer implementation
 */
export class VSCodeTreeExplorer implements ITreeDataProvider {
  private container: HTMLElement;
  private treeData: Map<string, TreeNode> = new Map();
  private expandedNodes: Set<string> = new Set();
  private visibleNodes: TreeNode[] = [];
  private selectedNode: TreeNode | null = null;
  private onFileSelectCallback: ((path: string) => void) | null = null;
  
  // Cache for loaded directories
  private directoryCache: Map<string, TreeNode[]> = new Map();

  constructor(container: HTMLElement) {
    this.container = container;
    this.initialize();
  }

  private async initialize() {
    console.log('[VSCodeTreeExplorer] Initializing...');
    this.setupContainer();
    this.attachStyles();
    await this.loadRootDirectory();
    this.render();
  }

  private setupContainer() {
    this.container.className = 'vscode-tree-explorer';
    this.container.innerHTML = `
      <div class="tree-container" role="tree" tabindex="0">
        <!-- Tree items will be rendered here -->
      </div>
    `;
  }

  private attachStyles() {
    if (document.getElementById('vscode-tree-styles')) return;
    
    const style = document.createElement('style');
    style.id = 'vscode-tree-styles';
    style.textContent = `
      .vscode-tree-explorer {
        height: 100%;
        overflow-y: auto;
        user-select: none;
        font-size: 13px;
        line-height: 22px;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      }

      .tree-container {
        padding: 0;
      }

      .tree-item {
        display: flex;
        align-items: center;
        height: 22px;
        cursor: pointer;
        position: relative;
      }

      .tree-item:hover {
        background-color: rgba(255, 255, 255, 0.04);
      }

      .tree-item.selected {
        background-color: rgba(255, 255, 255, 0.08);
      }

      .tree-item.focused {
        background-color: rgba(14, 99, 156, 0.6);
      }

      .tree-indent {
        display: inline-block;
        width: 20px;
        flex-shrink: 0;
      }

      .tree-chevron {
        width: 20px;
        height: 22px;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        cursor: pointer;
      }

      .tree-chevron.collapsible {
        position: relative;
      }
      
      .tree-chevron.collapsible .codicon::before {
        content: '\\eb6b'; /* chevron-right */
        font-family: 'codicon';
        font-size: 16px;
        transition: transform 0.1s;
        display: inline-block;
      }
      
      .tree-chevron.expanded .codicon::before {
        transform: rotate(90deg);
      }


      .tree-icon {
        width: 16px;
        height: 16px;
        margin: 0 4px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
      }

      .tree-label {
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }

      .tree-item[data-type="directory"] .tree-label {
        font-weight: 500;
      }
    `;
    document.head.appendChild(style);
  }

  // TreeDataProvider implementation
  async getChildren(element?: TreeNode): Promise<TreeNode[]> {
    if (!element) {
      // Return root items
      return this.getRootItems();
    }
    
    // Return children of a directory
    if (element.type === 'directory') {
      return this.getDirectoryChildren(element);
    }
    
    return [];
  }

  getTreeItem(element: TreeNode): TreeNode {
    return element;
  }

  getParent(element: TreeNode): TreeNode | undefined {
    return element.parent;
  }

  // Load root directory
  private async loadRootDirectory() {
    try {
      console.log('[VSCodeTreeExplorer] Loading root directory...');
      const rootItems = await window.fileAPI.getFileTree();
      console.log('[VSCodeTreeExplorer] Root items loaded:', rootItems?.length || 0);
      
      if (!rootItems || rootItems.length === 0) {
        console.warn('[VSCodeTreeExplorer] No root items returned from fileAPI');
        this.visibleNodes = [];
        return;
      }
      
      // Convert to TreeNodes
      this.visibleNodes = rootItems.map(item => this.createTreeNode(item, 0));
      
      // Store in treeData map
      this.visibleNodes.forEach(node => {
        this.treeData.set(node.path, node);
      });
      
      console.log('[VSCodeTreeExplorer] Visible nodes created:', this.visibleNodes.length);
    } catch (error) {
      console.error('[VSCodeTreeExplorer] Failed to load root directory:', error);
      this.visibleNodes = [];
    }
  }

  private async getRootItems(): Promise<TreeNode[]> {
    if (this.visibleNodes.length === 0) {
      await this.loadRootDirectory();
    }
    return this.visibleNodes;
  }

  private async getDirectoryChildren(directory: TreeNode): Promise<TreeNode[]> {
    // Check cache first
    if (this.directoryCache.has(directory.path)) {
      console.log('[VSCodeTreeExplorer] Using cached children for:', directory.path);
      return this.directoryCache.get(directory.path)!;
    }
    
    try {
      console.log('[VSCodeTreeExplorer] Loading children for:', directory.path);
      const children = await window.fileAPI.getDirectoryContents(directory.path);
      
      if (!children || children.length === 0) {
        console.log('[VSCodeTreeExplorer] No children found for:', directory.path);
        return [];
      }
      
      // Convert to TreeNodes
      const treeNodes = children.map(child => 
        this.createTreeNode(child, directory.level + 1, directory)
      );
      
      // Cache the results
      this.directoryCache.set(directory.path, treeNodes);
      
      // Store in treeData map
      treeNodes.forEach(node => {
        this.treeData.set(node.path, node);
      });
      
      // Update parent's children
      directory.children = treeNodes;
      
      console.log('[VSCodeTreeExplorer] Loaded', treeNodes.length, 'children for:', directory.path);
      return treeNodes;
    } catch (error) {
      console.error('[VSCodeTreeExplorer] Failed to load directory children:', error);
      return [];
    }
  }

  private createTreeNode(item: any, level: number, parent?: TreeNode): TreeNode {
    return {
      name: item.name,
      path: item.path,
      type: item.type,
      collapsibleState: item.type === 'directory' 
        ? TreeItemCollapsibleState.Collapsed 
        : TreeItemCollapsibleState.None,
      children: undefined,
      parent: parent,
      level: level
    };
  }

  // Rendering
  private async render() {
    const treeContainer = this.container.querySelector('.tree-container');
    if (!treeContainer) {
      console.error('[VSCodeTreeExplorer] Tree container not found');
      return;
    }

    console.log('[VSCodeTreeExplorer] Rendering tree...');
    
    // Get flattened visible nodes
    const flatNodes = await this.getFlattenedNodes();
    console.log('[VSCodeTreeExplorer] Rendering', flatNodes.length, 'nodes');
    
    if (flatNodes.length === 0) {
      treeContainer.innerHTML = '<div class="tree-item">No files to display</div>';
      return;
    }

    // Use document fragment for performance
    const fragment = document.createDocumentFragment();
    
    for (const node of flatNodes) {
      const element = this.createTreeElement(node);
      fragment.appendChild(element);
    }
    
    treeContainer.innerHTML = '';
    treeContainer.appendChild(fragment);
    
    // Attach event listeners using event delegation
    this.attachEventListeners();
  }

  private async getFlattenedNodes(): Promise<TreeNode[]> {
    const flat: TreeNode[] = [];
    
    const processNode = async (node: TreeNode) => {
      flat.push(node);
      
      // If expanded and has children, include them
      if (node.collapsibleState === TreeItemCollapsibleState.Expanded) {
        if (!node.children) {
          // Load children if not loaded
          node.children = await this.getDirectoryChildren(node);
        }
        
        for (const child of node.children || []) {
          await processNode(child);
        }
      }
    };
    
    // Process all root nodes
    for (const node of this.visibleNodes) {
      await processNode(node);
    }
    
    return flat;
  }

  private createTreeElement(node: TreeNode): HTMLElement {
    const item = document.createElement('div');
    item.className = 'tree-item';
    item.dataset.path = node.path;
    item.dataset.type = node.type;
    item.setAttribute('role', 'treeitem');
    
    if (this.selectedNode?.path === node.path) {
      item.classList.add('selected');
    }
    
    // Add indentation
    for (let i = 0; i < node.level; i++) {
      const indent = document.createElement('span');
      indent.className = 'tree-indent';
      item.appendChild(indent);
    }
    
    // Add chevron for directories
    const chevron = document.createElement('span');
    chevron.className = 'tree-chevron';
    
    if (node.type === 'directory') {
      chevron.classList.add('collapsible');
      if (node.collapsibleState === TreeItemCollapsibleState.Expanded) {
        chevron.classList.add('expanded');
      }
      // Add the codicon element inside
      const chevronIcon = document.createElement('i');
      chevronIcon.className = 'codicon';
      chevron.appendChild(chevronIcon);
    }
    
    item.appendChild(chevron);
    
    // Add icon with Codicon classes
    const icon = document.createElement('span');
    icon.className = 'tree-icon';
    
    const iconElement = document.createElement('i');
    if (node.type === 'directory') {
      iconElement.className = node.collapsibleState === TreeItemCollapsibleState.Expanded 
        ? 'codicon codicon-folder-opened'
        : 'codicon codicon-folder';
    } else {
      // File type icons based on extension
      const ext = node.name.split('.').pop()?.toLowerCase();
      const iconClass = this.getFileIconClass(ext || '');
      iconElement.className = `codicon ${iconClass}`;
    }
    
    icon.appendChild(iconElement);
    item.appendChild(icon);
    
    // Add label
    const label = document.createElement('span');
    label.className = 'tree-label';
    label.textContent = node.name;
    item.appendChild(label);
    
    return item;
  }

  private getFileIcon(node: TreeNode): string {
    // Return empty string - we'll use CSS classes instead
    return '';
  }

  private attachEventListeners() {
    const treeContainer = this.container.querySelector('.tree-container');
    if (!treeContainer) return;
    
    // Remove existing listeners
    treeContainer.replaceWith(treeContainer.cloneNode(true));
    const newContainer = this.container.querySelector('.tree-container') as HTMLElement;
    
    // Use event delegation for better performance
    newContainer.addEventListener('click', async (e) => {
      const target = e.target as HTMLElement;
      
      // Find the tree item element
      const treeItem = target.closest('.tree-item') as HTMLElement;
      if (!treeItem) return;
      
      const path = treeItem.dataset.path;
      if (!path) return;
      
      const node = this.treeData.get(path);
      if (!node) return;
      
      // Check if click was on chevron
      const chevron = target.closest('.tree-chevron');
      if (chevron && node.type === 'directory') {
        console.log('[VSCodeTreeExplorer] Chevron clicked for:', node.name);
        await this.toggleExpanded(node);
        return;
      }
      
      // Handle item selection
      this.selectedNode = node;
      
      if (node.type === 'file') {
        console.log('[VSCodeTreeExplorer] File selected:', node.path);
        if (this.onFileSelectCallback) {
          this.onFileSelectCallback(node.path);
        }
      } else {
        // Directory clicked (not on chevron) - toggle expansion
        console.log('[VSCodeTreeExplorer] Directory clicked:', node.name);
        await this.toggleExpanded(node);
      }
      
      // Update selection highlight
      this.updateSelection();
    });
  }

  private async toggleExpanded(node: TreeNode) {
    console.log('[VSCodeTreeExplorer] Toggling expansion for:', node.name, 'Current state:', node.collapsibleState);
    
    if (node.collapsibleState === TreeItemCollapsibleState.Collapsed) {
      node.collapsibleState = TreeItemCollapsibleState.Expanded;
      this.expandedNodes.add(node.path);
      
      // Load children if not loaded
      if (!node.children) {
        await this.getDirectoryChildren(node);
      }
    } else {
      node.collapsibleState = TreeItemCollapsibleState.Collapsed;
      this.expandedNodes.delete(node.path);
    }
    
    // Re-render the tree
    await this.render();
  }

  private updateSelection() {
    const items = this.container.querySelectorAll('.tree-item');
    items.forEach(item => {
      const path = (item as HTMLElement).dataset.path;
      if (path === this.selectedNode?.path) {
        item.classList.add('selected');
      } else {
        item.classList.remove('selected');
      }
    });
  }

  // Public API
  public onFileSelect(callback: (path: string) => void) {
    this.onFileSelectCallback = callback;
  }

  public async refresh() {
    console.log('[VSCodeTreeExplorer] Refreshing...');
    this.directoryCache.clear();
    this.treeData.clear();
    this.expandedNodes.clear();
    await this.loadRootDirectory();
    await this.render();
  }

  public async collapseAll() {
    console.log('[VSCodeTreeExplorer] Collapsing all folders...');
    // Collapse all expanded nodes
    this.treeData.forEach(node => {
      if (node.type === 'directory') {
        node.collapsibleState = TreeItemCollapsibleState.Collapsed;
      }
    });
    this.expandedNodes.clear();
    await this.render();
  }

  public async createFile(fileName: string) {
    console.log('[VSCodeTreeExplorer] Create file:', fileName);
    // TODO: Implement file creation
    await this.refresh();
  }

  public async createFolder(folderName: string) {
    console.log('[VSCodeTreeExplorer] Create folder:', folderName);
    // TODO: Implement folder creation
    await this.refresh();
  }

  private getFileIconClass(ext: string): string {
    // VS Code file type icons
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
      'toml': 'codicon-settings-gear',
      'yaml': 'codicon-file-code',
      'yml': 'codicon-file-code',
      'txt': 'codicon-file',
      'log': 'codicon-output',
      'gitignore': 'codicon-git',
      'env': 'codicon-gear',
      'lock': 'codicon-lock',
      'png': 'codicon-file-media',
      'jpg': 'codicon-file-media',
      'jpeg': 'codicon-file-media',
      'gif': 'codicon-file-media',
      'svg': 'codicon-file-media',
      'pdf': 'codicon-file-pdf',
      'zip': 'codicon-file-zip',
      'tar': 'codicon-file-zip',
      'gz': 'codicon-file-zip'
    };
    
    return iconMap[ext] || 'codicon-file';
  }

  public destroy() {
    this.container.innerHTML = '';
  }
}