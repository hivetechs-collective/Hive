"use strict";
/**
 * Native Terminal Integration for macOS
 * Opens commands in the system's native terminal application
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
exports.NativeTerminal = void 0;
const child_process_1 = require("child_process");
class NativeTerminal {
    /**
     * Open a command in Terminal.app with AppleScript
     */
    static openInTerminal(command, workingDirectory) {
        return new Promise((resolve, reject) => {
            const cwd = workingDirectory || process.env.HOME || '/Users/veronelazio';
            // AppleScript to open Terminal and run command
            const script = `
                tell application "Terminal"
                    activate
                    set newTab to do script "cd '${cwd}' && ${command}"
                end tell
            `;
            (0, child_process_1.exec)(`osascript -e "${script.replace(/"/g, '\\"')}"`, (error) => {
                if (error) {
                    reject(error);
                }
                else {
                    resolve();
                }
            });
        });
    }
    /**
     * Open in iTerm2 if available
     */
    static openInITerm(command, workingDirectory) {
        return new Promise((resolve, reject) => {
            const cwd = workingDirectory || process.env.HOME || '/Users/veronelazio';
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
            (0, child_process_1.exec)(`osascript -e "${script.replace(/"/g, '\\"')}"`, (error) => {
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
    static getAvailableTerminals() {
        return __awaiter(this, void 0, void 0, function* () {
            const terminals = [];
            // Check for common terminal apps
            const checkApp = (appName) => {
                return new Promise((resolve) => {
                    (0, child_process_1.exec)(`osascript -e 'tell application "System Events" to get name of every process whose name is "${appName}"'`, (error) => {
                        resolve(!error);
                    });
                });
            };
            if (yield checkApp('Terminal'))
                terminals.push('Terminal');
            if (yield checkApp('iTerm'))
                terminals.push('iTerm2');
            if (yield checkApp('Hyper'))
                terminals.push('Hyper');
            if (yield checkApp('Alacritty'))
                terminals.push('Alacritty');
            return terminals;
        });
    }
}
exports.NativeTerminal = NativeTerminal;
//# sourceMappingURL=native-terminal.js.map