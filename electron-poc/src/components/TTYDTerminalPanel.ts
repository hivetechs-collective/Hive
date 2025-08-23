/**
 * TTYDTerminalPanel - Terminal panel using ttyd server and webviews
 * Provides real terminal emulation with perfect TUI support
 */

interface TTYDTerminalTab {
    id: string;
    title: string;
    type: 'system-log' | 'terminal' | 'ai-tool';
    isActive: boolean;
    element?: HTMLElement;
    webview?: HTMLIFrameElement;  // Using iframe for webview in renderer
    toolId?: string;
    url?: string;
    port?: number;
}

export class TTYDTerminalPanel {
    private container: HTMLElement;
    private tabsContainer: HTMLElement;
    private contentContainer: HTMLElement;
    private tabs: Map<string, TTYDTerminalTab> = new Map();
    private activeTabId: string | null = null;
    private terminalCounter: number = 1;
    private tabScrollOffset: number = 0;

    constructor(container: HTMLElement) {
        this.container = container;
        this.tabsContainer = document.getElementById('isolated-terminal-tabs')!;
        this.contentContainer = document.getElementById('isolated-terminal-content')!;
        
        this.initialize();
    }

    private initialize(): void {
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
        } else {
            console.error('[TTYDTerminalPanel] New tab button not found!');
        }
        
        // Listen for terminal creation events from main process
        this.setupIpcListeners();
        
        // Set up tab navigation arrows
        this.setupTabNavigation();
    }
    
    private setupIpcListeners(): void {
        // Listen for terminal creation from main process
        window.terminalAPI.onTerminalCreated((terminalInfo: any) => {
            console.log('[TTYDTerminalPanel] Terminal created event:', terminalInfo);
            // Don't add tab here - we handle it in createTerminalTab to avoid duplicates
            // This event is mainly for external terminal creation (e.g., from AI tools)
        });
        
        // Listen for terminal ready events
        window.terminalAPI.onTerminalReady((terminalId: string, url: string) => {
            console.log('[TTYDTerminalPanel] Terminal ready:', terminalId, url);
            const tab = this.tabs.get(terminalId);
            if (tab && tab.webview) {
                // Update the iframe src to connect to ttyd
                tab.webview.src = url;
            }
        });
        
        // Listen for terminal close events
        window.terminalAPI.onTerminalExit((terminalId: string) => {
            console.log('[TTYDTerminalPanel] Terminal exited:', terminalId);
            this.removeTab(terminalId);
        });
    }

    private createSystemLogTab(): void {
        const tab: TTYDTerminalTab = {
            id: 'system-log',
            title: '📊 System Log',
            type: 'system-log',
            isActive: true
        };

        // Create tab button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'isolated-tab active';
        tabBtn.id = `tab-btn-${tab.id}`;  // Add ID for System Log tab too
        tabBtn.innerHTML = `
            <span>${tab.title}</span>
        `;
        tabBtn.style.cssText = `
            padding: 6px 12px;
            background: #1e1e1e;
            border: none;
            border-right: 1px solid #3c3c3c;
            color: #cccccc;
            cursor: pointer;
            font-size: 12px;
            display: flex;
            align-items: center;
            gap: 4px;
            flex-shrink: 0;
            white-space: nowrap;
        `;
        
        tabBtn.addEventListener('click', () => this.switchToTab(tab.id));
        this.tabsContainer.appendChild(tabBtn);

        // Create content area for system log
        const content = document.createElement('div');
        content.id = `isolated-content-${tab.id}`;
        content.className = 'isolated-tab-content active';
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: #1e1e1e;
            display: block;
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
        this.activeTabId = tab.id;
        
        // Set up console capture for system log
        this.setupConsoleCapture(content);
    }

    private setupConsoleCapture(logElement: HTMLElement): void {
        const originalLog = console.log;
        const originalError = console.error;
        const originalWarn = console.warn;
        
        const addLogEntry = (message: string, type: 'log' | 'error' | 'warn') => {
            const entry = document.createElement('div');
            const timestamp = new Date().toLocaleTimeString();
            
            let color = '#cccccc';
            let prefix = 'INFO';
            if (type === 'error') {
                color = '#f44747';
                prefix = 'ERROR';
            } else if (type === 'warn') {
                color = '#dcdcaa';
                prefix = 'WARN';
            }
            
            entry.style.color = color;
            entry.style.marginBottom = '2px';
            entry.textContent = `[${timestamp}] [${prefix}] ${message}`;
            
            logElement.appendChild(entry);
            
            // Auto-scroll to bottom
            logElement.scrollTop = logElement.scrollHeight;
            
            // Limit log entries to prevent memory issues
            while (logElement.children.length > 1000) {
                logElement.removeChild(logElement.firstChild!);
            }
        };
        
        console.log = function(...args: any[]) {
            originalLog.apply(console, args);
            addLogEntry(args.join(' '), 'log');
        };
        
        console.error = function(...args: any[]) {
            originalError.apply(console, args);
            addLogEntry(args.join(' '), 'error');
        };
        
        console.warn = function(...args: any[]) {
            originalWarn.apply(console, args);
            addLogEntry(args.join(' '), 'warn');
        };
    }

    async createTerminalTab(toolId?: string, command?: string, env?: Record<string, string>): Promise<void> {
        const terminalId = `terminal-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        console.log('[TTYDTerminalPanel] createTerminalTab called with toolId:', toolId, 'command:', command, 'env:', env);
        
        try {
            console.log('[TTYDTerminalPanel] Calling window.terminalAPI.createTerminalProcess...');
            // Request terminal creation from main process
            const result = await window.terminalAPI.createTerminalProcess({
                terminalId,
                toolId,
                command,
                cwd: window.currentOpenedFolder || undefined,
                env: env  // Pass environment variables if provided
            });
            
            console.log('[TTYDTerminalPanel] createTerminalProcess result:', result);
            
            if (result.success && result.terminal) {
                console.log('[TTYDTerminalPanel] Terminal created successfully:', result.terminal);
                // Add the terminal tab here since we control when it's called
                this.addTerminalTab(result.terminal);
            } else {
                console.error('[TTYDTerminalPanel] Failed to create terminal:', result.error);
            }
        } catch (error) {
            console.error('[TTYDTerminalPanel] Error creating terminal:', error);
        }
    }
    
    private addTerminalTab(terminalInfo: {
        id: string;
        title: string;
        url: string;
        port: number;
        toolId?: string;
    }): void {
        const tab: TTYDTerminalTab = {
            id: terminalInfo.id,
            title: terminalInfo.title || `Terminal ${this.terminalCounter++}`,
            type: terminalInfo.toolId ? 'ai-tool' : 'terminal',
            isActive: false,
            toolId: terminalInfo.toolId,
            url: terminalInfo.url,
            port: terminalInfo.port
        };

        // Create tab button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'isolated-tab';
        tabBtn.id = `tab-btn-${tab.id}`;
        tabBtn.innerHTML = `
            <span>${tab.title}</span>
            <span class="tab-close" style="margin-left: 8px; cursor: pointer;">×</span>
        `;
        tabBtn.style.cssText = `
            padding: 6px 12px;
            background: #2d2d2d;
            border: none;
            border-right: 1px solid #3c3c3c;
            color: #969696;
            cursor: pointer;
            font-size: 12px;
            display: flex;
            align-items: center;
            gap: 4px;
            position: relative;
            flex-shrink: 0;
            white-space: nowrap;
        `;
        
        // Add click handler for tab switching
        tabBtn.addEventListener('click', (e) => {
            if (!(e.target as HTMLElement).classList.contains('tab-close')) {
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
        `;
        
        // Create webview to embed ttyd terminal
        // Using webview instead of iframe for better Electron integration
        const webview = document.createElement('webview') as any;
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
        
        // Switch to the new tab
        this.switchToTab(tab.id);
        
        // Update navigation arrows after adding tab
        setTimeout(() => this.updateNavigationArrows(), 50);
    }

    private switchToTab(tabId: string): void {
        const tab = this.tabs.get(tabId);
        if (!tab) return;

        // Deactivate current tab
        if (this.activeTabId) {
            const currentTab = this.tabs.get(this.activeTabId);
            if (currentTab) {
                currentTab.isActive = false;
                const currentTabBtn = document.getElementById(`tab-btn-${this.activeTabId}`);
                if (currentTabBtn) {
                    currentTabBtn.classList.remove('active');
                    currentTabBtn.style.background = '#2d2d2d';
                    currentTabBtn.style.color = '#969696';
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
            tabBtn.style.background = '#1e1e1e';
            tabBtn.style.color = '#cccccc';
        }
        if (tab.element) {
            tab.element.style.display = 'block';
        }

        this.activeTabId = tabId;
        
        // Auto-scroll to show the active tab
        this.scrollToTab(tabId);
    }
    
    private scrollToTab(tabId: string): void {
        const tabBtn = document.getElementById(`tab-btn-${tabId}`);
        const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper') as HTMLElement;
        
        if (!tabBtn || !wrapper) return;
        
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
        } else if (tabRight > visibleRight) {
            // Scroll right to show the tab
            this.tabScrollOffset = Math.max(0, tabRight - wrapperWidth + 10); // 10px padding
            this.tabsContainer.style.transform = `translateX(-${this.tabScrollOffset}px)`;
            this.updateNavigationArrows();
        }
    }

    private async closeTab(tabId: string): Promise<void> {
        const tab = this.tabs.get(tabId);
        if (!tab || tab.type === 'system-log') return; // Can't close system log

        // Close the terminal in the backend
        try {
            await window.terminalAPI.killTerminalProcess(tabId);
        } catch (error) {
            console.error('[TTYDTerminalPanel] Error closing terminal:', error);
        }

        // Remove the tab
        this.removeTab(tabId);
    }
    
    private removeTab(tabId: string): void {
        const tab = this.tabs.get(tabId);
        if (!tab) return;
        
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
                this.switchToTab(remainingTabs[0]);
            }
        }
        
        // Update navigation arrows after removing tab
        setTimeout(() => this.updateNavigationArrows(), 50);
    }

    // Public method to launch AI tool in terminal
    public async launchAITool(toolId: string, command: string): Promise<void> {
        // Check if tool already has a terminal
        for (const [id, tab] of this.tabs) {
            if (tab.toolId === toolId) {
                // Switch to existing tab
                this.switchToTab(id);
                return;
            }
        }
        
        // Create new terminal for the tool
        await this.createTerminalTab(toolId, command);
    }

    // Get current active terminal ID
    public getActiveTerminalId(): string | null {
        return this.activeTabId;
    }
    
    // Set up tab navigation arrows
    private setupTabNavigation(): void {
        const leftArrow = document.getElementById('tab-nav-left');
        const rightArrow = document.getElementById('tab-nav-right');
        const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper') as HTMLElement;
        
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
        let resizeAnimationFrame: number | null = null;
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
                } else if (e.key === 'ArrowRight') {
                    e.preventDefault();
                    this.switchToNextTab();
                }
            }
        });
    }
    
    private switchToPreviousTab(): void {
        const tabIds = Array.from(this.tabs.keys());
        const currentIndex = tabIds.indexOf(this.activeTabId || '');
        if (currentIndex > 0) {
            this.switchToTab(tabIds[currentIndex - 1]);
        }
    }
    
    private switchToNextTab(): void {
        const tabIds = Array.from(this.tabs.keys());
        const currentIndex = tabIds.indexOf(this.activeTabId || '');
        if (currentIndex >= 0 && currentIndex < tabIds.length - 1) {
            this.switchToTab(tabIds[currentIndex + 1]);
        }
    }
    
    private scrollTabs(direction: 'left' | 'right'): void {
        const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper') as HTMLElement;
        if (!wrapper) return;
        
        const wrapperWidth = wrapper.offsetWidth;
        const scrollAmount = Math.floor(wrapperWidth * 0.8); // Scroll 80% of visible width
        
        if (direction === 'left') {
            this.tabScrollOffset = Math.max(0, this.tabScrollOffset - scrollAmount);
        } else {
            const maxScroll = this.tabsContainer.scrollWidth - wrapperWidth;
            this.tabScrollOffset = Math.min(maxScroll, this.tabScrollOffset + scrollAmount);
        }
        
        this.tabsContainer.style.transform = `translateX(-${this.tabScrollOffset}px)`;
        this.updateNavigationArrows();
    }
    
    private updateNavigationArrows(): void {
        const leftArrow = document.getElementById('tab-nav-left');
        const rightArrow = document.getElementById('tab-nav-right');
        const wrapper = document.querySelector('.isolated-terminal-tabs-wrapper') as HTMLElement;
        
        if (!leftArrow || !rightArrow || !wrapper) return;
        
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
            } else {
                leftArrow.style.opacity = '1';
                leftArrow.style.cursor = 'pointer';
                leftArrow.style.pointerEvents = 'auto';
            }
            
            const maxScroll = containerWidth - wrapperWidth;
            if (this.tabScrollOffset >= maxScroll) {
                rightArrow.style.opacity = '0.3';
                rightArrow.style.cursor = 'default';
                rightArrow.style.pointerEvents = 'none';
            } else {
                rightArrow.style.opacity = '1';
                rightArrow.style.cursor = 'pointer';
                rightArrow.style.pointerEvents = 'auto';
            }
        } else {
            // Hide arrows when not needed
            leftArrow.style.display = 'none';
            rightArrow.style.display = 'none';
            this.tabScrollOffset = 0;
            this.tabsContainer.style.transform = 'translateX(0)';
        }
    }
}

// Export singleton instance
export const ttydTerminalPanel = {
    instance: null as TTYDTerminalPanel | null,
    
    initialize(container: HTMLElement): TTYDTerminalPanel {
        if (!this.instance) {
            this.instance = new TTYDTerminalPanel(container);
        }
        return this.instance;
    },
    
    getInstance(): TTYDTerminalPanel | null {
        return this.instance;
    }
};