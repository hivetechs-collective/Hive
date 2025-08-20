"use strict";
/**
 * Settings Modal Implementation
 * In-app modal for managing API keys, profiles, and configuration
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
exports.SettingsModal = void 0;
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
class SettingsModal {
    constructor(onSettingsChanged) {
        this.selectedProfileId = null;
        this.modalElement = null;
        this.profileCreationModal = null;
        this.availableModels = [];
        this.customProfiles = [];
        this.notificationQueue = [];
        this.onSettingsChanged = null;
        this.currentContainer = null;
        this.onSettingsChanged = onSettingsChanged || null;
    }
    createProfileCreationModal() {
        return `
      <div class="profile-creation-modal-overlay" id="profile-creation-modal-overlay" style="display: none;">
        <div class="profile-creation-modal" style="background: #2d2d30; border: 1px solid #3e3e42; border-radius: 8px; padding: 0; width: 800px; max-height: 80vh; overflow-y: auto;">
          <div class="profile-creation-header" style="padding: 20px; border-bottom: 1px solid #3e3e42;">
            <h2 style="margin: 0;">Create New Profile</h2>
            <button class="close-btn" id="close-profile-creation" style="position: absolute; top: 20px; right: 20px; background: none; border: none; color: #ccc; font-size: 24px; cursor: pointer;">×</button>
          </div>
          
          <div class="profile-tabs" style="display: flex; border-bottom: 1px solid #3e3e42; padding: 0 20px;">
            <button class="tab-btn active" data-tab="templates" style="padding: 10px 20px; background: none; border: none; color: #fff; cursor: pointer; border-bottom: 2px solid #007acc;">🎯 Expert Templates</button>
            <button class="tab-btn" data-tab="existing" style="padding: 10px 20px; background: none; border: none; color: #ccc; cursor: pointer; border-bottom: 2px solid transparent;">📋 Existing Profiles</button>
            <button class="tab-btn" data-tab="custom" style="padding: 10px 20px; background: none; border: none; color: #ccc; cursor: pointer; border-bottom: 2px solid transparent;">🛠️ Custom Profile</button>
          </div>
          
          <div class="tab-content" style="padding: 20px;">
            <!-- Expert Templates Tab -->
            <div id="templates-tab" class="tab-panel">
              <p style="color: #ccc; margin-bottom: 20px;">Select an expert-configured template optimized for specific use cases:</p>
              <div class="template-grid" style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px;">
                ${this.renderExpertTemplates()}
              </div>
              <div class="template-actions" style="margin-top: 20px; display: none;" id="template-actions">
                <input type="text" id="profile-name-input" placeholder="Enter profile name" style="width: 100%; padding: 8px; background: #3c3c3c; border: 1px solid #555; border-radius: 4px; color: #fff; margin-bottom: 10px;">
                <button id="create-from-template" class="btn btn-primary" style="width: 100%;">Create Profile</button>
              </div>
            </div>
            
            <!-- Existing Profiles Tab -->
            <div id="existing-tab" class="tab-panel" style="display: none;">
              <p style="color: #ccc;">Your existing profiles:</p>
              <div id="existing-profiles-list">
                <!-- Will be populated dynamically -->
              </div>
              <p style="color: #888; margin-top: 20px; font-size: 12px;">Note: To use an existing profile, close this dialog and select it from the main profiles section.</p>
            </div>
            
            <!-- Custom Profile Tab -->
            <div id="custom-tab" class="tab-panel" style="display: none;">
              <p style="color: #ccc; margin-bottom: 20px;">Build a custom 4-stage consensus pipeline by selecting models for each stage:</p>
              <div class="custom-builder">
                <div class="stage-selector" style="margin-bottom: 20px;">
                  <label style="display: block; color: #ccc; margin-bottom: 5px;">Profile Name:</label>
                  <input type="text" id="custom-profile-name" placeholder="My Custom Profile" style="width: 100%; padding: 8px; background: #3c3c3c; border: 1px solid #555; border-radius: 4px; color: #fff;">
                </div>
                
                <div class="stage-selector" style="margin-bottom: 20px;">
                  <label style="display: block; color: #ccc; margin-bottom: 5px;">🎯 Generator (Stage 1):</label>
                  <select id="generator-model" style="width: 100%; padding: 8px; background: #3c3c3c; border: 1px solid #555; border-radius: 4px; color: #fff;">
                    ${this.renderModelOptions('generator')}
                  </select>
                </div>
                
                <div class="stage-selector" style="margin-bottom: 20px;">
                  <label style="display: block; color: #ccc; margin-bottom: 5px;">✨ Refiner (Stage 2):</label>
                  <select id="refiner-model" style="width: 100%; padding: 8px; background: #3c3c3c; border: 1px solid #555; border-radius: 4px; color: #fff;">
                    ${this.renderModelOptions('refiner')}
                  </select>
                </div>
                
                <div class="stage-selector" style="margin-bottom: 20px;">
                  <label style="display: block; color: #ccc; margin-bottom: 5px;">✅ Validator (Stage 3):</label>
                  <select id="validator-model" style="width: 100%; padding: 8px; background: #3c3c3c; border: 1px solid #555; border-radius: 4px; color: #fff;">
                    ${this.renderModelOptions('validator')}
                  </select>
                </div>
                
                <div class="stage-selector" style="margin-bottom: 20px;">
                  <label style="display: block; color: #ccc; margin-bottom: 5px;">🎨 Curator (Stage 4):</label>
                  <select id="curator-model" style="width: 100%; padding: 8px; background: #3c3c3c; border: 1px solid #555; border-radius: 4px; color: #fff;">
                    ${this.renderModelOptions('curator')}
                  </select>
                </div>
                
                <button id="create-custom-profile" class="btn btn-primary" style="width: 100%;">Create Custom Profile</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    `;
    }
    createModal() {
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
                <button id="create-profile" class="btn btn-secondary">+ New Profile</button>
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
    renderExpertTemplates() {
        const templates = [
            { id: 'lightning-fast', name: '⚡ Lightning Fast', desc: 'Ultra-high-speed for rapid prototyping', models: { g: 'claude-3-haiku', r: 'gpt-3.5-turbo', v: 'gemini-flash', c: 'claude-3-haiku' } },
            { id: 'balanced-performer', name: '🎯 Balanced Performer', desc: 'Optimal balance of speed, cost, and quality', models: { g: 'claude-3-sonnet', r: 'gpt-4-turbo', v: 'gemini-pro', c: 'claude-3-sonnet' } },
            { id: 'precision-architect', name: '🏗️ Precision Architect', desc: 'Maximum quality for complex decisions', models: { g: 'gpt-4', r: 'claude-3-opus', v: 'gpt-4', c: 'claude-3-opus' } },
            { id: 'budget-optimizer', name: '💰 Budget Optimizer', desc: 'Cost-efficient consensus', models: { g: 'gpt-3.5-turbo', r: 'claude-3-haiku', v: 'mistral-small', c: 'gpt-3.5-turbo' } },
            { id: 'research-deep-dive', name: '🔬 Research Deep Dive', desc: 'Comprehensive analysis and research', models: { g: 'claude-3-opus', r: 'gpt-4', v: 'claude-3-opus', c: 'gpt-4' } },
            { id: 'code-specialist', name: '💻 Code Specialist', desc: 'Optimized for software development', models: { g: 'deepseek-coder', r: 'codellama-70b', v: 'gpt-4-turbo', c: 'claude-3-sonnet' } },
            { id: 'creative-innovator', name: '🎨 Creative Innovator', desc: 'High creativity for innovative solutions', models: { g: 'claude-3-opus', r: 'gpt-4', v: 'mistral-large', c: 'claude-3-opus' } },
            { id: 'enterprise-grade', name: '🏢 Enterprise Grade', desc: 'Production-ready with reliability', models: { g: 'gpt-4', r: 'claude-3-opus', v: 'gpt-4', c: 'claude-3-opus' } },
            { id: 'ml-ai-specialist', name: '🤖 ML/AI Specialist', desc: 'Specialized for machine learning', models: { g: 'claude-3-opus', r: 'gpt-4', v: 'gemini-pro', c: 'gpt-4' } },
            { id: 'debugging-detective', name: '🔍 Debugging Detective', desc: 'Methodical debugging and troubleshooting', models: { g: 'gpt-4-turbo', r: 'claude-3-sonnet', v: 'gpt-4', c: 'claude-3-sonnet' } }
        ];
        return templates.map(t => `
      <div class="template-card" data-template-id="${t.id}" style="padding: 15px; background: #3c3c3c; border-radius: 6px; cursor: pointer; border: 2px solid transparent;">
        <h4 style="margin: 0 0 5px 0; color: #fff;">${t.name}</h4>
        <p style="margin: 0 0 10px 0; color: #aaa; font-size: 12px;">${t.desc}</p>
        <div style="font-size: 11px; color: #888;">
          <div>G: ${t.models.g}</div>
          <div>R: ${t.models.r}</div>
          <div>V: ${t.models.v}</div>
          <div>C: ${t.models.c}</div>
        </div>
      </div>
    `).join('');
    }
    loadModelsFromDatabase() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                this.availableModels = yield window.settingsAPI.loadModels();
                console.log(`Loaded ${this.availableModels.length} models from database`);
            }
            catch (error) {
                console.error('Failed to load models from database:', error);
                this.availableModels = [];
            }
        });
    }
    renderModelOptions(stage) {
        // Use models loaded from database, or fallback to basic set
        let models = this.availableModels;
        if (!models || models.length === 0) {
            // Fallback to basic set if database doesn't have models yet
            models = [
                // OpenAI Models
                { value: 'gpt-4', label: 'GPT-4', provider: 'OpenAI' },
                { value: 'gpt-4-turbo', label: 'GPT-4 Turbo', provider: 'OpenAI' },
                { value: 'gpt-4-32k', label: 'GPT-4 32K', provider: 'OpenAI' },
                { value: 'gpt-3.5-turbo', label: 'GPT-3.5 Turbo', provider: 'OpenAI' },
                { value: 'gpt-3.5-turbo-16k', label: 'GPT-3.5 Turbo 16K', provider: 'OpenAI' },
                // Anthropic Models
                { value: 'claude-3-opus', label: 'Claude 3 Opus', provider: 'Anthropic' },
                { value: 'claude-3-sonnet', label: 'Claude 3 Sonnet', provider: 'Anthropic' },
                { value: 'claude-3-haiku', label: 'Claude 3 Haiku', provider: 'Anthropic' },
                { value: 'claude-2.1', label: 'Claude 2.1', provider: 'Anthropic' },
                { value: 'claude-2', label: 'Claude 2', provider: 'Anthropic' },
                { value: 'claude-instant', label: 'Claude Instant', provider: 'Anthropic' },
                // Google Models
                { value: 'gemini-pro', label: 'Gemini Pro', provider: 'Google' },
                { value: 'gemini-pro-vision', label: 'Gemini Pro Vision', provider: 'Google' },
                { value: 'gemini-flash', label: 'Gemini Flash', provider: 'Google' },
                { value: 'palm-2', label: 'PaLM 2', provider: 'Google' },
                // Meta Models
                { value: 'llama-3-70b', label: 'Llama 3 70B', provider: 'Meta' },
                { value: 'llama-3-8b', label: 'Llama 3 8B', provider: 'Meta' },
                { value: 'llama-2-70b', label: 'Llama 2 70B', provider: 'Meta' },
                { value: 'llama-2-13b', label: 'Llama 2 13B', provider: 'Meta' },
                { value: 'codellama-70b', label: 'CodeLlama 70B', provider: 'Meta' },
                { value: 'codellama-34b', label: 'CodeLlama 34B', provider: 'Meta' },
                // Mistral Models
                { value: 'mistral-large', label: 'Mistral Large', provider: 'Mistral' },
                { value: 'mistral-medium', label: 'Mistral Medium', provider: 'Mistral' },
                { value: 'mistral-small', label: 'Mistral Small', provider: 'Mistral' },
                { value: 'mixtral-8x7b', label: 'Mixtral 8x7B', provider: 'Mistral' },
                { value: 'mixtral-8x22b', label: 'Mixtral 8x22B', provider: 'Mistral' },
                // Cohere Models
                { value: 'command-r-plus', label: 'Command R+', provider: 'Cohere' },
                { value: 'command-r', label: 'Command R', provider: 'Cohere' },
                { value: 'command', label: 'Command', provider: 'Cohere' },
                // Specialized Models
                { value: 'deepseek-coder', label: 'DeepSeek Coder', provider: 'DeepSeek' },
                { value: 'wizardlm-2-8x22b', label: 'WizardLM 2 8x22B', provider: 'Microsoft' },
                { value: 'wizardcoder-33b', label: 'WizardCoder 33B', provider: 'Microsoft' },
                { value: 'phind-codellama-34b', label: 'Phind CodeLlama 34B', provider: 'Phind' },
                { value: 'perplexity-online', label: 'Perplexity Online', provider: 'Perplexity' },
                // Open Models
                { value: 'nous-hermes-2-mixtral', label: 'Nous Hermes 2 Mixtral', provider: 'Nous' },
                { value: 'dolphin-mixtral-8x7b', label: 'Dolphin Mixtral 8x7B', provider: 'Cognitive' },
                { value: 'yi-34b', label: 'Yi 34B', provider: '01.AI' },
                { value: 'qwen-72b', label: 'Qwen 72B', provider: 'Alibaba' },
            ];
        }
        // Group by provider
        const grouped = models.reduce((acc, model) => {
            const provider = model.provider || 'Unknown';
            if (!acc[provider])
                acc[provider] = [];
            acc[provider].push(model);
            return acc;
        }, {});
        let options = '<option value="">Select a model...</option>';
        // Add recommended models for each stage
        const recommended = this.getRecommendedModels(stage);
        if (recommended.length > 0) {
            options += '<optgroup label="⭐ Recommended">';
            recommended.forEach(model => {
                options += `<option value="${model.value}">${model.label}</option>`;
            });
            options += '</optgroup>';
        }
        // Add all models grouped by provider
        Object.entries(grouped).sort(([a], [b]) => a.localeCompare(b)).forEach(([provider, providerModels]) => {
            options += `<optgroup label="${provider}">`;
            providerModels.forEach((model) => {
                options += `<option value="${model.value}">${model.label}</option>`;
            });
            options += '</optgroup>';
        });
        return options;
    }
    getRecommendedModels(stage) {
        const recommendations = {
            generator: [
                { value: 'claude-3-opus', label: 'Claude 3 Opus' },
                { value: 'gpt-4-turbo', label: 'GPT-4 Turbo' },
                { value: 'claude-3-sonnet', label: 'Claude 3 Sonnet' },
            ],
            refiner: [
                { value: 'gpt-4', label: 'GPT-4' },
                { value: 'claude-3-sonnet', label: 'Claude 3 Sonnet' },
                { value: 'gpt-4-turbo', label: 'GPT-4 Turbo' },
            ],
            validator: [
                { value: 'gpt-4', label: 'GPT-4' },
                { value: 'claude-3-opus', label: 'Claude 3 Opus' },
                { value: 'gemini-pro', label: 'Gemini Pro' },
            ],
            curator: [
                { value: 'claude-3-opus', label: 'Claude 3 Opus' },
                { value: 'gpt-4', label: 'GPT-4' },
                { value: 'claude-3-sonnet', label: 'Claude 3 Sonnet' },
            ],
        };
        return recommendations[stage] || [];
    }
    renderProfiles() {
        // Combine expert profiles with custom profiles
        const allProfiles = [...EXPERT_PROFILES, ...this.customProfiles];
        return allProfiles.map(profile => `
      <div class="profile-card" data-profile-id="${profile.id}">
        <h4>
          ${profile.name}
          ${profile.isDefault ? '<span class="profile-badge">DEFAULT</span>' : ''}
          ${profile.isCustom ? '<span class="profile-badge" style="background: #28a745;">CUSTOM</span>' : ''}
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
    initializeModal(parentElement) {
        // Add modal HTML to the parent element
        const modalHTML = this.createModal();
        const modalContainer = document.createElement('div');
        modalContainer.innerHTML = modalHTML;
        parentElement.appendChild(modalContainer.firstElementChild);
        this.modalElement = document.getElementById('settings-modal-overlay');
        // Add profile creation modal
        const profileCreationHTML = this.createProfileCreationModal();
        const profileCreationContainer = document.createElement('div');
        profileCreationContainer.innerHTML = profileCreationHTML;
        parentElement.appendChild(profileCreationContainer.firstElementChild);
        this.profileCreationModal = document.getElementById('profile-creation-modal-overlay');
        // Don't set a default - load from database
        // Initialize event handlers
        this.setupEventHandlers();
        this.setupProfileCreationHandlers();
        // Load models from database first
        this.loadModelsFromDatabase().then(() => {
            // Update model dropdowns if profile creation modal exists
            this.updateModelDropdowns();
        });
        // Load saved settings
        this.loadSettings();
    }
    setupEventHandlers() {
        var _a, _b, _c, _d, _e, _f, _g, _h;
        // Close button
        (_a = document.getElementById('close-settings')) === null || _a === void 0 ? void 0 : _a.addEventListener('click', () => {
            this.hideModal();
        });
        // Cancel button
        (_b = document.getElementById('cancel-settings')) === null || _b === void 0 ? void 0 : _b.addEventListener('click', () => {
            this.hideModal();
        });
        // Click outside to close
        (_c = document.getElementById('settings-modal-overlay')) === null || _c === void 0 ? void 0 : _c.addEventListener('click', (e) => {
            if (e.target === e.currentTarget) {
                this.hideModal();
            }
        });
        // Toggle visibility buttons
        document.querySelectorAll('.toggle-visibility').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const target = e.target.getAttribute('data-target');
                if (target) {
                    const input = document.getElementById(target);
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
                                e.target.textContent = '👁‍🗨';
                            }
                            else {
                                // Show masked key
                                input.value = this.maskApiKey(actualKey);
                                input.type = 'password';
                                input.setAttribute('data-masked', 'true');
                                e.target.textContent = '👁';
                            }
                        }
                        else {
                            // No stored key, just toggle password visibility
                            const isHidden = input.type === 'password';
                            input.type = isHidden ? 'text' : 'password';
                            e.target.textContent = isHidden ? '👁‍🗨' : '👁';
                        }
                    }
                }
            });
        });
        // Handle typing in API key fields
        ['openrouter-key', 'hive-key'].forEach(id => {
            const input = document.getElementById(id);
            if (input) {
                input.addEventListener('input', (e) => {
                    const target = e.target;
                    // If user starts typing, clear the masked state
                    if (target.value && !target.value.includes('•')) {
                        target.setAttribute('data-masked', 'false');
                        target.removeAttribute('data-actual-key');
                    }
                });
            }
        });
        // Profile selection
        this.attachProfileCardHandlers();
        // Test keys button
        (_d = document.getElementById('test-keys')) === null || _d === void 0 ? void 0 : _d.addEventListener('click', () => {
            this.testApiKeys();
        });
        // Save keys button
        (_e = document.getElementById('save-keys')) === null || _e === void 0 ? void 0 : _e.addEventListener('click', () => {
            this.saveApiKeys();
        });
        // Apply settings button
        (_f = document.getElementById('apply-settings')) === null || _f === void 0 ? void 0 : _f.addEventListener('click', () => {
            this.applyAllSettings();
        });
        // Reset settings button
        (_g = document.getElementById('reset-settings')) === null || _g === void 0 ? void 0 : _g.addEventListener('click', () => {
            this.resetToDefaults();
        });
        // Create profile button
        (_h = document.getElementById('create-profile')) === null || _h === void 0 ? void 0 : _h.addEventListener('click', () => {
            this.showProfileCreationModal();
        });
    }
    setupProfileCreationHandlers() {
        var _a, _b, _c;
        // Close button
        (_a = document.getElementById('close-profile-creation')) === null || _a === void 0 ? void 0 : _a.addEventListener('click', () => {
            this.hideProfileCreationModal();
        });
        // Tab switching
        document.querySelectorAll('.tab-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const target = e.target;
                const tab = target.dataset.tab;
                if (tab) {
                    // Update active tab
                    document.querySelectorAll('.tab-btn').forEach(b => {
                        b.classList.remove('active');
                        b.style.borderBottomColor = 'transparent';
                        b.style.color = '#ccc';
                    });
                    target.classList.add('active');
                    target.style.borderBottomColor = '#007acc';
                    target.style.color = '#fff';
                    // Show corresponding panel
                    document.querySelectorAll('.tab-panel').forEach(panel => {
                        panel.style.display = 'none';
                    });
                    document.getElementById(`${tab}-tab`).style.display = 'block';
                    // Load existing profiles if needed
                    if (tab === 'existing') {
                        this.loadExistingProfiles();
                    }
                }
            });
        });
        // Template card selection
        document.querySelectorAll('.template-card').forEach(card => {
            card.addEventListener('click', (e) => {
                // Deselect all cards
                document.querySelectorAll('.template-card').forEach(c => {
                    c.style.borderColor = 'transparent';
                });
                // Select this card
                const target = e.currentTarget;
                target.style.borderColor = '#007acc';
                // Show name input
                const actions = document.getElementById('template-actions');
                if (actions) {
                    actions.style.display = 'block';
                    document.getElementById('profile-name-input').focus();
                }
            });
        });
        // Create from template button
        (_b = document.getElementById('create-from-template')) === null || _b === void 0 ? void 0 : _b.addEventListener('click', () => {
            this.createFromTemplate();
        });
        // Create custom profile button
        (_c = document.getElementById('create-custom-profile')) === null || _c === void 0 ? void 0 : _c.addEventListener('click', () => {
            this.createCustomProfile();
        });
    }
    showModal() {
        if (this.modalElement) {
            this.modalElement.style.display = 'flex';
        }
    }
    hideModal() {
        if (this.modalElement) {
            this.modalElement.style.display = 'none';
        }
    }
    loadSettings() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const settings = yield window.settingsAPI.loadSettings();
                // Load all profiles including custom ones
                yield this.loadAllProfiles();
                // Load API keys - they come from database
                if (settings.openrouterKey) {
                    const input = document.getElementById('openrouter-key');
                    // Store the actual key but display it masked
                    input.setAttribute('data-actual-key', settings.openrouterKey);
                    input.value = this.maskApiKey(settings.openrouterKey);
                    input.setAttribute('data-masked', 'true');
                }
                if (settings.hiveKey) {
                    const input = document.getElementById('hive-key');
                    // Store the actual key but display it masked
                    input.setAttribute('data-actual-key', settings.hiveKey);
                    input.value = this.maskApiKey(settings.hiveKey);
                    input.setAttribute('data-masked', 'true');
                    // Automatically refresh usage from D1 when opening settings
                    // This also validates the key format and updates the display
                    this.refreshLicenseStatus(settings.hiveKey);
                }
                // Load selected profile from active_profile_id
                if (settings.activeProfileId || settings.activeProfileName) {
                    // Try to find matching profile in all profiles (expert + custom)
                    const allProfiles = [...EXPERT_PROFILES, ...this.customProfiles];
                    const matchingProfile = allProfiles.find(p => {
                        var _a;
                        return p.id === settings.activeProfileId ||
                            p.name.toLowerCase() === ((_a = settings.activeProfileName) === null || _a === void 0 ? void 0 : _a.toLowerCase());
                    });
                    if (matchingProfile) {
                        this.selectedProfileId = matchingProfile.id;
                        // Update UI after profiles are rendered
                        setTimeout(() => {
                            document.querySelectorAll('.profile-card').forEach(card => {
                                if (card.dataset.profileId === matchingProfile.id) {
                                    card.classList.add('selected');
                                }
                                else {
                                    card.classList.remove('selected');
                                }
                            });
                        }, 100);
                    }
                }
                else if (settings.selectedProfile) {
                    // Fallback to old selectedProfile if exists
                    this.selectedProfileId = settings.selectedProfile;
                    setTimeout(() => {
                        document.querySelectorAll('.profile-card').forEach(card => {
                            if (card.dataset.profileId === settings.selectedProfile) {
                                card.classList.add('selected');
                            }
                            else {
                                card.classList.remove('selected');
                            }
                        });
                    }, 100);
                }
                else {
                    // No profile set at all - don't select any default
                    setTimeout(() => {
                        document.querySelectorAll('.profile-card').forEach(card => {
                            card.classList.remove('selected');
                        });
                    }, 100);
                }
            }
            catch (error) {
                console.error('Failed to load settings:', error);
            }
        });
    }
    testApiKeys() {
        var _a, _b, _c;
        return __awaiter(this, void 0, void 0, function* () {
            const openrouterInput = document.getElementById('openrouter-key');
            const hiveInput = document.getElementById('hive-key');
            // Get actual keys (not masked versions)
            const openrouterKey = openrouterInput.getAttribute('data-actual-key') || openrouterInput.value;
            const hiveKey = hiveInput.getAttribute('data-actual-key') || hiveInput.value;
            // Clear existing notifications
            this.clearAllNotifications();
            try {
                const result = yield window.settingsAPI.testKeys({
                    openrouterKey,
                    hiveKey
                });
                // Build combined result message
                const results = [];
                let hasError = false;
                // Check OpenRouter key
                if (openrouterKey) {
                    if (result.openrouterValid) {
                        results.push('✅ OpenRouter key is valid');
                    }
                    else {
                        results.push('❌ OpenRouter key is invalid');
                        hasError = true;
                    }
                }
                else {
                    results.push('⚠️ No OpenRouter key provided');
                }
                // Check Hive key
                if (hiveKey) {
                    if (result.hiveValid) {
                        results.push('✅ Hive key is valid');
                        if (result.licenseInfo) {
                            results.push(`📊 Tier: ${result.licenseInfo.tier}`);
                            // Get actual usage from local database (same source as status bar)
                            try {
                                const localUsage = yield ((_a = window.electronAPI) === null || _a === void 0 ? void 0 : _a.getUsageCount());
                                if (localUsage) {
                                    console.log('Using local usage data in settings:', localUsage);
                                    if (localUsage.limit === 999999) {
                                        results.push(`📈 Used today: ${localUsage.used}`);
                                        results.push(`✅ Unlimited conversations`);
                                    }
                                    else {
                                        results.push(`📈 Used today: ${localUsage.used}`);
                                        results.push(`✅ Remaining today: ${localUsage.remaining}`);
                                        results.push(`💬 Daily limit: ${localUsage.limit}`);
                                    }
                                }
                                else {
                                    // Fallback to D1 data if local DB fails
                                    if (result.licenseInfo.remaining !== undefined) {
                                        if (result.licenseInfo.remaining === 'unlimited') {
                                            results.push(`✅ Unlimited conversations`);
                                        }
                                        else {
                                            if (result.licenseInfo.dailyUsed !== undefined) {
                                                results.push(`📈 Used today: ${result.licenseInfo.dailyUsed}`);
                                                results.push(`✅ Remaining today: ${result.licenseInfo.remaining}`);
                                            }
                                            else {
                                                results.push(`💬 Daily limit: ${result.licenseInfo.dailyLimit || '?'}`);
                                                results.push(`✅ Remaining today: ${result.licenseInfo.remaining}`);
                                            }
                                        }
                                    }
                                    else if (result.licenseInfo.dailyUsed !== undefined && result.licenseInfo.dailyLimit !== undefined) {
                                        const remaining = result.licenseInfo.dailyLimit - result.licenseInfo.dailyUsed;
                                        results.push(`📈 Used today: ${result.licenseInfo.dailyUsed}`);
                                        results.push(`✅ Remaining today: ${remaining}`);
                                    }
                                    else if (result.licenseInfo.dailyLimit !== undefined) {
                                        results.push(`💬 Daily limit: ${result.licenseInfo.dailyLimit} conversations`);
                                    }
                                }
                            }
                            catch (error) {
                                console.error('Failed to get local usage in settings:', error);
                                // Use D1 data as fallback
                                if (result.licenseInfo.dailyUsed !== undefined) {
                                    results.push(`📈 Used today: ${result.licenseInfo.dailyUsed} (from D1)`);
                                }
                            }
                            if (result.licenseInfo.email) {
                                results.push(`📧 Account: ${result.licenseInfo.email}`);
                            }
                            // Update license status with local usage data
                            const updatedLicenseInfo = Object.assign({}, result.licenseInfo);
                            try {
                                const localUsage = yield ((_b = window.electronAPI) === null || _b === void 0 ? void 0 : _b.getUsageCount());
                                if (localUsage) {
                                    updatedLicenseInfo.dailyUsed = localUsage.used;
                                    updatedLicenseInfo.remaining = localUsage.remaining;
                                }
                            }
                            catch (error) {
                                console.error('Failed to get local usage for license status:', error);
                            }
                            this.updateLicenseStatus(updatedLicenseInfo);
                        }
                    }
                    else {
                        results.push('❌ Hive key is invalid');
                        if ((_c = result.licenseInfo) === null || _c === void 0 ? void 0 : _c.error) {
                            results.push(`⚠️ ${result.licenseInfo.error}`);
                        }
                        hasError = true;
                    }
                }
                else {
                    results.push('⚠️ No Hive key provided');
                }
                // Show combined notification
                this.showCombinedMessage(results, hasError ? 'mixed' : 'success');
            }
            catch (error) {
                this.showMessage(`Failed to test keys: ${error}`, 'error');
            }
        });
    }
    saveApiKeys() {
        return __awaiter(this, void 0, void 0, function* () {
            const openrouterInput = document.getElementById('openrouter-key');
            const hiveInput = document.getElementById('hive-key');
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
                yield window.settingsAPI.saveKeys({
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
                // Update status bar when keys are saved
                if (this.onSettingsChanged) {
                    this.onSettingsChanged();
                }
            }
            catch (error) {
                this.showMessage(`Failed to save keys: ${error}`, 'error');
            }
        });
    }
    applyAllSettings() {
        return __awaiter(this, void 0, void 0, function* () {
            const openrouterInput = document.getElementById('openrouter-key');
            const hiveInput = document.getElementById('hive-key');
            // Get actual keys (not masked versions)
            const openrouterKey = openrouterInput.getAttribute('data-actual-key') || openrouterInput.value;
            const hiveKey = hiveInput.getAttribute('data-actual-key') || hiveInput.value;
            try {
                // Save all settings including the selected profile
                yield window.settingsAPI.saveAllSettings({
                    openrouterKey,
                    hiveKey,
                    selectedProfile: this.selectedProfileId
                });
                // Profile is now saved to database, the callback will reload it
                this.showMessage('All settings saved successfully!', 'success');
                // Call the callback if provided to update UI
                if (this.onSettingsChanged) {
                    this.onSettingsChanged();
                }
                // Don't close modal automatically when in tab mode
                if (this.modalElement && this.modalElement.style.display !== 'none') {
                    setTimeout(() => {
                        this.hideModal();
                    }, 1000);
                }
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
    maskApiKey(key) {
        if (!key || key.length < 8)
            return key;
        // Show first 6 characters and last 4 characters
        const firstPart = key.substring(0, 6);
        const lastPart = key.substring(key.length - 4);
        const maskedMiddle = '•'.repeat(Math.min(key.length - 10, 20));
        return `${firstPart}${maskedMiddle}${lastPart}`;
    }
    validateHiveKey(key) {
        // Validate HIVE-XXXX-XXXX-XXXX format (4 characters per segment)
        const parts = key.split('-');
        if (parts.length >= 3 && parts[0] === 'HIVE') {
            // Check each segment is 4 characters of alphanumeric
            const validSegments = parts.slice(1).every(segment => segment.length === 4 && /^[A-Z0-9]{4}$/.test(segment));
            if (validSegments) {
                this.updateLicenseStatus({ valid: true, tier: 'premium', dailyLimit: 1000 });
            }
            else {
                this.updateLicenseStatus({ valid: false });
            }
        }
        else {
            this.updateLicenseStatus({ valid: false });
        }
    }
    refreshLicenseStatus(hiveKey) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            const statusDiv = document.getElementById('license-status');
            if (statusDiv) {
                // Show loading state
                statusDiv.className = 'license-status valid';
                statusDiv.textContent = '🔄 Checking license status...';
            }
            try {
                // Test the key to get license info from D1
                const result = yield window.settingsAPI.testKeys({
                    hiveKey: hiveKey
                });
                if (result.hiveValid && result.licenseInfo) {
                    // Get actual usage from local database (same source as status bar)
                    const updatedLicenseInfo = Object.assign({}, result.licenseInfo);
                    try {
                        const localUsage = yield ((_a = window.electronAPI) === null || _a === void 0 ? void 0 : _a.getUsageCount());
                        if (localUsage) {
                            console.log('Using local usage for license status:', localUsage);
                            updatedLicenseInfo.dailyUsed = localUsage.used;
                            updatedLicenseInfo.remaining = localUsage.remaining;
                            updatedLicenseInfo.dailyLimit = localUsage.limit;
                        }
                    }
                    catch (error) {
                        console.error('Failed to get local usage for refresh:', error);
                    }
                    // Update the display with combined data (D1 license info + local usage)
                    this.updateLicenseStatus(updatedLicenseInfo);
                    // Also update the main status bar
                    if (this.onSettingsChanged) {
                        this.onSettingsChanged();
                    }
                }
                else {
                    // Key validation failed - show error
                    this.updateLicenseStatus({ valid: false });
                }
            }
            catch (error) {
                console.error('Failed to refresh license status:', error);
                // Show cached status or error
                this.updateLicenseStatus({
                    valid: false,
                    error: 'Unable to check license status - check network connection'
                });
            }
        });
    }
    updateLicenseStatus(info) {
        const statusDiv = document.getElementById('license-status');
        if (!statusDiv)
            return;
        // Check if info has license information (from D1 response)
        if (info && (info.tier || info.dailyLimit || info.valid === true)) {
            statusDiv.className = 'license-status valid';
            let statusText = `✓ Valid ${info.tier || 'standard'} license - ${info.dailyLimit || 10} conversations/day`;
            if (info.dailyUsed !== undefined && info.remaining !== undefined) {
                statusText += ` (${info.dailyUsed} used, ${info.remaining} remaining today)`;
            }
            else if (info.remaining !== undefined) {
                if (info.remaining === 'unlimited') {
                    statusText = `✓ Valid ${info.tier || 'standard'} license - Unlimited conversations`;
                }
                else {
                    statusText += ` (${info.remaining} remaining today)`;
                }
            }
            if (info.email) {
                statusText += ` - ${info.email}`;
            }
            statusDiv.textContent = statusText;
        }
        else {
            statusDiv.className = 'license-status invalid';
            statusDiv.textContent = (info === null || info === void 0 ? void 0 : info.error) || '✗ Invalid or expired license key';
        }
    }
    showProfileCreationModal() {
        if (this.profileCreationModal) {
            this.profileCreationModal.style.display = 'flex';
            // Update model dropdowns when showing modal
            this.updateModelDropdowns();
        }
    }
    updateModelDropdowns() {
        // Update all model dropdowns with latest data from database
        ['generator', 'refiner', 'validator', 'curator'].forEach(stage => {
            const selectEl = document.getElementById(`${stage}-model`);
            if (selectEl) {
                const currentValue = selectEl.value;
                selectEl.innerHTML = this.renderModelOptions(stage);
                // Restore previous selection if it still exists
                if (currentValue && Array.from(selectEl.options).some(opt => opt.value === currentValue)) {
                    selectEl.value = currentValue;
                }
            }
        });
    }
    hideProfileCreationModal() {
        if (this.profileCreationModal) {
            this.profileCreationModal.style.display = 'none';
        }
    }
    loadExistingProfiles() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const profiles = yield window.settingsAPI.loadProfiles();
                const listEl = document.getElementById('existing-profiles-list');
                if (listEl) {
                    if (profiles && profiles.length > 0) {
                        listEl.innerHTML = profiles.map((p) => `
            <div class="existing-profile-card" style="padding: 10px; background: #3c3c3c; border-radius: 4px; margin-bottom: 10px;">
              <h4 style="margin: 0; color: #fff;">${p.name}</h4>
              <p style="margin: 5px 0 0 0; color: #aaa; font-size: 12px;">G: ${p.generator} | R: ${p.refiner} | V: ${p.validator} | C: ${p.curator}</p>
            </div>
          `).join('');
                    }
                    else {
                        listEl.innerHTML = '<p style="color: #888;">No existing profiles found. Create one from templates or build a custom profile.</p>';
                    }
                }
            }
            catch (error) {
                console.error('Failed to load profiles:', error);
            }
        });
    }
    createFromTemplate() {
        return __awaiter(this, void 0, void 0, function* () {
            const selectedCard = document.querySelector('.template-card[style*="border-color: rgb(0, 122, 204)"]');
            const nameInput = document.getElementById('profile-name-input');
            if (!selectedCard || !nameInput.value.trim()) {
                this.showMessage('Please select a template and enter a profile name', 'error');
                return;
            }
            const templateId = selectedCard.dataset.templateId;
            const profileName = nameInput.value.trim();
            // Get template models
            const templates = {
                'lightning-fast': { generator: 'claude-3-haiku', refiner: 'gpt-3.5-turbo', validator: 'gemini-flash', curator: 'claude-3-haiku', desc: 'Based on Lightning Fast template' },
                'balanced-performer': { generator: 'claude-3-sonnet', refiner: 'gpt-4-turbo', validator: 'gemini-pro', curator: 'claude-3-sonnet', desc: 'Based on Balanced Performer template' },
                'precision-architect': { generator: 'gpt-4', refiner: 'claude-3-opus', validator: 'gpt-4', curator: 'claude-3-opus', desc: 'Based on Precision Architect template' },
                'budget-optimizer': { generator: 'gpt-3.5-turbo', refiner: 'claude-3-haiku', validator: 'mistral-small', curator: 'gpt-3.5-turbo', desc: 'Based on Budget Optimizer template' },
                'research-deep-dive': { generator: 'claude-3-opus', refiner: 'gpt-4', validator: 'claude-3-opus', curator: 'gpt-4', desc: 'Based on Research Deep Dive template' },
                'code-specialist': { generator: 'deepseek-coder', refiner: 'codellama-70b', validator: 'gpt-4-turbo', curator: 'claude-3-sonnet', desc: 'Based on Code Specialist template' },
                'creative-innovator': { generator: 'claude-3-opus', refiner: 'gpt-4', validator: 'mistral-large', curator: 'claude-3-opus', desc: 'Based on Creative Innovator template' },
                'enterprise-grade': { generator: 'gpt-4', refiner: 'claude-3-opus', validator: 'gpt-4', curator: 'claude-3-opus', desc: 'Based on Enterprise Grade template' },
                'ml-ai-specialist': { generator: 'claude-3-opus', refiner: 'gpt-4', validator: 'gemini-pro', curator: 'gpt-4', desc: 'Based on ML/AI Specialist template' },
                'debugging-detective': { generator: 'gpt-4-turbo', refiner: 'claude-3-sonnet', validator: 'gpt-4', curator: 'claude-3-sonnet', desc: 'Based on Debugging Detective template' },
            };
            const template = templates[templateId];
            if (!template)
                return;
            try {
                const profile = {
                    id: `${templateId}-${Date.now()}`,
                    name: profileName,
                    generator: template.generator,
                    refiner: template.refiner,
                    validator: template.validator,
                    curator: template.curator
                };
                yield window.settingsAPI.saveProfile(profile);
                this.showMessage(`Profile "${profileName}" created successfully!`, 'success');
                // Add the new custom profile to our local list
                const customProfile = Object.assign(Object.assign({}, profile), { description: template.desc, category: 'Custom', isCustom: true });
                this.customProfiles.push(customProfile);
                // Re-render the profiles grid
                const profilesGrid = document.getElementById('profiles-grid');
                if (profilesGrid) {
                    profilesGrid.innerHTML = this.renderProfiles();
                    // Re-attach event handlers
                    this.attachProfileCardHandlers();
                }
                // Select the newly created profile
                this.selectedProfileId = profile.id;
                setTimeout(() => {
                    document.querySelectorAll('.profile-card').forEach(card => {
                        if (card.dataset.profileId === profile.id) {
                            card.classList.add('selected');
                        }
                        else {
                            card.classList.remove('selected');
                        }
                    });
                }, 100);
                // Close modal
                setTimeout(() => {
                    this.hideProfileCreationModal();
                }, 1000);
            }
            catch (error) {
                this.showMessage(`Failed to create profile: ${error}`, 'error');
            }
        });
    }
    createCustomProfile() {
        return __awaiter(this, void 0, void 0, function* () {
            const nameInput = document.getElementById('custom-profile-name');
            const generatorSelect = document.getElementById('generator-model');
            const refinerSelect = document.getElementById('refiner-model');
            const validatorSelect = document.getElementById('validator-model');
            const curatorSelect = document.getElementById('curator-model');
            if (!nameInput.value.trim() || !generatorSelect.value || !refinerSelect.value || !validatorSelect.value || !curatorSelect.value) {
                this.showMessage('Please fill in all fields', 'error');
                return;
            }
            try {
                const profile = {
                    id: `custom-${Date.now()}`,
                    name: nameInput.value.trim(),
                    generator: generatorSelect.value,
                    refiner: refinerSelect.value,
                    validator: validatorSelect.value,
                    curator: curatorSelect.value
                };
                yield window.settingsAPI.saveProfile(profile);
                this.showMessage(`Custom profile "${profile.name}" created successfully!`, 'success');
                // Add the new custom profile to our local list
                const customProfile = Object.assign(Object.assign({}, profile), { description: 'Custom profile created by user', category: 'Custom', isCustom: true });
                this.customProfiles.push(customProfile);
                // Re-render the profiles grid
                const profilesGrid = document.getElementById('profiles-grid');
                if (profilesGrid) {
                    profilesGrid.innerHTML = this.renderProfiles();
                    // Re-attach event handlers
                    this.attachProfileCardHandlers();
                }
                // Select the newly created profile
                this.selectedProfileId = profile.id;
                setTimeout(() => {
                    document.querySelectorAll('.profile-card').forEach(card => {
                        if (card.dataset.profileId === profile.id) {
                            card.classList.add('selected');
                        }
                        else {
                            card.classList.remove('selected');
                        }
                    });
                }, 100);
                // Close modal
                setTimeout(() => {
                    this.hideProfileCreationModal();
                }, 1000);
            }
            catch (error) {
                this.showMessage(`Failed to create custom profile: ${error}`, 'error');
            }
        });
    }
    loadAllProfiles() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Load all profiles from database
                const profiles = yield window.settingsAPI.loadProfiles();
                // Separate custom profiles from predefined ones
                this.customProfiles = [];
                for (const profile of profiles) {
                    // Check if it's not a predefined profile
                    const isPredefined = EXPERT_PROFILES.some(p => p.id === profile.id);
                    if (!isPredefined) {
                        // It's a custom profile
                        this.customProfiles.push({
                            id: profile.id,
                            name: profile.name,
                            description: 'Custom profile created by user',
                            generator: profile.generator,
                            refiner: profile.refiner,
                            validator: profile.validator,
                            curator: profile.curator,
                            category: 'Custom',
                            isCustom: true
                        });
                    }
                }
                // Re-render profiles grid if it exists
                const profilesGrid = document.getElementById('profiles-grid');
                if (profilesGrid) {
                    profilesGrid.innerHTML = this.renderProfiles();
                    this.attachProfileCardHandlers();
                }
            }
            catch (error) {
                console.error('Failed to load profiles:', error);
            }
        });
    }
    attachProfileCardHandlers() {
        document.querySelectorAll('.profile-card').forEach(card => {
            card.addEventListener('click', () => __awaiter(this, void 0, void 0, function* () {
                document.querySelectorAll('.profile-card').forEach(c => c.classList.remove('selected'));
                card.classList.add('selected');
                this.selectedProfileId = card.dataset.profileId || null;
                // Instantly update the consensus panel with the selected profile
                if (this.selectedProfileId) {
                    const profile = [...EXPERT_PROFILES, ...this.customProfiles].find(p => p.id === this.selectedProfileId);
                    if (profile) {
                        try {
                            // Save the profile immediately to database
                            yield window.settingsAPI.saveProfile(profile);
                            // Call the settings changed callback to reload from database and update UI
                            if (this.onSettingsChanged) {
                                this.onSettingsChanged();
                            }
                            // Show feedback
                            this.showMessage(`Profile switched to: ${profile.name}`, 'success');
                        }
                        catch (error) {
                            console.error('Failed to apply profile:', error);
                        }
                    }
                }
            }));
        });
    }
    showMessage(message, type) {
        // Determine where to append the toast
        const targetContainer = this.currentContainer || document.body;
        const isInTab = this.currentContainer !== null;
        // Calculate position based on existing notifications
        const existingToasts = targetContainer.querySelectorAll('.toast');
        const topOffset = isInTab ? 20 + (existingToasts.length * 60) : 60 + (existingToasts.length * 60);
        // Create toast notification
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        toast.textContent = message;
        toast.style.cssText = `
      position: ${isInTab ? 'absolute' : 'fixed'};
      top: ${topOffset}px;
      right: 20px;
      padding: 12px 20px;
      background: ${type === 'success' ? '#4CAF50' : type === 'error' ? '#F44336' : '#2196F3'};
      color: white;
      border-radius: 4px;
      z-index: 10000;
      animation: slideIn 0.3s ease;
      box-shadow: 0 2px 5px rgba(0,0,0,0.2);
      max-width: 400px;
    `;
        targetContainer.appendChild(toast);
        this.notificationQueue.push(toast);
        setTimeout(() => {
            toast.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => {
                toast.remove();
                const index = this.notificationQueue.indexOf(toast);
                if (index > -1) {
                    this.notificationQueue.splice(index, 1);
                }
                // Reposition remaining notifications
                this.repositionNotifications();
            }, 300);
        }, 3000);
    }
    showCombinedMessage(messages, type) {
        // Clear existing notifications first
        this.clearAllNotifications();
        // Create combined toast notification
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        // Create formatted content
        const content = document.createElement('div');
        content.innerHTML = messages.map(msg => `<div style="margin: 2px 0;">${msg}</div>`).join('');
        toast.appendChild(content);
        const bgColor = type === 'success' ? '#4CAF50' : type === 'error' ? '#F44336' : '#FF9800';
        toast.style.cssText = `
      position: fixed;
      top: 60px;
      right: 20px;
      padding: 15px 20px;
      background: ${bgColor};
      color: white;
      border-radius: 6px;
      z-index: 10000;
      animation: slideIn 0.3s ease;
      box-shadow: 0 4px 10px rgba(0,0,0,0.3);
      max-width: 400px;
      font-size: 14px;
      line-height: 1.5;
    `;
        document.body.appendChild(toast);
        this.notificationQueue.push(toast);
        setTimeout(() => {
            toast.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => {
                toast.remove();
                const index = this.notificationQueue.indexOf(toast);
                if (index > -1) {
                    this.notificationQueue.splice(index, 1);
                }
            }, 300);
        }, 5000); // Longer duration for combined message
    }
    clearAllNotifications() {
        this.notificationQueue.forEach(toast => {
            toast.remove();
        });
        this.notificationQueue = [];
    }
    repositionNotifications() {
        this.notificationQueue.forEach((toast, index) => {
            toast.style.top = `${60 + (index * 60)}px`;
        });
    }
    renderInContainer(container) {
        // Store the container for context-aware toast notifications
        this.currentContainer = container;
        // Render the actual settings content (without modal wrapper) in the container
        const modalContent = this.createModal();
        // Extract just the settings-content div from the modal
        const tempDiv = document.createElement('div');
        tempDiv.innerHTML = modalContent;
        const settingsContent = tempDiv.querySelector('.settings-content');
        if (settingsContent) {
            container.innerHTML = settingsContent.outerHTML;
        }
        else {
            // Fallback to full modal content if settings-content not found
            container.innerHTML = modalContent;
        }
        this.initializePanelMode(container);
    }
    initializePanelMode(container) {
        // Load models from database first
        this.loadModelsFromDatabase().then(() => {
            // Update model dropdowns if they exist
            this.updateModelDropdowns();
        });
        // Use the same event handlers as the modal, but scoped to container
        this.setupEventHandlers();
        this.setupProfileCreationHandlers();
        // Load saved settings (this includes license validation)
        this.loadSettings();
    }
    /**
     * Get settings content as an HTMLElement for tab display
     */
    getSettingsTabContent() {
        const container = document.createElement('div');
        container.style.cssText = `
      width: 100%;
      height: 100%;
      overflow-y: auto;
      background: #1e1e1e;
      color: #cccccc;
      padding: 20px;
      position: relative;
    `;
        // Render the settings panel into the container
        this.renderInContainer(container);
        return container;
    }
    /**
     * Handle save when settings tab is closed
     */
    handleSave() {
        return __awaiter(this, void 0, void 0, function* () {
            // Apply all settings before closing
            yield this.applyAllSettings();
        });
    }
}
exports.SettingsModal = SettingsModal;
//# sourceMappingURL=settings-modal.js.map