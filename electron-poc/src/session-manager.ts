/**
 * Session Manager for Workspace Persistence
 * Handles saving and restoring open tabs, folders, and application state
 */

export class SessionManager {
  private currentFolder: string | null = null;
  private saveDebounceTimeout: NodeJS.Timeout | null = null;
  
  constructor() {
    this.init();
  }
  
  private init() {
    // Listen for tab changes to auto-save session
    window.addEventListener('tab-changed', () => this.debouncedSave());
    window.addEventListener('tab-closed', () => this.debouncedSave());
    window.addEventListener('tab-opened', () => this.debouncedSave());
    
    // Load recent folders on startup
    this.loadRecentFolders();
  }
  
  /**
   * Set the current working folder
   */
  public async setCurrentFolder(folderPath: string | null) {
    // Save session for previous folder if switching
    if (this.currentFolder && this.currentFolder !== folderPath) {
      await this.saveCurrentSession();
    }
    
    this.currentFolder = folderPath;
    
    // If opening a new folder, restore its session
    if (folderPath) {
      await this.restoreSession(folderPath);
      await this.updateRecentFolder(folderPath);
    }
  }
  
  /**
   * Get the current folder path
   */
  public getCurrentFolder(): string | null {
    return this.currentFolder;
  }
  
  /**
   * Save the current session (tabs and state)
   */
  public async saveCurrentSession() {
    if (!this.currentFolder || !window.editorTabs) {
      return;
    }
    
    try {
      const sessionData = window.editorTabs.getSessionData();
      const tabCount = window.editorTabs.getTabCount();
      
      // Save session to database
      await window.databaseAPI.saveSession(
        this.currentFolder,
        sessionData.tabs,
        sessionData.activeTab
      );
      
      // Update recent folder with tab count
      await window.databaseAPI.addRecentFolder(this.currentFolder, tabCount);
      
      console.log('[SessionManager] Session saved for', this.currentFolder);
    } catch (error) {
      console.error('[SessionManager] Failed to save session:', error);
    }
  }
  
  /**
   * Debounced save to avoid excessive database writes
   */
  private debouncedSave() {
    if (this.saveDebounceTimeout) {
      clearTimeout(this.saveDebounceTimeout);
    }
    
    this.saveDebounceTimeout = setTimeout(() => {
      this.saveCurrentSession();
    }, 1000); // Save after 1 second of inactivity
  }
  
  /**
   * Restore session for a folder
   */
  private async restoreSession(folderPath: string) {
    if (!window.editorTabs) {
      console.warn('[SessionManager] EditorTabs not initialized');
      return;
    }
    
    try {
      const sessionData = await window.databaseAPI.loadSession(folderPath);
      
      if (sessionData && sessionData.tabs && sessionData.tabs.length > 0) {
        console.log('[SessionManager] Restoring session for', folderPath);
        await window.editorTabs.restoreSession(sessionData);
      }
    } catch (error) {
      console.error('[SessionManager] Failed to restore session:', error);
    }
  }
  
  /**
   * Update recent folder in database
   */
  private async updateRecentFolder(folderPath: string) {
    try {
      const tabCount = window.editorTabs?.getTabCount() || 0;
      await window.databaseAPI.addRecentFolder(folderPath, tabCount);
      
      // Trigger update of recent folders display
      await this.loadRecentFolders();
    } catch (error) {
      console.error('[SessionManager] Failed to update recent folder:', error);
    }
  }
  
  /**
   * Load and display recent folders
   */
  public async loadRecentFolders() {
    try {
      const folders = await window.databaseAPI.getRecentFolders();
      
      // Dispatch event to update UI
      const event = new CustomEvent('recent-folders-updated', { 
        detail: { folders } 
      });
      window.dispatchEvent(event);
      
      return folders;
    } catch (error) {
      console.error('[SessionManager] Failed to load recent folders:', error);
      return [];
    }
  }
  
  /**
   * Clear session for a folder
   */
  public async clearSession(folderPath: string) {
    try {
      await window.databaseAPI.clearSession(folderPath);
      console.log('[SessionManager] Session cleared for', folderPath);
    } catch (error) {
      console.error('[SessionManager] Failed to clear session:', error);
    }
  }
  
  /**
   * Remove a folder from recent list
   */
  public async removeRecentFolder(folderPath: string) {
    try {
      await window.databaseAPI.removeRecentFolder(folderPath);
      await this.loadRecentFolders();
    } catch (error) {
      console.error('[SessionManager] Failed to remove recent folder:', error);
    }
  }
  
  /**
   * Close current folder and save session
   */
  public async closeCurrentFolder() {
    if (this.currentFolder) {
      await this.saveCurrentSession();
      
      // Clear all tabs
      if (window.editorTabs) {
        window.editorTabs.closeAllTabs();
      }
      
      this.currentFolder = null;
      
      // Trigger UI update to show welcome page
      const event = new CustomEvent('folder-closed');
      window.dispatchEvent(event);
      
      // Refresh recent folders
      await this.loadRecentFolders();
    }
  }
  
  /**
   * Open a recent folder with session restoration
   */
  public async openRecentFolder(folderPath: string) {
    console.log('[SessionManager] Opening recent folder:', folderPath);
    
    // Set current folder (will trigger session restoration)
    await this.setCurrentFolder(folderPath);
    
    // Dispatch event to open folder in explorer
    const event = new CustomEvent('open-folder-in-explorer', { 
      detail: { folderPath } 
    });
    window.dispatchEvent(event);
  }
}

// Create global instance
declare global {
  interface Window {
    sessionManager: SessionManager;
  }
}

// Initialize session manager when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    window.sessionManager = new SessionManager();
  });
} else {
  window.sessionManager = new SessionManager();
}

export default SessionManager;