/**
 * TTYDTerminalPanel - Terminal panel using xterm.js for direct terminal rendering
 * Provides real terminal emulation with perfect TUI support
 */

import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';
import { CanvasAddon } from '@xterm/addon-canvas';

interface TTYDTerminalTab {
    id: string;
    title: string;
    type: 'system-log' | 'terminal' | 'ai-tool';
    isActive: boolean;
    element?: HTMLElement;
    terminal?: Terminal;
    fitAddon?: FitAddon;
    searchAddon?: SearchAddon;
    toolId?: string;
}

export class TTYDTerminalPanel {
    private container: HTMLElement;
    private tabsContainer: HTMLElement;
    private contentContainer: HTMLElement;
    private tabs: Map<string, TTYDTerminalTab> = new Map();
    private activeTabId: string | null = null;
    private terminalCounter: number = 1;
    private tabScrollOffset: number = 0;
    private systemLogContainer: HTMLElement | null = null;

    constructor(container: HTMLElement) {
        this.container = container;
        this.tabsContainer = document.getElementById('isolated-terminal-tabs')!;
        this.contentContainer = document.getElementById('isolated-terminal-content')!;
        
        // Ensure content container has proper dimensions to prevent 139x9 issue
        this.contentContainer.style.position = 'relative';
        this.contentContainer.style.width = '100%';
        this.contentContainer.style.height = 'calc(100% - 35px)'; // Account for tab bar
        this.contentContainer.style.minHeight = '400px'; // Prevent tiny dimensions
        this.contentContainer.style.flex = '1 1 auto';
        
        this.initialize();
        this.setupResizeObserver();
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
        
        // Set up System Log toggle button
        const systemLogToggle = document.getElementById('isolated-terminal-system-log-toggle');
        if (systemLogToggle) {
            console.log('[TTYDTerminalPanel] Found System Log toggle button, adding listener');
            systemLogToggle.addEventListener('click', () => {
                console.log('[TTYDTerminalPanel] System Log toggle button clicked');
                this.toggleSystemLog();
            });
            // Initialize button appearance - keep it bright but indicate state with slight opacity
            systemLogToggle.style.opacity = '0.7';  // More visible even when System Log is hidden
        } else {
            console.error('[TTYDTerminalPanel] System Log toggle button not found!');
        }
        
        // Listen for terminal creation events from main process
        this.setupIpcListeners();
        
        // Set up tab navigation arrows
        this.setupTabNavigation();
        
        // Set up keyboard shortcuts
        this.setupKeyboardShortcuts();
    }
    
    private setupIpcListeners(): void {
        // Listen for terminal creation from main process
        window.terminalAPI.onTerminalCreated((terminalInfo: any) => {
            console.log('[TTYDTerminalPanel] Terminal created event:', terminalInfo);
            // Check if this tab already exists to avoid duplicates
            if (!this.tabs.has(terminalInfo.id)) {
                // This is from an external source (like AI CLI tools) - add the tab
                console.log('[TTYDTerminalPanel] Adding tab for external terminal:', terminalInfo.id);
                this.addTerminalTab(terminalInfo);
            } else {
                console.log('[TTYDTerminalPanel] Tab already exists for:', terminalInfo.id);
            }
        });

        // Listen for terminal data from PTY
        window.terminalAPI.onTerminalData((terminalId: string, data: string) => {
            const tab = this.tabs.get(terminalId);
            if (tab && tab.terminal) {
                tab.terminal.write(data);
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
            isActive: false  // System Log should NOT be active by default
        };

        // Create tab button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'ttyd-tab-btn';  // Remove 'active' class
        tabBtn.id = `tab-btn-${tab.id}`;  // Add ID for System Log tab too
        tabBtn.innerHTML = `
            <span>${tab.title}</span>
        `;
        tabBtn.style.display = 'none';  // Hide System Log tab by default
        
        tabBtn.addEventListener('click', () => this.switchToTab(tab.id));
        this.tabsContainer.appendChild(tabBtn);

        // Create content area for system log with wrapper
        const content = document.createElement('div');
        content.id = `isolated-content-${tab.id}`;
        content.className = 'isolated-tab-content';  // Remove 'active' class
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: #1e1e1e;
            display: none;  // Hide by default since it's not active
        `;
        
        // Create the actual scrollable log container
        const logContainer = document.createElement('div');
        logContainer.id = 'system-log-container';
        logContainer.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            overflow-y: auto;
            overflow-x: hidden;
            padding: 10px;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 12px;
            color: #cccccc;
            line-height: 1.4;
            scroll-behavior: auto;  // Changed from smooth for more reliable scrolling
            -webkit-overflow-scrolling: touch;
        `;
        
        // Add initial message
        logContainer.innerHTML = `<div style="color: #569cd6;">[System Log initialized]</div>`;
        
        content.appendChild(logContainer);
        
        this.contentContainer.appendChild(content);
        
        tab.element = content;
        this.tabs.set(tab.id, tab);
        this.systemLogContainer = logContainer;
        // Don't set activeTabId here - let the first terminal or AI tool take focus
        
        // Set up console capture for system log - pass the actual log container
        this.setupConsoleCapture(logContainer);
    }

    private setupConsoleCapture(logElement: HTMLElement): void {
        const originalLog = console.log;
        const originalError = console.error;
        const originalWarn = console.warn;
        
        // Track if user has manually scrolled up
        let userHasScrolledUp = false;
        let isAutoScrolling = false;
        
        // Function to scroll to bottom
        const scrollToBottom = () => {
            isAutoScrolling = true;
            // Multiple attempts to ensure it works
            logElement.scrollTop = logElement.scrollHeight;
            
            setTimeout(() => {
                logElement.scrollTop = logElement.scrollHeight;
                requestAnimationFrame(() => {
                    logElement.scrollTop = logElement.scrollHeight;
                    // Clear the auto-scrolling flag after animation frame
                    setTimeout(() => {
                        isAutoScrolling = false;
                    }, 100);
                });
            }, 10);
        };
        
        // Initially scroll to bottom
        setTimeout(scrollToBottom, 100);
        
        // Listen for user scroll events
        logElement.addEventListener('scroll', () => {
            // Skip if this is our programmatic scroll
            if (isAutoScrolling) return;
            
            const scrollTop = logElement.scrollTop;
            const scrollHeight = logElement.scrollHeight;
            const clientHeight = logElement.clientHeight;
            const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
            
            // If user is more than 100px from bottom, they've scrolled up
            if (distanceFromBottom > 100) {
                userHasScrolledUp = true;
            } else if (distanceFromBottom < 20) {
                // If they're very close to bottom, resume auto-scroll
                userHasScrolledUp = false;
            }
        });
        
        const addLogEntry = (message: string, type: 'log' | 'error' | 'warn') => {
            // Check if we were at bottom before adding entry
            const wasAtBottom = !userHasScrolledUp;
            
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
            
            entry.style.cssText = `
                color: ${color};
                margin-bottom: 2px;
                word-wrap: break-word;
                white-space: pre-wrap;
                font-family: inherit;
                font-size: inherit;
            `;
            entry.textContent = `[${timestamp}] [${prefix}] ${message}`;
            
            logElement.appendChild(entry);
            
            // If we were at bottom (or user hasn't scrolled), stay at bottom
            if (wasAtBottom) {
                scrollToBottom();
            }
            
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

    async createTerminalTab(toolId?: string, command?: string, env?: Record<string, string>, scriptContent?: string): Promise<void> {
        const terminalId = `terminal-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        console.log('[TTYDTerminalPanel] createTerminalTab called with toolId:', toolId, 'command:', command, 'env:', env);
        
        // Mark that we're creating this terminal internally to avoid duplicate tabs
        this.tabs.set(terminalId, {
            id: terminalId,
            title: 'Creating...',
            type: 'terminal',
            isActive: false
        } as TTYDTerminalTab);
        
        try {
            console.log('[TTYDTerminalPanel] Calling window.terminalAPI.createTerminalProcess...');
            // Request terminal creation from main process
            const result = await window.terminalAPI.createTerminalProcess({
                terminalId,
                toolId,
                command,
                cwd: window.currentOpenedFolder || undefined,
                env: env,  // Pass environment variables if provided
                scriptContent
            });
            
            console.log('[TTYDTerminalPanel] createTerminalProcess result:', result);
            
            if (result.success && result.terminal) {
                console.log('[TTYDTerminalPanel] Terminal created successfully:', result.terminal);
                // Remove the placeholder
                this.tabs.delete(terminalId);
                // Add the real terminal tab
                this.addTerminalTab(result.terminal);
            } else {
                console.error('[TTYDTerminalPanel] Failed to create terminal:', result.error);
                // Remove the placeholder on failure
                this.tabs.delete(terminalId);
            }
        } catch (error) {
            console.error('[TTYDTerminalPanel] Error creating terminal:', error);
            // Remove the placeholder on error
            this.tabs.delete(terminalId);
        }
    }
    
    private addTerminalTab(terminalInfo: {
        id: string;
        title: string;
        url?: string;
        port?: number;
        toolId?: string;
    }): void {
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
                    } else {
                        this.tabsContainer = existingTabs as HTMLElement;
                        console.log('[TTYDTerminalPanel] Found existing tabs container');
                    }
                } else {
                    console.error('[TTYDTerminalPanel] Cannot create tabs container - wrapper not found');
                    return;
                }
            }
        }

        const tab: TTYDTerminalTab = {
            id: terminalInfo.id,
            title: terminalInfo.title || `Terminal ${this.terminalCounter++}`,
            type: terminalInfo.toolId ? 'ai-tool' : 'terminal',
            isActive: false,
            toolId: terminalInfo.toolId
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
        console.log('[TTYDTerminalPanel] Tab button added to container:', tab.id, 'Container children:', this.tabsContainer.children.length);

        // Create content area with xterm.js terminal
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

        // Create xterm.js terminal instance with VS Code-like styling
        const terminal = new Terminal({
            fontSize: 13,
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            theme: {
                background: '#1e1e1e',
                foreground: '#cccccc',
                cursor: '#aeafad',
                black: '#000000',
                red: '#cd3131',
                green: '#0dbc79',
                yellow: '#e5e510',
                blue: '#2472c8',
                magenta: '#bc3fbc',
                cyan: '#11a8cd',
                white: '#e5e5e5',
                brightBlack: '#666666',
                brightRed: '#f14c4c',
                brightGreen: '#23d18b',
                brightYellow: '#f5f543',
                brightBlue: '#3b8eea',
                brightMagenta: '#d670d6',
                brightCyan: '#29b8db',
                brightWhite: '#e5e5e5'
            },
            cursorBlink: true,
            cursorStyle: 'block',
            scrollback: 10000,
            allowTransparency: false
        });

        // Add addons for enhanced functionality
        const fitAddon = new FitAddon();
        terminal.loadAddon(fitAddon);

        const searchAddon = new SearchAddon();
        terminal.loadAddon(searchAddon);

        // Add web links support (clickable URLs)
        const webLinksAddon = new WebLinksAddon();
        terminal.loadAddon(webLinksAddon);

        // Add canvas renderer for better performance
        const canvasAddon = new CanvasAddon();
        terminal.loadAddon(canvasAddon);

        // Open terminal in the container
        terminal.open(content);

        // Set up input handler - send data to PTY via IPC
        terminal.onData((data) => {
            window.terminalAPI.writeToTerminal(terminalInfo.id, data);
        });

        // Set up resize handler - notify PTY of size changes
        terminal.onResize((size) => {
            window.terminalAPI.resizeTerminal(terminalInfo.id, size.cols, size.rows);
        });

        // Fit terminal to container after a short delay
        setTimeout(() => {
            fitAddon.fit();
            console.log(`[TTYDTerminalPanel] Terminal ${tab.id} fitted to container`);
        }, 100);

        this.contentContainer.appendChild(content);

        tab.element = content;
        tab.terminal = terminal;
        tab.fitAddon = fitAddon;
        tab.searchAddon = searchAddon;
        this.tabs.set(tab.id, tab);

        // Switch to the new tab
        this.switchToTab(tab.id);

        // Update navigation arrows after adding tab
        setTimeout(() => this.updateNavigationArrows(), 50);
    }

    private switchToTab(tabId: string): void {
        const tab = this.tabs.get(tabId);
        if (!tab) return;

        // Check if panel is collapsed
        const panel = document.querySelector('.isolated-terminal-panel');
        const isCollapsed = panel?.classList.contains('collapsed');

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

        // Dispose of terminal instance
        if (tab.terminal) {
            tab.terminal.dispose();
        }

        // Remove content element
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
                } else {
                    // Only switch to System Log if it's the only tab left
                    this.switchToTab(remainingTabs[0]);
                }
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

    public async closeActiveTab(): Promise<void> {
        if (!this.activeTabId) return;
        await this.closeTab(this.activeTabId);
    }

    public showSystemLogTab(): void {
        this.toggleSystemLog('show');
        this.switchToTab('system-log');
    }

    public hideSystemLogTab(): void {
        this.toggleSystemLog('hide');
    }

    public clearSystemLog(): void {
        if (this.systemLogContainer) {
            this.systemLogContainer.innerHTML = '<div style="color:#569cd6;">[System log cleared]</div>';
        }
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
    
    public toggleSystemLog(force?: 'show' | 'hide'): void {
        const systemLogTab = this.tabs.get('system-log');
        const systemLogTabBtn = document.getElementById('tab-btn-system-log');
        const systemLogToggle = document.getElementById('isolated-terminal-system-log-toggle');
        
        if (!systemLogTab || !systemLogTabBtn || !systemLogToggle) {
            console.error('[TTYDTerminalPanel] System Log tab or button not found');
            return;
        }
        
        const currentlyHidden = systemLogTabBtn.style.display === 'none' || systemLogTabBtn.style.display === '';
        const shouldShow = force === 'show' || (force === undefined && currentlyHidden);
        const shouldHide = force === 'hide' || (force === undefined && !currentlyHidden);

        if (shouldShow) {
            // Show System Log tab
            systemLogTabBtn.style.display = 'flex';
            systemLogToggle.style.opacity = '1';  // Full brightness when visible
            console.log('[TTYDTerminalPanel] System Log tab shown');
            
            // If no other tabs are active, make System Log active
            if (!this.activeTabId || this.activeTabId === 'system-log') {
                this.switchToTab('system-log');
            }
        } else if (shouldHide) {
            // Hide System Log tab
            systemLogTabBtn.style.display = 'none';
            systemLogToggle.style.opacity = '0.7';  // Still fairly bright when hidden
            console.log('[TTYDTerminalPanel] System Log tab hidden');
            
            // If System Log was active, switch to another tab
            if (this.activeTabId === 'system-log') {
                const remainingTabs = Array.from(this.tabs.keys()).filter(id => id !== 'system-log');
                if (remainingTabs.length > 0) {
                    // Switch to the last tab (most recently created)
                    this.switchToTab(remainingTabs[remainingTabs.length - 1]);
                } else {
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
    
    private setupKeyboardShortcuts(): void {
        // Add keyboard shortcuts for terminal control
        document.addEventListener('keydown', (e) => {
            const isolatedPanel = document.querySelector('.isolated-terminal-panel');
            if (!isolatedPanel || isolatedPanel.classList.contains('collapsed')) return;

            // Ctrl/Cmd + L to clear terminal
            if ((e.ctrlKey || e.metaKey) && e.key === 'l') {
                e.preventDefault();
                this.clearActiveTerminal();
            }
        });
    }

    private clearActiveTerminal(): void {
        if (!this.activeTabId) return;

        const tab = this.tabs.get(this.activeTabId);
        if (tab && tab.terminal) {
            tab.terminal.clear();
            console.log('[TTYDTerminalPanel] Terminal cleared:', this.activeTabId);
        }
    }
    
    private setupResizeObserver(): void {
        // Observe the container for size changes and refit terminals
        let resizeTimeout: NodeJS.Timeout | null = null;

        const resizeObserver = new ResizeObserver(() => {
            // Debounce resize events to avoid excessive redraws
            if (resizeTimeout) {
                clearTimeout(resizeTimeout);
            }

            resizeTimeout = setTimeout(() => {
                // Refit all active terminals to their containers
                this.tabs.forEach((tab) => {
                    if (tab.terminal && tab.fitAddon && tab.isActive) {
                        try {
                            tab.fitAddon.fit();
                            console.log(`[TTYDTerminalPanel] Refitted terminal ${tab.id} to container`);
                        } catch (error) {
                            console.error(`[TTYDTerminalPanel] Error refitting terminal ${tab.id}:`, error);
                        }
                    }
                });

                resizeTimeout = null;
            }, 100);
        });

        // Observe the content container element
        if (this.contentContainer) {
            resizeObserver.observe(this.contentContainer);
        }

        // Also handle window resize events
        window.addEventListener('resize', () => {
            if (resizeTimeout) {
                clearTimeout(resizeTimeout);
            }

            resizeTimeout = setTimeout(() => {
                this.tabs.forEach((tab) => {
                    if (tab.terminal && tab.fitAddon && tab.isActive) {
                        try {
                            tab.fitAddon.fit();
                        } catch (error) {
                            console.error(`[TTYDTerminalPanel] Error refitting terminal on window resize:`, error);
                        }
                    }
                });
            }, 100);
        });
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
