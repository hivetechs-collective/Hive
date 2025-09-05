/**
 * Settings Window Implementation
 * Manages API keys, profiles, and configuration
 */

import './settings.css';

// Profile definition matching Rust structure
interface ConsensusProfile {
  id: string;
  name: string;
  description: string;
  generator: string;
  refiner: string;
  validator: string;
  curator: string;
  category: 'Speed' | 'Quality' | 'Cost' | 'Research';
  isDefault?: boolean;
}

// Predefined profiles matching Rust implementation
const EXPERT_PROFILES: ConsensusProfile[] = [
  {
    id: 'lightning-fast',
    name: 'Lightning Fast',
    description: 'Ultra-high-speed consensus for rapid prototyping',
    generator: 'claude-3-haiku',
    refiner: 'gpt-3.5-turbo',
    validator: 'gemini-flash',
    curator: 'claude-3-haiku',
    category: 'Speed'
  },
  {
    id: 'cost-conscious',
    name: 'Cost Conscious',
    description: 'Minimal cost while maintaining quality',
    generator: 'gpt-3.5-turbo',
    refiner: 'claude-3-haiku',
    validator: 'mistral-small',
    curator: 'gpt-3.5-turbo',
    category: 'Cost'
  },
  {
    id: 'balanced-performer',
    name: 'Balanced Performer',
    description: 'Optimal balance of speed, cost, and quality',
    generator: 'claude-3-sonnet',
    refiner: 'gpt-4-turbo',
    validator: 'gemini-pro',
    curator: 'claude-3-sonnet',
    category: 'Quality',
    isDefault: true
  },
  {
    id: 'production-grade',
    name: 'Production Grade',
    description: 'High-quality consensus for production systems',
    generator: 'gpt-4-turbo',
    refiner: 'claude-3-opus',
    validator: 'gpt-4',
    curator: 'claude-3-opus',
    category: 'Quality'
  },
  {
    id: 'deep-researcher',
    name: 'Deep Researcher',
    description: 'Maximum depth analysis for complex problems',
    generator: 'claude-3-opus',
    refiner: 'gpt-4',
    validator: 'claude-3-opus',
    curator: 'gpt-4',
    category: 'Research'
  },
  {
    id: 'code-specialist',
    name: 'Code Specialist',
    description: 'Optimized for software development tasks',
    generator: 'deepseek-coder',
    refiner: 'codellama-70b',
    validator: 'gpt-4-turbo',
    curator: 'claude-3-sonnet',
    category: 'Research'
  },
  {
    id: 'creative-writer',
    name: 'Creative Writer',
    description: 'Enhanced creativity for content generation',
    generator: 'claude-3-opus',
    refiner: 'gpt-4',
    validator: 'mistral-large',
    curator: 'claude-3-opus',
    category: 'Quality'
  },
  {
    id: 'fact-checker',
    name: 'Fact Checker',
    description: 'High accuracy with multiple validation stages',
    generator: 'gpt-4',
    refiner: 'claude-3-sonnet',
    validator: 'perplexity-online',
    curator: 'gpt-4-turbo',
    category: 'Research'
  },
  {
    id: 'rapid-iteration',
    name: 'Rapid Iteration',
    description: 'Fast feedback loops for iterative development',
    generator: 'gemini-flash',
    refiner: 'claude-3-haiku',
    validator: 'gpt-3.5-turbo',
    curator: 'gemini-flash',
    category: 'Speed'
  },
  {
    id: 'enterprise-grade',
    name: 'Enterprise Grade',
    description: 'Premium models for mission-critical applications',
    generator: 'gpt-4',
    refiner: 'claude-3-opus',
    validator: 'gpt-4',
    curator: 'claude-3-opus',
    category: 'Quality'
  }
];

class SettingsManager {
  private selectedProfileId: string | null = null;
  private openrouterKey: string = '';
  private hiveKey: string = '';

  constructor() {
    this.initializeUI();
    this.loadSettings();
  }

  private initializeUI() {
    // Close button
    document.getElementById('close-settings')?.addEventListener('click', () => {
      window.close();
    });

    // Cancel button
    document.getElementById('cancel-settings')?.addEventListener('click', () => {
      window.close();
    });

    // Toggle visibility buttons
    document.querySelectorAll('.toggle-visibility').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const target = (e.target as HTMLElement).getAttribute('data-target');
        if (target) {
          const input = document.getElementById(target) as HTMLInputElement;
          if (input) {
            input.type = input.type === 'password' ? 'text' : 'password';
            (e.target as HTMLElement).textContent = input.type === 'password' ? '👁' : '👁‍🗨';
          }
        }
      });
    });

    // Test keys button
    document.getElementById('test-keys')?.addEventListener('click', () => {
      this.testApiKeys();
    });

    // Save keys button
    document.getElementById('save-keys')?.addEventListener('click', () => {
      this.saveApiKeys();
    });

    // Save profile button
    document.getElementById('save-profile')?.addEventListener('click', () => {
      this.saveProfile();
    });

    // Apply settings button
    document.getElementById('apply-settings')?.addEventListener('click', () => {
      this.applyAllSettings();
    });

    // Reset settings button
    document.getElementById('reset-settings')?.addEventListener('click', () => {
      this.resetToDefaults();
    });

    // Load profiles
    this.loadProfiles();
  }

  private loadProfiles() {
    const grid = document.getElementById('profiles-grid');
    if (!grid) return;

    grid.innerHTML = '';

    EXPERT_PROFILES.forEach(profile => {
      const card = document.createElement('div');
      card.className = 'profile-card';
      if (profile.isDefault) {
        card.classList.add('selected');
        this.selectedProfileId = profile.id;
      }
      card.dataset.profileId = profile.id;

      card.innerHTML = `
        <h4>
          ${profile.name}
          ${profile.isDefault ? '<span class="profile-badge">DEFAULT</span>' : ''}
        </h4>
        <div class="profile-description">${profile.description}</div>
        <div class="profile-models">
          <div class="model-stage">
            <span class="stage-name">Generator:</span>
            <span class="model-name">${profile.generator}</span>
          </div>
          <div class="model-stage">
            <span class="stage-name">Refiner:</span>
            <span class="model-name">${profile.refiner}</span>
          </div>
          <div class="model-stage">
            <span class="stage-name">Validator:</span>
            <span class="model-name">${profile.validator}</span>
          </div>
          <div class="model-stage">
            <span class="stage-name">Curator:</span>
            <span class="model-name">${profile.curator}</span>
          </div>
        </div>
      `;

      card.addEventListener('click', () => {
        document.querySelectorAll('.profile-card').forEach(c => c.classList.remove('selected'));
        card.classList.add('selected');
        this.selectedProfileId = profile.id;
      });

      grid.appendChild(card);
    });
  }

  private async loadSettings() {
    try {
      const settings = await (window as any).settingsAPI.loadSettings();
      
      // Load API keys
      if (settings.openrouterKey) {
        (document.getElementById('openrouter-key') as HTMLInputElement).value = settings.openrouterKey;
        this.openrouterKey = settings.openrouterKey;
      }
      
      if (settings.hiveKey) {
        (document.getElementById('hive-key') as HTMLInputElement).value = settings.hiveKey;
        this.hiveKey = settings.hiveKey;
        this.validateHiveKey(settings.hiveKey);
      }

      // Load selected profile
      if (settings.selectedProfile) {
        this.selectedProfileId = settings.selectedProfile;
        document.querySelectorAll('.profile-card').forEach(card => {
          if ((card as HTMLElement).dataset.profileId === settings.selectedProfile) {
            card.classList.add('selected');
          } else {
            card.classList.remove('selected');
          }
        });
      }

      // Load advanced settings
      if (settings.autoSave !== undefined) {
        (document.getElementById('auto-save') as HTMLInputElement).checked = settings.autoSave;
      }
      if (settings.showCosts !== undefined) {
        (document.getElementById('show-costs') as HTMLInputElement).checked = settings.showCosts;
      }
      if (settings.maxDailyConversations !== undefined) {
        (document.getElementById('max-daily-conversations') as HTMLInputElement).value = settings.maxDailyConversations;
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  }

  private async testApiKeys() {
    const openrouterKey = (document.getElementById('openrouter-key') as HTMLInputElement).value;
    const hiveKey = (document.getElementById('hive-key') as HTMLInputElement).value;

    try {
      const result = await (window as any).settingsAPI.testKeys({
        openrouterKey,
        hiveKey
      });

      if (result.openrouterValid) {
        this.showMessage('OpenRouter key is valid!', 'success');
      } else {
        this.showMessage('OpenRouter key is invalid', 'error');
      }

      if (result.hiveValid) {
        this.showMessage('Hive key is valid!', 'success');
        this.updateLicenseStatus(result.licenseInfo);
      } else {
        this.showMessage('Hive key is invalid', 'error');
      }
    } catch (error) {
      this.showMessage(`Failed to test keys: ${error}`, 'error');
    }
  }

  private async saveApiKeys() {
    const openrouterKey = (document.getElementById('openrouter-key') as HTMLInputElement).value;
    const hiveKey = (document.getElementById('hive-key') as HTMLInputElement).value;

    try {
      await (window as any).settingsAPI.saveKeys({
        openrouterKey,
        hiveKey
      });
      this.showMessage('API keys saved successfully!', 'success');
    } catch (error) {
      this.showMessage(`Failed to save keys: ${error}`, 'error');
    }
  }

  private async saveProfile() {
    if (!this.selectedProfileId) {
      this.showMessage('Please select a profile', 'error');
      return;
    }

    try {
      const profile = EXPERT_PROFILES.find(p => p.id === this.selectedProfileId);
      if (!profile) return;

      await (window as any).settingsAPI.saveProfile(profile);
      this.showMessage('Profile saved successfully!', 'success');
    } catch (error) {
      this.showMessage(`Failed to save profile: ${error}`, 'error');
    }
  }

  private async applyAllSettings() {
    const openrouterKey = (document.getElementById('openrouter-key') as HTMLInputElement).value;
    const hiveKey = (document.getElementById('hive-key') as HTMLInputElement).value;
    const autoSave = (document.getElementById('auto-save') as HTMLInputElement).checked;
    const showCosts = (document.getElementById('show-costs') as HTMLInputElement).checked;
    const maxDaily = (document.getElementById('max-daily-conversations') as HTMLInputElement).value;

    try {
      await (window as any).settingsAPI.saveAllSettings({
        openrouterKey,
        hiveKey,
        selectedProfile: this.selectedProfileId,
        autoSave,
        showCosts,
        maxDailyConversations: parseInt(maxDaily)
      });
      
      this.showMessage('All settings saved successfully!', 'success');
      
      // Close window after successful save
      setTimeout(() => {
        window.close();
      }, 1000);
    } catch (error) {
      this.showMessage(`Failed to save settings: ${error}`, 'error');
    }
  }

  private async resetToDefaults() {
    if (!confirm('Are you sure you want to reset all settings to defaults?')) {
      return;
    }

    try {
      await (window as any).settingsAPI.resetSettings();
      this.loadSettings();
      this.showMessage('Settings reset to defaults', 'success');
    } catch (error) {
      this.showMessage(`Failed to reset settings: ${error}`, 'error');
    }
  }

  private validateHiveKey(key: string) {
    // Basic format validation
    if (key.startsWith('hive-') && key.length > 10) {
      this.updateLicenseStatus({ valid: true, tier: 'premium', dailyLimit: 1000 });
    } else {
      this.updateLicenseStatus({ valid: false });
    }
  }

  private updateLicenseStatus(info: any) {
    const statusDiv = document.getElementById('license-status');
    if (!statusDiv) return;

    if (info.valid) {
      statusDiv.className = 'license-status valid';
      statusDiv.textContent = `✓ Valid ${info.tier} license - ${info.dailyLimit} conversations/day`;
    } else {
      statusDiv.className = 'license-status invalid';
      statusDiv.textContent = '✗ Invalid or expired license key';
    }
  }

  private showMessage(message: string, type: 'success' | 'error') {
    // Create toast notification
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    toast.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      padding: 12px 20px;
      background: ${type === 'success' ? '#4CAF50' : '#F44336'};
      color: white;
      border-radius: 4px;
      z-index: 1000;
      animation: slideIn 0.3s ease;
    `;
    
    document.body.appendChild(toast);
    
    setTimeout(() => {
      toast.remove();
    }, 3000);
  }
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
  new SettingsManager();
});