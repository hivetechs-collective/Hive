/**
 * CLI Tool Detector
 * Checks for installed AI CLI tools and provides information about them
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

export interface CliToolInfo {
    id: string;
    name: string;
    command: string;
    installed: boolean;
    version?: string;
    path?: string;
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
     * Check if a specific tool is installed
     */
    async checkTool(toolId: string): Promise<CliToolInfo | null> {
        const tool = this.tools.get(toolId);
        if (!tool) return null;
        
        try {
            // Try to find the command in PATH
            const { stdout: pathOutput } = await execAsync(`which ${tool.command}`);
            if (pathOutput) {
                tool.path = pathOutput.trim();
                tool.installed = true;
                
                // Try to get version
                try {
                    const versionCmd = `${tool.command} --version`;
                    const { stdout: versionOutput } = await execAsync(versionCmd);
                    tool.version = versionOutput.trim().split('\n')[0];
                } catch {
                    // Some tools might not support --version
                    tool.version = 'unknown';
                }
            }
        } catch {
            // Command not found
            tool.installed = false;
        }
        
        return tool;
    }
    
    /**
     * Check all known tools
     */
    async checkAllTools(): Promise<Map<string, CliToolInfo>> {
        const checkPromises = Array.from(this.tools.keys()).map(id => this.checkTool(id));
        await Promise.all(checkPromises);
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