import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface AppState {
  // Theme
  darkMode: boolean;
  toggleDarkMode: () => void;
  
  // API Keys
  hasApiKey: boolean;
  apiKeys: {
    openrouter: boolean;
    hive: boolean;
  };
  checkApiKeys: () => Promise<void>;
  setApiKey: (key: string) => Promise<void>;
  
  // File Explorer
  currentFile: string | null;
  setCurrentFile: (file: string | null) => void;
  
  // Settings
  settings: {
    autoSave: boolean;
    fontSize: number;
    tabSize: number;
    wordWrap: boolean;
  };
  updateSettings: (settings: Partial<AppState['settings']>) => void;
  
  // Git
  gitBranch: string;
  gitStatus: {
    modified: number;
    added: number;
    deleted: number;
  };
  updateGitStatus: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  // Theme
  darkMode: true,
  toggleDarkMode: () => set(state => ({ darkMode: !state.darkMode })),
  
  // API Keys
  hasApiKey: false,
  apiKeys: {
    openrouter: false,
    hive: false,
  },
  checkApiKeys: async () => {
    try {
      const status = await invoke<{
        openrouter: { configured: boolean };
        anthropic: { configured: boolean };
        hive: { configured: boolean };
      }>('get_api_keys_status');
      set({ 
        hasApiKey: status.openrouter.configured,
        apiKeys: {
          openrouter: status.openrouter.configured,
          hive: status.hive.configured,
        }
      });
    } catch (error) {
      console.error('Failed to check API keys:', error);
      set({ 
        hasApiKey: false,
        apiKeys: { openrouter: false, hive: false }
      });
    }
  },
  
  setApiKey: async (key: string) => {
    try {
      await invoke('set_api_key', { key });
      set({ hasApiKey: true });
    } catch (error) {
      console.error('Failed to set API key:', error);
      throw error;
    }
  },
  
  // File Explorer
  currentFile: null,
  setCurrentFile: (file) => set({ currentFile: file }),
  
  // Settings
  settings: {
    autoSave: true,
    fontSize: 14,
    tabSize: 2,
    wordWrap: true,
  },
  updateSettings: (newSettings) => set(state => ({
    settings: { ...state.settings, ...newSettings }
  })),
  
  // Git
  gitBranch: 'main',
  gitStatus: {
    modified: 0,
    added: 0,
    deleted: 0,
  },
  updateGitStatus: async () => {
    // TODO: Implement git status check
    // For now, just set mock data
    set({
      gitBranch: 'main',
      gitStatus: {
        modified: 0,
        added: 0,
        deleted: 0,
      }
    });
  },
}));