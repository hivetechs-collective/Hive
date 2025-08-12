/**
 * High-Performance File Explorer Component
 * Uses virtual scrolling and lazy loading for optimal performance
 */

export interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: FileNode[];
  expanded?: boolean;
  size?: number;
  modified?: Date;
  icon?: string;
}

export class FileExplorer {
  private container: HTMLElement;
  private fileTree: FileNode[] = [];
  private visibleNodes: FileNode[] = [];
  private selectedPath: string | null = null;
  private expandedPaths = new Set<string>();
  private fileChangeCallbacks: ((path: string) => void)[] = [];
  private virtualScroller: VirtualScroller | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
    this.init();
  }

  private async init() {
    console.log('Initializing FileExplorer with container:', this.container);
    
    // Load initial file tree
    await this.loadFileTree();
    
    // Set up virtual scrolling for performance
    console.log('Creating VirtualScroller...');
    this.virtualScroller = new VirtualScroller(this.container, {
      itemHeight: 22,
      buffer: 10,
      renderItem: (node: FileNode & { level: number }) => this.renderFileNode(node)
    });
    console.log('VirtualScroller created successfully');
    
    // Initial render
    this.render();
  }

  private async loadFileTree() {
    try {
      console.log('Loading file tree...');
      this.fileTree = await window.fileAPI.getFileTree();
      console.log('File tree loaded:', this.fileTree?.length, 'items');
      this.updateVisibleNodes();
    } catch (error) {
      console.error('Failed to load file tree:', error);
    }
  }

  private updateVisibleNodes() {
    this.visibleNodes = [];
    this.flattenTree(this.fileTree, 0);
  }

  private flattenTree(nodes: FileNode[], level: number) {
    for (const node of nodes) {
      this.visibleNodes.push({ ...node, level } as any);
      
      if (node.type === 'directory' && node.expanded && node.children) {
        this.flattenTree(node.children, level + 1);
      }
    }
  }

  private renderFileNode(node: FileNode & { level: number }): HTMLElement {
    const element = document.createElement('div');
    element.className = 'file-node';
    element.style.paddingLeft = `${node.level * 16 + 8}px`;
    element.dataset.path = node.path;
    
    // Add click handler
    element.addEventListener('click', (e) => this.handleNodeClick(node, e));
    
    // Icon
    const icon = document.createElement('span');
    icon.className = 'file-icon';
    icon.innerHTML = this.getFileIcon(node);
    
    // Name
    const name = document.createElement('span');
    name.className = 'file-name';
    name.textContent = node.name;
    
    // Git status indicator (if file is modified)
    if (node.type === 'file') {
      this.addGitStatus(element, node.path);
    }
    
    element.appendChild(icon);
    element.appendChild(name);
    
    if (this.selectedPath === node.path) {
      element.classList.add('selected');
    }
    
    return element;
  }

  private async addGitStatus(element: HTMLElement, path: string) {
    // Check Git status asynchronously to avoid blocking
    requestIdleCallback(async () => {
      try {
        const gitStatus = await window.gitAPI.getStatus();
        const file = gitStatus.files.find(f => f.path === path);
        if (file) {
          const indicator = document.createElement('span');
          const status = file.index !== ' ' ? 'staged' : file.working !== ' ' ? 'modified' : '';
          if (status) {
            indicator.className = `git-indicator ${status}`;
            indicator.textContent = this.getGitStatusIcon(status);
            element.appendChild(indicator);
          }
        }
      } catch (error) {
        // Ignore git status errors
      }
    });
  }

  private getFileIcon(node: FileNode): string {
    if (node.type === 'directory') {
      return node.expanded ? '📂' : '📁';
    }
    
    // File type icons
    const ext = node.name.split('.').pop()?.toLowerCase();
    const iconMap: Record<string, string> = {
      'ts': '📘',
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
      'yml': '📑'
    };
    
    return iconMap[ext || ''] || '📄';
  }

  private getGitStatusIcon(status: string): string {
    const statusMap: Record<string, string> = {
      'modified': 'M',
      'added': 'A',
      'deleted': 'D',
      'renamed': 'R',
      'untracked': 'U'
    };
    return statusMap[status] || '';
  }

  private async handleNodeClick(node: FileNode & { level: number }, event: Event) {
    event.stopPropagation();
    console.log('Node clicked:', node.name, node.type, 'expanded:', node.expanded);
    
    if (node.type === 'directory') {
      // Toggle expansion
      node.expanded = !node.expanded;
      console.log('Directory toggled to:', node.expanded);
      
      if (node.expanded) {
        this.expandedPaths.add(node.path);
        // Lazy load children if not loaded
        if (!node.children) {
          console.log('Loading directory contents for:', node.path);
          try {
            node.children = await window.fileAPI.getDirectoryContents(node.path);
            console.log('Loaded', node.children?.length, 'children for', node.path);
          } catch (error) {
            console.error('Failed to load directory contents:', error);
            node.children = [];
          }
        }
      } else {
        this.expandedPaths.delete(node.path);
      }
      this.updateVisibleNodes();
      this.render();
    } else {
      // Open file in editor
      console.log('File selected:', node.path);
      this.selectedPath = node.path;
      this.fileChangeCallbacks.forEach(cb => cb(node.path));
      this.render();
    }
  }

  public onFileSelect(callback: (path: string) => void) {
    this.fileChangeCallbacks.push(callback);
  }

  public async refresh() {
    await this.loadFileTree();
    this.render();
  }
  
  public setFileTree(tree: FileNode[]) {
    this.fileTree = tree;
    this.updateVisibleNodes();
    this.render();
  }
  
  public async createFile(fileName: string) {
    try {
      // For now, just log the creation - implement actual file creation later
      console.log('Creating file:', fileName);
      // TODO: Call file system API to create file
      // await window.fileAPI.writeFile(path.join(currentPath, fileName), '');
      // this.refresh();
    } catch (error) {
      console.error('Failed to create file:', error);
    }
  }
  
  public async createFolder(folderName: string) {
    try {
      // For now, just log the creation - implement actual folder creation later
      console.log('Creating folder:', folderName);
      // TODO: Call file system API to create folder
      // await window.fileAPI.createDirectory(path.join(currentPath, folderName));
      // this.refresh();
    } catch (error) {
      console.error('Failed to create folder:', error);
    }
  }

  private render() {
    if (this.virtualScroller) {
      console.log('Rendering virtual scroller with', this.visibleNodes.length, 'visible nodes');
      this.virtualScroller.setItems(this.visibleNodes);
    } else {
      console.error('Virtual scroller not initialized');
    }
  }

  public destroy() {
    if (this.virtualScroller) {
      this.virtualScroller.destroy();
    }
  }
}

/**
 * Virtual Scroller for handling large file trees efficiently
 */
class VirtualScroller {
  private container: HTMLElement;
  private items: any[] = [];
  private itemHeight: number;
  private buffer: number;
  private renderItem: (item: any) => HTMLElement;
  private scrollElement: HTMLElement;
  private contentElement: HTMLElement;
  private visibleStart = 0;
  private visibleEnd = 0;
  private scrollHandler: any;

  constructor(container: HTMLElement, options: {
    itemHeight: number;
    buffer: number;
    renderItem: (item: any) => HTMLElement;
  }) {
    this.container = container;
    this.itemHeight = options.itemHeight;
    this.buffer = options.buffer;
    this.renderItem = options.renderItem;
    
    this.scrollElement = document.createElement('div');
    this.scrollElement.className = 'virtual-scroller';
    this.scrollElement.style.height = '100%';
    this.scrollElement.style.overflow = 'auto';
    
    this.contentElement = document.createElement('div');
    this.contentElement.className = 'virtual-content';
    
    this.scrollElement.appendChild(this.contentElement);
    this.container.appendChild(this.scrollElement);
    
    this.scrollHandler = this.handleScroll.bind(this);
    this.scrollElement.addEventListener('scroll', this.scrollHandler, { passive: true });
  }

  setItems(items: any[]) {
    this.items = items;
    this.contentElement.style.height = `${items.length * this.itemHeight}px`;
    this.updateVisibleItems();
  }

  private handleScroll() {
    requestAnimationFrame(() => this.updateVisibleItems());
  }

  private updateVisibleItems() {
    const scrollTop = this.scrollElement.scrollTop;
    const containerHeight = this.scrollElement.clientHeight;
    
    const newVisibleStart = Math.max(0, Math.floor(scrollTop / this.itemHeight) - this.buffer);
    const newVisibleEnd = Math.min(
      this.items.length,
      Math.ceil((scrollTop + containerHeight) / this.itemHeight) + this.buffer
    );
    
    if (newVisibleStart !== this.visibleStart || newVisibleEnd !== this.visibleEnd) {
      this.visibleStart = newVisibleStart;
      this.visibleEnd = newVisibleEnd;
      this.renderVisibleItems();
    }
  }

  private renderVisibleItems() {
    // Clear content
    this.contentElement.innerHTML = '';
    
    // Create a fragment for better performance
    const fragment = document.createDocumentFragment();
    
    for (let i = this.visibleStart; i < this.visibleEnd; i++) {
      const item = this.items[i];
      if (item) {
        const element = this.renderItem(item);
        element.style.position = 'absolute';
        element.style.top = `${i * this.itemHeight}px`;
        element.style.height = `${this.itemHeight}px`;
        element.style.width = '100%';
        fragment.appendChild(element);
      }
    }
    
    this.contentElement.appendChild(fragment);
  }

  destroy() {
    this.scrollElement.removeEventListener('scroll', this.scrollHandler);
    this.container.removeChild(this.scrollElement);
  }
}