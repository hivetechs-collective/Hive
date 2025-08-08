import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface ConsensusProfile {
  id: string;
  name: string;
  generator_model: string;
  refiner_model: string;
  validator_model: string;
  curator_model: string;
}

interface ConsensusState {
  // Status
  isRunning: boolean;
  currentStage: string | null;
  progress: {
    [stage: string]: {
      progress: number;
      tokens: number;
      cost: number;
    };
  };
  
  // Results
  lastResult: string | null;
  history: Array<{
    id: string;
    query: string;
    result: string;
    timestamp: Date;
    cost: number;
  }>;
  
  // Profiles
  profiles: ConsensusProfile[];
  activeProfile: ConsensusProfile | null;
  
  // Actions
  runConsensus: (query: string) => Promise<void>;
  cancelConsensus: () => Promise<void>;
  loadProfiles: () => Promise<void>;
  setActiveProfile: (profileName: string) => Promise<void>;
  clearHistory: () => void;
}

export const useConsensusStore = create<ConsensusState>((set, get) => ({
  // Status
  isRunning: false,
  currentStage: null,
  progress: {},
  
  // Results
  lastResult: null,
  history: [],
  
  // Profiles
  profiles: [],
  activeProfile: null,
  
  // Actions
  runConsensus: async (query: string) => {
    set({ isRunning: true, currentStage: null, progress: {} });
    
    try {
      // This will be handled by streaming events
      await invoke('run_consensus_streaming', { query });
    } catch (error) {
      console.error('Consensus failed:', error);
      set({ isRunning: false });
      throw error;
    }
  },
  
  cancelConsensus: async () => {
    try {
      await invoke('cancel_consensus');
      set({ isRunning: false, currentStage: null });
    } catch (error) {
      console.error('Failed to cancel consensus:', error);
    }
  },
  
  loadProfiles: async () => {
    try {
      const profiles = await invoke<ConsensusProfile[]>('get_profiles');
      const activeProfile = await invoke<ConsensusProfile>('get_active_profile');
      set({ profiles, activeProfile });
    } catch (error) {
      console.error('Failed to load profiles:', error);
    }
  },
  
  setActiveProfile: async (profileName: string) => {
    try {
      await invoke('set_active_profile', { profileName });
      const activeProfile = await invoke<ConsensusProfile>('get_active_profile');
      set({ activeProfile });
    } catch (error) {
      console.error('Failed to set active profile:', error);
      throw error;
    }
  },
  
  clearHistory: () => set({ history: [] }),
}));