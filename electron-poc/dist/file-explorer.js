"use strict";
/**
 * High-Performance File Explorer Component
 * Uses virtual scrolling and lazy loading for optimal performance
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
exports.FileExplorer = void 0;
class FileExplorer {
    constructor(container) {
        this.fileTree = [];
        this.visibleNodes = [];
        this.selectedPath = null;
        this.expandedPaths = new Set();
        this.fileChangeCallbacks = [];
        this.virtualScroller = null;
        this.container = container;
        this.init();
    }
    init() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('Initializing FileExplorer with container:', this.container);
            // Load initial file tree
            yield this.loadFileTree();
            // Set up virtual scrolling for performance
            console.log('Creating VirtualScroller...');
            this.virtualScroller = new VirtualScroller(this.container, {
                itemHeight: 22,
                buffer: 10,
                renderItem: (node) => this.renderFileNode(node)
            });
            console.log('VirtualScroller created successfully');
            // Initial render
            this.render();
        });
    }
    loadFileTree() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            try {
                console.log('Loading file tree...');
                this.fileTree = yield window.fileAPI.getFileTree();
                console.log('File tree loaded:', (_a = this.fileTree) === null || _a === void 0 ? void 0 : _a.length, 'items');
                this.updateVisibleNodes();
            }
            catch (error) {
                console.error('Failed to load file tree:', error);
            }
        });
    }
    updateVisibleNodes() {
        this.visibleNodes = [];
        this.flattenTree(this.fileTree, 0);
    }
    flattenTree(nodes, level) {
        for (const node of nodes) {
            this.visibleNodes.push(Object.assign(Object.assign({}, node), { level }));
            if (node.type === 'directory' && node.expanded && node.children) {
                this.flattenTree(node.children, level + 1);
            }
        }
    }
    renderFileNode(node) {
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
    addGitStatus(element, path) {
        return __awaiter(this, void 0, void 0, function* () {
            // Check Git status asynchronously to avoid blocking
            requestIdleCallback(() => __awaiter(this, void 0, void 0, function* () {
                try {
                    const gitStatus = yield window.gitAPI.getStatus();
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
                }
                catch (error) {
                    // Ignore git status errors
                }
            }));
        });
    }
    getFileIcon(node) {
        var _a;
        if (node.type === 'directory') {
            return node.expanded ? '📂' : '📁';
        }
        // File type icons
        const ext = (_a = node.name.split('.').pop()) === null || _a === void 0 ? void 0 : _a.toLowerCase();
        const iconMap = {
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
    getGitStatusIcon(status) {
        const statusMap = {
            'modified': 'M',
            'added': 'A',
            'deleted': 'D',
            'renamed': 'R',
            'untracked': 'U'
        };
        return statusMap[status] || '';
    }
    handleNodeClick(node, event) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
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
                            node.children = yield window.fileAPI.getDirectoryContents(node.path);
                            console.log('Loaded', (_a = node.children) === null || _a === void 0 ? void 0 : _a.length, 'children for', node.path);
                        }
                        catch (error) {
                            console.error('Failed to load directory contents:', error);
                            node.children = [];
                        }
                    }
                }
                else {
                    this.expandedPaths.delete(node.path);
                }
                this.updateVisibleNodes();
                this.render();
            }
            else {
                // Open file in editor
                console.log('File selected:', node.path);
                this.selectedPath = node.path;
                this.fileChangeCallbacks.forEach(cb => cb(node.path));
                this.render();
            }
        });
    }
    onFileSelect(callback) {
        this.fileChangeCallbacks.push(callback);
    }
    refresh() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.loadFileTree();
            this.render();
        });
    }
    setFileTree(tree) {
        this.fileTree = tree;
        this.updateVisibleNodes();
        this.render();
    }
    createFile(fileName) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // For now, just log the creation - implement actual file creation later
                console.log('Creating file:', fileName);
                // TODO: Call file system API to create file
                // await window.fileAPI.writeFile(path.join(currentPath, fileName), '');
                // this.refresh();
            }
            catch (error) {
                console.error('Failed to create file:', error);
            }
        });
    }
    createFolder(folderName) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // For now, just log the creation - implement actual folder creation later
                console.log('Creating folder:', folderName);
                // TODO: Call file system API to create folder
                // await window.fileAPI.createDirectory(path.join(currentPath, folderName));
                // this.refresh();
            }
            catch (error) {
                console.error('Failed to create folder:', error);
            }
        });
    }
    render() {
        if (this.virtualScroller) {
            console.log('Rendering virtual scroller with', this.visibleNodes.length, 'visible nodes');
            this.virtualScroller.setItems(this.visibleNodes);
        }
        else {
            console.error('Virtual scroller not initialized');
        }
    }
    destroy() {
        if (this.virtualScroller) {
            this.virtualScroller.destroy();
        }
    }
}
exports.FileExplorer = FileExplorer;
/**
 * Virtual Scroller for handling large file trees efficiently
 */
class VirtualScroller {
    constructor(container, options) {
        this.items = [];
        this.visibleStart = 0;
        this.visibleEnd = 0;
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
    setItems(items) {
        this.items = items;
        this.contentElement.style.height = `${items.length * this.itemHeight}px`;
        this.updateVisibleItems();
    }
    handleScroll() {
        requestAnimationFrame(() => this.updateVisibleItems());
    }
    updateVisibleItems() {
        const scrollTop = this.scrollElement.scrollTop;
        const containerHeight = this.scrollElement.clientHeight;
        const newVisibleStart = Math.max(0, Math.floor(scrollTop / this.itemHeight) - this.buffer);
        const newVisibleEnd = Math.min(this.items.length, Math.ceil((scrollTop + containerHeight) / this.itemHeight) + this.buffer);
        if (newVisibleStart !== this.visibleStart || newVisibleEnd !== this.visibleEnd) {
            this.visibleStart = newVisibleStart;
            this.visibleEnd = newVisibleEnd;
            this.renderVisibleItems();
        }
    }
    renderVisibleItems() {
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
//# sourceMappingURL=file-explorer.js.map