// Mock Tauri API for browser development
// This allows us to test the UI without the Tauri backend

const mockProfiles = [
  {
    id: 'lightning-fast',
    name: 'Lightning Fast',
    description: 'Optimized for speed with minimal token usage',
    category: 'Speed',
    is_active: false,
    is_custom: false,
    expert_level: 'Basic',
    use_cases: ['Quick answers', 'Simple queries'],
    tags: ['fast', 'economical']
  },
  {
    id: 'balanced-generalist',
    name: 'Balanced Generalist',
    description: 'Balanced performance and quality for general use',
    category: 'Balanced',
    is_active: true,
    is_custom: false,
    expert_level: 'Intermediate',
    use_cases: ['General development', 'Code review'],
    tags: ['balanced', 'versatile']
  },
  {
    id: 'precision-architect',
    name: 'Precision Architect',
    description: 'Maximum accuracy for complex architectural decisions',
    category: 'Quality',
    is_active: false,
    is_custom: false,
    expert_level: 'Expert',
    use_cases: ['Architecture', 'Complex systems'],
    tags: ['precise', 'thorough']
  },
  {
    id: 'budget-optimizer',
    name: 'Budget Optimizer',
    description: 'Minimize costs while maintaining quality',
    category: 'Economy',
    is_active: false,
    is_custom: false,
    expert_level: 'Basic',
    use_cases: ['Cost-sensitive', 'High volume'],
    tags: ['economical', 'efficient']
  },
  {
    id: 'research-specialist',
    name: 'Research Specialist',
    description: 'Deep analysis and comprehensive research',
    category: 'Research',
    is_active: false,
    is_custom: false,
    expert_level: 'Expert',
    use_cases: ['Research', 'Analysis'],
    tags: ['thorough', 'analytical']
  },
  {
    id: 'debug-specialist',
    name: 'Debug Specialist',
    description: 'Focused on debugging and error resolution',
    category: 'Debug',
    is_active: false,
    is_custom: false,
    expert_level: 'Expert',
    use_cases: ['Debugging', 'Error fixing'],
    tags: ['debugging', 'problem-solving']
  },
  {
    id: 'enterprise-architect',
    name: 'Enterprise Architect',
    description: 'Enterprise-grade solutions and best practices',
    category: 'Enterprise',
    is_active: false,
    is_custom: false,
    expert_level: 'Expert',
    use_cases: ['Enterprise', 'Best practices'],
    tags: ['enterprise', 'scalable']
  },
  {
    id: 'creative-innovator',
    name: 'Creative Innovator',
    description: 'Creative solutions and innovative approaches',
    category: 'Creative',
    is_active: false,
    is_custom: false,
    expert_level: 'Advanced',
    use_cases: ['Innovation', 'Creative solutions'],
    tags: ['creative', 'innovative']
  },
  {
    id: 'teaching-assistant',
    name: 'Teaching Assistant',
    description: 'Educational focus with clear explanations',
    category: 'Education',
    is_active: false,
    is_custom: false,
    expert_level: 'Intermediate',
    use_cases: ['Teaching', 'Learning'],
    tags: ['educational', 'clear']
  },
  {
    id: 'security-auditor',
    name: 'Security Auditor',
    description: 'Security-focused analysis and recommendations',
    category: 'Security',
    is_active: false,
    is_custom: false,
    expert_level: 'Expert',
    use_cases: ['Security', 'Auditing'],
    tags: ['security', 'audit']
  }
];

const mockFileSystem = [
  {
    name: 'src',
    path: '/Users/veronelazio/Developer/Private/hive/src',
    is_dir: true,
    size: 0,
    modified: Date.now()
  },
  {
    name: 'frontend',
    path: '/Users/veronelazio/Developer/Private/hive/frontend',
    is_dir: true,
    size: 0,
    modified: Date.now()
  },
  {
    name: 'README.md',
    path: '/Users/veronelazio/Developer/Private/hive/README.md',
    is_dir: false,
    size: 1024,
    modified: Date.now()
  },
  {
    name: 'Cargo.toml',
    path: '/Users/veronelazio/Developer/Private/hive/Cargo.toml',
    is_dir: false,
    size: 2048,
    modified: Date.now()
  }
];

// Check if we're running in Tauri or browser
export const isTauri = () => {
  return (window as any).__TAURI__ !== undefined;
};

// Mock invoke function for browser development
export const mockInvoke = async (cmd: string, args?: any): Promise<any> => {
  console.log(`Mock invoke: ${cmd}`, args);
  
  // Add delay to simulate network
  await new Promise(resolve => setTimeout(resolve, 100));
  
  switch (cmd) {
    case 'get_available_profiles':
      return mockProfiles;
      
    case 'get_profiles':
      return mockProfiles.map(p => ({
        id: p.id,
        name: p.name,
        generator_model: 'claude-3-haiku',
        refiner_model: 'claude-3-sonnet',
        validator_model: 'gpt-4-turbo',
        curator_model: 'claude-3-opus'
      }));
      
    case 'get_active_profile':
      const active = mockProfiles.find(p => p.is_active);
      return active ? {
        id: active.id,
        name: active.name,
        generator_model: 'claude-3-haiku',
        refiner_model: 'claude-3-sonnet',
        validator_model: 'gpt-4-turbo',
        curator_model: 'claude-3-opus'
      } : null;
      
    case 'set_active_profile':
      // Update mock data
      mockProfiles.forEach(p => {
        p.is_active = p.id === args.profileId;
      });
      return true;
      
    case 'get_profile_config':
      return {
        generator_model: 'claude-3-haiku',
        generator_temperature: 0.7,
        refiner_model: 'claude-3-sonnet',
        refiner_temperature: 0.5,
        validator_model: 'gpt-4-turbo',
        validator_temperature: 0.3,
        curator_model: 'claude-3-opus',
        curator_temperature: 0.2
      };
      
    case 'read_directory':
      return mockFileSystem;
      
    case 'get_api_keys_status':
      return {
        openrouter: { configured: false },
        anthropic: { configured: false },
        hive: { configured: false }
      };
      
    case 'get_settings':
      return {
        theme: 'dark',
        autoSave: true,
        autoAccept: false,
        fontSize: 14,
        showWelcome: true
      };
      
    case 'run_consensus_streaming':
      // Simulate streaming consensus
      setTimeout(() => {
        window.dispatchEvent(new CustomEvent('consensus-progress', {
          detail: {
            stage: 'Generator',
            progress: 50,
            tokens: 100,
            cost: 0.01,
            message: 'Generating initial response...'
          }
        }));
      }, 500);
      
      setTimeout(() => {
        window.dispatchEvent(new CustomEvent('consensus-complete', {
          detail: 'This is a mock consensus response for testing the UI.'
        }));
      }, 2000);
      
      return true;
      
    default:
      console.warn(`Mock: Unhandled command ${cmd}`);
      return null;
  }
};

// Export a wrapper that uses real Tauri or mock based on environment
export const invoke = async (cmd: string, args?: any): Promise<any> => {
  if (isTauri()) {
    // Use real Tauri API
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return tauriInvoke(cmd, args);
  } else {
    // Use mock for browser development
    return mockInvoke(cmd, args);
  }
};