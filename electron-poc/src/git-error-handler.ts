/**
 * Git Error Handler
 * Provides intelligent error handling and recovery options for Git operations
 */

export interface GitErrorOptions {
  title: string;
  message: string;
  actions?: ErrorAction[];
  type: 'size-limit' | 'auth' | 'network' | 'conflict' | 'general';
}

export interface ErrorAction {
  label: string;
  action: () => void | Promise<void>;
  primary?: boolean;
}

export class GitErrorHandler {
  /**
   * Parse git error messages and return structured error information
   */
  static parseError(error: Error | string): GitErrorOptions {
    const errorMessage = typeof error === 'string' ? error : error.message || '';
    
    // Check for 2GB pack size limit
    if (errorMessage.includes('pack exceeds maximum allowed size') || 
        errorMessage.includes('2.00 GiB') ||
        errorMessage.includes('2147483648')) {
      return {
        type: 'size-limit',
        title: 'Repository Size Limit Exceeded',
        message: 'GitHub has a 2GB pack size limit. Your repository changes exceed this limit.',
        actions: [
          {
            label: 'Try Chunked Push',
            action: async () => {
              // Try pushing with chunked strategy
              if (window.scmView?.pushWithChunks) {
                await window.scmView.pushWithChunks();
              } else {
                console.error('Chunked push not available');
              }
            },
            primary: true
          },
          {
            label: 'Analyze Repository',
            action: async () => {
              // Call through SCM view which has the analyzer
              if (window.scmView?.analyzeRepository) {
                await window.scmView.analyzeRepository();
              } else {
                console.error('Repository analyzer not available');
              }
            }
          },
          {
            label: 'View Solutions',
            action: () => {
              this.showSizeLimitSolutions();
            }
          },
          {
            label: 'Learn About Git LFS',
            action: () => {
              this.openGitLFSDocumentation();
            }
          }
        ]
      };
    }
    
    // Check for authentication errors
    if (errorMessage.includes('authentication failed') || 
        errorMessage.includes('Permission denied') ||
        errorMessage.includes('could not read Username')) {
      return {
        type: 'auth',
        title: 'Authentication Failed',
        message: 'Unable to authenticate with the remote repository.',
        actions: [
          {
            label: 'Configure Credentials',
            action: () => {
              this.openCredentialConfig();
            },
            primary: true
          },
          {
            label: 'Setup SSH Key',
            action: () => {
              this.openSSHSetup();
            }
          }
        ]
      };
    }
    
    // Check for network errors
    if (errorMessage.includes('unable to access') || 
        errorMessage.includes('Could not resolve host') ||
        errorMessage.includes('Connection refused')) {
      return {
        type: 'network',
        title: 'Network Error',
        message: 'Unable to connect to the remote repository.',
        actions: [
          {
            label: 'Retry',
            action: () => {
              // Caller should handle retry
            },
            primary: true
          },
          {
            label: 'Check Proxy Settings',
            action: () => {
              this.openProxySettings();
            }
          }
        ]
      };
    }
    
    // Check for merge conflicts
    if (errorMessage.includes('merge conflict') || 
        errorMessage.includes('fix conflicts')) {
      return {
        type: 'conflict',
        title: 'Merge Conflict',
        message: 'There are merge conflicts that need to be resolved.',
        actions: [
          {
            label: 'Open Merge Tool',
            action: () => {
              // TODO: Implement merge tool
              console.log('Merge tool not yet implemented');
              alert('Please resolve conflicts manually in your editor');
            },
            primary: true
          }
        ]
      };
    }
    
    // Default error
    return {
      type: 'general',
      title: 'Git Operation Failed',
      message: errorMessage || 'An unknown error occurred',
      actions: []
    };
  }
  
  /**
   * Show solutions dialog for repository size issues
   */
  static showSizeLimitSolutions() {
    const solutions = `
# Solutions for Repository Size Limit

## 1. Use Git LFS (Large File Storage)
Git LFS is designed to handle large files efficiently:
- Move large files to LFS: \`git lfs track "*.psd"\`
- Migrate existing files: \`git lfs migrate import --include="*.zip"\`

## 2. Clean Repository History
Remove large files from history:
- Use BFG Repo-Cleaner: \`bfg --strip-blobs-bigger-than 100M\`
- Run garbage collection: \`git gc --aggressive --prune=now\`

## 3. Split Large Commits
Break up your changes into smaller commits:
- Reset to previous commit: \`git reset HEAD~1\`
- Stage and commit files in smaller batches

## 4. Remove Unnecessary Files
- Delete build artifacts and dependencies
- Add them to .gitignore
- Clean working directory: \`git clean -fd\`

## 5. Archive Old History
- Create a new repository for current work
- Archive old repository for reference
`;
    
    // Create a dialog to show solutions
    const dialog = document.createElement('div');
    dialog.className = 'git-solutions-dialog';
    dialog.innerHTML = `
      <div class="dialog-overlay"></div>
      <div class="dialog-content">
        <div class="dialog-header">
          <h2>Repository Size Solutions</h2>
          <button class="dialog-close">×</button>
        </div>
        <div class="dialog-body">
          <pre>${solutions}</pre>
        </div>
        <div class="dialog-footer">
          <button class="btn-primary copy-solutions">Copy Solutions</button>
          <button class="btn-secondary close-dialog">Close</button>
        </div>
      </div>
    `;
    
    document.body.appendChild(dialog);
    
    // Add event listeners
    dialog.querySelector('.dialog-close')?.addEventListener('click', () => {
      document.body.removeChild(dialog);
    });
    
    dialog.querySelector('.close-dialog')?.addEventListener('click', () => {
      document.body.removeChild(dialog);
    });
    
    dialog.querySelector('.copy-solutions')?.addEventListener('click', () => {
      navigator.clipboard.writeText(solutions);
      const btn = dialog.querySelector('.copy-solutions') as HTMLButtonElement;
      btn.textContent = 'Copied!';
      setTimeout(() => {
        btn.textContent = 'Copy Solutions';
      }, 2000);
    });
  }
  
  /**
   * Open Git LFS documentation
   */
  static openGitLFSDocumentation() {
    window.electronAPI?.openExternal('https://git-lfs.github.com/');
  }
  
  /**
   * Open credential configuration
   */
  static openCredentialConfig() {
    // Show credential configuration dialog
    const dialog = document.createElement('div');
    dialog.className = 'git-credential-dialog';
    dialog.innerHTML = `
      <div class="dialog-overlay"></div>
      <div class="dialog-content">
        <div class="dialog-header">
          <h2>Configure Git Credentials</h2>
          <button class="dialog-close">×</button>
        </div>
        <div class="dialog-body">
          <p>Choose your authentication method:</p>
          <div class="auth-options">
            <button class="auth-option" data-method="https">
              <h3>HTTPS with Token</h3>
              <p>Use a personal access token for authentication</p>
            </button>
            <button class="auth-option" data-method="ssh">
              <h3>SSH Key</h3>
              <p>Use SSH keys for secure authentication</p>
            </button>
          </div>
        </div>
      </div>
    `;
    
    document.body.appendChild(dialog);
    
    dialog.querySelector('.dialog-close')?.addEventListener('click', () => {
      document.body.removeChild(dialog);
    });
    
    dialog.querySelectorAll('.auth-option').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const method = (e.currentTarget as HTMLElement).dataset.method;
        if (method === 'https') {
          window.electronAPI?.openExternal('https://github.com/settings/tokens');
        } else {
          this.openSSHSetup();
        }
        document.body.removeChild(dialog);
      });
    });
  }
  
  /**
   * Open SSH setup guide
   */
  static openSSHSetup() {
    window.electronAPI?.openExternal('https://docs.github.com/en/authentication/connecting-to-github-with-ssh');
  }
  
  /**
   * Open proxy settings
   */
  static openProxySettings() {
    // Show proxy configuration info
    const info = `
Configure Git proxy settings:

# Set proxy
git config --global http.proxy http://proxy.example.com:8080
git config --global https.proxy https://proxy.example.com:8080

# Unset proxy
git config --global --unset http.proxy
git config --global --unset https.proxy
`;
    
    alert(info);
  }
}

// Add CSS for dialogs
const style = document.createElement('style');
style.textContent = `
.git-solutions-dialog,
.git-credential-dialog {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
}

.dialog-content {
  position: relative;
  background: var(--vscode-editor-background, #1e1e1e);
  border: 1px solid var(--vscode-panel-border, #444);
  border-radius: 6px;
  max-width: 600px;
  max-height: 80vh;
  width: 90%;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--vscode-panel-border, #444);
}

.dialog-header h2 {
  margin: 0;
  font-size: 18px;
  color: var(--vscode-foreground, #ccc);
}

.dialog-close {
  background: transparent;
  border: none;
  color: var(--vscode-foreground, #ccc);
  font-size: 24px;
  cursor: pointer;
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.dialog-close:hover {
  background: var(--vscode-toolbar-hoverBackground, rgba(90, 93, 94, 0.31));
}

.dialog-body {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  color: var(--vscode-foreground, #ccc);
}

.dialog-body pre {
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
  margin: 0;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--vscode-panel-border, #444);
}

.btn-primary,
.btn-secondary {
  padding: 6px 14px;
  border-radius: 4px;
  border: none;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: var(--vscode-button-background, #0e639c);
  color: var(--vscode-button-foreground, #fff);
}

.btn-primary:hover {
  background: var(--vscode-button-hoverBackground, #1177bb);
}

.btn-secondary {
  background: var(--vscode-button-secondaryBackground, #3a3d41);
  color: var(--vscode-button-secondaryForeground, #ccc);
}

.btn-secondary:hover {
  background: var(--vscode-button-secondaryHoverBackground, #45494e);
}

.auth-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 16px;
}

.auth-option {
  padding: 16px;
  border: 1px solid var(--vscode-panel-border, #444);
  border-radius: 6px;
  background: var(--vscode-editor-background, #1e1e1e);
  text-align: left;
  cursor: pointer;
  transition: all 0.2s;
}

.auth-option:hover {
  background: var(--vscode-list-hoverBackground, #2a2d2e);
  border-color: var(--vscode-focusBorder, #007acc);
}

.auth-option h3 {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: var(--vscode-foreground, #ccc);
}

.auth-option p {
  margin: 0;
  font-size: 12px;
  color: var(--vscode-descriptionForeground, #999);
}
`;
document.head.appendChild(style);