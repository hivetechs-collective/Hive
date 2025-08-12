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
    // Load initial file tree
    await this.loadFileTree();
    
    // Set up virtual scrolling for performance
    this.virtualScroller = new VirtualScroller(this.container, {
      itemHeight: 22,
      buffer: 10,
      renderItem: (node: FileNode) => this.renderFileNode(node)
    });
    
    // Initial render
    this.render();
  }

  private async loadFileTree() {
    try {
      this.fileTree = await window.fileAPI.getFileTree();
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
      const status = await window.gitAPI.getFileStatus(path);
      if (status) {
        const indicator = document.createElement('span');
        indicator.className = `git-indicator ${status}`;
        indicator.textContent = this.getGitStatusIcon(status);
        element.appendChild(indicator);
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
    
    if (node.type === 'directory') {
      // Toggle expansion
      node.expanded = !node.expanded;
      if (node.expanded) {
        this.expandedPaths.add(node.path);
        // Lazy load children if not loaded
        if (!node.children) {
          node.children = await window.fileAPI.getDirectoryContents(node.path);
        }
      } else {
        this.expandedPaths.delete(node.path);
      }
      this.updateVisibleNodes();
      this.render();
    } else {
      // Open file in editor
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

  private render() {
    if (this.virtualScroller) {
      this.virtualScroller.setItems(this.visibleNodes);
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