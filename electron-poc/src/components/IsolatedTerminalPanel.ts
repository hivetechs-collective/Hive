/**
 * IsolatedTerminalPanel - A completely isolated terminal implementation
 * This component is designed to have zero impact on the rest of the application
 */

interface TerminalTab {
    id: string;
    title: string;
    type: 'system-log' | 'terminal';
    isActive: boolean;
    element?: HTMLElement;
}

export class IsolatedTerminalPanel {
    private container: HTMLElement;
    private tabsContainer: HTMLElement;
    private contentContainer: HTMLElement;
    private tabs: Map<string, TerminalTab> = new Map();
    private activeTabId: string | null = null;
    private terminalCounter: number = 1;

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
            newTabBtn.addEventListener('click', () => this.createTerminalTab());
        }
    }

    private createSystemLogTab(): void {
        const tab: TerminalTab = {
            id: 'system-log',
            title: '📊 System Log',
            type: 'system-log',
            isActive: true
        };

        // Create tab button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'isolated-tab active';
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
        `;
        
        tabBtn.addEventListener('click', () => this.switchToTab(tab.id));
        this.tabsContainer.appendChild(tabBtn);

        // Create content area
        const content = document.createElement('div');
        content.id = `isolated-content-${tab.id}`;
        content.className = 'isolated-tab-content active';
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            padding: 10px;
            overflow-y: auto;
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 12px;
            color: #cccccc;
            display: block;
        `;
        
        // Add some initial content
        content.innerHTML = `
            <div style="color: #569cd6;">[INFO] Isolated Terminal System Initialized</div>
            <div style="color: #569cd6;">[INFO] System Log tab created</div>
            <div style="color: #4ec9b0;">[INFO] This is a completely isolated terminal panel</div>
            <div style="color: #4ec9b0;">[INFO] It will not affect any other part of the application</div>
        `;
        
        this.contentContainer.appendChild(content);
        
        tab.element = content;
        this.tabs.set(tab.id, tab);
        this.activeTabId = tab.id;

        // Start capturing console output
        this.startConsoleCapture(content);
    }

    private createTerminalTab(): void {
        const tabId = `terminal-${this.terminalCounter++}`;
        const tab: TerminalTab = {
            id: tabId,
            title: `Terminal ${this.terminalCounter - 1}`,
            type: 'terminal',
            isActive: false
        };

        // Create tab button with close button
        const tabBtn = document.createElement('button');
        tabBtn.className = 'isolated-tab';
        tabBtn.style.cssText = `
            padding: 6px 12px;
            background: transparent;
            border: none;
            border-right: 1px solid #3c3c3c;
            color: #969696;
            cursor: pointer;
            font-size: 12px;
            display: flex;
            align-items: center;
            gap: 4px;
            position: relative;
        `;
        
        const tabLabel = document.createElement('span');
        tabLabel.textContent = tab.title;
        tabBtn.appendChild(tabLabel);

        const closeBtn = document.createElement('span');
        closeBtn.textContent = '×';
        closeBtn.style.cssText = `
            margin-left: 8px;
            opacity: 0.6;
            font-size: 16px;
        `;
        closeBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            this.closeTab(tabId);
        });
        tabBtn.appendChild(closeBtn);
        
        tabBtn.addEventListener('click', () => this.switchToTab(tabId));
        this.tabsContainer.appendChild(tabBtn);

        // Create content area
        const content = document.createElement('div');
        content.id = `isolated-content-${tabId}`;
        content.className = 'isolated-tab-content';
        content.style.cssText = `
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            padding: 10px;
            overflow-y: auto;
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 12px;
            color: #cccccc;
            background: #000000;
            display: none;
        `;
        
        // Add placeholder content for now
        content.innerHTML = `
            <div style="color: #4ec9b0;">$ Welcome to ${tab.title}</div>
            <div style="color: #969696;">This will be a fully functional terminal once xterm.js is integrated</div>
            <div style="color: #969696;">For now, it's just a placeholder showing isolation works</div>
            <div style="color: #cccccc;">$ <span style="border-right: 2px solid #cccccc; animation: blink 1s infinite;">_</span></div>
        `;
        
        this.contentContainer.appendChild(content);
        
        tab.element = content;
        this.tabs.set(tabId, tab);
        
        // Switch to the new tab
        this.switchToTab(tabId);
    }

    private switchToTab(tabId: string): void {
        // Deactivate all tabs and contents
        this.tabs.forEach((tab, id) => {
            const tabBtn = this.tabsContainer.querySelector(`button:nth-child(${Array.from(this.tabs.keys()).indexOf(id) + 1})`) as HTMLElement;
            if (tabBtn) {
                tabBtn.classList.remove('active');
                if (id === tabId) {
                    tabBtn.style.background = '#1e1e1e';
                    tabBtn.style.color = '#cccccc';
                } else {
                    tabBtn.style.background = 'transparent';
                    tabBtn.style.color = '#969696';
                }
            }
            
            if (tab.element) {
                tab.element.style.display = 'none';
            }
            tab.isActive = false;
        });

        // Activate selected tab
        const selectedTab = this.tabs.get(tabId);
        if (selectedTab) {
            if (selectedTab.element) {
                selectedTab.element.style.display = 'block';
            }
            selectedTab.isActive = true;
            this.activeTabId = tabId;
        }
    }

    private closeTab(tabId: string): void {
        // Don't allow closing the System Log tab
        if (tabId === 'system-log') return;

        const tab = this.tabs.get(tabId);
        if (tab) {
            // Remove tab button
            const tabIndex = Array.from(this.tabs.keys()).indexOf(tabId);
            const tabBtn = this.tabsContainer.querySelector(`button:nth-child(${tabIndex + 1})`);
            if (tabBtn) {
                tabBtn.remove();
            }

            // Remove content
            if (tab.element) {
                tab.element.remove();
            }

            // Remove from map
            this.tabs.delete(tabId);

            // If this was the active tab, switch to system log
            if (this.activeTabId === tabId) {
                this.switchToTab('system-log');
            }
        }
    }

    private startConsoleCapture(logElement: HTMLElement): void {
        // Capture console.log output and add to System Log
        const originalLog = console.log;
        const originalError = console.error;
        const originalWarn = console.warn;

        // Override console methods to capture output
        console.log = (...args: any[]) => {
            this.addLogEntry(logElement, 'INFO', args.join(' '));
            originalLog.apply(console, args);
        };

        console.error = (...args: any[]) => {
            this.addLogEntry(logElement, 'ERROR', args.join(' '));
            originalError.apply(console, args);
        };

        console.warn = (...args: any[]) => {
            this.addLogEntry(logElement, 'WARN', args.join(' '));
            originalWarn.apply(console, args);
        };
    }

    private addLogEntry(element: HTMLElement, level: string, message: string): void {
        // Only add to System Log if it's the active tab
        const systemLogTab = this.tabs.get('system-log');
        if (!systemLogTab || !systemLogTab.element) return;

        const entry = document.createElement('div');
        const timestamp = new Date().toLocaleTimeString();
        
        let color = '#cccccc';
        switch (level) {
            case 'ERROR': color = '#f44747'; break;
            case 'WARN': color = '#dcdcaa'; break;
            case 'INFO': color = '#569cd6'; break;
        }
        
        entry.style.color = color;
        entry.textContent = `[${timestamp}] [${level}] ${message}`;
        
        systemLogTab.element.appendChild(entry);
        
        // Auto-scroll to bottom
        systemLogTab.element.scrollTop = systemLogTab.element.scrollHeight;
    }
}

// Add blinking cursor animation
const style = document.createElement('style');
style.textContent = `
    @keyframes blink {
        0%, 50% { opacity: 1; }
        51%, 100% { opacity: 0; }
    }
`;
document.head.appendChild(style);