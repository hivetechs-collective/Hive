/**
 * CLI Tool Detector
 * Checks for installed AI CLI tools and provides information about them
 * Note: This runs in the renderer process, actual detection happens in main process via IPC
 */

export interface CliToolInfo {
    id: string;
    name: string;
    command: string;
    installed: boolean;
    version?: string;
    path?: string;
}

export interface CliToolStatus {
    id: string;
    name: string;
    installed: boolean;
    version?: string;
    path?: string;
}

// Placeholder function for Claude Code detection
export async function detectClaudeCode(): Promise<CliToolStatus> {
    // This will be replaced with IPC call to main process
    return {
        id: 'claude-code',
        name: 'Claude Code',
        installed: false
    };
}

export class CliToolDetector {
    private tools: Map<string, CliToolInfo> = new Map();
    
    constructor() {
        this.initializeTools();
    }
    
    /**
     * Initialize the list of known tools
     */
    private initializeTools(): void {
        const toolDefinitions: Array<{id: string, name: string, command: string}> = [
            { id: 'claude-code', name: 'Claude Code', command: 'claude' },
            { id: 'aider', name: 'Aider', command: 'aider' },
            { id: 'cursor', name: 'Cursor', command: 'cursor' },
            { id: 'continue', name: 'Continue', command: 'continue' },
            { id: 'codewhisperer', name: 'Amazon Q', command: 'aws' },
            { id: 'cody', name: 'Cody', command: 'cody' },
            { id: 'qwen-code', name: 'Qwen Code', command: 'qwen' },
            { id: 'gemini-cli', name: 'Gemini CLI', command: 'gemini' }
        ];
        
        toolDefinitions.forEach(tool => {
            this.tools.set(tool.id, {
                ...tool,
                installed: false
            });
        });
    }
    
    /**
     * Check if a specific tool is installed (via IPC to main process)
     */
    async checkTool(toolId: string): Promise<CliToolInfo | null> {
        const tool = this.tools.get(toolId);
        if (!tool) return null;
        
        // This would normally call IPC, but for now just return the tool info
        // The actual detection happens in the AI CLI Tools panel
        return tool;
    }
    
    /**
     * Check all known tools
     */
    async checkAllTools(): Promise<Map<string, CliToolInfo>> {
        // For now, just return the tools map
        // Actual detection happens in the AI CLI Tools panel via IPC
        return this.tools;
    }
    
    /**
     * Get info for a specific tool
     */
    getToolInfo(toolId: string): CliToolInfo | undefined {
        return this.tools.get(toolId);
    }
    
    /**
     * Get all tools
     */
    getAllTools(): Map<string, CliToolInfo> {
        return this.tools;
    }
    
    /**
     * Quick check if tool exists (synchronous, from cache)
     */
    isToolInstalled(toolId: string): boolean {
        const tool = this.tools.get(toolId);
        return tool ? tool.installed : false;
    }
}

// Export singleton instance for renderer process
export const cliToolDetector = new CliToolDetector();

// Make it available globally in renderer
if (typeof window !== 'undefined') {
    (window as any).cliToolDetector = cliToolDetector;
}