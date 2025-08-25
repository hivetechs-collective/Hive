"use strict";
/**
 * TTYDTerminalPanel - Terminal panel using ttyd server and webviews
 * Provides real terminal emulation with perfect TUI support
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
exports.ttydTerminalPanel = exports.TTYDTerminalPanel = void 0;
class TTYDTerminalPanel {
    constructor(container) {
        this.tabs = new Map();
        this.activeTabId = null;
        this.terminalCounter = 1;
        this.tabScrollOffset = 0;
        this.container = container;
        this.tabsContainer = document.getElementById('isolated-terminal-tabs');
        this.contentContainer = document.getElementById('isolated-terminal-content');
        // Ensure content container has proper dimensions to prevent 139x9 issue
        this.contentContainer.style.position = 'relative';
        this.contentContainer.style.width = '100%';
        this.contentContainer.style.height = 'calc(100% - 35px)'; // Account for tab bar
        this.contentContainer.style.minHeight = '400px'; // Prevent tiny dimensions
        this.contentContainer.style.flex = '1 1 auto';
        this.initialize();
        this.setupResizeObserver();
    }
    initialize() {
        // Create System Log as the first tab
        this.createSystemLogTab();
        // Set up new tab button
        const newTabBtn = document.getElementById('isolated-terminal-new-tab');
        if (newTabBtn) {
            console.log('[TTYDTerminalPanel] Found new tab button, adding listener');
            newTabBtn.addEventListener('click', () => {
                console.log('[TTYDTerminalPanel] New tab button clicked');
                this.createTerminalTab();
            });
        }
        else {
            console.error('[TTYDTerminalPanel] New tab button not found!');
        }
        // Set up System Log toggle button
        const systemLogToggle = document.getElementById('isolated-terminal-system-log-toggle');
        if (systemLogToggle) {
            console.log('[TTYDTerminalPanel] Found System Log toggle button, adding listener');
            systemLogToggle.addEventListener('click', () => {
                console.log('[TTYDTerminalPanel] System Log toggle button clicked');
                this.toggleSystemLog();
            });
            // Initialize button appearance - keep it bright but indicate state with slight opacity
            systemLogToggle.style.opacity = '0.7'; // More visible even when System Log is hidden
        }
        else {
            console.error('[TTYDTerminalPanel] System Log toggle button not found!');
        }
        // Listen for terminal creation events from main process
        this.setupIpcListeners();
        // Set up tab navigation arrows
        this.setupTabNavigation();
        // Set up keyboard shortcuts
        this.setupKeyboardShortcuts();
    }
    setupIpcListeners() {
        // Listen for terminal creation from main process
        window.terminalAPI.onTerminalCreated((terminalInfo) => {
            console.log('[TTYDTerminalPanel] Terminal created event:', terminalInfo);
            // Check if this tab already exists to avoid duplicates
            if (!this.tabs.has(terminalInfo.id)) {
                // This is from an external source (like AI CLI tools) - add the tab
                console.log('[TTYDTerminalPanel] Adding tab for external terminal:', terminalInfo.id);
                this.addTerminalTab(terminalInfo);
            }
            else {
                console.log('[TTYDTerminalPanel] Tab already exists for:', terminalInfo.id);
            }
        });
        // Listen for terminal ready events
        window.terminalAPI.onTerminalReady((terminalId, url) => {
            console.log('[TTYDTerminalPanel] Terminal ready:', terminalId, url);
            const tab = this.tabs.get(terminalId);
            if (tab && tab.webview) {
                // Update the iframe src to connect to ttyd
                tab.webview.src = url;
            }
        });
        // Listen for terminal close events
        window.terminalAPI.onTerminalExit((terminalId) => {
            console.log('[TTYDTerminalPanel] Terminal exited:', terminalId);
            this.removeTab(terminalId);
        });
    }
    createSystemLogTab() {
        const tab = {
            id: 'system-log',
            title: '📊 System Log',
            type: 'system-log',
            isActive: false // System Log should NOT be active by default
        };
        // Create tab button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'ttyd-tab-btn'; // Remove 'active' class
        tabBtn.id = `tab-btn-${tab.id}`; // Add ID for System Log tab too
        tabBtn.innerHTML = `
            <span>${tab.title}</span>
        `;
        tabBtn.style.display = 'none'; // Hide System Log tab by default
        tabBtn.addEventListener('click', () => this.switchToTab(tab.id));
        this.tabsContainer.appendChild(tabBtn);
        // Create content area for system log
        const content = document.createElement('div');
        content.id = `isolated-content-${tab.id}`;
        content.className = 'isolated-tab-content'; // Remove 'active' class
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: #1e1e1e;
            display: none;  // Hide by default since it's not active
            overflow-y: auto;
            padding: 10px;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 12px;
            color: #cccccc;
        `;
        // Add initial message
        content.innerHTML = `<div style="color: #569cd6;">[System Log initialized]</div>`;
        this.contentContainer.appendChild(content);
        tab.element = content;
        this.tabs.set(tab.id, tab);
        // Don't set activeTabId here - let the first terminal or AI tool take focus
        // Set up console capture for system log
        this.setupConsoleCapture(content);
    }
    setupConsoleCapture(logElement) {
        const originalLog = console.log;
        const originalError = console.error;
        const originalWarn = console.warn;
        const addLogEntry = (message, type) => {
            const entry = document.createElement('div');
            const timestamp = new Date().toLocaleTimeString();
            let color = '#cccccc';
            let prefix = 'INFO';
            if (type === 'error') {
                color = '#f44747';
                prefix = 'ERROR';
            }
            else if (type === 'warn') {
                color = '#dcdcaa';
                prefix = 'WARN';
            }
            entry.style.color = color;
            entry.style.marginBottom = '2px';
            entry.textContent = `[${timestamp}] [${prefix}] ${message}`;
            logElement.appendChild(entry);
            // Auto-scroll to bottom - use requestAnimationFrame for smooth scrolling
            requestAnimationFrame(() => {
                if (logElement && logElement.scrollHeight) {
                    logElement.scrollTop = logElement.scrollHeight;
                    // Force a reflow to ensure scroll happens
                    logElement.scrollIntoView({ behavior: 'auto', block: 'end' });
                }
            });
            // Limit log entries to prevent memory issues
            while (logElement.children.length > 1000) {
                logElement.removeChild(logElement.firstChild);
            }
        };
        console.log = function (...args) {
            originalLog.apply(console, args);
            addLogEntry(args.join(' '), 'log');
        };
        console.error = function (...args) {
            originalError.apply(console, args);
            addLogEntry(args.join(' '), 'error');
        };
        console.warn = function (...args) {
            originalWarn.apply(console, args);
            addLogEntry(args.join(' '), 'warn');
        };
    }
    createTerminalTab(toolId, command, env) {
        return __awaiter(this, void 0, void 0, function* () {
            const terminalId = `terminal-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
            console.log('[TTYDTerminalPanel] createTerminalTab called with toolId:', toolId, 'command:', command, 'env:', env);
            // Mark that we're creating this terminal internally to avoid duplicate tabs
            this.tabs.set(terminalId, {
                id: terminalId,
                title: 'Creating...',
                type: 'terminal',
                isActive: false
            });
            try {
                console.log('[TTYDTerminalPanel] Calling window.terminalAPI.createTerminalProcess...');
                // Request terminal creation from main process
                const result = yield window.terminalAPI.createTerminalProcess({
                    terminalId,
                    toolId,
                    command,
                    cwd: window.currentOpenedFolder || undefined,
                    env: env // Pass environment variables if provided
                });
                console.log('[TTYDTerminalPanel] createTerminalProcess result:', result);
                if (result.success && result.terminal) {
                    console.log('[TTYDTerminalPanel] Terminal created successfully:', result.terminal);
                    // Remove the placeholder
                    this.tabs.delete(terminalId);
                    // Add the real terminal tab
                    this.addTerminalTab(result.terminal);
                }
                else {
                    console.error('[TTYDTerminalPanel] Failed to create terminal:', result.error);
                    // Remove the placeholder on failure
                    this.tabs.delete(terminalId);
                }
            }
            catch (error) {
                console.error('[TTYDTerminalPanel] Error creating terminal:', error);
                // Remove the placeholder on error
                this.tabs.delete(terminalId);
            }
        });
    }
    addTerminalTab(terminalInfo) {
        // Ensure tabs container exists
        if (!this.tabsContainer) {
            this.tabsContainer = document.getElementById('isolated-terminal-tabs');
            if (!this.tabsContainer) {
                console.error('[TTYDTerminalPanel] Tabs container not found! Attempting to recreate...');
                // Try to find and recreate if missing
                const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper');
                if (wrapper) {
                    // Check if it already has the tabs container as a child
                    let existingTabs = wrapper.querySelector('#isolated-terminal-tabs');
                    if (!existingTabs) {
                        this.tabsContainer = document.createElement('div');
                        this.tabsContainer.id = 'isolated-terminal-tabs';
                        this.tabsContainer.className = 'isolated-terminal-tabs';
                        this.tabsContainer.style.cssText = 'display: flex; align-items: center; transition: transform 0.3s ease; white-space: nowrap;';
                        wrapper.appendChild(this.tabsContainer);
                        console.log('[TTYDTerminalPanel] Recreated tabs container');
                    }
                    else {
                        this.tabsContainer = existingTabs;
                        console.log('[TTYDTerminalPanel] Found existing tabs container');
                    }
                }
                else {
                    console.error('[TTYDTerminalPanel] Cannot create tabs container - wrapper not found');
                    return;
                }
            }
        }
        const tab = {
            id: terminalInfo.id,
            title: terminalInfo.title || `Terminal ${this.terminalCounter++}`,
            type: terminalInfo.toolId ? 'ai-tool' : 'terminal',
            isActive: false,
            toolId: terminalInfo.toolId,
            url: terminalInfo.url,
            port: terminalInfo.port
        };
        // Add tab to our internal map
        this.tabs.set(tab.id, tab);
        // Create tab button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'ttyd-tab-btn';
        tabBtn.id = `tab-btn-${tab.id}`;
        tabBtn.innerHTML = `
            <span>${tab.title}</span>
            <span class="tab-close" style="margin-left: 8px; cursor: pointer;">×</span>
        `;
        // Add click handler for tab switching
        tabBtn.addEventListener('click', (e) => {
            if (!e.target.classList.contains('tab-close')) {
                this.switchToTab(tab.id);
            }
        });
        // Add click handler for close button
        const closeBtn = tabBtn.querySelector('.tab-close');
        if (closeBtn) {
            closeBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                this.closeTab(tab.id);
            });
        }
        // Append to the tabs container (which is separate from the new tab button)
        this.tabsContainer.appendChild(tabBtn);
        console.log('[TTYDTerminalPanel] Tab button added to container:', tab.id, 'Container children:', this.tabsContainer.children.length);
        // Create content area with iframe for ttyd
        const content = document.createElement('div');
        content.id = `isolated-content-${tab.id}`;
        content.className = 'isolated-tab-content';
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: #1e1e1e;
            display: none;
            width: 100%;
            height: 100%;
        `;
        // Create webview to embed ttyd terminal
        // Using webview instead of iframe for better Electron integration
        const webview = document.createElement('webview');
        // CRITICAL: Always delay loading to ensure container has proper size
        // This prevents the 9-row issue by ensuring ttyd never sees a small container
        webview.dataset.originalSrc = terminalInfo.url;
        webview.src = 'about:blank';
        // Load the webview immediately - flex layout ensures proper size
        console.log('[TTYDTerminalPanel] Loading ttyd webview');
        webview.src = terminalInfo.url;
        webview.style.cssText = `
            width: 100%;
            height: 100%;
            border: none;
            background: #1e1e1e;
        `;
        // Enable node integration for local content (safe for localhost)
        webview.setAttribute('nodeintegration', 'true');
        webview.setAttribute('disablewebsecurity', 'true');
        webview.setAttribute('allowpopups', 'true');
        content.appendChild(webview);
        this.contentContainer.appendChild(content);
        tab.element = content;
        tab.webview = webview;
        this.tabs.set(tab.id, tab);
        // Handle webview ready event to ensure proper sizing
        webview.addEventListener('dom-ready', () => {
            console.log('[TTYDTerminalPanel] Webview DOM ready, ensuring proper size');
            // Ensure the webview and its container have proper dimensions
            const panel = document.querySelector('.isolated-terminal-panel');
            const contentHeight = this.contentContainer.offsetHeight;
            const contentWidth = this.contentContainer.offsetWidth;
            console.log(`[TTYDTerminalPanel] Container dimensions: ${contentWidth}x${contentHeight}`);
            // Calculate proper terminal rows and columns based on actual dimensions
            const charHeight = 17; // Approximate character height in pixels
            const charWidth = 9; // Approximate character width in pixels
            const rows = Math.floor(contentHeight / charHeight);
            const cols = Math.floor(contentWidth / charWidth);
            console.log(`[TTYDTerminalPanel] Calculated terminal size: ${cols}x${rows} (from ${contentWidth}x${contentHeight}px)`);
            // Always force a reload after a short delay to ensure ttyd gets proper dimensions
            // This fixes the 139x9 issue where height is stuck at 9 rows
            setTimeout(() => {
                // First, try to send a resize signal through the webview
                try {
                    // Send window resize event to force ttyd to recalculate
                    webview.executeJavaScript(`
                        window.dispatchEvent(new Event('resize'));
                        if (window.term) {
                            window.term.fit();
                        }
                    `).catch(() => {
                        // If executeJavaScript fails (security), just reload
                        console.log('[TTYDTerminalPanel] Cannot execute JS, reloading webview');
                    });
                }
                catch (e) {
                    console.log('[TTYDTerminalPanel] executeJavaScript not available');
                }
                // If dimensions are still wrong, force a full reload
                if (rows < 20 || contentHeight < 400) {
                    console.log('[TTYDTerminalPanel] Height still too small, forcing reload');
                    const currentSrc = webview.src;
                    webview.src = '';
                    setTimeout(() => {
                        webview.src = currentSrc;
                    }, 100);
                }
            }, 500);
        });
        // Handle webview load errors
        webview.addEventListener('did-fail-load', (event) => {
            console.error('[TTYDTerminalPanel] Webview failed to load:', event);
        });
        // Switch to the new tab
        this.switchToTab(tab.id);
        // Update navigation arrows after adding tab
        setTimeout(() => this.updateNavigationArrows(), 50);
    }
    switchToTab(tabId) {
        const tab = this.tabs.get(tabId);
        if (!tab)
            return;
        // Check if panel is collapsed
        const panel = document.querySelector('.isolated-terminal-panel');
        const isCollapsed = panel === null || panel === void 0 ? void 0 : panel.classList.contains('collapsed');
        // Deactivate current tab
        if (this.activeTabId) {
            const currentTab = this.tabs.get(this.activeTabId);
            if (currentTab) {
                currentTab.isActive = false;
                const currentTabBtn = document.getElementById(`tab-btn-${this.activeTabId}`);
                if (currentTabBtn) {
                    currentTabBtn.classList.remove('active');
                }
                if (currentTab.element) {
                    currentTab.element.style.display = 'none';
                }
            }
        }
        // Activate new tab
        tab.isActive = true;
        const tabBtn = document.getElementById(`tab-btn-${tabId}`);
        if (tabBtn) {
            tabBtn.classList.add('active');
        }
        // Only show element if panel is not collapsed (prevents ttyd from seeing small container)
        if (tab.element && !isCollapsed) {
            tab.element.style.display = 'block';
        }
        this.activeTabId = tabId;
        // Auto-scroll to show the active tab
        this.scrollToTab(tabId);
    }
    scrollToTab(tabId) {
        const tabBtn = document.getElementById(`tab-btn-${tabId}`);
        const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper');
        if (!tabBtn || !wrapper)
            return;
        const tabLeft = tabBtn.offsetLeft;
        const tabRight = tabLeft + tabBtn.offsetWidth;
        const wrapperWidth = wrapper.offsetWidth;
        const currentScroll = this.tabScrollOffset;
        const visibleLeft = currentScroll;
        const visibleRight = currentScroll + wrapperWidth;
        // If tab is not fully visible, scroll to show it
        if (tabLeft < visibleLeft) {
            // Scroll left to show the tab
            this.tabScrollOffset = Math.max(0, tabLeft - 10); // 10px padding
            this.tabsContainer.style.transform = `translateX(-${this.tabScrollOffset}px)`;
            this.updateNavigationArrows();
        }
        else if (tabRight > visibleRight) {
            // Scroll right to show the tab
            this.tabScrollOffset = Math.max(0, tabRight - wrapperWidth + 10); // 10px padding
            this.tabsContainer.style.transform = `translateX(-${this.tabScrollOffset}px)`;
            this.updateNavigationArrows();
        }
    }
    closeTab(tabId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tab = this.tabs.get(tabId);
            if (!tab || tab.type === 'system-log')
                return; // Can't close system log
            // Close the terminal in the backend
            try {
                yield window.terminalAPI.killTerminalProcess(tabId);
            }
            catch (error) {
                console.error('[TTYDTerminalPanel] Error closing terminal:', error);
            }
            // Remove the tab
            this.removeTab(tabId);
        });
    }
    removeTab(tabId) {
        const tab = this.tabs.get(tabId);
        if (!tab)
            return;
        // Remove tab button
        const tabBtn = document.getElementById(`tab-btn-${tabId}`);
        if (tabBtn) {
            tabBtn.remove();
        }
        // Remove content
        if (tab.element) {
            tab.element.remove();
        }
        // Remove from map
        this.tabs.delete(tabId);
        // If this was the active tab, switch to another
        if (this.activeTabId === tabId) {
            const remainingTabs = Array.from(this.tabs.keys());
            if (remainingTabs.length > 0) {
                // Switch to the last tab (most recently created) instead of first (System Log)
                // But prefer non-system-log tabs
                const nonSystemLogTabs = remainingTabs.filter(id => id !== 'system-log');
                if (nonSystemLogTabs.length > 0) {
                    // Switch to the last non-system-log tab
                    this.switchToTab(nonSystemLogTabs[nonSystemLogTabs.length - 1]);
                }
                else {
                    // Only switch to System Log if it's the only tab left
                    this.switchToTab(remainingTabs[0]);
                }
            }
        }
        // Update navigation arrows after removing tab
        setTimeout(() => this.updateNavigationArrows(), 50);
    }
    // Public method to launch AI tool in terminal
    launchAITool(toolId, command) {
        return __awaiter(this, void 0, void 0, function* () {
            // Check if tool already has a terminal
            for (const [id, tab] of this.tabs) {
                if (tab.toolId === toolId) {
                    // Switch to existing tab
                    this.switchToTab(id);
                    return;
                }
            }
            // Create new terminal for the tool
            yield this.createTerminalTab(toolId, command);
        });
    }
    // Get current active terminal ID
    getActiveTerminalId() {
        return this.activeTabId;
    }
    // Set up tab navigation arrows
    setupTabNavigation() {
        const leftArrow = document.getElementById('tab-nav-left');
        const rightArrow = document.getElementById('tab-nav-right');
        const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper');
        if (!leftArrow || !rightArrow || !wrapper) {
            console.error('[TTYDTerminalPanel] Tab navigation elements not found');
            return;
        }
        // Add arrow hover effects
        leftArrow.addEventListener('mouseenter', () => {
            if (this.tabScrollOffset > 0) {
                leftArrow.style.color = '#cccccc';
            }
        });
        leftArrow.addEventListener('mouseleave', () => {
            leftArrow.style.color = '#969696';
        });
        rightArrow.addEventListener('mouseenter', () => {
            rightArrow.style.color = '#cccccc';
        });
        rightArrow.addEventListener('mouseleave', () => {
            rightArrow.style.color = '#969696';
        });
        // Arrow click handlers
        leftArrow.addEventListener('click', () => {
            this.scrollTabs('left');
        });
        rightArrow.addEventListener('click', () => {
            this.scrollTabs('right');
        });
        // Check if arrows are needed on resize
        // Use requestAnimationFrame to prevent ResizeObserver loop errors
        let resizeAnimationFrame = null;
        const resizeObserver = new ResizeObserver(() => {
            // Cancel any pending animation frame
            if (resizeAnimationFrame !== null) {
                cancelAnimationFrame(resizeAnimationFrame);
            }
            // Schedule update for next animation frame
            resizeAnimationFrame = requestAnimationFrame(() => {
                this.updateNavigationArrows();
                resizeAnimationFrame = null;
            });
        });
        resizeObserver.observe(wrapper);
        resizeObserver.observe(this.tabsContainer);
        // Initial check
        setTimeout(() => this.updateNavigationArrows(), 100);
        // Add keyboard shortcuts for tab navigation
        document.addEventListener('keydown', (e) => {
            // Ctrl/Cmd + Shift + Left/Right to navigate tabs
            if ((e.ctrlKey || e.metaKey) && e.shiftKey) {
                if (e.key === 'ArrowLeft') {
                    e.preventDefault();
                    this.switchToPreviousTab();
                }
                else if (e.key === 'ArrowRight') {
                    e.preventDefault();
                    this.switchToNextTab();
                }
            }
        });
    }
    switchToPreviousTab() {
        const tabIds = Array.from(this.tabs.keys());
        const currentIndex = tabIds.indexOf(this.activeTabId || '');
        if (currentIndex > 0) {
            this.switchToTab(tabIds[currentIndex - 1]);
        }
    }
    switchToNextTab() {
        const tabIds = Array.from(this.tabs.keys());
        const currentIndex = tabIds.indexOf(this.activeTabId || '');
        if (currentIndex >= 0 && currentIndex < tabIds.length - 1) {
            this.switchToTab(tabIds[currentIndex + 1]);
        }
    }
    scrollTabs(direction) {
        const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper');
        if (!wrapper)
            return;
        const wrapperWidth = wrapper.offsetWidth;
        const scrollAmount = Math.floor(wrapperWidth * 0.8); // Scroll 80% of visible width
        if (direction === 'left') {
            this.tabScrollOffset = Math.max(0, this.tabScrollOffset - scrollAmount);
        }
        else {
            const maxScroll = this.tabsContainer.scrollWidth - wrapperWidth;
            this.tabScrollOffset = Math.min(maxScroll, this.tabScrollOffset + scrollAmount);
        }
        this.tabsContainer.style.transform = `translateX(-${this.tabScrollOffset}px)`;
        this.updateNavigationArrows();
    }
    updateNavigationArrows() {
        const leftArrow = document.getElementById('tab-nav-left');
        const rightArrow = document.getElementById('tab-nav-right');
        const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper');
        if (!leftArrow || !rightArrow || !wrapper)
            return;
        const containerWidth = this.tabsContainer.scrollWidth;
        const wrapperWidth = wrapper.offsetWidth;
        const needsNavigation = containerWidth > wrapperWidth;
        if (needsNavigation) {
            // Show arrows
            leftArrow.style.display = 'flex';
            rightArrow.style.display = 'flex';
            // Enable/disable based on scroll position
            if (this.tabScrollOffset <= 0) {
                leftArrow.style.opacity = '0.3';
                leftArrow.style.cursor = 'default';
                leftArrow.style.pointerEvents = 'none';
            }
            else {
                leftArrow.style.opacity = '1';
                leftArrow.style.cursor = 'pointer';
                leftArrow.style.pointerEvents = 'auto';
            }
            const maxScroll = containerWidth - wrapperWidth;
            if (this.tabScrollOffset >= maxScroll) {
                rightArrow.style.opacity = '0.3';
                rightArrow.style.cursor = 'default';
                rightArrow.style.pointerEvents = 'none';
            }
            else {
                rightArrow.style.opacity = '1';
                rightArrow.style.cursor = 'pointer';
                rightArrow.style.pointerEvents = 'auto';
            }
        }
        else {
            // Hide arrows when not needed
            leftArrow.style.display = 'none';
            rightArrow.style.display = 'none';
            this.tabScrollOffset = 0;
            this.tabsContainer.style.transform = 'translateX(0)';
        }
    }
    toggleSystemLog() {
        const systemLogTab = this.tabs.get('system-log');
        const systemLogTabBtn = document.getElementById('tab-btn-system-log');
        const systemLogToggle = document.getElementById('isolated-terminal-system-log-toggle');
        if (!systemLogTab || !systemLogTabBtn || !systemLogToggle) {
            console.error('[TTYDTerminalPanel] System Log tab or button not found');
            return;
        }
        // Toggle visibility
        if (systemLogTabBtn.style.display === 'none') {
            // Show System Log tab
            systemLogTabBtn.style.display = 'flex';
            systemLogToggle.style.opacity = '1'; // Full brightness when visible
            console.log('[TTYDTerminalPanel] System Log tab shown');
            // If no other tabs are active, make System Log active
            if (!this.activeTabId || this.activeTabId === 'system-log') {
                this.switchToTab('system-log');
            }
        }
        else {
            // Hide System Log tab
            systemLogTabBtn.style.display = 'none';
            systemLogToggle.style.opacity = '0.7'; // Still fairly bright when hidden
            console.log('[TTYDTerminalPanel] System Log tab hidden');
            // If System Log was active, switch to another tab
            if (this.activeTabId === 'system-log') {
                const remainingTabs = Array.from(this.tabs.keys()).filter(id => id !== 'system-log');
                if (remainingTabs.length > 0) {
                    // Switch to the last tab (most recently created)
                    this.switchToTab(remainingTabs[remainingTabs.length - 1]);
                }
                else {
                    // No other tabs, hide the content
                    if (systemLogTab.element) {
                        systemLogTab.element.style.display = 'none';
                    }
                    this.activeTabId = null;
                }
            }
        }
        // Update navigation arrows
        this.updateNavigationArrows();
    }
    setupKeyboardShortcuts() {
        // Add keyboard shortcuts for terminal control
        // Use capture phase to intercept before other handlers
        document.addEventListener('keydown', (e) => __awaiter(this, void 0, void 0, function* () {
            // Use Ctrl/Cmd + Shift + R for terminal refresh (avoids conflict with browser refresh)
            if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'R') {
                // Check if the TTYD panel is visible
                const isolatedPanel = document.querySelector('.isolated-terminal-panel');
                if (isolatedPanel && !isolatedPanel.classList.contains('collapsed')) {
                    e.preventDefault();
                    e.stopPropagation();
                    console.log('[TTYDTerminalPanel] Refresh shortcut triggered');
                    this.refreshActiveTerminal();
                    return false;
                }
            }
            // Ctrl/Cmd + Shift + T to RESTART all terminals (complete fix for size issues)
            if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'T') {
                const isolatedPanel = document.querySelector('.isolated-terminal-panel');
                if (isolatedPanel && !isolatedPanel.classList.contains('collapsed')) {
                    e.preventDefault();
                    e.stopPropagation();
                    console.log('[TTYDTerminalPanel] RESTART ALL TERMINALS triggered (Cmd+Shift+T)');
                    yield this.restartAllTerminals();
                    return false;
                }
            }
            // Ctrl/Cmd + Shift + S to try resizing terminal without reload
            if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'S') {
                const isolatedPanel = document.querySelector('.isolated-terminal-panel');
                if (isolatedPanel && !isolatedPanel.classList.contains('collapsed')) {
                    e.preventDefault();
                    e.stopPropagation();
                    console.log('[TTYDTerminalPanel] RESIZE TERMINAL triggered (Cmd+Shift+S)');
                    yield this.tryResizeTerminal();
                    return false;
                }
            }
            // Alternative: F5 key for refresh when terminal panel is visible
            if (e.key === 'F5') {
                const isolatedPanel = document.querySelector('.isolated-terminal-panel');
                if (isolatedPanel && !isolatedPanel.classList.contains('collapsed')) {
                    e.preventDefault();
                    e.stopPropagation();
                    console.log('[TTYDTerminalPanel] F5 refresh triggered');
                    this.refreshActiveTerminal();
                    return false;
                }
            }
        }), true); // Use capture phase
    }
    hideAllWebviews() {
        // Hide all webviews by clearing their src to prevent ttyd from seeing tiny container
        console.log('[TTYDTerminalPanel] Hiding all webviews to prevent terminal resize');
        this.tabs.forEach((tab) => {
            if (tab.webview && tab.type !== 'system-log') {
                const webview = tab.webview;
                // Store the URL for later restoration
                if (!webview.dataset.originalSrc && webview.src && webview.src !== 'about:blank') {
                    webview.dataset.originalSrc = webview.src;
                }
                // Clear the webview to disconnect from ttyd
                webview.src = 'about:blank';
                webview.style.visibility = 'hidden';
            }
        });
    }
    showAndReloadWebviews() {
        // Show and reload webviews with proper container size
        console.log('[TTYDTerminalPanel] Showing and reloading webviews with proper size');
        // First ensure container has proper size
        const contentHeight = this.contentContainer.offsetHeight;
        const contentWidth = this.contentContainer.offsetWidth;
        console.log(`[TTYDTerminalPanel] Container size before reload: ${contentWidth}x${contentHeight}px`);
        // Only proceed if container has reasonable size
        if (contentHeight < 200 || contentWidth < 200) {
            console.log('[TTYDTerminalPanel] Container still too small, waiting...');
            setTimeout(() => this.showAndReloadWebviews(), 200);
            return;
        }
        this.tabs.forEach((tab) => {
            if (tab.webview && tab.type !== 'system-log') {
                const webview = tab.webview;
                const originalSrc = webview.dataset.originalSrc || tab.url;
                if (originalSrc && originalSrc !== 'about:blank') {
                    console.log(`[TTYDTerminalPanel] Restoring webview for tab: ${tab.id}`);
                    webview.style.visibility = 'visible';
                    // Reload with proper size
                    webview.src = originalSrc;
                    // Log expected terminal size
                    const rows = Math.floor(contentHeight / 17);
                    const cols = Math.floor(contentWidth / 9);
                    console.log(`[TTYDTerminalPanel] Expected terminal size: ${cols}x${rows}`);
                }
            }
        });
    }
    restartAllTerminals() {
        return __awaiter(this, void 0, void 0, function* () {
            // Restart all ttyd processes to get fresh PTY with correct size
            // This is the only reliable way to fix the terminal size issue
            console.log('[TTYDTerminalPanel] RESTARTING all terminals for proper size');
            // Get current container dimensions
            const contentHeight = this.contentContainer.offsetHeight;
            const contentWidth = this.contentContainer.offsetWidth;
            console.log(`[TTYDTerminalPanel] Container size: ${contentWidth}x${contentHeight}px`);
            // Store terminal info for recreation
            const terminalsToRestart = [];
            this.tabs.forEach((tab) => {
                if (tab.type !== 'system-log') {
                    terminalsToRestart.push({
                        id: tab.id,
                        title: tab.title,
                        toolId: tab.toolId,
                        wasActive: tab.isActive
                    });
                }
            });
            // Kill and recreate each terminal
            for (const termInfo of terminalsToRestart) {
                console.log(`[TTYDTerminalPanel] Restarting terminal: ${termInfo.title}`);
                try {
                    // Kill the old ttyd process
                    yield window.terminalAPI.killTerminalProcess(termInfo.id);
                    // Remove the old tab
                    this.removeTab(termInfo.id);
                    // Wait a bit for process to fully terminate
                    yield new Promise(resolve => setTimeout(resolve, 200));
                    // Create a new terminal with same configuration
                    yield this.createTerminalTab(termInfo.toolId);
                    // Note: We can't restore exact terminal content, but at least we get proper size
                }
                catch (error) {
                    console.error(`[TTYDTerminalPanel] Error restarting terminal ${termInfo.id}:`, error);
                }
            }
            const rows = Math.floor(contentHeight / 17);
            const cols = Math.floor(contentWidth / 9);
            console.log(`[TTYDTerminalPanel] Terminals restarted with expected size: ${cols}x${rows}`);
        });
    }
    forceCompleteReload() {
        // For less aggressive cases, just reload webviews
        console.log('[TTYDTerminalPanel] Starting complete webview reload');
        const contentHeight = this.contentContainer.offsetHeight;
        const contentWidth = this.contentContainer.offsetWidth;
        console.log(`[TTYDTerminalPanel] Current container: ${contentWidth}x${contentHeight}px`);
        this.tabs.forEach((tab) => {
            if (tab.webview && tab.type !== 'system-log') {
                const webview = tab.webview;
                const currentSrc = webview.src;
                if (currentSrc && currentSrc !== 'about:blank') {
                    console.log(`[TTYDTerminalPanel] Reloading webview for tab: ${tab.id}`);
                    webview.src = 'about:blank';
                    setTimeout(() => {
                        webview.src = currentSrc;
                    }, 100);
                }
            }
        });
    }
    fixPanelLayout() {
        // Fix the panel layout after window state changes or panel expansion
        const panel = document.querySelector('.isolated-terminal-panel');
        const contentContainer = document.getElementById('isolated-terminal-content');
        const tabsContainer = document.getElementById('isolated-terminal-tabs');
        const headerContainer = document.querySelector('.isolated-terminal-header');
        if (!panel || !contentContainer || !tabsContainer)
            return;
        // Check if panel is in expand-to-fill mode
        const isExpanded = panel.classList.contains('expand-to-fill');
        // Reset any weird dimensions
        const tabsHeight = 35; // Height of the tabs bar
        // Fix the panel height
        if (isExpanded) {
            // When expanded, ensure it fills available height
            panel.style.height = '100%';
        }
        // Ensure header has fixed height
        if (headerContainer) {
            headerContainer.style.height = `${tabsHeight}px`;
            headerContainer.style.minHeight = `${tabsHeight}px`;
            headerContainer.style.maxHeight = `${tabsHeight}px`;
            headerContainer.style.flex = `0 0 ${tabsHeight}px`;
        }
        // Ensure content container fills the remaining space
        contentContainer.style.flex = '1 1 auto';
        contentContainer.style.position = 'relative';
        contentContainer.style.width = '100%';
        contentContainer.style.height = `calc(100% - ${tabsHeight}px)`;
        contentContainer.style.minHeight = '0';
        contentContainer.style.overflow = 'hidden';
        // Force all webviews to recalculate their dimensions
        this.tabs.forEach((tab) => {
            if (tab.element) {
                tab.element.style.position = 'absolute';
                tab.element.style.top = '0';
                tab.element.style.left = '0';
                tab.element.style.right = '0';
                tab.element.style.bottom = '0';
                tab.element.style.width = '100%';
                tab.element.style.height = '100%';
                // If it's the active tab, ensure it's visible
                if (tab.isActive) {
                    tab.element.style.display = 'block';
                }
                else {
                    tab.element.style.display = 'none';
                }
            }
            // Also refresh the webview if it's active
            if (tab.webview && tab.isActive) {
                const webview = tab.webview;
                // Ensure webview fills its container
                webview.style.position = 'absolute';
                webview.style.top = '0';
                webview.style.left = '0';
                webview.style.right = '0';
                webview.style.bottom = '0';
                webview.style.width = '100%';
                webview.style.height = '100%';
                // Force ttyd to recalculate terminal size by reloading
                // This is the most reliable way to fix the 37x9 issue
                const currentSrc = webview.src;
                if (currentSrc && currentSrc !== 'about:blank') {
                    console.log('[TTYDTerminalPanel] Reloading webview to fix terminal size');
                    webview.src = '';
                    setTimeout(() => {
                        webview.src = currentSrc;
                    }, 100);
                }
            }
        });
        console.log('[TTYDTerminalPanel] Layout fixed after window state change');
    }
    tryResizeTerminal() {
        return __awaiter(this, void 0, void 0, function* () {
            // Try to resize the active terminal by injecting JavaScript
            if (!this.activeTabId)
                return;
            const tab = this.tabs.get(this.activeTabId);
            if (tab && tab.webview) {
                const webview = tab.webview;
                // Calculate proper size based on container
                const contentHeight = this.contentContainer.offsetHeight;
                const contentWidth = this.contentContainer.offsetWidth;
                const rows = Math.floor(contentHeight / 17); // ~17px per row
                const cols = Math.floor(contentWidth / 9); // ~9px per column
                console.log(`[TTYDTerminalPanel] Trying to resize terminal to ${cols}x${rows}`);
                try {
                    // Inject JavaScript to access ttyd's terminal
                    const script = `
                    (function() {
                        try {
                            // Method 1: Direct terminal access
                            if (window.term) {
                                window.term.resize(${cols}, ${rows});
                                return 'Resized via window.term';
                            }
                            
                            // Method 2: Send resize through ttyd's WebSocket
                            if (window.ws && window.ws.readyState === 1) {
                                // ttyd expects binary message with specific format
                                const encoder = new TextEncoder();
                                const resizeMsg = encoder.encode('1' + String.fromCharCode(${rows}) + String.fromCharCode(${cols}));
                                window.ws.send(resizeMsg);
                                return 'Sent resize via WebSocket';
                            }
                            
                            // Method 3: Trigger resize event
                            window.dispatchEvent(new Event('resize'));
                            return 'Triggered resize event';
                        } catch (e) {
                            return 'Error: ' + e.message;
                        }
                    })();
                `;
                    // Try to execute the script
                    if (webview.executeJavaScript) {
                        const result = yield webview.executeJavaScript(script);
                        console.log('[TTYDTerminalPanel] Resize result:', result);
                    }
                    else if (webview.send) {
                        // Try using webview.send
                        webview.send('resize', { cols, rows });
                    }
                    else {
                        console.log('[TTYDTerminalPanel] No way to inject JavaScript');
                    }
                }
                catch (error) {
                    console.error('[TTYDTerminalPanel] Error trying to resize:', error);
                }
            }
        });
    }
    refreshActiveTerminal() {
        // Manually refresh the active terminal to fix symbol distortion
        if (!this.activeTabId)
            return;
        const tab = this.tabs.get(this.activeTabId);
        if (tab && tab.webview) {
            const webview = tab.webview;
            console.log('[TTYDTerminalPanel] Manually refreshing terminal:', this.activeTabId);
            // First try to resize, then reload if needed
            this.tryResizeTerminal().then(() => {
                console.log('[TTYDTerminalPanel] Tried resize, now refreshing display');
            });
            // Reload the webview to refresh the terminal display
            // This is the most reliable way to fix symbol distortion
            const currentSrc = webview.src;
            if (currentSrc && currentSrc !== 'about:blank') {
                // Show a brief loading state
                webview.style.opacity = '0.5';
                // Clear and reload
                webview.src = '';
                // Reload after a brief delay
                setTimeout(() => {
                    webview.src = currentSrc;
                    webview.style.opacity = '1';
                    console.log('[TTYDTerminalPanel] Terminal refreshed');
                }, 100);
            }
        }
    }
    setupResizeObserver() {
        // Watch for center panel changes that affect our width
        const centerPanel = document.querySelector('.editor-container');
        if (centerPanel) {
            const centerObserver = new MutationObserver((mutations) => {
                mutations.forEach((mutation) => {
                    if (mutation.type === 'attributes' &&
                        (mutation.attributeName === 'style' || mutation.attributeName === 'class')) {
                        // Check if center panel is hidden or collapsed
                        const target = mutation.target;
                        const isHidden = target.style.display === 'none' ||
                            target.classList.contains('collapsed') ||
                            target.offsetWidth === 0;
                        if (isHidden) {
                            console.log('[TTYDTerminalPanel] Center panel collapsed, TTYD will expand');
                            // Don't do anything - flex layout handles the expansion automatically
                        }
                    }
                });
            });
            centerObserver.observe(centerPanel, {
                attributes: true,
                attributeFilter: ['style', 'class']
            });
        }
        // Observe the container for size changes
        let resizeTimeout = null;
        let resizeCompleteTimeout = null;
        // Handle window minimize/maximize events
        let lastWindowState = 'normal';
        // Detect when window is restored from minimize
        window.addEventListener('resize', () => {
            const currentWidth = window.innerWidth;
            const currentHeight = window.innerHeight;
            // Check if window was likely minimized (very small) and now restored
            if (lastWindowState === 'minimized' && currentWidth > 100 && currentHeight > 100) {
                console.log('[TTYDTerminalPanel] Window restored from minimize, fixing layout');
                // Force recalculate the panel dimensions
                setTimeout(() => {
                    this.fixPanelLayout();
                }, 100);
            }
            // Update state
            if (currentWidth < 100 || currentHeight < 100) {
                lastWindowState = 'minimized';
            }
            else {
                lastWindowState = 'normal';
            }
        });
        // Also listen for visibility changes
        document.addEventListener('visibilitychange', () => {
            if (!document.hidden) {
                console.log('[TTYDTerminalPanel] Document became visible, checking layout');
                setTimeout(() => {
                    this.fixPanelLayout();
                }, 100);
            }
        });
        // Watch for panel collapse/expand changes
        const panel = document.querySelector('.isolated-terminal-panel');
        if (panel) {
            let wasCollapsed = panel.classList.contains('collapsed');
            let wasExpanded = panel.classList.contains('expanded');
            const classObserver = new MutationObserver((mutations) => {
                mutations.forEach((mutation) => {
                    if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
                        const target = mutation.target;
                        const isCollapsed = target.classList.contains('collapsed');
                        const isExpanded = target.classList.contains('expanded');
                        // Handle collapse state change
                        if (isCollapsed !== wasCollapsed) {
                            console.log('[TTYDTerminalPanel] Panel collapse state changed:', wasCollapsed, '->', isCollapsed);
                            wasCollapsed = isCollapsed;
                            if (isCollapsed) {
                                // Just log, don't hide webviews or restart terminals
                                console.log('[TTYDTerminalPanel] Panel collapsed');
                            }
                            else {
                                // Panel expanded from collapsed state - just log, don't restart
                                console.log('[TTYDTerminalPanel] Panel expanded from collapsed');
                                // Don't restart terminals - they should maintain their size
                            }
                        }
                        // Handle expanded state change (when center panel collapses)
                        if (!isCollapsed && isExpanded !== wasExpanded) {
                            console.log('[TTYDTerminalPanel] Panel expand state changed:', wasExpanded, '->', isExpanded);
                            wasExpanded = isExpanded;
                            // Don't reload or restart - flex layout handles the size change
                        }
                    }
                });
            });
            classObserver.observe(panel, {
                attributes: true,
                attributeFilter: ['class']
            });
        }
        const resizeObserver = new ResizeObserver(() => {
            // Debounce resize events to avoid excessive redraws
            if (resizeTimeout) {
                clearTimeout(resizeTimeout);
            }
            if (resizeCompleteTimeout) {
                clearTimeout(resizeCompleteTimeout);
            }
            resizeTimeout = setTimeout(() => {
                // First pass: DO NOT toggle display as it may cause terminal issues
                // Just ensure the webview maintains proper size
                this.tabs.forEach((tab) => {
                    if (tab.webview && tab.isActive) {
                        const webview = tab.webview;
                        // Just ensure size is correct without any display toggling
                        // The terminal inside should handle its own resize
                        webview.style.width = '100%';
                        webview.style.height = '100%';
                    }
                });
                resizeTimeout = null;
            }, 50); // Faster initial response
            // Second pass: DO NOT reload webview as it causes terminal to reset to 9 rows
            // The terminal size is set server-side in ttyd and reload breaks it
            resizeCompleteTimeout = setTimeout(() => {
                this.tabs.forEach((tab) => {
                    if (tab.webview && tab.isActive) {
                        const webview = tab.webview;
                        // Instead of reloading, just ensure the webview is properly sized
                        // The terminal will handle its own resize via xterm.js fit addon
                        webview.style.width = '100%';
                        webview.style.height = '100%';
                        // Try to send a resize signal through the webview without reload
                        // This won't work due to security but at least we tried
                        try {
                            // Attempt to trigger resize inside the terminal
                            // Note: This likely won't work due to webview security restrictions
                            webview.executeJavaScript(`
                                if (window.term && window.term.fit) {
                                    window.term.fit();
                                    console.log('Terminal resized via fit()');
                                }
                            `).catch(() => {
                                // Expected to fail due to security
                                console.log('[TTYDTerminalPanel] Cannot execute resize in webview (security)');
                            });
                        }
                        catch (e) {
                            // Ignore - expected due to security
                        }
                    }
                });
                resizeCompleteTimeout = null;
            }, 500); // Wait for resize to complete
        });
        // Observe the container element
        if (this.container) {
            resizeObserver.observe(this.container);
        }
    }
}
exports.TTYDTerminalPanel = TTYDTerminalPanel;
// Export singleton instance
exports.ttydTerminalPanel = {
    instance: null,
    initialize(container) {
        if (!this.instance) {
            this.instance = new TTYDTerminalPanel(container);
        }
        return this.instance;
    },
    getInstance() {
        return this.instance;
    }
};
//# sourceMappingURL=TTYDTerminalPanel.js.map