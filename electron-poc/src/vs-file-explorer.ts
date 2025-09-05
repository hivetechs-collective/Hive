/**
 * VS Code-style File Explorer
 * Based on VS Code's actual implementation patterns
 */

export interface FileTreeNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: FileTreeNode[];
  expanded?: boolean;
  depth: number;
  hasChildren?: boolean;
}

export class VSCodeFileExplorer {
  private container: HTMLElement;
  private treeNodes: FileTreeNode[] = [];
  private flattenedNodes: FileTreeNode[] = [];
  private selectedPath: string | null = null;
  private onFileSelectCallback: ((path: string) => void) | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
    this.init();
  }

  private async init() {
    console.log('Initializing VS Code File Explorer');
    this.setupContainer();
    await this.loadRootDirectory();
    this.render();
  }

  private setupContainer() {
    this.container.className = 'vs-file-explorer';
    this.container.innerHTML = `
      <div class="file-tree" role="tree">
        <!-- File tree items will be rendered here -->
      </div>
    `;
    
    // Add CSS styles
    this.addStyles();
  }

  private addStyles() {
    if (document.getElementById('vs-file-explorer-styles')) return;
    
    const style = document.createElement('style');
    style.id = 'vs-file-explorer-styles';
    style.textContent = `
      .vs-file-explorer {
        height: 100%;
        overflow-y: auto;
        user-select: none;
        font-size: 13px;
        line-height: 22px;
      }

      .file-tree {
        padding: 0;
      }

      .file-tree-item {
        display: flex;
        align-items: center;
        padding: 0 8px;
        height: 22px;
        cursor: pointer;
        white-space: nowrap;
      }

      .file-tree-item:hover {
        background-color: rgba(255, 255, 255, 0.1);
      }

      .file-tree-item.selected {
        background-color: rgba(255, 255, 255, 0.2);
      }

      .file-tree-item.focused {
        background-color: rgba(14, 99, 156, 0.8);
      }

      .file-tree-indent {
        display: inline-block;
        width: 8px;
      }

      .file-tree-twistie {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 16px;
        height: 16px;
        margin-right: 4px;
        cursor: pointer;
        opacity: 0.8;
      }

      .file-tree-twistie.collapsible {
        opacity: 1;
      }

      .file-tree-twistie.expanded {
        transform: rotate(90deg);
      }

      .file-tree-twistie svg {
        width: 16px;
        height: 16px;
        fill: currentColor;
      }

      .file-tree-icon {
        width: 16px;
        height: 16px;
        margin-right: 6px;
        display: flex;
        align-items: center;
        justify-content: center;
      }

      .file-tree-label {
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
      }
    `;
    document.head.appendChild(style);
  }

  private async loadRootDirectory() {
    try {
      console.log('Loading root directory...');
      const rootItems = await window.fileAPI.getFileTree();
      console.log('Root items loaded:', rootItems?.length || 0);
      
      this.treeNodes = (rootItems || []).map((item, index) => ({
        name: item.name,
        path: item.path,
        type: item.type,
        children: item.type === 'directory' ? [] : undefined,
        expanded: false,
        depth: 0,
        hasChildren: item.type === 'directory'
      }));
      
      this.updateFlattenedNodes();
    } catch (error) {
      console.error('Failed to load root directory:', error);
      this.treeNodes = [];
      this.updateFlattenedNodes();
    }
  }

  private updateFlattenedNodes() {
    this.flattenedNodes = [];
    this.flattenNodes(this.treeNodes);
    console.log('Flattened nodes updated:', this.flattenedNodes.length);
  }

  private flattenNodes(nodes: FileTreeNode[], depth = 0) {
    for (const node of nodes) {
      this.flattenedNodes.push({ ...node, depth });
      
      if (node.expanded && node.children) {
        this.flattenNodes(node.children, depth + 1);
      }
    }
  }

  private render() {
    const treeContainer = this.container.querySelector('.file-tree');
    if (!treeContainer) return;

    console.log('Rendering file tree with', this.flattenedNodes.length, 'nodes');

    if (this.flattenedNodes.length === 0) {
      treeContainer.innerHTML = '<div class="file-tree-item">No files to display</div>';
      return;
    }

    const fragment = document.createDocumentFragment();
    
    for (const node of this.flattenedNodes) {
      const item = this.createTreeItem(node);
      fragment.appendChild(item);
    }

    treeContainer.innerHTML = '';
    treeContainer.appendChild(fragment);
  }

  private createTreeItem(node: FileTreeNode): HTMLElement {
    const item = document.createElement('div');
    item.className = 'file-tree-item';
    item.dataset.path = node.path;
    item.setAttribute('role', 'treeitem');
    item.setAttribute('aria-expanded', node.expanded?.toString() || 'false');
    
    if (this.selectedPath === node.path) {
      item.classList.add('selected');
    }

    // Add indentation
    for (let i = 0; i < node.depth; i++) {
      const indent = document.createElement('span');
      indent.className = 'file-tree-indent';
      item.appendChild(indent);
    }

    // Add twistie (expand/collapse arrow)
    const twistie = document.createElement('span');
    twistie.className = 'file-tree-twistie';
    
    if (node.type === 'directory') {
      twistie.classList.add('collapsible');
      if (node.expanded) {
        twistie.classList.add('expanded');
      }
      
      twistie.innerHTML = `
        <svg viewBox="0 0 16 16">
          <path d="M6 4v8l4-4z"/>
        </svg>
      `;
    }
    
    item.appendChild(twistie);

    // Add file/folder icon
    const icon = document.createElement('span');
    icon.className = 'file-tree-icon';
    icon.textContent = this.getIcon(node);
    item.appendChild(icon);

    // Add label
    const label = document.createElement('span');
    label.className = 'file-tree-label';
    label.textContent = node.name;
    item.appendChild(label);

    // Add event listeners
    item.addEventListener('click', (e) => this.handleItemClick(node, e));
    twistie.addEventListener('click', (e) => this.handleTwistieClick(node, e));

    return item;
  }

  private getIcon(node: FileTreeNode): string {
    if (node.type === 'directory') {
      return node.expanded ? '📂' : '📁';
    }
    
    // File type icons
    const ext = node.name.split('.').pop()?.toLowerCase();
    const iconMap: Record<string, string> = {
      'ts': '🔷',
      'tsx': '⚛️',
      'js': '📄',
      'jsx': '⚛️',
      'json': '📋',
      'md': '📝',
      'css': '🎨',
      'html': '🌐',
      'rs': '🦀',
      'toml': '⚙️',
      'yaml': '📑',
      'yml': '📑',
      'png': '🖼️',
      'jpg': '🖼️',
      'jpeg': '🖼️',
      'gif': '🖼️',
      'svg': '🖼️'
    };
    
    return iconMap[ext || ''] || '📄';
  }

  private async handleTwistieClick(node: FileTreeNode, event: Event) {
    event.stopPropagation();
    
    if (node.type !== 'directory') return;
    
    console.log('Twistie clicked for:', node.name, 'current expanded:', node.expanded);
    
    if (!node.expanded) {
      // Expand the directory
      await this.expandDirectory(node);
    } else {
      // Collapse the directory
      node.expanded = false;
    }
    
    this.updateFlattenedNodes();
    this.render();
  }

  private async handleItemClick(node: FileTreeNode, event: Event) {
    event.stopPropagation();
    
    console.log('Item clicked:', node.name, node.type);
    
    // Update selection
    this.selectedPath = node.path;
    
    if (node.type === 'file') {
      // Open file in editor
      if (this.onFileSelectCallback) {
        this.onFileSelectCallback(node.path);
      }
    } else if (node.type === 'directory') {
      // Toggle directory expansion
      await this.handleTwistieClick(node, event);
      return; // Don't re-render since handleTwistieClick already does it
    }
    
    this.render(); // Update selection highlighting
  }

  private async expandDirectory(node: FileTreeNode) {
    try {
      console.log('Expanding directory:', node.path);
      
      if (!node.children) {
        node.children = [];
      }
      
      if (node.children.length === 0) {
        // Load children from file system
        const children = await window.fileAPI.getDirectoryContents(node.path);
        console.log('Loaded', children?.length || 0, 'children for', node.path);
        
        node.children = (children || []).map(child => ({
          name: child.name,
          path: child.path,
          type: child.type,
          children: child.type === 'directory' ? [] : undefined,
          expanded: false,
          depth: node.depth + 1,
          hasChildren: child.type === 'directory'
        }));
      }
      
      node.expanded = true;
    } catch (error) {
      console.error('Failed to expand directory:', error);
    }
  }

  public onFileSelect(callback: (path: string) => void) {
    this.onFileSelectCallback = callback;
  }

  public async refresh() {
    console.log('Refreshing file explorer');
    await this.loadRootDirectory();
    this.render();
  }

  public async createFile(fileName: string) {
    console.log('Create file requested:', fileName);
    // TODO: Implement file creation
    // For now, just refresh to show any changes
    await this.refresh();
  }

  public async createFolder(folderName: string) {
    console.log('Create folder requested:', folderName);
    // TODO: Implement folder creation
    // For now, just refresh to show any changes
    await this.refresh();
  }

  public destroy() {
    // Clean up event listeners and resources
    this.container.innerHTML = '';
  }
}