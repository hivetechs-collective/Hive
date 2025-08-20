"use strict";
/**
 * Exact VS Code File Explorer Implementation
 * Matches VS Code's precise HTML structure and CSS classes
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
exports.VSCodeExplorerExact = void 0;
const file_icons_1 = require("./file-icons");
const git_decoration_provider_1 = require("./git-decoration-provider");
// VS Code's TreeItemCollapsibleState
var TreeItemCollapsibleState;
(function (TreeItemCollapsibleState) {
    TreeItemCollapsibleState[TreeItemCollapsibleState["None"] = 0] = "None";
    TreeItemCollapsibleState[TreeItemCollapsibleState["Collapsed"] = 1] = "Collapsed";
    TreeItemCollapsibleState[TreeItemCollapsibleState["Expanded"] = 2] = "Expanded";
})(TreeItemCollapsibleState || (TreeItemCollapsibleState = {}));
class VSCodeExplorerExact {
    constructor(container) {
        this.treeData = new Map();
        this.expandedNodes = new Set();
        this.selectedNode = null;
        this.onFileSelectCallback = null;
        this.rootNodes = [];
        this.draggedNode = null;
        this.dropTarget = null;
        this.gitDecorationProvider = null;
        this.rootPath = '';
        this.container = container;
        // Don't initialize automatically - wait for initialize() to be called with a path
    }
    // Public method to initialize/reinitialize with a specific path
    initialize(rootPath) {
        return __awaiter(this, void 0, void 0, function* () {
            if (rootPath) {
                this.rootPath = rootPath;
                console.log('[VSCodeExplorer] Setting root path to:', rootPath);
            }
            // Setup container and styles if not already done
            if (!this.container.querySelector('.explorer-folders-view')) {
                this.setupContainer();
                this.attachStyles();
            }
            // If no root path set, show welcome message
            if (!this.rootPath) {
                console.log('[VSCodeExplorer] No root path set, showing welcome message');
                this.showWelcomeMessage();
                return;
            }
            // Re-initialize Git decorations with new path
            yield this.initializeGitDecorations();
            // Reload the directory tree
            yield this.loadRootDirectory();
            this.render();
        });
    }
    initializeInternal() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeExplorer] Initializing exact VS Code implementation...');
            this.setupContainer();
            this.attachStyles();
            // Initialize Git decoration provider
            yield this.initializeGitDecorations();
            yield this.loadRootDirectory();
            this.render();
        });
    }
    initializeGitDecorations() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                console.log('[VSCodeExplorer] Initializing Git decorations...');
                this.gitDecorationProvider = new git_decoration_provider_1.GitDecorationProvider(this.rootPath);
                // Listen for decoration changes
                this.gitDecorationProvider.on('decorationsChanged', (changedPaths) => {
                    console.log('[VSCodeExplorer] Git decorations changed, re-rendering...');
                    this.render(); // Re-render when Git status changes
                });
                // Initialize the provider
                yield this.gitDecorationProvider.initialize();
                console.log('[VSCodeExplorer] Git decorations initialized');
            }
            catch (error) {
                console.error('[VSCodeExplorer] Failed to initialize Git decorations:', error);
            }
        });
    }
    setupContainer() {
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
    showWelcomeMessage() {
        const rowsContainer = this.container.querySelector('.monaco-list-rows');
        if (!rowsContainer)
            return;
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
    attachStyles() {
        if (document.getElementById('vscode-exact-styles'))
            return;
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
    loadRootDirectory() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                console.log('[VSCodeExplorer] Loading root directory:', this.rootPath);
                const rootItems = yield window.fileAPI.getFileTree(this.rootPath);
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
            }
            catch (error) {
                console.error('[VSCodeExplorer] Failed to load root directory:', error);
            }
        });
    }
    createTreeNode(item, depth, parent) {
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
    render() {
        return __awaiter(this, void 0, void 0, function* () {
            const rowsContainer = this.container.querySelector('.monaco-list-rows');
            if (!rowsContainer) {
                console.error('[VSCodeExplorer] Rows container not found');
                return;
            }
            console.log('[VSCodeExplorer] Rendering tree...');
            // Get flattened visible nodes
            const flatNodes = yield this.getFlattenedNodes();
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
        });
    }
    createTreeElement(node) {
        var _a;
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
                    case 'M':
                        status = 'modified';
                        break;
                    case 'A':
                        status = 'added';
                        break;
                    case 'D':
                        status = 'deleted';
                        break;
                    case 'U':
                        status = 'untracked';
                        break;
                    case 'R':
                        status = 'renamed';
                        break;
                }
                if (status) {
                    row.setAttribute('data-git-status', status);
                }
            }
        }
        // Make draggable
        row.draggable = true;
        if (((_a = this.selectedNode) === null || _a === void 0 ? void 0 : _a.path) === node.path) {
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
            const folderIcon = (0, file_icons_1.getFolderIcon)(node.name, node.collapsibleState === TreeItemCollapsibleState.Expanded);
            icon.className = `codicon codicon-${folderIcon.icon}`;
            icon.style.cssText = `color: ${folderIcon.color || '#dcb67a'};`;
        }
        else {
            const fileIcon = (0, file_icons_1.getFileIcon)(node.name);
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
        let gitIndicator = null;
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
                    }
                    else if (decoration.badge === 'A') {
                        gitIndicator.classList.add('added');
                    }
                    else if (decoration.badge === 'D') {
                        gitIndicator.classList.add('deleted');
                    }
                    else if (decoration.badge === 'U') {
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
    getFileIconClass(filename) {
        const iconMapping = (0, file_icons_1.getFileIcon)(filename);
        return `codicon-${iconMapping.icon}`;
    }
    getFileIconStyle(filename) {
        const iconMapping = (0, file_icons_1.getFileIcon)(filename);
        return `color: ${iconMapping.color || '#969696'};`;
    }
    getFlattenedNodes() {
        return __awaiter(this, void 0, void 0, function* () {
            const flat = [];
            const processNode = (node) => __awaiter(this, void 0, void 0, function* () {
                flat.push(node);
                // If expanded and directory, include children
                if (node.type === 'directory' &&
                    node.collapsibleState === TreeItemCollapsibleState.Expanded) {
                    if (!node.children) {
                        // Load children if not loaded
                        node.children = yield this.loadDirectoryChildren(node);
                    }
                    for (const child of node.children || []) {
                        yield processNode(child);
                    }
                }
            });
            // Process all root nodes
            for (const node of this.rootNodes) {
                yield processNode(node);
            }
            return flat;
        });
    }
    loadDirectoryChildren(directory) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                console.log('[VSCodeExplorer] Loading children for:', directory.path);
                const children = yield window.fileAPI.getDirectoryContents(directory.path);
                if (!children || children.length === 0) {
                    return [];
                }
                // Convert to TreeNodes
                const treeNodes = children.map(child => this.createTreeNode(child, directory.depth + 1, directory));
                // Store in treeData map
                treeNodes.forEach(node => {
                    this.treeData.set(node.path, node);
                });
                return treeNodes;
            }
            catch (error) {
                console.error('[VSCodeExplorer] Failed to load directory children:', error);
                return [];
            }
        });
    }
    attachEventListeners() {
        var _a;
        const listElement = this.container.querySelector('.monaco-list');
        if (!listElement)
            return;
        // Remove existing listeners
        const newList = listElement.cloneNode(true);
        (_a = listElement.parentNode) === null || _a === void 0 ? void 0 : _a.replaceChild(newList, listElement);
        // Use event delegation with proper error handling
        const handleClick = (e) => {
            try {
                e.preventDefault();
                e.stopPropagation();
                const target = e.target;
                const row = target.closest('.monaco-list-row');
                if (!row)
                    return;
                const path = row.getAttribute('data-path');
                if (!path)
                    return;
                const node = this.treeData.get(path);
                if (!node)
                    return;
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
                            var _a;
                            try {
                                console.log('[VSCodeExplorer] About to call onFileSelectCallback');
                                this.onFileSelectCallback(node.path);
                                console.log('[VSCodeExplorer] onFileSelectCallback completed');
                            }
                            catch (err) {
                                console.error('[VSCodeExplorer] Error in file select callback:', err);
                                console.error('[VSCodeExplorer] Error type:', (_a = err === null || err === void 0 ? void 0 : err.constructor) === null || _a === void 0 ? void 0 : _a.name);
                                console.error('[VSCodeExplorer] Is it an Event?', err instanceof Event);
                            }
                        }, 0);
                    }
                }
                else {
                    // Directory clicked - toggle expansion
                    this.toggleExpanded(node).catch(err => {
                        console.error('[VSCodeExplorer] Error toggling directory:', err);
                    });
                }
                // Update selection
                this.updateSelection();
            }
            catch (error) {
                console.error('[VSCodeExplorer] Error in click handler:', error);
                // Don't re-throw to prevent webpack overlay from catching it
            }
        };
        newList.addEventListener('click', handleClick);
        // Add drag and drop event delegation
        this.setupDragAndDrop(newList);
    }
    setupDragAndDrop(container) {
        // Drag start
        container.addEventListener('dragstart', (e) => {
            const row = e.target.closest('.monaco-list-row');
            if (!row)
                return;
            const path = row.getAttribute('data-path');
            if (!path)
                return;
            const node = this.treeData.get(path);
            if (!node)
                return;
            this.handleDragStart(e, node);
        });
        // Drag over
        container.addEventListener('dragover', (e) => {
            e.preventDefault(); // Required to allow drop
            const row = e.target.closest('.monaco-list-row');
            if (!row)
                return;
            const path = row.getAttribute('data-path');
            if (!path)
                return;
            const node = this.treeData.get(path);
            if (!node)
                return;
            this.handleDragOver(e, node);
        });
        // Drag enter
        container.addEventListener('dragenter', (e) => {
            const row = e.target.closest('.monaco-list-row');
            if (!row)
                return;
            const path = row.getAttribute('data-path');
            if (!path)
                return;
            const node = this.treeData.get(path);
            if (!node)
                return;
            this.handleDragEnter(e, node);
        });
        // Drag leave
        container.addEventListener('dragleave', (e) => {
            this.handleDragLeave(e);
        });
        // Drop
        container.addEventListener('drop', (e) => {
            e.preventDefault();
            e.stopPropagation();
            const row = e.target.closest('.monaco-list-row');
            if (!row)
                return;
            const path = row.getAttribute('data-path');
            if (!path)
                return;
            const node = this.treeData.get(path);
            if (!node)
                return;
            this.handleDrop(e, node);
        });
        // Drag end
        container.addEventListener('dragend', (e) => {
            this.handleDragEnd(e);
        });
    }
    toggleExpanded(node) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeExplorer] Toggling:', node.name);
            if (node.collapsibleState === TreeItemCollapsibleState.Collapsed) {
                node.collapsibleState = TreeItemCollapsibleState.Expanded;
                this.expandedNodes.add(node.path);
            }
            else {
                node.collapsibleState = TreeItemCollapsibleState.Collapsed;
                this.expandedNodes.delete(node.path);
            }
            yield this.render();
        });
    }
    updateSelection() {
        const rows = this.container.querySelectorAll('.monaco-list-row');
        rows.forEach(row => {
            var _a;
            const path = row.getAttribute('data-path');
            if (path === ((_a = this.selectedNode) === null || _a === void 0 ? void 0 : _a.path)) {
                row.classList.add('selected', 'focused');
            }
            else {
                row.classList.remove('selected', 'focused');
            }
        });
    }
    // Public API
    onFileSelect(callback) {
        this.onFileSelectCallback = callback;
    }
    refresh() {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeExplorer] Refreshing...');
            this.treeData.clear();
            this.expandedNodes.clear();
            // Refresh Git decorations
            if (this.gitDecorationProvider) {
                yield this.gitDecorationProvider.refreshStatus();
            }
            yield this.loadRootDirectory();
            yield this.render();
        });
    }
    collapseAll() {
        return __awaiter(this, void 0, void 0, function* () {
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
            yield this.render();
        });
    }
    createFile(fileName) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeExplorer] Create file:', fileName);
            // Get the current directory (use selected directory or root)
            let targetDir = this.rootPath;
            if (this.selectedNode && this.selectedNode.type === 'directory') {
                targetDir = this.selectedNode.path;
            }
            else if (this.selectedNode && this.selectedNode.parent) {
                targetDir = this.selectedNode.parent.path;
            }
            console.log('[VSCodeExplorer] Target directory:', targetDir);
            console.log('[VSCodeExplorer] Full path will be:', targetDir + '/' + fileName);
            try {
                // Create the file through IPC
                const result = yield window.fileAPI.createFile(targetDir, fileName);
                console.log('[VSCodeExplorer] IPC result:', result);
                console.log('[VSCodeExplorer] File created successfully, refreshing...');
                // Give Git a moment to detect the new file
                setTimeout(() => __awaiter(this, void 0, void 0, function* () {
                    yield this.refresh();
                    console.log('[VSCodeExplorer] Refresh completed');
                }), 500);
            }
            catch (error) {
                console.error('[VSCodeExplorer] Failed to create file:', error);
                alert('Failed to create file: ' + error.message);
            }
        });
    }
    createFolder(folderName) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeExplorer] Create folder:', folderName);
            // Get the current directory (use selected directory or root)
            let targetDir = this.rootPath;
            if (this.selectedNode && this.selectedNode.type === 'directory') {
                targetDir = this.selectedNode.path;
            }
            else if (this.selectedNode && this.selectedNode.parent) {
                targetDir = this.selectedNode.parent.path;
            }
            console.log('[VSCodeExplorer] Target directory:', targetDir);
            console.log('[VSCodeExplorer] Full path will be:', targetDir + '/' + folderName);
            try {
                // Create the folder through IPC
                const result = yield window.fileAPI.createFolder(targetDir, folderName);
                console.log('[VSCodeExplorer] IPC result:', result);
                console.log('[VSCodeExplorer] Folder created successfully, refreshing...');
                // Give Git a moment to detect the new folder
                setTimeout(() => __awaiter(this, void 0, void 0, function* () {
                    yield this.refresh();
                    console.log('[VSCodeExplorer] Refresh completed');
                }), 500);
            }
            catch (error) {
                console.error('[VSCodeExplorer] Failed to create folder:', error);
                alert('Failed to create folder: ' + error.message);
            }
        });
    }
    destroy() {
        // Clean up Git decoration provider
        if (this.gitDecorationProvider) {
            this.gitDecorationProvider.dispose();
            this.gitDecorationProvider = null;
        }
        this.container.innerHTML = '';
    }
    // Drag and Drop Handlers
    handleDragStart(e, node) {
        console.log('[VSCodeExplorer] Drag start:', node.name);
        this.draggedNode = node;
        // Set drag effect
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = 'move';
            e.dataTransfer.setData('text/plain', node.path);
        }
        // Add dragging class to the row element
        const row = e.target.closest('.monaco-list-row');
        if (row) {
            row.classList.add('dragging');
            row.style.opacity = '0.5';
        }
    }
    handleDragOver(e, node) {
        e.preventDefault(); // Allow drop
        if (!this.draggedNode)
            return;
        if (this.draggedNode === node)
            return; // Can't drop on itself
        // Only allow dropping on directories
        if (node.type === 'directory') {
            if (e.dataTransfer) {
                e.dataTransfer.dropEffect = 'move';
            }
        }
    }
    handleDragEnter(e, node) {
        if (!this.draggedNode)
            return;
        if (this.draggedNode === node)
            return; // Can't drop on itself
        const row = e.target.closest('.monaco-list-row');
        if (!row)
            return;
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
    handleDragLeave(e) {
        // Only clear if we're leaving the row entirely
        const relatedTarget = e.relatedTarget;
        const row = e.target.closest('.monaco-list-row');
        if (row && relatedTarget && !row.contains(relatedTarget)) {
            row.classList.remove('drop-target');
            row.style.background = '';
            row.style.border = '';
        }
    }
    handleDrop(e, targetNode) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            e.preventDefault();
            e.stopPropagation();
            console.log('[VSCodeExplorer] Drop on:', targetNode.name);
            console.log('[VSCodeExplorer] Dragged node:', (_a = this.draggedNode) === null || _a === void 0 ? void 0 : _a.name);
            if (!this.draggedNode) {
                console.log('[VSCodeExplorer] No dragged node, aborting drop');
                return;
            }
            if (this.draggedNode === targetNode) {
                console.log('[VSCodeExplorer] Cannot drop on itself');
                return;
            }
            // Clear drop target styles
            const row = e.target.closest('.monaco-list-row');
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
                    yield this.moveItem(this.draggedNode.path, targetNode.path);
                    console.log('[VSCodeExplorer] Move completed, refreshing tree...');
                    // Refresh the tree
                    yield this.refresh();
                    console.log('[VSCodeExplorer] Tree refreshed');
                }
                catch (error) {
                    console.error('[VSCodeExplorer] Failed to move item:', error);
                    alert('Failed to move: ' + error.message);
                }
            }
            else {
                console.log('[VSCodeExplorer] Target is not a directory, cannot drop');
            }
            this.draggedNode = null;
        });
    }
    handleDragEnd(e) {
        console.log('[VSCodeExplorer] Drag end');
        // Clear dragging styles
        const draggingElements = this.container.querySelectorAll('.dragging');
        draggingElements.forEach(el => {
            el.classList.remove('dragging');
            el.style.opacity = '';
        });
        // Clean up any drop targets
        const dropTargets = this.container.querySelectorAll('.drop-target');
        dropTargets.forEach(target => {
            target.classList.remove('drop-target');
            target.style.background = '';
            target.style.border = '';
        });
        this.draggedNode = null;
        this.dropTarget = null;
    }
    moveItem(sourcePath, targetDir) {
        return __awaiter(this, void 0, void 0, function* () {
            console.log('[VSCodeExplorer] Moving', sourcePath, 'to', targetDir);
            // Extract the item name from the source path
            const itemName = sourcePath.split('/').pop();
            if (!itemName)
                throw new Error('Invalid source path');
            const newPath = targetDir + '/' + itemName;
            // Check if target already exists
            if (yield window.fileAPI.fileExists(newPath)) {
                throw new Error(`Item '${itemName}' already exists in the target directory`);
            }
            // Move the item using file system operations
            yield window.fileAPI.moveFile(sourcePath, newPath);
            console.log('[VSCodeExplorer] Item moved successfully');
        });
    }
}
exports.VSCodeExplorerExact = VSCodeExplorerExact;
//# sourceMappingURL=vscode-explorer-exact.js.map