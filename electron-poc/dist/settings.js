"use strict";
/**
 * Settings Window Implementation
 * Manages API keys, profiles, and configuration
 */
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
require("./settings.css");
// Predefined profiles matching Rust implementation
const EXPERT_PROFILES = [
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
    constructor() {
        this.selectedProfileId = null;
        this.openrouterKey = '';
        this.hiveKey = '';
        this.initializeUI();
        this.loadSettings();
    }
    initializeUI() {
        var _a, _b, _c, _d, _e, _f, _g;
        // Close button
        (_a = document.getElementById('close-settings')) === null || _a === void 0 ? void 0 : _a.addEventListener('click', () => {
            window.close();
        });
        // Cancel button
        (_b = document.getElementById('cancel-settings')) === null || _b === void 0 ? void 0 : _b.addEventListener('click', () => {
            window.close();
        });
        // Toggle visibility buttons
        document.querySelectorAll('.toggle-visibility').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const target = e.target.getAttribute('data-target');
                if (target) {
                    const input = document.getElementById(target);
                    if (input) {
                        input.type = input.type === 'password' ? 'text' : 'password';
                        e.target.textContent = input.type === 'password' ? '👁' : '👁‍🗨';
                    }
                }
            });
        });
        // Test keys button
        (_c = document.getElementById('test-keys')) === null || _c === void 0 ? void 0 : _c.addEventListener('click', () => {
            this.testApiKeys();
        });
        // Save keys button
        (_d = document.getElementById('save-keys')) === null || _d === void 0 ? void 0 : _d.addEventListener('click', () => {
            this.saveApiKeys();
        });
        // Save profile button
        (_e = document.getElementById('save-profile')) === null || _e === void 0 ? void 0 : _e.addEventListener('click', () => {
            this.saveProfile();
        });
        // Apply settings button
        (_f = document.getElementById('apply-settings')) === null || _f === void 0 ? void 0 : _f.addEventListener('click', () => {
            this.applyAllSettings();
        });
        // Reset settings button
        (_g = document.getElementById('reset-settings')) === null || _g === void 0 ? void 0 : _g.addEventListener('click', () => {
            this.resetToDefaults();
        });
        // Load profiles
        this.loadProfiles();
    }
    loadProfiles() {
        const grid = document.getElementById('profiles-grid');
        if (!grid)
            return;
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
    loadSettings() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const settings = yield window.settingsAPI.loadSettings();
                // Load API keys
                if (settings.openrouterKey) {
                    document.getElementById('openrouter-key').value = settings.openrouterKey;
                    this.openrouterKey = settings.openrouterKey;
                }
                if (settings.hiveKey) {
                    document.getElementById('hive-key').value = settings.hiveKey;
                    this.hiveKey = settings.hiveKey;
                    this.validateHiveKey(settings.hiveKey);
                }
                // Load selected profile
                if (settings.selectedProfile) {
                    this.selectedProfileId = settings.selectedProfile;
                    document.querySelectorAll('.profile-card').forEach(card => {
                        if (card.dataset.profileId === settings.selectedProfile) {
                            card.classList.add('selected');
                        }
                        else {
                            card.classList.remove('selected');
                        }
                    });
                }
                // Load advanced settings
                if (settings.autoSave !== undefined) {
                    document.getElementById('auto-save').checked = settings.autoSave;
                }
                if (settings.showCosts !== undefined) {
                    document.getElementById('show-costs').checked = settings.showCosts;
                }
                if (settings.maxDailyConversations !== undefined) {
                    document.getElementById('max-daily-conversations').value = settings.maxDailyConversations;
                }
            }
            catch (error) {
                console.error('Failed to load settings:', error);
            }
        });
    }
    testApiKeys() {
        return __awaiter(this, void 0, void 0, function* () {
            const openrouterKey = document.getElementById('openrouter-key').value;
            const hiveKey = document.getElementById('hive-key').value;
            try {
                const result = yield window.settingsAPI.testKeys({
                    openrouterKey,
                    hiveKey
                });
                if (result.openrouterValid) {
                    this.showMessage('OpenRouter key is valid!', 'success');
                }
                else {
                    this.showMessage('OpenRouter key is invalid', 'error');
                }
                if (result.hiveValid) {
                    this.showMessage('Hive key is valid!', 'success');
                    this.updateLicenseStatus(result.licenseInfo);
                }
                else {
                    this.showMessage('Hive key is invalid', 'error');
                }
            }
            catch (error) {
                this.showMessage(`Failed to test keys: ${error}`, 'error');
            }
        });
    }
    saveApiKeys() {
        return __awaiter(this, void 0, void 0, function* () {
            const openrouterKey = document.getElementById('openrouter-key').value;
            const hiveKey = document.getElementById('hive-key').value;
            try {
                yield window.settingsAPI.saveKeys({
                    openrouterKey,
                    hiveKey
                });
                this.showMessage('API keys saved successfully!', 'success');
            }
            catch (error) {
                this.showMessage(`Failed to save keys: ${error}`, 'error');
            }
        });
    }
    saveProfile() {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.selectedProfileId) {
                this.showMessage('Please select a profile', 'error');
                return;
            }
            try {
                const profile = EXPERT_PROFILES.find(p => p.id === this.selectedProfileId);
                if (!profile)
                    return;
                yield window.settingsAPI.saveProfile(profile);
                this.showMessage('Profile saved successfully!', 'success');
            }
            catch (error) {
                this.showMessage(`Failed to save profile: ${error}`, 'error');
            }
        });
    }
    applyAllSettings() {
        return __awaiter(this, void 0, void 0, function* () {
            const openrouterKey = document.getElementById('openrouter-key').value;
            const hiveKey = document.getElementById('hive-key').value;
            const autoSave = document.getElementById('auto-save').checked;
            const showCosts = document.getElementById('show-costs').checked;
            const maxDaily = document.getElementById('max-daily-conversations').value;
            try {
                yield window.settingsAPI.saveAllSettings({
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
            }
            catch (error) {
                this.showMessage(`Failed to save settings: ${error}`, 'error');
            }
        });
    }
    resetToDefaults() {
        return __awaiter(this, void 0, void 0, function* () {
            if (!confirm('Are you sure you want to reset all settings to defaults?')) {
                return;
            }
            try {
                yield window.settingsAPI.resetSettings();
                this.loadSettings();
                this.showMessage('Settings reset to defaults', 'success');
            }
            catch (error) {
                this.showMessage(`Failed to reset settings: ${error}`, 'error');
            }
        });
    }
    validateHiveKey(key) {
        // Basic format validation
        if (key.startsWith('hive-') && key.length > 10) {
            this.updateLicenseStatus({ valid: true, tier: 'premium', dailyLimit: 1000 });
        }
        else {
            this.updateLicenseStatus({ valid: false });
        }
    }
    updateLicenseStatus(info) {
        const statusDiv = document.getElementById('license-status');
        if (!statusDiv)
            return;
        if (info.valid) {
            statusDiv.className = 'license-status valid';
            statusDiv.textContent = `✓ Valid ${info.tier} license - ${info.dailyLimit} conversations/day`;
        }
        else {
            statusDiv.className = 'license-status invalid';
            statusDiv.textContent = '✗ Invalid or expired license key';
        }
    }
    showMessage(message, type) {
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
//# sourceMappingURL=settings.js.map