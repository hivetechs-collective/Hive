"use strict";
/**
 * Simple, bulletproof file explorer without any fancy features
 * Just works without throwing Event objects
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
exports.SimpleFileExplorer = void 0;
class SimpleFileExplorer {
    constructor(container) {
        this.onFileSelect = null;
        this.nodes = new Map();
        this.container = container;
        this.init();
    }
    init() {
        return __awaiter(this, void 0, void 0, function* () {
            this.container.innerHTML = '<div class="simple-explorer">Loading...</div>';
            try {
                const files = yield window.fileAPI.getFileTree();
                this.renderTree(files);
            }
            catch (error) {
                this.container.innerHTML = '<div class="simple-explorer">Failed to load files</div>';
            }
        });
    }
    renderTree(files) {
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
            if (node)
                explorer.appendChild(node);
        });
        this.container.innerHTML = '';
        this.container.appendChild(explorer);
    }
    createNode(item, level) {
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
        const node = {
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
                }
                catch (e) {
                    console.log('File select failed:', e);
                }
            }
            else if (item.type === 'directory') {
                this.toggleDirectory(div, item, level);
            }
        };
        return div;
    }
    toggleDirectory(div, item, level) {
        var _a, _b;
        return __awaiter(this, void 0, void 0, function* () {
            const node = this.nodes.get(item.path);
            if (!node)
                return;
            if (node.expanded) {
                // Collapse
                node.expanded = false;
                let nextSibling = div.nextSibling;
                while (nextSibling && ((_a = nextSibling.dataset) === null || _a === void 0 ? void 0 : _a.parent) === item.path) {
                    const toRemove = nextSibling;
                    nextSibling = nextSibling.nextSibling;
                    (_b = toRemove.parentNode) === null || _b === void 0 ? void 0 : _b.removeChild(toRemove);
                }
                div.textContent = '📁 ' + item.name;
            }
            else {
                // Expand
                node.expanded = true;
                div.textContent = '📂 ' + item.name;
                try {
                    const children = yield window.fileAPI.getDirectoryContents(item.path);
                    children.forEach(child => {
                        var _a;
                        const childDiv = this.createNode(child, level + 1);
                        if (childDiv) {
                            childDiv.dataset.parent = item.path;
                            (_a = div.parentNode) === null || _a === void 0 ? void 0 : _a.insertBefore(childDiv, div.nextSibling);
                        }
                    });
                }
                catch (e) {
                    console.log('Failed to load directory:', e);
                }
            }
        });
    }
    onFileSelected(callback) {
        this.onFileSelect = callback;
    }
    refresh() {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.init();
        });
    }
    createFile(name) {
        return __awaiter(this, void 0, void 0, function* () {
            // Not implemented
        });
    }
    createFolder(name) {
        return __awaiter(this, void 0, void 0, function* () {
            // Not implemented
        });
    }
    collapseAll() {
        return __awaiter(this, void 0, void 0, function* () {
            this.nodes.forEach(node => node.expanded = false);
            yield this.init();
        });
    }
}
exports.SimpleFileExplorer = SimpleFileExplorer;
//# sourceMappingURL=simple-file-explorer.js.map