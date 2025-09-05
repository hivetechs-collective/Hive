import { spawn, ChildProcess } from 'child_process';
import * as path from 'path';

export enum GitErrorCode {
  BadConfigFile = 'BadConfigFile',
  AuthenticationFailed = 'AuthenticationFailed',
  NoUserNameConfigured = 'NoUserNameConfigured',
  NoUserEmailConfigured = 'NoUserEmailConfigured',
  NoRemoteRepositorySpecified = 'NoRemoteRepositorySpecified',
  NotAGitRepository = 'NotAGitRepository',
  NotAtRepositoryRoot = 'NotAtRepositoryRoot',
  Conflict = 'Conflict',
  StashConflict = 'StashConflict',
  UnmergedChanges = 'UnmergedChanges',
  PushRejected = 'PushRejected',
  RemoteConnectionError = 'RemoteConnectionError',
  DirtyWorkTree = 'DirtyWorkTree',
  CantOpenResource = 'CantOpenResource',
  GitNotFound = 'GitNotFound',
  CantCreatePipe = 'CantCreatePipe',
  PermissionDenied = 'PermissionDenied',
  CantAccessRemote = 'CantAccessRemote',
  RepositoryNotFound = 'RepositoryNotFound',
  RepositoryIsLocked = 'RepositoryIsLocked',
  BranchNotFullyMerged = 'BranchNotFullyMerged',
  NoRemoteReference = 'NoRemoteReference',
  InvalidBranchName = 'InvalidBranchName',
  BranchAlreadyExists = 'BranchAlreadyExists',
  NoLocalChanges = 'NoLocalChanges',
  NoStashFound = 'NoStashFound',
  LocalChangesOverwritten = 'LocalChangesOverwritten',
  NoUpstreamBranch = 'NoUpstreamBranch',
  IsInSubmodule = 'IsInSubmodule',
  WrongCase = 'WrongCase',
  CantLockRef = 'CantLockRef',
  CantRebaseMultipleBranches = 'CantRebaseMultipleBranches',
  PatchDoesNotApply = 'PatchDoesNotApply',
  NoPathFound = 'NoPathFound',
  UnknownPath = 'UnknownPath',
}

export interface GitExecOptions {
  cwd?: string;
  env?: NodeJS.ProcessEnv;
  input?: string;
  encoding?: BufferEncoding;
  log?: boolean;
  cancellationToken?: AbortSignal;
  onSpawn?: (process: ChildProcess) => void;
}

export interface GitExecResult {
  exitCode: number;
  stdout: string;
  stderr: string;
}

export class GitError extends Error {
  constructor(
    public readonly data: {
      message: string;
      exitCode?: number;
      stdout?: string;
      stderr?: string;
      gitErrorCode?: GitErrorCode;
      gitCommand?: string;
      gitArgs?: string[];
    }
  ) {
    super(data.message);
    this.name = 'GitError';
  }
}

export class GitExecutor {
  private gitPath: string = 'git';
  private env: NodeJS.ProcessEnv;
  
  constructor(private repoPath: string) {
    // Set up environment for Git
    this.env = {
      ...process.env,
      GIT_TERMINAL_PROMPT: '0', // Disable prompting for credentials
      GIT_ASKPASS: '', // Disable GUI askpass
      LANG: 'en_US.UTF-8',
      LC_ALL: 'en_US.UTF-8',
    };

    // For macOS, ensure we use the credential helper
    if (process.platform === 'darwin') {
      this.env.GIT_ASKPASS = ''; // Let it use osxkeychain
    }
  }

  private spawn(args: string[], options: GitExecOptions = {}): ChildProcess {
    const spawnOptions = {
      cwd: options.cwd || this.repoPath,
      env: options.env || this.env,
      windowsHide: true,
    };

    console.log(`[GitExecutor] Spawning: git ${args.join(' ')}`);
    return spawn(this.gitPath, args, spawnOptions);
  }

  private detectGitErrorCode(stderr: string): GitErrorCode | undefined {
    if (/Another git process seems to be running/.test(stderr)) {
      return GitErrorCode.RepositoryIsLocked;
    }
    if (/Authentication failed/i.test(stderr)) {
      return GitErrorCode.AuthenticationFailed;
    }
    if (/Not a git repository/i.test(stderr)) {
      return GitErrorCode.NotAGitRepository;
    }
    if (/bad config file/.test(stderr)) {
      return GitErrorCode.BadConfigFile;
    }
    if (/cannot make pipe for command substitution|cannot create standard input pipe/.test(stderr)) {
      return GitErrorCode.CantCreatePipe;
    }
    if (/Repository not found/.test(stderr)) {
      return GitErrorCode.RepositoryNotFound;
    }
    if (/unable to access/.test(stderr)) {
      return GitErrorCode.CantAccessRemote;
    }
    if (/branch '.+' is not fully merged/.test(stderr)) {
      return GitErrorCode.BranchNotFullyMerged;
    }
    if (/Couldn't find remote ref/.test(stderr)) {
      return GitErrorCode.NoRemoteReference;
    }
    if (/A branch named '.+' already exists/.test(stderr)) {
      return GitErrorCode.BranchAlreadyExists;
    }
    if (/'.+' is not a valid branch name/.test(stderr)) {
      return GitErrorCode.InvalidBranchName;
    }
    if (/Please,? commit your changes or stash them/.test(stderr)) {
      return GitErrorCode.DirtyWorkTree;
    }
    if (/Your local changes to the following files would be overwritten/.test(stderr)) {
      return GitErrorCode.LocalChangesOverwritten;
    }
    if (/No upstream branch/.test(stderr) || /does not have an upstream branch/.test(stderr)) {
      return GitErrorCode.NoUpstreamBranch;
    }
    if (/failed to push some refs/.test(stderr)) {
      return GitErrorCode.PushRejected;
    }
    if (/CONFLICT/.test(stderr)) {
      return GitErrorCode.Conflict;
    }
    if (/Pulling is not possible because you have unmerged files|Cannot pull with rebase: You have unstaged changes|Your local changes to the following files would be overwritten|Please, commit your changes before you can merge/i.test(stderr)) {
      return GitErrorCode.UnmergedChanges;
    }
    if (/cannot lock ref|unable to update local ref/.test(stderr)) {
      return GitErrorCode.CantLockRef;
    }
    if (/cannot rebase onto multiple branches/.test(stderr)) {
      return GitErrorCode.CantRebaseMultipleBranches;
    }
    if (/Patch does not apply/.test(stderr)) {
      return GitErrorCode.PatchDoesNotApply;
    }
    if (/No stash found/.test(stderr)) {
      return GitErrorCode.NoStashFound;
    }
    if (/error: unable to unlink old/.test(stderr)) {
      return GitErrorCode.PermissionDenied;
    }
    
    return undefined;
  }

  async exec(args: string[], options: GitExecOptions = {}): Promise<GitExecResult> {
    return new Promise((resolve, reject) => {
      const child = this.spawn(args, options);
      
      if (options.onSpawn) {
        options.onSpawn(child);
      }

      let stdout = '';
      let stderr = '';
      const encoding = options.encoding || 'utf8';

      if (child.stdout) {
        child.stdout.setEncoding(encoding);
        child.stdout.on('data', (data) => {
          stdout += data;
        });
      }

      if (child.stderr) {
        child.stderr.setEncoding(encoding);
        child.stderr.on('data', (data) => {
          stderr += data;
        });
      }

      if (options.input) {
        child.stdin?.end(options.input, 'utf8');
      }

      // Handle cancellation
      if (options.cancellationToken) {
        const onAbort = () => {
          child.kill('SIGTERM');
          reject(new Error('Cancelled'));
        };
        
        if (options.cancellationToken.aborted) {
          onAbort();
          return;
        }
        
        options.cancellationToken.addEventListener('abort', onAbort, { once: true });
      }

      child.on('error', (err) => {
        reject(new GitError({
          message: 'Failed to spawn git process',
          gitErrorCode: GitErrorCode.GitNotFound,
          gitCommand: args[0],
          gitArgs: args,
        }));
      });

      child.on('close', (exitCode) => {
        console.log(`[GitExecutor] Command completed with code ${exitCode}`);
        
        const result: GitExecResult = {
          exitCode: exitCode || 0,
          stdout,
          stderr,
        };

        if (exitCode !== 0) {
          const errorCode = this.detectGitErrorCode(stderr);
          reject(new GitError({
            message: stderr || 'Git command failed',
            exitCode,
            stdout,
            stderr,
            gitErrorCode: errorCode,
            gitCommand: args[0],
            gitArgs: args,
          }));
        } else {
          resolve(result);
        }
      });
    });
  }

  // High-level Git operations using robust execution
  async status(): Promise<any> {
    const result = await this.exec(['status', '--porcelain=v2', '--branch', '--untracked-files=all']);
    return this.parseStatus(result.stdout);
  }

  async push(options: { remote?: string; branch?: string; setUpstream?: boolean } = {}): Promise<void> {
    const args = ['push'];
    
    if (options.setUpstream) {
      args.push('--set-upstream');
    }
    
    if (options.remote) {
      args.push(options.remote);
    }
    
    if (options.branch) {
      args.push(options.branch);
    }

    await this.exec(args);
  }

  async pull(options: { remote?: string; branch?: string; rebase?: boolean } = {}): Promise<void> {
    const args = ['pull'];
    
    if (options.rebase) {
      args.push('--rebase');
    }
    
    if (options.remote) {
      args.push(options.remote);
    }
    
    if (options.branch) {
      args.push(options.branch);
    }

    await this.exec(args);
  }

  async fetch(options: { all?: boolean; prune?: boolean } = {}): Promise<void> {
    const args = ['fetch'];
    
    if (options.all) {
      args.push('--all');
    }
    
    if (options.prune) {
      args.push('--prune');
    }

    await this.exec(args);
  }

  async add(files: string[]): Promise<void> {
    await this.exec(['add', '--', ...files]);
  }

  async reset(files: string[]): Promise<void> {
    await this.exec(['reset', 'HEAD', '--', ...files]);
  }

  async commit(message: string): Promise<void> {
    await this.exec(['commit', '-m', message]);
  }

  async checkout(target: string): Promise<void> {
    await this.exec(['checkout', target]);
  }

  async branch(options?: { all?: boolean; remotes?: boolean }): Promise<any> {
    const args = ['branch'];
    
    if (options?.all) {
      args.push('-a');
    } else if (options?.remotes) {
      args.push('-r');
    }
    
    args.push('-vv'); // Verbose with upstream info
    
    const result = await this.exec(args);
    return this.parseBranches(result.stdout);
  }

  async log(options: { maxCount?: number; graph?: boolean; oneline?: boolean; limit?: number } = {}): Promise<any> {
    const args = ['log'];
    
    // Add graph option if requested (critical for graph view!)
    if (options.graph) {
      args.push('--graph');
    }
    
    if (options.maxCount) {
      args.push(`-${options.maxCount}`);
    } else if (options.limit) {
      args.push(`-${options.limit}`);
    }
    
    if (options.oneline) {
      args.push('--oneline');
    } else {
      args.push('--pretty=format:%H|%an|%ae|%ad|%s');
    }
    
    const result = await this.exec(args);
    return result.stdout;
  }

  async diff(options: { cached?: boolean; nameOnly?: boolean } = {}): Promise<string> {
    const args = ['diff'];
    
    if (options.cached) {
      args.push('--cached');
    }
    
    if (options.nameOnly) {
      args.push('--name-only');
    }
    
    const result = await this.exec(args);
    return result.stdout;
  }

  // Helper methods for parsing Git output
  private parseStatus(output: string): any {
    const lines = output.split('\n').filter(l => l);
    const status: any = {
      branch: '',
      upstream: null,
      ahead: 0,
      behind: 0,
      files: [],
    };

    for (const line of lines) {
      if (line.startsWith('# branch.head ')) {
        status.branch = line.substring(14);
      } else if (line.startsWith('# branch.upstream ')) {
        status.upstream = line.substring(18);
      } else if (line.startsWith('# branch.ab ')) {
        const match = line.match(/\+(\d+) -(\d+)$/);
        if (match) {
          status.ahead = parseInt(match[1], 10);
          status.behind = parseInt(match[2], 10);
        }
      } else if (line.startsWith('1 ') || line.startsWith('2 ')) {
        // Parse file entries
        const parts = line.split(' ');
        if (parts.length >= 9) {
          const xy = parts[1];
          const path = parts[8];
          status.files.push({
            path,
            index: xy[0],
            working: xy[1],
          });
        }
      } else if (line.startsWith('? ')) {
        // Untracked files
        status.files.push({
          path: line.substring(2),
          index: '?',
          working: '?',
        });
      }
    }

    return status;
  }

  private parseBranches(output: string): any {
    const lines = output.split('\n').filter(l => l);
    const branches: any[] = [];

    for (const line of lines) {
      const current = line.startsWith('*');
      const parts = line.substring(2).trim().split(/\s+/);
      const name = parts[0];
      const commit = parts[1];
      
      // Parse upstream tracking info if present
      let upstream = null;
      let ahead = 0;
      let behind = 0;
      
      const trackingMatch = line.match(/\[([^\]]+)\]/);
      if (trackingMatch) {
        const tracking = trackingMatch[1];
        const upstreamMatch = tracking.match(/^([^:]+)/);
        if (upstreamMatch) {
          upstream = upstreamMatch[1];
        }
        
        const aheadMatch = tracking.match(/ahead (\d+)/);
        if (aheadMatch) {
          ahead = parseInt(aheadMatch[1], 10);
        }
        
        const behindMatch = tracking.match(/behind (\d+)/);
        if (behindMatch) {
          behind = parseInt(behindMatch[1], 10);
        }
      }

      branches.push({
        name,
        current,
        commit,
        upstream,
        ahead,
        behind,
      });
    }

    return branches;
  }
}