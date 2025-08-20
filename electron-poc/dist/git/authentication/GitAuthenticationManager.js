"use strict";
/**
 * Git Authentication Manager
 * Central manager for all Git authentication operations
 * Based on VS Code's proven architecture
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
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
exports.GitAuthenticationManager = void 0;
const child_process_1 = require("child_process");
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const crypto = __importStar(require("crypto"));
class GitAuthenticationManager {
    constructor(options = {}) {
        var _a, _b, _c, _d;
        this.credentialProviders = new Map();
        this.credentialCache = new Map();
        this.isInitialized = false;
        this.options = {
            enableCache: (_a = options.enableCache) !== null && _a !== void 0 ? _a : true,
            cacheDuration: (_b = options.cacheDuration) !== null && _b !== void 0 ? _b : 60000,
            useSystemCredentialManager: (_c = options.useSystemCredentialManager) !== null && _c !== void 0 ? _c : true,
            enableOAuth: (_d = options.enableOAuth) !== null && _d !== void 0 ? _d : true,
        };
        // Set up paths
        const tempDir = os.tmpdir();
        const sessionId = crypto.randomBytes(8).toString('hex');
        this.pipePath = path.join(tempDir, `git-askpass-${sessionId}.pipe`);
        this.askpassPath = path.join(__dirname, 'askpass.sh');
        this.sshAskpassPath = path.join(__dirname, 'ssh-askpass.sh');
        this.askpassMainPath = path.join(__dirname, 'askpass-main.js');
    }
    /**
     * Initialize the authentication system
     */
    initialize() {
        return __awaiter(this, void 0, void 0, function* () {
            if (this.isInitialized)
                return;
            console.log('[GitAuth] Initializing authentication system...');
            // Create askpass scripts
            yield this.createAskpassScripts();
            // Set up IPC handlers
            this.setupIpcHandlers();
            // Register default credential providers
            yield this.registerDefaultProviders();
            this.isInitialized = true;
            console.log('[GitAuth] Authentication system initialized');
        });
    }
    /**
     * Create the askpass scripts that Git will use
     */
    createAskpassScripts() {
        return __awaiter(this, void 0, void 0, function* () {
            // Create askpass.sh for HTTPS
            const askpassScript = `#!/bin/bash
# Git askpass script for HTTPS authentication
PIPE=$(mktemp -u)
mkfifo "$PIPE"
export ELECTRON_RUN_AS_NODE="1"
export VSCODE_GIT_ASKPASS_PIPE="$PIPE"
export VSCODE_GIT_ASKPASS_TYPE="https"
"${process.execPath}" "${this.askpassMainPath}" "$@"
cat "$PIPE"
rm "$PIPE"
`;
            // Create ssh-askpass.sh for SSH
            const sshAskpassScript = `#!/bin/bash
# SSH askpass script
PIPE=$(mktemp -u)
mkfifo "$PIPE"
export ELECTRON_RUN_AS_NODE="1"
export VSCODE_GIT_ASKPASS_PIPE="$PIPE"
export VSCODE_GIT_ASKPASS_TYPE="ssh"
"${process.execPath}" "${this.askpassMainPath}" "$@"
cat "$PIPE"
rm "$PIPE"
`;
            // Write scripts
            yield fs.promises.writeFile(this.askpassPath, askpassScript, { mode: 0o755 });
            yield fs.promises.writeFile(this.sshAskpassPath, sshAskpassScript, { mode: 0o755 });
            // Create askpass-main.js
            yield this.createAskpassMain();
        });
    }
    /**
     * Create the main askpass handler script
     */
    createAskpassMain() {
        return __awaiter(this, void 0, void 0, function* () {
            const askpassMainScript = `
const fs = require('fs');
const { ipcRenderer } = require('electron');

// Parse command line arguments
const args = process.argv.slice(2);
const prompt = args[0] || '';

// Get pipe path from environment
const pipePath = process.env.VSCODE_GIT_ASKPASS_PIPE;
const askpassType = process.env.VSCODE_GIT_ASKPASS_TYPE;

if (!pipePath) {
  console.error('No pipe path specified');
  process.exit(1);
}

// Parse the prompt to determine what's being requested
let request = {
  type: 'unknown',
  prompt: prompt,
  askpassType: askpassType
};

if (prompt.includes('Username')) {
  request.type = 'username';
  const match = prompt.match(/Username for '([^']+)':/);
  if (match) request.host = match[1];
} else if (prompt.includes('Password')) {
  request.type = 'password';
  const match = prompt.match(/Password for '([^']+)':/);
  if (match) {
    const url = match[1];
    const urlMatch = url.match(/https?:\\/\\/([^@]+)@(.+)/);
    if (urlMatch) {
      request.username = urlMatch[1];
      request.host = 'https://' + urlMatch[2];
    } else {
      request.host = url;
    }
  }
} else if (prompt.includes('passphrase')) {
  request.type = 'ssh-passphrase';
  const match = prompt.match(/Enter passphrase for key '([^']+)':/);
  if (match) request.keyPath = match[1];
}

// Send request to main process and wait for response
process.parentPort.once('message', (response) => {
  // Write response to pipe
  fs.writeFileSync(pipePath, response || '');
  process.exit(0);
});

// Send request
process.parentPort.postMessage(request);
`;
            yield fs.promises.writeFile(this.askpassMainPath, askpassMainScript);
        });
    }
    /**
     * Set up IPC handlers for credential requests
     */
    setupIpcHandlers() {
        // Only register handlers once globally
        if (GitAuthenticationManager.ipcHandlersRegistered) {
            console.log('[GitAuth] IPC handlers already registered, skipping...');
            return;
        }
        // Handle askpass requests from Git processes
        electron_1.ipcMain.handle('git-auth-request', (event, request) => __awaiter(this, void 0, void 0, function* () {
            console.log('[GitAuth] Received credential request:', request.type);
            return yield this.handleCredentialRequest(request);
        }));
        // Handle credential storage requests
        electron_1.ipcMain.handle('git-auth-store', (event, host, credential) => __awaiter(this, void 0, void 0, function* () {
            this.storeCredential(host, credential);
            return { success: true };
        }));
        // Handle credential removal requests
        electron_1.ipcMain.handle('git-auth-remove', (event, host) => __awaiter(this, void 0, void 0, function* () {
            this.removeCredential(host);
            return { success: true };
        }));
        GitAuthenticationManager.ipcHandlersRegistered = true;
    }
    /**
     * Register default credential providers
     */
    registerDefaultProviders() {
        return __awaiter(this, void 0, void 0, function* () {
            // System credential manager provider
            if (this.options.useSystemCredentialManager) {
                const { SystemCredentialProvider } = yield Promise.resolve().then(() => __importStar(require('./providers/SystemCredentialProvider')));
                this.registerProvider(new SystemCredentialProvider());
            }
            // GitHub OAuth provider
            if (this.options.enableOAuth) {
                const { GitHubOAuthProvider } = yield Promise.resolve().then(() => __importStar(require('./providers/GitHubOAuthProvider')));
                this.registerProvider(new GitHubOAuthProvider());
            }
            // Generic credential provider (prompts user)
            const { GenericCredentialProvider } = yield Promise.resolve().then(() => __importStar(require('./providers/GenericCredentialProvider')));
            this.registerProvider(new GenericCredentialProvider());
        });
    }
    /**
     * Register a credential provider
     */
    registerProvider(provider) {
        console.log(`[GitAuth] Registering credential provider: ${provider.name}`);
        this.credentialProviders.set(provider.id, provider);
    }
    /**
     * Handle a credential request
     */
    handleCredentialRequest(request) {
        return __awaiter(this, void 0, void 0, function* () {
            // Check cache first
            if (this.options.enableCache && request.host) {
                const cached = this.getCredentialFromCache(request.host);
                if (cached) {
                    console.log('[GitAuth] Using cached credential');
                    return { success: true, credential: cached };
                }
            }
            // Try each provider
            for (const provider of this.credentialProviders.values()) {
                if (provider.canHandle(request)) {
                    console.log(`[GitAuth] Trying provider: ${provider.name}`);
                    try {
                        const credential = yield provider.getCredentials(request);
                        if (credential) {
                            // Cache the credential
                            if (this.options.enableCache && request.host) {
                                this.storeCredential(request.host, credential);
                            }
                            return { success: true, credential };
                        }
                    }
                    catch (error) {
                        console.error(`[GitAuth] Provider ${provider.name} failed:`, error);
                    }
                }
            }
            return {
                success: false,
                error: 'No credential provider could handle this request'
            };
        });
    }
    /**
     * Get credential from cache
     */
    getCredentialFromCache(host) {
        const cached = this.credentialCache.get(host);
        if (cached && Date.now() - cached.timestamp < this.options.cacheDuration) {
            return cached;
        }
        // Remove expired credential
        if (cached) {
            this.credentialCache.delete(host);
        }
        return null;
    }
    /**
     * Store credential in cache
     */
    storeCredential(host, credential) {
        credential.timestamp = Date.now();
        this.credentialCache.set(host, credential);
    }
    /**
     * Remove credential from cache
     */
    removeCredential(host) {
        this.credentialCache.delete(host);
    }
    /**
     * Get environment variables for Git operations
     */
    getGitEnvironment() {
        return Object.assign({ GIT_ASKPASS: this.askpassPath, SSH_ASKPASS: this.sshAskpassPath, GIT_TERMINAL_PROMPT: '0', SSH_ASKPASS_REQUIRE: 'force', ELECTRON_RUN_AS_NODE: '1', VSCODE_GIT_ASKPASS_NODE: process.execPath, VSCODE_GIT_ASKPASS_MAIN: this.askpassMainPath, VSCODE_GIT_ASKPASS_PIPE: this.pipePath }, process.env);
    }
    /**
     * Execute a Git command with authentication support
     */
    executeGitCommand(args, cwd) {
        return __awaiter(this, void 0, void 0, function* () {
            return new Promise((resolve, reject) => {
                const env = this.getGitEnvironment();
                console.log('[GitAuth] Executing git command with auth:', args.join(' '));
                const gitProcess = (0, child_process_1.spawn)('git', args, {
                    cwd,
                    env,
                    windowsHide: true,
                });
                let stdout = '';
                let stderr = '';
                gitProcess.stdout.on('data', (data) => {
                    stdout += data.toString();
                });
                gitProcess.stderr.on('data', (data) => {
                    stderr += data.toString();
                });
                gitProcess.on('error', (error) => {
                    console.error('[GitAuth] Git process error:', error);
                    reject(error);
                });
                gitProcess.on('close', (code) => {
                    resolve({ stdout, stderr, code: code || 0 });
                });
            });
        });
    }
    /**
     * Clean up resources
     */
    dispose() {
        // Clean up cache
        this.credentialCache.clear();
        // Clean up temp files
        try {
            if (fs.existsSync(this.pipePath)) {
                fs.unlinkSync(this.pipePath);
            }
            if (fs.existsSync(this.askpassPath)) {
                fs.unlinkSync(this.askpassPath);
            }
            if (fs.existsSync(this.sshAskpassPath)) {
                fs.unlinkSync(this.sshAskpassPath);
            }
            if (fs.existsSync(this.askpassMainPath)) {
                fs.unlinkSync(this.askpassMainPath);
            }
        }
        catch (error) {
            console.error('[GitAuth] Error cleaning up:', error);
        }
    }
}
exports.GitAuthenticationManager = GitAuthenticationManager;
GitAuthenticationManager.ipcHandlersRegistered = false;
//# sourceMappingURL=GitAuthenticationManager.js.map