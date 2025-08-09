/**
 * Settings Modal Implementation
 * In-app modal for managing API keys, profiles, and configuration
 */

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

export class SettingsModal {
  private selectedProfileId: string | null = null;
  private modalElement: HTMLElement | null = null;

  public createModal(): string {
    return `
      <div class="settings-modal-overlay" id="settings-modal-overlay" style="display: none;">
        <div class="settings-modal">
          <div class="settings-header">
            <h2>Settings</h2>
            <button class="close-btn" id="close-settings">×</button>
          </div>
          
          <div class="settings-content">
            <!-- API Keys Section -->
            <div class="settings-section">
              <h3>API Keys</h3>
              
              <div class="form-group">
                <label for="openrouter-key">OpenRouter API Key</label>
                <div class="input-group">
                  <input 
                    type="password" 
                    id="openrouter-key" 
                    placeholder="sk-or-v1-..." 
                    class="api-key-input"
                  />
                  <button class="toggle-visibility" data-target="openrouter-key">👁</button>
                </div>
                <small class="help-text">Your OpenRouter API key for AI model access</small>
              </div>
              
              <div class="form-group">
                <label for="hive-key">Hive License Key</label>
                <div class="input-group">
                  <input 
                    type="password" 
                    id="hive-key" 
                    placeholder="HIVE-XXXX-XXXX-XXXX" 
                    class="api-key-input"
                  />
                  <button class="toggle-visibility" data-target="hive-key">👁</button>
                </div>
                <small class="help-text">Your Hive license key determines your subscription tier</small>
                <div id="license-status" class="license-status"></div>
              </div>
              
              <div class="button-group">
                <button id="test-keys" class="btn btn-secondary">Test Keys</button>
                <button id="save-keys" class="btn btn-primary">Save Keys</button>
              </div>
            </div>
            
            <!-- Consensus Profiles Section -->
            <div class="settings-section">
              <h3>Consensus Profiles</h3>
              <p class="section-description">Select an expert profile for the 4-stage consensus pipeline</p>
              
              <div class="profiles-grid" id="profiles-grid">
                ${this.renderProfiles()}
              </div>
              
              <div class="button-group">
                <button id="save-profile" class="btn btn-primary">Save Profile Selection</button>
              </div>
            </div>
            
          </div>
          
          <div class="settings-footer">
            <button id="reset-settings" class="btn btn-danger">Reset to Defaults</button>
            <div class="footer-actions">
              <button id="cancel-settings" class="btn btn-secondary">Cancel</button>
              <button id="apply-settings" class="btn btn-primary">Apply</button>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  private renderProfiles(): string {
    return EXPERT_PROFILES.map(profile => `
      <div class="profile-card ${profile.isDefault ? 'selected' : ''}" data-profile-id="${profile.id}">
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
      </div>
    `).join('');
  }

  public initializeModal(parentElement: HTMLElement) {
    // Add modal HTML to the parent element
    const modalHTML = this.createModal();
    const modalContainer = document.createElement('div');
    modalContainer.innerHTML = modalHTML;
    parentElement.appendChild(modalContainer.firstElementChild!);

    this.modalElement = document.getElementById('settings-modal-overlay');
    
    // Don't set a default - load from database
    
    // Initialize event handlers
    this.setupEventHandlers();
    
    // Load saved settings
    this.loadSettings();
  }

  private setupEventHandlers() {
    // Close button
    document.getElementById('close-settings')?.addEventListener('click', () => {
      this.hideModal();
    });

    // Cancel button
    document.getElementById('cancel-settings')?.addEventListener('click', () => {
      this.hideModal();
    });

    // Click outside to close
    document.getElementById('settings-modal-overlay')?.addEventListener('click', (e) => {
      if (e.target === e.currentTarget) {
        this.hideModal();
      }
    });

    // Toggle visibility buttons
    document.querySelectorAll('.toggle-visibility').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const target = (e.target as HTMLElement).getAttribute('data-target');
        if (target) {
          const input = document.getElementById(target) as HTMLInputElement;
          if (input) {
            const actualKey = input.getAttribute('data-actual-key');
            const isMasked = input.getAttribute('data-masked') === 'true';
            
            if (actualKey) {
              // Toggle between masked and actual key
              if (isMasked) {
                // Show actual key
                input.value = actualKey;
                input.type = 'text';
                input.setAttribute('data-masked', 'false');
                (e.target as HTMLElement).textContent = '👁‍🗨';
              } else {
                // Show masked key
                input.value = this.maskApiKey(actualKey);
                input.type = 'password';
                input.setAttribute('data-masked', 'true');
                (e.target as HTMLElement).textContent = '👁';
              }
            } else {
              // No stored key, just toggle password visibility
              const isHidden = input.type === 'password';
              input.type = isHidden ? 'text' : 'password';
              (e.target as HTMLElement).textContent = isHidden ? '👁‍🗨' : '👁';
            }
          }
        }
      });
    });

    // Handle typing in API key fields
    ['openrouter-key', 'hive-key'].forEach(id => {
      const input = document.getElementById(id) as HTMLInputElement;
      if (input) {
        input.addEventListener('input', (e) => {
          const target = e.target as HTMLInputElement;
          // If user starts typing, clear the masked state
          if (target.value && !target.value.includes('•')) {
            target.setAttribute('data-masked', 'false');
            target.removeAttribute('data-actual-key');
          }
        });
      }
    });

    // Profile selection
    document.querySelectorAll('.profile-card').forEach(card => {
      card.addEventListener('click', () => {
        document.querySelectorAll('.profile-card').forEach(c => c.classList.remove('selected'));
        card.classList.add('selected');
        this.selectedProfileId = (card as HTMLElement).dataset.profileId || null;
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
  }

  public showModal() {
    if (this.modalElement) {
      this.modalElement.style.display = 'flex';
    }
  }

  public hideModal() {
    if (this.modalElement) {
      this.modalElement.style.display = 'none';
    }
  }

  private async loadSettings() {
    try {
      const settings = await (window as any).settingsAPI.loadSettings();
      
      // Load API keys - they come from database
      if (settings.openrouterKey) {
        const input = document.getElementById('openrouter-key') as HTMLInputElement;
        // Store the actual key but display it masked
        input.setAttribute('data-actual-key', settings.openrouterKey);
        input.value = this.maskApiKey(settings.openrouterKey);
        input.setAttribute('data-masked', 'true');
      }
      
      if (settings.hiveKey) {
        const input = document.getElementById('hive-key') as HTMLInputElement;
        // Store the actual key but display it masked
        input.setAttribute('data-actual-key', settings.hiveKey);
        input.value = this.maskApiKey(settings.hiveKey);
        input.setAttribute('data-masked', 'true');
        this.validateHiveKey(settings.hiveKey);
      }

      // Load selected profile from active_profile_id
      if (settings.activeProfileId || settings.activeProfileName) {
        // Try to find a matching predefined profile
        const matchingProfile = EXPERT_PROFILES.find(p => {
          // Match by name (case-insensitive)
          if (settings.activeProfileName) {
            return p.name.toLowerCase() === settings.activeProfileName.toLowerCase();
          }
          return false;
        });
        
        if (matchingProfile) {
          this.selectedProfileId = matchingProfile.id;
          document.querySelectorAll('.profile-card').forEach(card => {
            if ((card as HTMLElement).dataset.profileId === matchingProfile.id) {
              card.classList.add('selected');
            } else {
              card.classList.remove('selected');
            }
          });
        } else if (settings.activeProfileName) {
          // Custom profile is active, show a note
          const profilesGrid = document.getElementById('profiles-grid');
          if (profilesGrid) {
            const note = document.createElement('div');
            note.className = 'custom-profile-note';
            note.style.cssText = 'padding: 10px; background: #3c3c3c; border-radius: 4px; margin-bottom: 10px; color: #ffcc00;';
            note.textContent = `Currently using custom profile: "${settings.activeProfileName}". Select a profile below to change.`;
            profilesGrid.parentElement?.insertBefore(note, profilesGrid);
          }
        }
      } else if (settings.selectedProfile) {
        // Fallback to old selectedProfile if exists
        this.selectedProfileId = settings.selectedProfile;
        document.querySelectorAll('.profile-card').forEach(card => {
          if ((card as HTMLElement).dataset.profileId === settings.selectedProfile) {
            card.classList.add('selected');
          } else {
            card.classList.remove('selected');
          }
        });
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  }

  private async testApiKeys() {
    const openrouterInput = document.getElementById('openrouter-key') as HTMLInputElement;
    const hiveInput = document.getElementById('hive-key') as HTMLInputElement;
    
    // Get actual keys (not masked versions)
    const openrouterKey = openrouterInput.getAttribute('data-actual-key') || openrouterInput.value;
    const hiveKey = hiveInput.getAttribute('data-actual-key') || hiveInput.value;

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
    const openrouterInput = document.getElementById('openrouter-key') as HTMLInputElement;
    const hiveInput = document.getElementById('hive-key') as HTMLInputElement;
    
    // Get the actual key value (could be masked or not)
    let openrouterKey = openrouterInput.getAttribute('data-actual-key') || '';
    let hiveKey = hiveInput.getAttribute('data-actual-key') || '';
    
    // If user typed something new, use that instead
    const openrouterMasked = openrouterInput.getAttribute('data-masked') === 'true';
    const hiveMasked = hiveInput.getAttribute('data-masked') === 'true';
    
    if (!openrouterMasked && openrouterInput.value && !openrouterInput.value.includes('•')) {
      openrouterKey = openrouterInput.value.trim();
    }
    if (!hiveMasked && hiveInput.value && !hiveInput.value.includes('•')) {
      hiveKey = hiveInput.value.trim();
    }

    // Only save if there's actual input
    if (!openrouterKey && !hiveKey) {
      this.showMessage('Please enter at least one API key', 'error');
      return;
    }

    try {
      await (window as any).settingsAPI.saveKeys({
        openrouterKey,
        hiveKey
      });
      
      // Update stored keys and display masked versions
      if (openrouterKey) {
        openrouterInput.setAttribute('data-actual-key', openrouterKey);
        openrouterInput.value = this.maskApiKey(openrouterKey);
        openrouterInput.setAttribute('data-masked', 'true');
        openrouterInput.type = 'password';
      }
      if (hiveKey) {
        hiveInput.setAttribute('data-actual-key', hiveKey);
        hiveInput.value = this.maskApiKey(hiveKey);
        hiveInput.setAttribute('data-masked', 'true');
        hiveInput.type = 'password';
        this.validateHiveKey(hiveKey);
      }
      
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
    const openrouterInput = document.getElementById('openrouter-key') as HTMLInputElement;
    const hiveInput = document.getElementById('hive-key') as HTMLInputElement;
    
    // Get actual keys (not masked versions)
    const openrouterKey = openrouterInput.getAttribute('data-actual-key') || openrouterInput.value;
    const hiveKey = hiveInput.getAttribute('data-actual-key') || hiveInput.value;

    try {
      await (window as any).settingsAPI.saveAllSettings({
        openrouterKey,
        hiveKey,
        selectedProfile: this.selectedProfileId
      });
      
      this.showMessage('All settings saved successfully!', 'success');
      
      // Close modal after successful save
      setTimeout(() => {
        this.hideModal();
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

  private maskApiKey(key: string): string {
    if (!key || key.length < 8) return key;
    
    // Show first 6 characters and last 4 characters
    const firstPart = key.substring(0, 6);
    const lastPart = key.substring(key.length - 4);
    const maskedMiddle = '•'.repeat(Math.min(key.length - 10, 20));
    
    return `${firstPart}${maskedMiddle}${lastPart}`;
  }

  private validateHiveKey(key: string) {
    // Validate HIVE-XXXX-XXXX-XXXX format (4 characters per segment)
    const parts = key.split('-');
    if (parts.length >= 3 && parts[0] === 'HIVE') {
      // Check each segment is 4 characters of alphanumeric
      const validSegments = parts.slice(1).every(segment => 
        segment.length === 4 && /^[A-Z0-9]{4}$/.test(segment)
      );
      
      if (validSegments) {
        this.updateLicenseStatus({ valid: true, tier: 'premium', dailyLimit: 1000 });
      } else {
        this.updateLicenseStatus({ valid: false });
      }
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
      top: 60px;
      right: 20px;
      padding: 12px 20px;
      background: ${type === 'success' ? '#4CAF50' : '#F44336'};
      color: white;
      border-radius: 4px;
      z-index: 10000;
      animation: slideIn 0.3s ease;
    `;
    
    document.body.appendChild(toast);
    
    setTimeout(() => {
      toast.remove();
    }, 3000);
  }
}