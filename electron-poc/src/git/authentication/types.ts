/**
 * Git Authentication Types
 * Core types for the Git authentication system
 */

export interface Credential {
  username?: string;
  password?: string;
  token?: string;
  timestamp: number;
}

export interface CredentialRequest {
  type: 'username' | 'password' | 'token' | 'ssh-passphrase' | 'host-verification';
  host?: string;
  protocol?: string;
  path?: string;
  username?: string;
  fingerprint?: string;
  keyPath?: string;
}

export interface CredentialProvider {
  id: string;
  name: string;
  
  /**
   * Check if this provider can handle the request
   */
  canHandle(request: CredentialRequest): boolean;
  
  /**
   * Get credentials for the request
   */
  getCredentials(request: CredentialRequest): Promise<Credential | null>;
}

export interface AuthenticationResult {
  success: boolean;
  credential?: Credential;
  error?: string;
}

export interface GitAuthOptions {
  /**
   * Enable credential caching
   */
  enableCache?: boolean;
  
  /**
   * Cache duration in milliseconds
   */
  cacheDuration?: number;
  
  /**
   * Enable system credential manager integration
   */
  useSystemCredentialManager?: boolean;
  
  /**
   * Enable OAuth for supported providers
   */
  enableOAuth?: boolean;
}

export interface GitEnvironment {
  GIT_ASKPASS: string;
  SSH_ASKPASS: string;
  GIT_TERMINAL_PROMPT: string;
  ELECTRON_RUN_AS_NODE?: string;
  VSCODE_GIT_ASKPASS_NODE?: string;
  VSCODE_GIT_ASKPASS_MAIN?: string;
  VSCODE_GIT_ASKPASS_PIPE?: string;
  SSH_ASKPASS_REQUIRE?: string;
  [key: string]: string | undefined;
}