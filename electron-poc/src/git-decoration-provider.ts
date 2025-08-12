/**
 * Git Decoration Provider for File Explorer
 * Provides Git status decorations for files and folders
 */

import { EventEmitter } from 'events';
import * as path from 'path';
import * as chokidar from 'chokidar';

export interface GitDecoration {
  badge?: string;      // Single letter badge (M, A, D, U, etc.)
  color?: string;      // Color for the file name
  tooltip?: string;    // Tooltip on hover
  propagate?: boolean; // Whether to show on parent folders
  priority?: number;   // For determining which decoration wins in conflicts
}

export interface GitFileStatus {
  path: string;
  index: string;   // Staged status
  working: string; // Working tree status
  renamed?: string;
}

// VS Code-style status codes
export enum GitStatus {
  INDEX_MODIFIED = 'M ',
  MODIFIED = ' M',
  INDEX_ADDED = 'A ',
  INDEX_DELETED = 'D ',
  DELETED = ' D',
  INDEX_RENAMED = 'R ',
  INDEX_COPIED = 'C ',
  UNTRACKED = '??',
  IGNORED = '!!',
  BOTH_DELETED = 'DD',
  ADDED_BY_US = 'AU',
  ADDED_BY_THEM = 'UA',
  DELETED_BY_US = 'DU',
  DELETED_BY_THEM = 'UD',
  BOTH_ADDED = 'AA',
  BOTH_MODIFIED = 'UU'
}

export class GitDecorationProvider extends EventEmitter {
  private decorations: Map<string, GitDecoration> = new Map();
  private fileWatcher: chokidar.FSWatcher | null = null;
  private gitWatcher: chokidar.FSWatcher | null = null;
  private updateTimer: NodeJS.Timeout | null = null;
  private rootPath: string;
  private gitStatus: Map<string, GitFileStatus> = new Map();

  // VS Code-inspired color scheme
  private readonly colors = {
    modified: '#e2c08d',     // Orange
    added: '#73c991',        // Green
    deleted: '#f48771',      // Red
    untracked: '#6b6b6b',    // Gray (was purple in VS Code)
    ignored: '#5a5a5a',      // Dark gray
    conflicted: '#fd7e14',   // Dark orange
    renamed: '#4fc3f7',      // Light blue
    staged: '#007acc'        // VS Code blue
  };

  // Badge text for different statuses
  private readonly badges = {
    modified: 'M',
    added: 'A',
    deleted: 'D',
    untracked: 'U',
    renamed: 'R',
    copied: 'C',
    conflicted: '!',
    ignored: '',
    staged: '●'
  };

  constructor(rootPath: string) {
    super();
    this.rootPath = rootPath;
  }

  public async initialize(): Promise<void> {
    console.log('[GitDecorationProvider] Initializing...');
    
    // Initial Git status load
    await this.updateGitStatus();
    
    // Set up file system watchers
    this.setupWatchers();
    
    console.log('[GitDecorationProvider] Initialized with', this.decorations.size, 'decorations');
  }

  private setupWatchers(): void {
    // Watch .git directory for Git operations
    const gitPath = path.join(this.rootPath, '.git');
    this.gitWatcher = chokidar.watch(gitPath, {
      ignored: /(^|[\/\\])\../, // Ignore dotfiles except .git
      persistent: true,
      ignoreInitial: true,
      depth: 2
    });

    // Watch working directory for file changes
    this.fileWatcher = chokidar.watch(this.rootPath, {
      ignored: [
        /(^|[\/\\])\../, // Ignore dotfiles
        /node_modules/,
        /\.git/
      ],
      persistent: true,
      ignoreInitial: true
    });

    // Git directory changes
    this.gitWatcher.on('all', (event, filePath) => {
      console.log('[GitDecorationProvider] Git change detected:', event, filePath);
      this.scheduleUpdate();
    });

    // Working directory changes
    this.fileWatcher.on('all', (event, filePath) => {
      console.log('[GitDecorationProvider] File change detected:', event, filePath);
      this.scheduleUpdate();
    });
  }

  private scheduleUpdate(): void {
    // Debounce updates to avoid excessive Git calls
    if (this.updateTimer) {
      clearTimeout(this.updateTimer);
    }
    
    this.updateTimer = setTimeout(() => {
      this.updateGitStatus();
    }, 300);
  }

  private async updateGitStatus(): Promise<void> {
    try {
      // Get Git status from the main process
      const status = await (window as any).gitAPI.getStatus();
      
      if (!status.isRepo) {
        console.log('[GitDecorationProvider] Not a Git repository');
        this.decorations.clear();
        this.emit('decorationsChanged', []);
        return;
      }

      console.log('[GitDecorationProvider] Git status updated:', status.files.length, 'files');
      
      // Clear old decorations
      this.decorations.clear();
      this.gitStatus.clear();
      
      // Process each file status
      for (const file of status.files) {
        this.gitStatus.set(file.path, file);
        const decoration = this.createDecoration(file);
        if (decoration) {
          const fullPath = path.join(this.rootPath, file.path);
          this.decorations.set(fullPath, decoration);
          
          // Also add decorations for parent folders
          if (decoration.propagate) {
            this.propagateToParents(fullPath, decoration);
          }
        }
      }
      
      // Emit change event
      this.emit('decorationsChanged', Array.from(this.decorations.keys()));
      
    } catch (error) {
      console.error('[GitDecorationProvider] Failed to update Git status:', error);
    }
  }

  private createDecoration(fileStatus: GitFileStatus): GitDecoration | null {
    const statusCode = fileStatus.index + fileStatus.working;
    
    // Determine the decoration based on Git status
    let badge = '';
    let color = '';
    let tooltip = '';
    let priority = 0;
    let propagate = true;

    // Check for conflicts first (highest priority)
    if (statusCode === 'UU' || statusCode === 'AA' || statusCode === 'DD') {
      badge = this.badges.conflicted;
      color = this.colors.conflicted;
      tooltip = 'Merge conflict';
      priority = 100;
    }
    // Staged modifications
    else if (fileStatus.index === 'M') {
      badge = this.badges.modified;
      color = this.colors.staged;
      tooltip = 'Staged changes';
      priority = 80;
    }
    // Staged additions
    else if (fileStatus.index === 'A') {
      badge = this.badges.added;
      color = this.colors.added;
      tooltip = 'Staged new file';
      priority = 75;
    }
    // Staged deletions
    else if (fileStatus.index === 'D') {
      badge = this.badges.deleted;
      color = this.colors.deleted;
      tooltip = 'Staged for deletion';
      priority = 70;
    }
    // Renamed files
    else if (fileStatus.index === 'R') {
      badge = this.badges.renamed;
      color = this.colors.renamed;
      tooltip = `Renamed from ${fileStatus.renamed}`;
      priority = 65;
    }
    // Working tree modifications
    else if (fileStatus.working === 'M') {
      badge = this.badges.modified;
      color = this.colors.modified;
      tooltip = 'Modified';
      priority = 50;
    }
    // Working tree deletions
    else if (fileStatus.working === 'D') {
      badge = this.badges.deleted;
      color = this.colors.deleted;
      tooltip = 'Deleted';
      priority = 45;
    }
    // Untracked files
    else if (statusCode === '??') {
      badge = this.badges.untracked;
      color = this.colors.untracked;
      tooltip = 'Untracked';
      priority = 30;
      propagate = false; // Don't propagate untracked status to parents
    }
    // Ignored files
    else if (statusCode === '!!') {
      badge = this.badges.ignored;
      color = this.colors.ignored;
      tooltip = 'Ignored';
      priority = 10;
      propagate = false;
    }
    
    if (badge || color) {
      return {
        badge,
        color,
        tooltip,
        priority,
        propagate
      };
    }
    
    return null;
  }

  private propagateToParents(filePath: string, decoration: GitDecoration): void {
    let currentPath = path.dirname(filePath);
    
    while (currentPath && currentPath !== this.rootPath && currentPath !== path.dirname(currentPath)) {
      const existingDecoration = this.decorations.get(currentPath);
      
      // Only update parent if this decoration has higher priority
      if (!existingDecoration || (decoration.priority || 0) > (existingDecoration.priority || 0)) {
        this.decorations.set(currentPath, {
          ...decoration,
          tooltip: 'Contains ' + decoration.tooltip?.toLowerCase()
        });
      }
      
      currentPath = path.dirname(currentPath);
    }
  }

  public getDecoration(filePath: string): GitDecoration | undefined {
    return this.decorations.get(filePath);
  }

  public getAllDecorations(): Map<string, GitDecoration> {
    return new Map(this.decorations);
  }

  public getFileStatus(filePath: string): GitFileStatus | undefined {
    // Convert absolute path to relative path
    const relativePath = path.relative(this.rootPath, filePath);
    return this.gitStatus.get(relativePath);
  }

  public async refreshStatus(): Promise<void> {
    console.log('[GitDecorationProvider] Manual refresh requested');
    await this.updateGitStatus();
  }

  public dispose(): void {
    if (this.fileWatcher) {
      this.fileWatcher.close();
    }
    if (this.gitWatcher) {
      this.gitWatcher.close();
    }
    if (this.updateTimer) {
      clearTimeout(this.updateTimer);
    }
    this.decorations.clear();
    this.gitStatus.clear();
    this.removeAllListeners();
  }
}