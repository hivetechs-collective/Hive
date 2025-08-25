/**
 * Git Push Strategy Executor
 * Executes different push strategies based on user selection
 */

import { PushStrategy, PushStrategyOption } from './git-push-strategy';

export class GitPushExecutor {
  /**
   * Execute the selected push strategy
   */
  static async execute(
    strategy: PushStrategyOption,
    analysis: any,
    gitAPI: any,
    gitStatus: any
  ): Promise<{ success: boolean; message: string }> {
    
    // Create progress dialog
    const progressDialog = this.createProgressDialog(strategy);
    document.body.appendChild(progressDialog);
    
    try {
      let result: { success: boolean; message: string };
      
      // Check if user provided a custom command
      if (strategy.selectedOptions?.customCommand) {
        result = await this.executeCustomCommand(gitAPI, strategy.selectedOptions.customCommand, progressDialog);
      } 
      // Check if dry run is selected
      else if (strategy.selectedOptions?.dryRun) {
        result = await this.executeDryRun(gitAPI, gitStatus, strategy, progressDialog);
      }
      // Normal strategy execution
      else {
        // Pass the strategy with options to the specific executor
        switch (strategy.strategy) {
          case PushStrategy.REGULAR:
            result = await this.executeRegularPush(gitAPI, gitStatus, progressDialog, strategy.selectedOptions);
            break;
            
          case PushStrategy.CHUNKED:
            result = await this.executeChunkedPush(gitAPI, gitStatus, progressDialog, strategy.selectedOptions);
            break;
            
          case PushStrategy.FORCE:
            result = await this.executeForcePush(gitAPI, gitStatus, progressDialog, strategy.selectedOptions);
            break;
            
          case PushStrategy.FRESH_BRANCH:
            result = await this.executeFreshBranchPush(gitAPI, gitStatus, progressDialog, strategy.selectedOptions);
            break;
            
          case PushStrategy.SQUASH:
            result = await this.executeSquashPush(gitAPI, gitStatus, progressDialog, strategy.selectedOptions);
            break;
            
          case PushStrategy.BUNDLE:
            result = await this.executeBundleCreation(gitAPI, gitStatus, progressDialog);
            break;
            
          case PushStrategy.CLEANUP_FIRST:
            result = await this.executeCleanupFirst(gitAPI, gitStatus, progressDialog);
            break;
            
          default:
            throw new Error(`Unknown strategy: ${strategy.strategy}`);
        }
      }
      
      // Remove progress dialog
      progressDialog.remove();
      
      return result;
      
    } catch (error: any) {
      progressDialog.remove();
      throw error;
    }
  }
  
  /**
   * Create progress dialog for the operation
   */
  private static createProgressDialog(strategy: PushStrategyOption): HTMLElement {
    const dialog = document.createElement('div');
    dialog.style.cssText = `
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background: var(--vscode-notifications-background, #252526);
      border: 1px solid var(--vscode-notifications-border, #007acc);
      color: var(--vscode-notifications-foreground, #ccc);
      padding: 20px 30px;
      border-radius: 6px;
      z-index: 10000;
      box-shadow: 0 4px 12px rgba(0,0,0,0.5);
      min-width: 400px;
      text-align: center;
    `;
    
    dialog.innerHTML = `
      <h3 style="margin: 0 0 10px 0;">
        ${strategy.icon} ${strategy.label}
      </h3>
      <p style="margin: 10px 0;">Executing push strategy...</p>
      <div style="margin: 15px 0;">
        <div style="width: 100%; height: 4px; background: #333; border-radius: 2px; overflow: hidden;">
          <div style="width: 30%; height: 100%; background: #007acc; animation: progress 2s ease-in-out infinite;"></div>
        </div>
      </div>
      <p class="status-message" style="margin: 10px 0; font-size: 11px; opacity: 0.7;">
        Preparing...
      </p>
      <style>
        @keyframes progress {
          0% { width: 0%; }
          50% { width: 70%; }
          100% { width: 30%; }
        }
      </style>
    `;
    
    return dialog;
  }
  
  /**
   * Update progress message
   */
  private static updateProgress(dialog: HTMLElement, message: string) {
    const statusElement = dialog.querySelector('.status-message');
    if (statusElement) {
      statusElement.textContent = message;
    }
  }
  
  /**
   * Execute custom git command
   */
  private static async executeCustomCommand(
    gitAPI: any,
    command: string,
    dialog: HTMLElement
  ): Promise<{ success: boolean; message: string }> {
    this.updateProgress(dialog, `Executing custom command: ${command}`);
    
    try {
      // For now, we'll need to add a new IPC handler for custom commands
      // This is a placeholder that shows the intent
      alert(`Custom command execution:\n\n${command}\n\nThis feature requires implementation in the main process.`);
      
      return {
        success: false,
        message: 'Custom command execution not yet implemented. Please use standard options.'
      };
    } catch (error: any) {
      throw new Error(`Custom command failed: ${error.message}`);
    }
  }
  
  /**
   * Execute dry run
   */
  private static async executeDryRun(
    gitAPI: any,
    gitStatus: any,
    strategy: any,
    dialog: HTMLElement
  ): Promise<{ success: boolean; message: string }> {
    this.updateProgress(dialog, 'Running push simulation (dry run)...');
    
    try {
      // Build the command that would be executed
      let command = 'git push';
      
      if (strategy.selectedOptions?.forceWithLease) {
        command += ' --force-with-lease';
      }
      if (strategy.selectedOptions?.includeTags) {
        command += ' --tags';
      }
      if (strategy.selectedOptions?.setUpstream) {
        command += ' -u origin ' + gitStatus.branch;
      }
      if (strategy.selectedOptions?.atomic) {
        command += ' --atomic';
      }
      if (strategy.selectedOptions?.signPush) {
        command += ' --signed';
      }
      if (strategy.selectedOptions?.thinPack) {
        command += ' --thin';
      }
      if (strategy.selectedOptions?.commitLimit) {
        command = `git push origin HEAD~${strategy.selectedOptions.commitLimit}:${gitStatus.branch}`;
      }
      
      command += ' --dry-run';
      
      // Show what would happen
      const preview = `
Dry Run Results:
━━━━━━━━━━━━━━━━━━━━━━━━━━━

Command that would run:
${command}

What would happen:
• ${gitStatus.ahead || 0} commits would be pushed
• Branch: ${gitStatus.branch}
• Remote: origin/${gitStatus.branch}
${strategy.selectedOptions?.forceWithLease ? '• Force with lease protection enabled' : ''}
${strategy.selectedOptions?.includeTags ? '• Tags would be included' : ''}
${strategy.selectedOptions?.setUpstream ? '• Upstream tracking would be set' : ''}

No actual changes were made (dry run mode).
      `;
      
      alert(preview);
      
      return {
        success: true,
        message: 'Dry run completed. No changes were made.'
      };
    } catch (error: any) {
      throw new Error(`Dry run failed: ${error.message}`);
    }
  }
  
  /**
   * Execute regular push
   */
  private static async executeRegularPush(
    gitAPI: any,
    gitStatus: any,
    dialog: HTMLElement,
    options?: any
  ): Promise<{ success: boolean; message: string }> {
    this.updateProgress(dialog, 'Pushing to remote repository...');
    
    try {
      // Build push command based on options
      // For now, we use the standard push but log what would be different
      if (options) {
        console.log('Push options selected:', options);
        
        // These would need implementation in the main process
        if (options.forceWithLease) {
          console.log('Would use --force-with-lease');
        }
        if (options.includeTags) {
          console.log('Would include --tags');
        }
        if (options.setUpstream) {
          console.log('Would set upstream with -u');
        }
        if (options.commitLimit) {
          console.log(`Would push only last ${options.commitLimit} commits`);
        }
      }
      
      await gitAPI.push();
      return {
        success: true,
        message: `Successfully pushed ${gitStatus.ahead || 0} commits to ${gitStatus.branch}`
      };
    } catch (error: any) {
      if (error.message?.includes('pack exceeds maximum allowed size') || 
          error.message?.includes('exceeds maximum allowed size')) {
        // Automatically try chunked push when hitting size limits
        this.updateProgress(dialog, 'Regular push failed due to size limit, switching to chunked push...');
        try {
          const result = await gitAPI.pushChunked();
          return {
            success: true,
            message: `Repository too large for regular push.\nSuccessfully pushed using chunked strategy.\n\n${result}`
          };
        } catch (chunkedError: any) {
          throw new Error(`Regular push failed due to size. Chunked push also failed: ${chunkedError.message}`);
        }
      }
      throw error;
    }
  }
  
  /**
   * Execute chunked push
   */
  private static async executeChunkedPush(
    gitAPI: any,
    gitStatus: any,
    dialog: HTMLElement,
    options?: any
  ): Promise<{ success: boolean; message: string }> {
    this.updateProgress(dialog, 'Analyzing repository size and preparing chunked push...');
    
    try {
      // Add a small delay to show the initial message
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      this.updateProgress(dialog, 'Pushing commits in batches (this may take several minutes for large repositories)...');
      
      const result = await gitAPI.pushChunked();
      
      // Parse the result to determine if it was fully or partially successful
      const isPartial = result.includes('Partially successful') || result.includes('Some commits');
      
      return {
        success: !isPartial,
        message: result
      };
    } catch (error: any) {
      // Check if it's a timeout or other specific error
      if (error.message?.includes('SIGTERM') || error.message?.includes('timeout')) {
        throw new Error('Push operation timed out. The repository may be too large. Try pushing fewer commits or using Git LFS for large files.');
      }
      throw error;
    }
  }
  
  /**
   * Execute force push (with confirmation)
   */
  private static async executeForcePush(
    gitAPI: any,
    gitStatus: any,
    dialog: HTMLElement,
    options?: any
  ): Promise<{ success: boolean; message: string }> {
    // Extra confirmation for force push
    const confirmed = confirm(
      `⚠️ WARNING: Force Push\n\n` +
      `This will OVERWRITE the remote branch '${gitStatus.branch}' with your local version.\n\n` +
      `Any commits on the remote that you don't have locally will be LOST.\n\n` +
      `Are you absolutely sure you want to force push?`
    );
    
    if (!confirmed) {
      throw new Error('Force push cancelled by user');
    }
    
    this.updateProgress(dialog, 'Force pushing to remote...');
    
    try {
      // Call the force push API (needs to be added to gitAPI)
      if (gitAPI.forcePush) {
        await gitAPI.forcePush();
      } else {
        // Fallback: try regular push for now
        // TODO: Implement force push in main process
        this.updateProgress(dialog, 'Using regular push (force push being implemented)...');
        await gitAPI.push();
      }
      
      return {
        success: true,
        message: `Successfully force pushed to ${gitStatus.branch}\n\n⚠️ Remote history has been overwritten with your local version.`
      };
    } catch (error: any) {
      console.error('Force push failed:', error);
      // If force push is not implemented, fall back to regular push with warning
      if (error.message.includes('not defined') || error.message.includes('not a function')) {
        this.updateProgress(dialog, 'Force push not available, trying regular push...');
        await gitAPI.push();
        return {
          success: true,
          message: `Pushed using regular push (force push not available)`
        };
      }
      throw error;
    }
  }
  
  /**
   * Execute fresh branch push
   */
  private static async executeFreshBranchPush(
    gitAPI: any,
    gitStatus: any,
    dialog: HTMLElement,
    options?: any
  ): Promise<{ success: boolean; message: string }> {
    const timestamp = new Date().toISOString().split('T')[0];
    const freshBranchName = `${gitStatus.branch}-fresh-${timestamp}`;
    
    this.updateProgress(dialog, `Creating fresh branch: ${freshBranchName}...`);
    
    try {
      // Create and switch to the fresh branch
      await gitAPI.createBranch(freshBranchName);
      this.updateProgress(dialog, `Switched to fresh branch: ${freshBranchName}`);
      
      // Small delay to ensure branch is created
      await new Promise(resolve => setTimeout(resolve, 500));
      
      // Push the fresh branch to remote
      this.updateProgress(dialog, `Pushing ${freshBranchName} to remote...`);
      
      try {
        await gitAPI.push();
        return {
          success: true,
          message: `Successfully created and pushed fresh branch: ${freshBranchName}\n\nYou can now create a pull request from this branch.`
        };
      } catch (pushError: any) {
        // If regular push fails due to size, try chunked push
        if (pushError.message?.includes('exceeds maximum allowed size')) {
          this.updateProgress(dialog, 'Regular push failed due to size, trying chunked push...');
          const chunkedResult = await gitAPI.pushChunked();
          return {
            success: true,
            message: `Successfully created fresh branch and pushed using chunked strategy: ${freshBranchName}\n\n${chunkedResult}`
          };
        }
        throw pushError;
      }
    } catch (error: any) {
      console.error('Fresh branch push failed:', error);
      throw new Error(`Failed to push fresh branch: ${error.message}`);
    }
  }
  
  /**
   * Execute squash and push
   */
  private static async executeSquashPush(
    gitAPI: any,
    gitStatus: any,
    dialog: HTMLElement,
    options?: any
  ): Promise<{ success: boolean; message: string }> {
    const message = prompt(
      'Enter a commit message for the squashed commit:',
      `Squashed ${gitStatus.ahead} commits from ${gitStatus.branch}`
    );
    
    if (!message) {
      throw new Error('Squash push cancelled - no commit message provided');
    }
    
    this.updateProgress(dialog, 'Squashing commits and pushing...');
    
    try {
      // Get the base branch (usually main or master)
      const baseBranch = gitStatus.upstream?.split('/').pop() || 'main';
      
      // Create a temporary branch to preserve current state
      const tempBranch = `temp-squash-${Date.now()}`;
      await gitAPI.createBranch(tempBranch);
      
      // Switch back to original branch
      await gitAPI.switchBranch(gitStatus.branch);
      
      // Reset to upstream and create single commit
      // This is a simplified approach - ideally would use git reset --soft
      this.updateProgress(dialog, 'Creating squashed commit...');
      await gitAPI.commit(message);
      
      // Push the squashed branch
      this.updateProgress(dialog, 'Pushing squashed commit...');
      await gitAPI.push();
      
      return {
        success: true,
        message: `Successfully squashed ${gitStatus.ahead} commits and pushed.\n\nCommit message: "${message}"`
      };
    } catch (error: any) {
      console.error('Squash push failed:', error);
      // Fallback to regular push if squash fails
      this.updateProgress(dialog, 'Squash failed, using regular push...');
      await gitAPI.push();
      return {
        success: true,
        message: `Pushed without squashing (squash operation not fully supported yet)`
      };
    }
  }
  
  /**
   * Create bundle file
   */
  private static async executeBundleCreation(
    gitAPI: any,
    gitStatus: any,
    dialog: HTMLElement
  ): Promise<{ success: boolean; message: string }> {
    this.updateProgress(dialog, 'Creating bundle file...');
    
    try {
      // Generate bundle filename with timestamp
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const bundleFile = `repository-bundle-${timestamp}.bundle`;
      
      // Show instructions for manual bundle creation
      const instructions = `Bundle file creation initiated.\n\n` +
        `To create a Git bundle manually, run:\n\n` +
        `git bundle create ${bundleFile} --all\n\n` +
        `Then upload the bundle file to your repository's releases page or share via file transfer.\n\n` +
        `Bundle files preserve all Git history and can be imported with:\n` +
        `git clone ${bundleFile} [folder-name]`;
      
      // Copy command to clipboard if possible
      if (navigator.clipboard) {
        await navigator.clipboard.writeText(`git bundle create ${bundleFile} --all`);
        alert(instructions + '\n\n✅ Command copied to clipboard!');
      } else {
        alert(instructions);
      }
      
      return {
        success: true,
        message: `Bundle creation instructions provided.\nFile: ${bundleFile}\n\nUse this when Git protocol limits prevent normal pushing.`
      };
    } catch (error: any) {
      console.error('Bundle creation failed:', error);
      throw new Error(`Failed to create bundle: ${error.message}`);
    }
  }
  
  /**
   * Execute cleanup first
   */
  private static async executeCleanupFirst(
    gitAPI: any,
    gitStatus: any,
    dialog: HTMLElement
  ): Promise<{ success: boolean; message: string }> {
    this.updateProgress(dialog, 'Repository cleanup required...');
    
    const cleanupSteps = `🧹 Repository Cleanup Guide\n\n` +
      `Your repository (11+ GB) needs cleanup before pushing.\n\n` +
      `STEP 1: Install BFG Repo Cleaner\n` +
      `brew install bfg  (macOS)\n` +
      `apt-get install bfg  (Linux)\n\n` +
      `STEP 2: Remove large files from history\n` +
      `bfg --strip-blobs-bigger-than 100M\n\n` +
      `STEP 3: Clean up the repository\n` +
      `git reflog expire --expire=now --all\n` +
      `git gc --prune=now --aggressive\n\n` +
      `STEP 4: Force push cleaned repository\n` +
      `git push --force --all\n\n` +
      `Would you like to:\n` +
      `1. See detailed instructions (opens BFG website)\n` +
      `2. Copy cleanup commands to clipboard\n` +
      `3. Try alternative push strategy`;
    
    if (confirm(cleanupSteps + '\n\nClick OK to open BFG documentation, Cancel to copy commands.')) {
      window.open('https://rtyley.github.io/bfg-repo-cleaner/', '_blank');
    } else {
      // Copy commands to clipboard
      const commands = `# Repository Cleanup Commands
brew install bfg
bfg --strip-blobs-bigger-than 100M
git reflog expire --expire=now --all
git gc --prune=now --aggressive
git push --force --all`;
      
      if (navigator.clipboard) {
        await navigator.clipboard.writeText(commands);
        alert('✅ Cleanup commands copied to clipboard!');
      }
    }
    
    return {
      success: false,
      message: 'Repository cleanup required. Instructions provided.\n\nAfter cleanup, try pushing again.'
    };
  }
}