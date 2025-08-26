/**
 * Native Terminal Integration for macOS
 * Opens commands in the system's native terminal application
 */

import { exec } from 'child_process';
import * as path from 'path';

export class NativeTerminal {
    /**
     * Open a command in Terminal.app with AppleScript
     */
    static openInTerminal(command: string, workingDirectory?: string): Promise<void> {
        return new Promise((resolve, reject) => {
            const cwd = workingDirectory || process.env.HOME || process.cwd();
            
            // AppleScript to open Terminal and run command
            const script = `
                tell application "Terminal"
                    activate
                    set newTab to do script "cd '${cwd}' && ${command}"
                end tell
            `;
            
            exec(`osascript -e "${script.replace(/"/g, '\\"')}"`, (error) => {
                if (error) {
                    reject(error);
                } else {
                    resolve();
                }
            });
        });
    }
    
    /**
     * Open in iTerm2 if available
     */
    static openInITerm(command: string, workingDirectory?: string): Promise<void> {
        return new Promise((resolve, reject) => {
            const cwd = workingDirectory || process.env.HOME || process.cwd();
            
            // AppleScript for iTerm2
            const script = `
                tell application "iTerm"
                    activate
                    tell current window
                        create tab with default profile
                        tell current session
                            write text "cd '${cwd}'"
                            write text "${command}"
                        end tell
                    end tell
                end tell
            `;
            
            exec(`osascript -e "${script.replace(/"/g, '\\"')}"`, (error) => {
                if (error) {
                    // Fall back to Terminal.app
                    return NativeTerminal.openInTerminal(command, workingDirectory)
                        .then(resolve)
                        .catch(reject);
                }
                resolve();
            });
        });
    }
    
    /**
     * Check which terminal apps are available
     */
    static async getAvailableTerminals(): Promise<string[]> {
        const terminals: string[] = [];
        
        // Check for common terminal apps
        const checkApp = (appName: string): Promise<boolean> => {
            return new Promise((resolve) => {
                exec(`osascript -e 'tell application "System Events" to get name of every process whose name is "${appName}"'`, (error) => {
                    resolve(!error);
                });
            });
        };
        
        if (await checkApp('Terminal')) terminals.push('Terminal');
        if (await checkApp('iTerm')) terminals.push('iTerm2');
        if (await checkApp('Hyper')) terminals.push('Hyper');
        if (await checkApp('Alacritty')) terminals.push('Alacritty');
        
        return terminals;
    }
}