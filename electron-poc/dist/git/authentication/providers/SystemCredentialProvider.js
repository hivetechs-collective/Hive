"use strict";
/**
 * System Credential Provider
 * Integrates with the operating system's credential manager
 * - macOS: Keychain
 * - Windows: Credential Manager
 * - Linux: Secret Service API / libsecret
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
exports.SystemCredentialProvider = void 0;
const child_process_1 = require("child_process");
const util_1 = require("util");
const execAsync = (0, util_1.promisify)(child_process_1.exec);
class SystemCredentialProvider {
    constructor() {
        this.id = 'system';
        this.name = 'System Credential Manager';
        this.platform = process.platform;
    }
    canHandle(request) {
        // Only handle HTTPS credentials with a host
        return request.host !== undefined &&
            (request.type === 'username' || request.type === 'password');
    }
    getCredentials(request) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!request.host)
                return null;
            try {
                switch (this.platform) {
                    case 'darwin':
                        return yield this.getFromKeychain(request);
                    case 'win32':
                        return yield this.getFromWindowsCredentialManager(request);
                    case 'linux':
                        return yield this.getFromLibsecret(request);
                    default:
                        console.log(`[SystemCredentialProvider] Unsupported platform: ${this.platform}`);
                        return null;
                }
            }
            catch (error) {
                console.error('[SystemCredentialProvider] Error getting credentials:', error);
                return null;
            }
        });
    }
    /**
     * Get credentials from macOS Keychain
     */
    getFromKeychain(request) {
        return __awaiter(this, void 0, void 0, function* () {
            const service = this.getServiceName(request.host);
            try {
                // Try to get existing credentials
                const { stdout } = yield execAsync(`security find-internet-password -s "${service}" -w 2>/dev/null`);
                const password = stdout.trim();
                // Get username
                const { stdout: userOutput } = yield execAsync(`security find-internet-password -s "${service}" | grep "acct" | cut -d '"' -f 4`);
                const username = userOutput.trim();
                if (password && username) {
                    console.log(`[SystemCredentialProvider] Found credentials in Keychain for ${service}`);
                    return {
                        username,
                        password,
                        timestamp: Date.now(),
                    };
                }
            }
            catch (error) {
                // Credentials not found or error accessing keychain
                console.log(`[SystemCredentialProvider] No credentials in Keychain for ${service}`);
            }
            return null;
        });
    }
    /**
     * Get credentials from Windows Credential Manager
     */
    getFromWindowsCredentialManager(request) {
        return __awaiter(this, void 0, void 0, function* () {
            const target = this.getServiceName(request.host);
            try {
                // Use cmdkey to check for stored credentials
                const { stdout } = yield execAsync(`cmdkey /list:${target}`);
                if (stdout.includes(target)) {
                    // Credentials exist, but we need to use a different method to retrieve them
                    // Windows doesn't allow direct password retrieval via cmdkey
                    // We would need to use Windows Credential Manager API via native module
                    console.log(`[SystemCredentialProvider] Found credentials in Windows Credential Manager for ${target}`);
                    // For now, return null and let the user re-enter
                    // In production, you'd use a native module like node-windows-credential-manager
                    return null;
                }
            }
            catch (error) {
                console.log(`[SystemCredentialProvider] No credentials in Windows Credential Manager for ${target}`);
            }
            return null;
        });
    }
    /**
     * Get credentials from Linux Secret Service (libsecret)
     */
    getFromLibsecret(request) {
        return __awaiter(this, void 0, void 0, function* () {
            const service = this.getServiceName(request.host);
            try {
                // Use secret-tool to get password
                const { stdout: password } = yield execAsync(`secret-tool lookup service "${service}" 2>/dev/null`);
                if (password) {
                    // Try to get username (stored as attribute)
                    const { stdout: username } = yield execAsync(`secret-tool search service "${service}" 2>/dev/null | grep "attribute.username" | cut -d '=' -f2`);
                    console.log(`[SystemCredentialProvider] Found credentials in libsecret for ${service}`);
                    return {
                        username: username.trim(),
                        password: password.trim(),
                        timestamp: Date.now(),
                    };
                }
            }
            catch (error) {
                console.log(`[SystemCredentialProvider] No credentials in libsecret for ${service}`);
            }
            return null;
        });
    }
    /**
     * Store credentials in the system credential manager
     */
    storeCredentials(host, credential) {
        return __awaiter(this, void 0, void 0, function* () {
            const service = this.getServiceName(host);
            try {
                switch (this.platform) {
                    case 'darwin':
                        return yield this.storeInKeychain(service, credential);
                    case 'win32':
                        return yield this.storeInWindowsCredentialManager(service, credential);
                    case 'linux':
                        return yield this.storeInLibsecret(service, credential);
                    default:
                        return false;
                }
            }
            catch (error) {
                console.error('[SystemCredentialProvider] Error storing credentials:', error);
                return false;
            }
        });
    }
    /**
     * Store credentials in macOS Keychain
     */
    storeInKeychain(service, credential) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!credential.username || !credential.password)
                return false;
            try {
                // Delete existing entry if it exists
                yield execAsync(`security delete-internet-password -s "${service}" 2>/dev/null`).catch(() => { }); // Ignore error if doesn't exist
                // Add new entry
                yield execAsync(`security add-internet-password -s "${service}" -a "${credential.username}" -w "${credential.password}" -U`);
                console.log(`[SystemCredentialProvider] Stored credentials in Keychain for ${service}`);
                return true;
            }
            catch (error) {
                console.error(`[SystemCredentialProvider] Failed to store in Keychain:`, error);
                return false;
            }
        });
    }
    /**
     * Store credentials in Windows Credential Manager
     */
    storeInWindowsCredentialManager(service, credential) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!credential.username || !credential.password)
                return false;
            try {
                // Use cmdkey to store credentials
                yield execAsync(`cmdkey /add:${service} /user:${credential.username} /pass:${credential.password}`);
                console.log(`[SystemCredentialProvider] Stored credentials in Windows Credential Manager for ${service}`);
                return true;
            }
            catch (error) {
                console.error(`[SystemCredentialProvider] Failed to store in Windows Credential Manager:`, error);
                return false;
            }
        });
    }
    /**
     * Store credentials in Linux Secret Service (libsecret)
     */
    storeInLibsecret(service, credential) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!credential.username || !credential.password)
                return false;
            try {
                // Store password with secret-tool
                yield execAsync(`echo -n "${credential.password}" | secret-tool store --label="${service}" service "${service}" username "${credential.username}"`);
                console.log(`[SystemCredentialProvider] Stored credentials in libsecret for ${service}`);
                return true;
            }
            catch (error) {
                console.error(`[SystemCredentialProvider] Failed to store in libsecret:`, error);
                return false;
            }
        });
    }
    /**
     * Get a consistent service name for the credential store
     */
    getServiceName(host) {
        // Remove protocol if present
        let service = host.replace(/^https?:\/\//, '');
        // Remove path if present
        service = service.split('/')[0];
        // Prefix with our app name
        return `hive-ide:${service}`;
    }
}
exports.SystemCredentialProvider = SystemCredentialProvider;
//# sourceMappingURL=SystemCredentialProvider.js.map