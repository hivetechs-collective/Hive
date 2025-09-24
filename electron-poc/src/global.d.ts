declare module '*.jpg' {
  const value: string;
  export default value;
}

declare module '*.jpeg' {
  const value: string;
  export default value;
}

declare module '*.png' {
  const value: string;
  export default value;
}

declare module '*.svg' {
  const value: string;
  export default value;
}

declare module '*.gif' {
  const value: string;
  export default value;
}

// Extended database API for session persistence
declare global {
  interface Window {
    databaseAPI: {
      getSetting: (key: string) => Promise<string>;
      setSetting: (key: string, value: string) => Promise<{ success: boolean }>;
      
      // Session persistence
      saveSession: (folderPath: string, tabs: any[], activeTab: string | null) => Promise<{ success: boolean }>;
      loadSession: (folderPath: string) => Promise<{ tabs: any[], activeTab: string | null }>;
      clearSession: (folderPath: string) => Promise<{ success: boolean }>;
      
      // Recent folders
      addRecentFolder: (folderPath: string, tabCount: number) => Promise<{ success: boolean }>;
      getRecentFolders: () => Promise<Array<{ folder_path: string, last_opened: string, tab_count: number }>>;
      removeRecentFolder: (folderPath: string) => Promise<{ success: boolean }>;
    };
  }
}

export {};