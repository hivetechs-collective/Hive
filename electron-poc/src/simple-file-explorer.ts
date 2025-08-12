/**
 * Simple, bulletproof file explorer without any fancy features
 * Just works without throwing Event objects
 */

interface SimpleTreeNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  children?: SimpleTreeNode[];
  expanded?: boolean;
}

export class SimpleFileExplorer {
  private container: HTMLElement;
  private onFileSelect: ((path: string) => void) | null = null;
  private nodes: Map<string, SimpleTreeNode> = new Map();

  constructor(container: HTMLElement) {
    this.container = container;
    this.init();
  }

  private async init() {
    this.container.innerHTML = '<div class="simple-explorer">Loading...</div>';
    
    try {
      const files = await window.fileAPI.getFileTree();
      this.renderTree(files);
    } catch (error) {
      this.container.innerHTML = '<div class="simple-explorer">Failed to load files</div>';
    }
  }

  private renderTree(files: any[]) {
    const explorer = document.createElement('div');
    explorer.className = 'simple-explorer';
    explorer.style.cssText = `
      height: 100%;
      overflow-y: auto;
      font-family: monospace;
      font-size: 13px;
      line-height: 20px;
      padding: 4px;
    `;

    files.forEach(file => {
      const node = this.createNode(file, 0);
      if (node) explorer.appendChild(node);
    });

    this.container.innerHTML = '';
    this.container.appendChild(explorer);
  }

  private createNode(item: any, level: number): HTMLElement | null {
    const div = document.createElement('div');
    div.style.cssText = `
      padding-left: ${level * 20}px;
      cursor: pointer;
      user-select: none;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    `;
    
    // Store node data
    const node: SimpleTreeNode = {
      name: item.name,
      path: item.path,
      type: item.type,
      expanded: false
    };
    this.nodes.set(item.path, node);

    // Create the display
    const icon = item.type === 'directory' ? '📁 ' : '📄 ';
    div.textContent = icon + item.name;
    div.dataset.path = item.path;
    div.dataset.type = item.type;

    // Simple hover effect
    div.onmouseenter = () => {
      div.style.background = 'rgba(255,255,255,0.1)';
    };
    div.onmouseleave = () => {
      div.style.background = '';
    };

    // Handle clicks WITHOUT throwing events
    div.onclick = () => {
      if (item.type === 'file' && this.onFileSelect) {
        // Just call the callback, no promises, no async, no events
        try {
          this.onFileSelect(item.path);
        } catch (e) {
          console.log('File select failed:', e);
        }
      } else if (item.type === 'directory') {
        this.toggleDirectory(div, item, level);
      }
    };

    return div;
  }

  private async toggleDirectory(div: HTMLElement, item: any, level: number) {
    const node = this.nodes.get(item.path);
    if (!node) return;

    if (node.expanded) {
      // Collapse
      node.expanded = false;
      let nextSibling = div.nextSibling;
      while (nextSibling && (nextSibling as HTMLElement).dataset?.parent === item.path) {
        const toRemove = nextSibling;
        nextSibling = nextSibling.nextSibling;
        toRemove.parentNode?.removeChild(toRemove);
      }
      div.textContent = '📁 ' + item.name;
    } else {
      // Expand
      node.expanded = true;
      div.textContent = '📂 ' + item.name;
      
      try {
        const children = await window.fileAPI.getDirectoryContents(item.path);
        children.forEach(child => {
          const childDiv = this.createNode(child, level + 1);
          if (childDiv) {
            childDiv.dataset.parent = item.path;
            div.parentNode?.insertBefore(childDiv, div.nextSibling);
          }
        });
      } catch (e) {
        console.log('Failed to load directory:', e);
      }
    }
  }

  public onFileSelected(callback: (path: string) => void) {
    this.onFileSelect = callback;
  }

  public async refresh() {
    await this.init();
  }

  public async createFile(name: string) {
    // Not implemented
  }

  public async createFolder(name: string) {
    // Not implemented
  }

  public async collapseAll() {
    this.nodes.forEach(node => node.expanded = false);
    await this.init();
  }
}