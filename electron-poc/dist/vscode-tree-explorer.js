"use strict";
/**
 * VS Code-style File Explorer using their actual patterns
 * Based on VS Code's AsyncDataTree implementation
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.VSCodeTreeExplorer = void 0;
// TreeItemCollapsibleState enum (from VS Code)
var TreeItemCollapsibleState;
(function (TreeItemCollapsibleState) {
    TreeItemCollapsibleState[TreeItemCollapsibleState["None"] = 0] = "None";
    TreeItemCollapsibleState[TreeItemCollapsibleState["Collapsed"] = 1] = "Collapsed";
    TreeItemCollapsibleState[TreeItemCollapsibleState["Expanded"] = 2] = "Expanded";
})(TreeItemCollapsibleState || (TreeItemCollapsibleState = {}));
/**
 * VS Code-style File Explorer implementation
 */
class VSCodeTreeExplorer {
    constructor(container) {
        this.treeData = new Map();
        this.expandedNodes = new Set();
        this.visibleNodes = [];
        this.selectedNode = null;
        this.onFileSelectCallback = null;
        // Cache for loaded directories
        this.directoryCache = new Map();
        this.container = container;
        this.initialize();
    }
    initialize() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeTreeExplorer] Initializing...');
            this.setupContainer();
            this.attachStyles();
            yield this.loadRootDirectory();
            this.render();
        });
    }
    setupContainer() {
        this.container.className = 'vscode-tree-explorer';
        this.container.innerHTML = `
      <div class="tree-container" role="tree" tabindex="0">
        <!-- Tree items will be rendered here -->
      </div>
    `;
    }
    attachStyles() {
        if (document.getElementById('vscode-tree-styles'))
            return;
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
    getChildren(element) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!element) {
                // Return root items
                return this.getRootItems();
            }
            // Return children of a directory
            if (element.type === 'directory') {
                return this.getDirectoryChildren(element);
            }
            return [];
        });
    }
    getTreeItem(element) {
        return element;
    }
    getParent(element) {
        return element.parent;
    }
    // Load root directory
    loadRootDirectory() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                console.log('[VSCodeTreeExplorer] Loading root directory...');
                const rootItems = yield window.fileAPI.getFileTree();
                console.log('[VSCodeTreeExplorer] Root items loaded:', (rootItems === null || rootItems === void 0 ? void 0 : rootItems.length) || 0);
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
            }
            catch (error) {
                console.error('[VSCodeTreeExplorer] Failed to load root directory:', error);
                this.visibleNodes = [];
            }
        });
    }
    getRootItems() {
        return __awaiter(this, void 0, void 0, function* () {
            if (this.visibleNodes.length === 0) {
                yield this.loadRootDirectory();
            }
            return this.visibleNodes;
        });
    }
    getDirectoryChildren(directory) {
        return __awaiter(this, void 0, void 0, function* () {
            // Check cache first
            if (this.directoryCache.has(directory.path)) {
                console.log('[VSCodeTreeExplorer] Using cached children for:', directory.path);
                return this.directoryCache.get(directory.path);
            }
            try {
                console.log('[VSCodeTreeExplorer] Loading children for:', directory.path);
                const children = yield window.fileAPI.getDirectoryContents(directory.path);
                if (!children || children.length === 0) {
                    console.log('[VSCodeTreeExplorer] No children found for:', directory.path);
                    return [];
                }
                // Convert to TreeNodes
                const treeNodes = children.map(child => this.createTreeNode(child, directory.level + 1, directory));
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
            }
            catch (error) {
                console.error('[VSCodeTreeExplorer] Failed to load directory children:', error);
                return [];
            }
        });
    }
    createTreeNode(item, level, parent) {
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
    render() {
        return __awaiter(this, void 0, void 0, function* () {
            const treeContainer = this.container.querySelector('.tree-container');
            if (!treeContainer) {
                console.error('[VSCodeTreeExplorer] Tree container not found');
                return;
            }
            console.log('[VSCodeTreeExplorer] Rendering tree...');
            // Get flattened visible nodes
            const flatNodes = yield this.getFlattenedNodes();
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
        });
    }
    getFlattenedNodes() {
        return __awaiter(this, void 0, void 0, function* () {
            const flat = [];
            const processNode = (node) => __awaiter(this, void 0, void 0, function* () {
                flat.push(node);
                // If expanded and has children, include them
                if (node.collapsibleState === TreeItemCollapsibleState.Expanded) {
                    if (!node.children) {
                        // Load children if not loaded
                        node.children = yield this.getDirectoryChildren(node);
                    }
                    for (const child of node.children || []) {
                        yield processNode(child);
                    }
                }
            });
            // Process all root nodes
            for (const node of this.visibleNodes) {
                yield processNode(node);
            }
            return flat;
        });
    }
    createTreeElement(node) {
        var _a, _b;
        const item = document.createElement('div');
        item.className = 'tree-item';
        item.dataset.path = node.path;
        item.dataset.type = node.type;
        item.setAttribute('role', 'treeitem');
        if (((_a = this.selectedNode) === null || _a === void 0 ? void 0 : _a.path) === node.path) {
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
        }
        else {
            // File type icons based on extension
            const ext = (_b = node.name.split('.').pop()) === null || _b === void 0 ? void 0 : _b.toLowerCase();
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
    getFileIcon(node) {
        // Return empty string - we'll use CSS classes instead
        return '';
    }
    attachEventListeners() {
        const treeContainer = this.container.querySelector('.tree-container');
        if (!treeContainer)
            return;
        // Remove existing listeners
        treeContainer.replaceWith(treeContainer.cloneNode(true));
        const newContainer = this.container.querySelector('.tree-container');
        // Use event delegation for better performance
        newContainer.addEventListener('click', (e) => __awaiter(this, void 0, void 0, function* () {
            const target = e.target;
            // Find the tree item element
            const treeItem = target.closest('.tree-item');
            if (!treeItem)
                return;
            const path = treeItem.dataset.path;
            if (!path)
                return;
            const node = this.treeData.get(path);
            if (!node)
                return;
            // Check if click was on chevron
            const chevron = target.closest('.tree-chevron');
            if (chevron && node.type === 'directory') {
                console.log('[VSCodeTreeExplorer] Chevron clicked for:', node.name);
                yield this.toggleExpanded(node);
                return;
            }
            // Handle item selection
            this.selectedNode = node;
            if (node.type === 'file') {
                console.log('[VSCodeTreeExplorer] File selected:', node.path);
                if (this.onFileSelectCallback) {
                    this.onFileSelectCallback(node.path);
                }
            }
            else {
                // Directory clicked (not on chevron) - toggle expansion
                console.log('[VSCodeTreeExplorer] Directory clicked:', node.name);
                yield this.toggleExpanded(node);
            }
            // Update selection highlight
            this.updateSelection();
        }));
    }
    toggleExpanded(node) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeTreeExplorer] Toggling expansion for:', node.name, 'Current state:', node.collapsibleState);
            if (node.collapsibleState === TreeItemCollapsibleState.Collapsed) {
                node.collapsibleState = TreeItemCollapsibleState.Expanded;
                this.expandedNodes.add(node.path);
                // Load children if not loaded
                if (!node.children) {
                    yield this.getDirectoryChildren(node);
                }
            }
            else {
                node.collapsibleState = TreeItemCollapsibleState.Collapsed;
                this.expandedNodes.delete(node.path);
            }
            // Re-render the tree
            yield this.render();
        });
    }
    updateSelection() {
        const items = this.container.querySelectorAll('.tree-item');
        items.forEach(item => {
            var _a;
            const path = item.dataset.path;
            if (path === ((_a = this.selectedNode) === null || _a === void 0 ? void 0 : _a.path)) {
                item.classList.add('selected');
            }
            else {
                item.classList.remove('selected');
            }
        });
    }
    // Public API
    onFileSelect(callback) {
        this.onFileSelectCallback = callback;
    }
    refresh() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeTreeExplorer] Refreshing...');
            this.directoryCache.clear();
            this.treeData.clear();
            this.expandedNodes.clear();
            yield this.loadRootDirectory();
            yield this.render();
        });
    }
    collapseAll() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeTreeExplorer] Collapsing all folders...');
            // Collapse all expanded nodes
            this.treeData.forEach(node => {
                if (node.type === 'directory') {
                    node.collapsibleState = TreeItemCollapsibleState.Collapsed;
                }
            });
            this.expandedNodes.clear();
            yield this.render();
        });
    }
    createFile(fileName) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeTreeExplorer] Create file:', fileName);
            // TODO: Implement file creation
            yield this.refresh();
        });
    }
    createFolder(folderName) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeTreeExplorer] Create folder:', folderName);
            // TODO: Implement folder creation
            yield this.refresh();
        });
    }
    getFileIconClass(ext) {
        // VS Code file type icons
        const iconMap = {
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
    destroy() {
        this.container.innerHTML = '';
    }
}
exports.VSCodeTreeExplorer = VSCodeTreeExplorer;
//# sourceMappingURL=vscode-tree-explorer.js.map