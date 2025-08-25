/**
 * Git Push Strategy Dialog
 * Shows intelligent push options with recommendations
 */

import { PushStrategy, PushStrategyOption } from './git-push-strategy';

export class GitPushDialog {
  /**
   * Show the push strategy selection dialog
   */
  static async show(
    analysis: any,
    strategies: PushStrategyOption[],
    explanation: string
  ): Promise<PushStrategyOption | null> {
    return new Promise((resolve) => {
      // Create modal overlay
      const overlay = document.createElement('div');
      overlay.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.8);
        z-index: 10000;
        display: flex;
        align-items: center;
        justify-content: center;
        animation: fadeIn 0.2s ease-in;
      `;
      
      // Create dialog
      const dialog = document.createElement('div');
      dialog.style.cssText = `
        background: var(--vscode-notifications-background, #252526);
        border: 1px solid var(--vscode-notifications-border, #007acc);
        color: var(--vscode-notifications-foreground, #ccc);
        border-radius: 8px;
        box-shadow: 0 8px 32px rgba(0,0,0,0.5);
        max-width: 800px;
        width: 90%;
        max-height: 80vh;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        animation: slideIn 0.3s ease-out;
      `;
      
      // Header
      const header = document.createElement('div');
      header.style.cssText = `
        padding: 20px;
        border-bottom: 1px solid var(--vscode-panel-border, #333);
        background: linear-gradient(135deg, #1e1e1e 0%, #2d2d30 100%);
      `;
      header.innerHTML = `
        <h2 style="margin: 0; font-size: 20px; display: flex; align-items: center;">
          <span style="margin-right: 10px;">📤</span>
          Smart Push Strategy Selector
        </h2>
        <p style="margin: 10px 0 0 0; opacity: 0.8; font-size: 13px;">
          Repository analysis complete. Choose the best strategy for your situation.
        </p>
      `;
      
      // Repository stats
      const statsSection = document.createElement('div');
      statsSection.style.cssText = `
        padding: 15px 20px;
        background: rgba(0, 123, 255, 0.1);
        border-bottom: 1px solid var(--vscode-panel-border, #333);
      `;
      statsSection.innerHTML = `
        <div style="display: flex; justify-content: space-around; text-align: center;">
          <div>
            <div style="font-size: 24px; font-weight: bold; color: #007acc;">
              ${analysis.totalSize}
            </div>
            <div style="font-size: 11px; opacity: 0.7;">Repository Size</div>
          </div>
          <div>
            <div style="font-size: 24px; font-weight: bold; color: #73c991;">
              ${analysis.commitCount}
            </div>
            <div style="font-size: 11px; opacity: 0.7;">Total Commits</div>
          </div>
          <div>
            <div style="font-size: 24px; font-weight: bold; color: #e2c08d;">
              ${analysis.hasUnpushedCommits}
            </div>
            <div style="font-size: 11px; opacity: 0.7;">Unpushed</div>
          </div>
          <div>
            <div style="font-size: 18px; font-weight: bold; color: ${analysis.isMainBranch ? '#f48771' : '#73c991'};">
              ${analysis.isMainBranch ? '⚠️ Main' : '✓ Feature'}
            </div>
            <div style="font-size: 11px; opacity: 0.7;">Branch Type</div>
          </div>
        </div>
      `;
      
      // Recommendation
      const recommendationSection = document.createElement('div');
      recommendationSection.style.cssText = `
        padding: 15px 20px;
        background: linear-gradient(135deg, rgba(40, 167, 69, 0.1) 0%, rgba(40, 167, 69, 0.05) 100%);
        border-bottom: 1px solid var(--vscode-panel-border, #333);
      `;
      recommendationSection.innerHTML = `
        <div style="display: flex; align-items: center;">
          <span style="font-size: 24px; margin-right: 15px;">💡</span>
          <div style="flex: 1;">
            <div style="font-weight: bold; margin-bottom: 5px; color: #73c991;">
              Recommended Approach
            </div>
            <div style="font-size: 13px; line-height: 1.5;">
              ${explanation}
            </div>
            ${analysis.reasoning.length > 0 ? `
              <ul style="margin: 10px 0 0 0; padding-left: 20px; font-size: 12px; opacity: 0.8;">
                ${analysis.reasoning.map((r: string) => `<li>${r}</li>`).join('')}
              </ul>
            ` : ''}
          </div>
        </div>
        ${analysis.risks.length > 0 ? `
          <div style="margin-top: 10px; padding: 10px; background: rgba(244, 135, 113, 0.1); border-radius: 4px;">
            <div style="font-size: 12px; color: #f48771;">
              <strong>⚠️ Risks:</strong>
              ${analysis.risks.join(' • ')}
            </div>
          </div>
        ` : ''}
      `;
      
      // Strategies list
      const strategiesSection = document.createElement('div');
      strategiesSection.style.cssText = `
        flex: 1;
        overflow-y: auto;
        padding: 20px;
      `;
      
      const strategiesHtml = strategies.map((strategy, index) => `
        <div class="strategy-option" data-strategy="${index}" style="
          margin-bottom: 15px;
          padding: 15px;
          background: ${strategy.recommended ? 'rgba(40, 167, 69, 0.1)' : 'rgba(255, 255, 255, 0.02)'};
          border: 1px solid ${strategy.recommended ? '#28a745' : 'var(--vscode-panel-border, #333)'};
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s;
          ${strategy.recommended ? 'box-shadow: 0 0 0 2px rgba(40, 167, 69, 0.2);' : ''}
        " onmouseover="this.style.background='rgba(255,255,255,0.05)'" 
           onmouseout="this.style.background='${strategy.recommended ? 'rgba(40, 167, 69, 0.1)' : 'rgba(255, 255, 255, 0.02)'}'">
          <div style="display: flex; align-items: flex-start;">
            <div style="font-size: 24px; margin-right: 15px;">${strategy.icon}</div>
            <div style="flex: 1;">
              <div style="display: flex; align-items: center; margin-bottom: 5px;">
                <strong style="font-size: 14px;">${strategy.label}</strong>
                ${strategy.recommended ? 
                  '<span style="margin-left: 10px; padding: 2px 8px; background: #28a745; color: white; border-radius: 3px; font-size: 10px;">RECOMMENDED</span>' : 
                  ''}
              </div>
              <p style="margin: 5px 0; font-size: 12px; opacity: 0.8;">
                ${strategy.description}
              </p>
              ${strategy.estimatedTime ? `
                <p style="margin: 5px 0; font-size: 11px; opacity: 0.6;">
                  ⏱ Estimated time: ${strategy.estimatedTime}
                </p>
              ` : ''}
              <div style="display: flex; gap: 20px; margin-top: 10px;">
                <div style="flex: 1;">
                  <div style="font-size: 11px; font-weight: bold; color: #73c991; margin-bottom: 3px;">Pros:</div>
                  <ul style="margin: 0; padding-left: 15px; font-size: 11px; opacity: 0.8;">
                    ${strategy.pros.map(pro => `<li>${pro}</li>`).join('')}
                  </ul>
                </div>
                <div style="flex: 1;">
                  <div style="font-size: 11px; font-weight: bold; color: #f48771; margin-bottom: 3px;">Cons:</div>
                  <ul style="margin: 0; padding-left: 15px; font-size: 11px; opacity: 0.8;">
                    ${strategy.cons.map(con => `<li>${con}</li>`).join('')}
                  </ul>
                </div>
              </div>
              ${strategy.command ? `
                <div style="margin-top: 10px; padding: 8px; background: #1e1e1e; border-radius: 3px;">
                  <code style="font-size: 11px; color: #569cd6;">${strategy.command}</code>
                </div>
              ` : ''}
            </div>
          </div>
        </div>
      `).join('');
      
      strategiesSection.innerHTML = strategiesHtml;
      
      // Push Options Section
      const optionsSection = document.createElement('div');
      optionsSection.style.cssText = `
        padding: 15px 20px;
        border-top: 1px solid var(--vscode-panel-border, #333);
        background: var(--vscode-editor-background, #1e1e1e);
      `;
      
      optionsSection.innerHTML = `
        <h4 style="margin: 0 0 12px 0; font-size: 13px; font-weight: 600; color: var(--vscode-foreground, #ccc);">
          ⚙️ Push Options
        </h4>
        
        <!-- Common Options Grid -->
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 12px;">
          <label style="display: flex; align-items: center; font-size: 12px; cursor: pointer;">
            <input type="checkbox" id="push-opt-force-lease" style="margin-right: 6px;" />
            <span>Force with lease <span style="opacity: 0.6;">(safer)</span></span>
          </label>
          <label style="display: flex; align-items: center; font-size: 12px; cursor: pointer;">
            <input type="checkbox" id="push-opt-tags" style="margin-right: 6px;" />
            <span>Include tags <span style="opacity: 0.6;">(--tags)</span></span>
          </label>
          <label style="display: flex; align-items: center; font-size: 12px; cursor: pointer;">
            <input type="checkbox" id="push-opt-upstream" style="margin-right: 6px;" />
            <span>Set upstream <span style="opacity: 0.6;">(-u)</span></span>
          </label>
          <label style="display: flex; align-items: center; font-size: 12px; cursor: pointer;">
            <input type="checkbox" id="push-opt-dry-run" style="margin-right: 6px;" />
            <span>Dry run first <span style="opacity: 0.6;">(preview)</span></span>
          </label>
        </div>
        
        <!-- Advanced Options (Collapsible) -->
        <details id="advanced-options" style="margin-top: 12px;">
          <summary style="cursor: pointer; font-size: 12px; color: var(--vscode-textLink-foreground, #3794ff); user-select: none;">
            ▶ Advanced Options
          </summary>
          <div style="padding: 12px; margin-top: 8px; background: rgba(0,0,0,0.2); border-radius: 4px;">
            <div style="margin-bottom: 10px;">
              <label style="display: block; font-size: 11px; margin-bottom: 4px; opacity: 0.8;">
                Push only last N commits (leave empty for all):
              </label>
              <input type="number" id="push-opt-commit-limit" min="1" placeholder="All commits" 
                     style="width: 100px; padding: 4px; background: var(--vscode-input-background, #3c3c3c); 
                            border: 1px solid var(--vscode-input-border, #3c3c3c); 
                            color: var(--vscode-input-foreground, #ccc); border-radius: 2px;" />
            </div>
            
            <div style="margin-bottom: 10px;">
              <label style="display: block; font-size: 11px; margin-bottom: 4px; opacity: 0.8;">
                Custom git command (overrides all options):
              </label>
              <input type="text" id="push-opt-custom-cmd" placeholder="e.g., git push origin HEAD~5:branch --force-with-lease" 
                     style="width: 100%; padding: 4px; background: var(--vscode-input-background, #3c3c3c); 
                            border: 1px solid var(--vscode-input-border, #3c3c3c); 
                            color: var(--vscode-input-foreground, #ccc); border-radius: 2px; font-family: monospace; font-size: 11px;" />
            </div>
            
            <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px;">
              <label style="display: flex; align-items: center; font-size: 11px; cursor: pointer;">
                <input type="checkbox" id="push-opt-atomic" style="margin-right: 4px;" />
                <span>Atomic <span style="opacity: 0.6;">(all or nothing)</span></span>
              </label>
              <label style="display: flex; align-items: center; font-size: 11px; cursor: pointer;">
                <input type="checkbox" id="push-opt-sign" style="margin-right: 4px;" />
                <span>Sign push <span style="opacity: 0.6;">(GPG)</span></span>
              </label>
              <label style="display: flex; align-items: center; font-size: 11px; cursor: pointer;">
                <input type="checkbox" id="push-opt-thin" style="margin-right: 4px;" />
                <span>Thin pack <span style="opacity: 0.6;">(optimize)</span></span>
              </label>
            </div>
          </div>
        </details>
      `;
      
      // Footer
      const footer = document.createElement('div');
      footer.style.cssText = `
        padding: 15px 20px;
        border-top: 1px solid var(--vscode-panel-border, #333);
        display: flex;
        justify-content: flex-end;
        gap: 10px;
      `;
      
      const cancelButton = document.createElement('button');
      cancelButton.textContent = 'Cancel';
      cancelButton.style.cssText = `
        padding: 8px 20px;
        background: var(--vscode-button-secondaryBackground, #3a3d41);
        color: var(--vscode-button-secondaryForeground, #ccc);
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 13px;
      `;
      cancelButton.onclick = () => {
        overlay.remove();
        resolve(null);
      };
      
      footer.appendChild(cancelButton);
      
      // Assemble dialog
      dialog.appendChild(header);
      dialog.appendChild(statsSection);
      dialog.appendChild(recommendationSection);
      dialog.appendChild(strategiesSection);
      dialog.appendChild(optionsSection);
      dialog.appendChild(footer);
      overlay.appendChild(dialog);
      
      // Add click handlers for strategies
      const strategyElements = dialog.querySelectorAll('.strategy-option');
      strategyElements.forEach((element) => {
        element.addEventListener('click', () => {
          const index = parseInt(element.getAttribute('data-strategy') || '0');
          const selectedStrategy = strategies[index];
          
          // Collect selected options
          selectedStrategy.selectedOptions = {
            forceWithLease: (document.getElementById('push-opt-force-lease') as HTMLInputElement)?.checked,
            includeTags: (document.getElementById('push-opt-tags') as HTMLInputElement)?.checked,
            setUpstream: (document.getElementById('push-opt-upstream') as HTMLInputElement)?.checked,
            dryRun: (document.getElementById('push-opt-dry-run') as HTMLInputElement)?.checked,
            commitLimit: parseInt((document.getElementById('push-opt-commit-limit') as HTMLInputElement)?.value) || undefined,
            customCommand: (document.getElementById('push-opt-custom-cmd') as HTMLInputElement)?.value || undefined,
            atomic: (document.getElementById('push-opt-atomic') as HTMLInputElement)?.checked,
            signPush: (document.getElementById('push-opt-sign') as HTMLInputElement)?.checked,
            thinPack: (document.getElementById('push-opt-thin') as HTMLInputElement)?.checked
          };
          
          overlay.remove();
          resolve(selectedStrategy);
        });
      });
      
      // Add styles
      const style = document.createElement('style');
      style.textContent = `
        @keyframes fadeIn {
          from { opacity: 0; }
          to { opacity: 1; }
        }
        @keyframes slideIn {
          from { transform: translateY(-20px); opacity: 0; }
          to { transform: translateY(0); opacity: 1; }
        }
        .strategy-option:hover {
          transform: translateX(5px);
        }
      `;
      overlay.appendChild(style);
      
      // Show dialog
      document.body.appendChild(overlay);
      
      // Focus first strategy
      (strategyElements[0] as HTMLElement)?.focus();
    });
  }
}