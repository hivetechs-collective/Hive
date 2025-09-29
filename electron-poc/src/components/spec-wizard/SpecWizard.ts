/*
 * SpecWizard - UI guide for Specification-Driven Development using Spec Kit
 * Non-intrusive: generates a short shell script that runs in our terminal
 * to initialize (if needed) and create spec/plan/tasks under specs/NNN-slug
 */

export class SpecWizard {
  private overlay: HTMLElement | null = null;
  private container: HTMLElement | null = null;
  private currentTab: 'start'|'clarify'|'contracts' = 'start';
  private completed = { start: false, clarify: false, contracts: false };
  private lastClarifyCount = -1;
  private lastCheckPassed = false;
  private contractsCreated = 0;

  open() {
    if (this.overlay) return; // already open
    this.overlay = document.createElement('div');
    this.overlay.className = 'wizard-overlay';
    this.overlay.innerHTML = this.render();

    document.body.appendChild(this.overlay);

    // Wire events
    const closeBtn = this.overlay.querySelector('#wiz-close');
    closeBtn?.addEventListener('click', () => this.close());

    const browseBtn = this.overlay.querySelector('#wiz-browse');
    browseBtn?.addEventListener('click', async () => {
      try {
        const res = await (window as any).electronAPI?.showOpenDialog?.({ properties: ['openDirectory'] });
        if (res && !res.canceled && res.filePaths?.[0]) {
          (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value = res.filePaths[0];
        }
      } catch {}
    });

    const titleInput = this.overlay.querySelector('#wiz-title') as HTMLInputElement;
    const slugInput = this.overlay.querySelector('#wiz-slug') as HTMLInputElement;
    titleInput?.addEventListener('input', () => {
      if (!slugInput.dataset.userEdited) {
        slugInput.value = this.slugify(titleInput.value);
      }
    });
    slugInput?.addEventListener('input', () => {
      slugInput.dataset.userEdited = '1';
      slugInput.value = this.slugify(slugInput.value);
    });

    const runBtn = this.overlay.querySelector('#wiz-run');
    runBtn?.addEventListener('click', () => this.run());

    // Step navigation
    this.overlay.querySelector('#wiz-tab-start')?.addEventListener('click', () => this.switchTab('start'));
    this.overlay.querySelector('#wiz-tab-clarify')?.addEventListener('click', () => this.switchTab('clarify'));
    this.overlay.querySelector('#wiz-tab-contracts')?.addEventListener('click', () => this.switchTab('contracts'));

    // Clarify/Validate actions
    this.overlay.querySelector('#wiz-refresh-clarify')?.addEventListener('click', () => this.refreshClarify());
    this.overlay.querySelector('#wiz-run-check')?.addEventListener('click', () => this.runSpecifyCheck());
    this.overlay.querySelector('#wiz-clear-needs')?.addEventListener('click', () => this.removeNeedsClarification());
    this.overlay.querySelector('#wiz-undo-clarify')?.addEventListener('click', () => this.undoClarify());

    // Contracts actions
    this.overlay.querySelector('#wiz-add-endpoint')?.addEventListener('click', () => this.addEndpointRow());
    this.overlay.querySelector('#wiz-scaffold-contracts')?.addEventListener('click', () => this.scaffoldContracts());

    // Start verification
    this.overlay.querySelector('#wiz-verify-created')?.addEventListener('click', () => this.verifyStartCreated());

    // Mode and selectors
    (this.overlay.querySelector('#wiz-mode-create') as HTMLInputElement)?.addEventListener('change', () => this.updateModeUI());
    (this.overlay.querySelector('#wiz-mode-update') as HTMLInputElement)?.addEventListener('change', () => this.updateModeUI());
    this.overlay.querySelector('#wiz-refresh-specs')?.addEventListener('click', () => this.populateSpecSelects());
    this.overlay.querySelector('#wiz-load-specs-clarify')?.addEventListener('click', () => this.populateSpecSelects());
    this.overlay.querySelector('#wiz-load-specs-contracts')?.addEventListener('click', () => this.populateSpecSelects());
    this.overlay.querySelector('#wiz-ensure-files')?.addEventListener('click', () => this.ensureSpecFiles());
    (this.overlay.querySelector('#wiz-existing-spec') as HTMLSelectElement)?.addEventListener('change', () => this.loadExistingFields());

    // Footer navigation
    this.overlay.querySelector('#wiz-next')?.addEventListener('click', () => this.handleNext());
    this.overlay.querySelector('#wiz-prev')?.addEventListener('click', () => this.handlePrev());

    // Initial UI state
    this.switchTab('start');
    this.updateStepperUI();

    // Default project path
    const projectField = this.overlay.querySelector('#wiz-project') as HTMLInputElement;
    projectField.value = (window as any).currentOpenedFolder || '';
    this.populateSpecSelects();
    this.updateModeUI();
    this.loadExistingFields();
  }

  close() {
    if (!this.overlay) return;
    document.body.removeChild(this.overlay);
    this.overlay = null;
  }

  private slugify(s: string): string {
    return (s || '')
      .toLowerCase()
      .replace(/[^a-z0-9\-\s_]+/g, '')
      .replace(/[\s_]+/g, '-')
      .replace(/-+/g, '-')
      .replace(/^-+|-+$/g, '');
  }

  private render(): string {
    return `
    <div class="wizard-root">
      <div class="wizard-modal">
        <div class="wizard-header">
          <div class="wizard-title">Spec‑Kit Wizard</div>
          <button id="wiz-close" class="wizard-btn ghost" aria-label="Close">×</button>
        </div>
        <div class="wizard-body">
          <div class="wizard-tabs" style="display:flex;gap:8px;margin-bottom:10px;">
            <button class="wizard-btn secondary" id="wiz-tab-start">1. Start <span class="wiz-badge wiz-required">Required</span></button>
            <button class="wizard-btn secondary" id="wiz-tab-clarify">2. Clarify & Validate <span class="wiz-badge wiz-required">Required</span></button>
            <button class="wizard-btn secondary" id="wiz-tab-contracts">3. Contracts <span class="wiz-badge wiz-required">Required</span></button>
          </div>

          <div class="wizard-grid wizard-tab" data-tab="start">
            <div class="wizard-card">
              <div class="wizard-card-title">Project</div>
              <div class="wizard-field-row">
                <input id="wiz-project" type="text" placeholder="/path/to/project" class="wizard-input" />
                <button id="wiz-browse" class="wizard-btn secondary">Browse…</button>
              </div>
              <div class="wizard-hint">We’ll run in this folder. If not initialized, we’ll do <code>specify init --here</code>.</div>
            </div>

            <div class="wizard-card">
              <div class="wizard-card-title">Mode</div>
              <div class="wizard-hint">Create a new feature (default) or update an existing spec safely (no overwrite).</div>
              <div class="wizard-field-row" style="gap:16px;">
                <label><input type="radio" name="wiz-mode" id="wiz-mode-create" checked> Create new feature</label>
                <label><input type="radio" name="wiz-mode" id="wiz-mode-update"> Update existing spec</label>
              </div>
              <div id="wiz-update-area" style="display:none;">
                <div class="wizard-field-row" style="margin-top:6px; gap:8px;">
                  <select id="wiz-existing-spec" class="wizard-input" style="height:32px;"></select>
                  <button id="wiz-refresh-specs" class="wizard-btn secondary">Refresh</button>
                </div>
                <div class="wizard-field-row" style="justify-content:flex-end;margin-top:6px;gap:8px;">
                  <button id="wiz-ensure-files" class="wizard-btn secondary">Create Missing Files</button>
                </div>
              </div>
            </div>
            <div class="wizard-card">
              <div class="wizard-card-title">Feature</div>
              <label class="wizard-label">Title</label>
              <input id="wiz-title" type="text" placeholder="Photo Albums (initial)" class="wizard-input" />
              <label class="wizard-label">Slug</label>
              <input id="wiz-slug" type="text" placeholder="photo-albums" class="wizard-input" />
              <div class="wizard-hint">What & Why only. Technical details belong to the Plan step; we’ll get there next.</div>
            </div>

            <div class="wizard-card">
              <div class="wizard-card-title">Vision</div>
              <textarea id="wiz-vision" class="wizard-textarea" rows="3" placeholder="Enable users to organize photos into dated albums with drag‑and‑drop."></textarea>
              <div class="wizard-subgrid">
                <div>
                  <div class="wizard-card-subtitle">User Stories (3)</div>
                  <textarea id="wiz-stories" class="wizard-textarea" rows="5" placeholder="- As a user, I can create an album with a title.\n- As a user, I can drag a photo between albums.\n- As a user, I can browse album thumbnails quickly."></textarea>
                </div>
                <div>
                  <div class="wizard-card-subtitle">Acceptance Criteria</div>
                  <textarea id="wiz-accept" class="wizard-textarea" rows="5" placeholder="- Creating an album succeeds in <10s with a visible confirmation.\n- Dragging a photo updates order without reload.\n- The grid renders 200 thumbnails < 300ms on M1."></textarea>
                </div>
              </div>
              <div class="wizard-hint">Done when: a new directory appears under <code>specs/NNN-slug/</code> with spec.md, plan.md, tasks.md.</div>
              <div class="wizard-field-row" style="justify-content:flex-end;margin-top:6px;">
                <button id="wiz-verify-created" class="wizard-btn secondary">Verify Spec Created</button>
              </div>
            </div>
          </div>

          <div class="wizard-grid wizard-tab" data-tab="clarify" style="display:none;">
            <div class="wizard-card">
              <div class="wizard-card-title">Clarify Ambiguities</div>
              <div class="wizard-hint">Why: SDD requires executable specs — ambiguous items must be resolved. Look for <code>[NEEDS CLARIFICATION: …]</code> and refine.</div>
              <div class="wizard-field-row" style="gap:8px;">
                <span class="wizard-label" style="margin:0;">Target Spec</span>
                <select id="wiz-target-spec-clarify" class="wizard-input" style="height:32px;flex:1"></select>
                <button id="wiz-load-specs-clarify" class="wizard-btn secondary">Refresh</button>
              </div>
              <div class="wizard-field-row" style="justify-content:flex-end;">
                <button id="wiz-refresh-clarify" class="wizard-btn secondary">Refresh</button>
                <button id="wiz-clear-needs" class="wizard-btn secondary">Remove [NEEDS CLARIFICATION] Tags</button>
                <button id="wiz-undo-clarify" class="wizard-btn secondary">Undo Last Cleanup</button>
              </div>
              <div id="wiz-clarify-list" class="wizard-code" style="min-height:120px;white-space:pre-wrap;"></div>
            </div>
            <div class="wizard-card">
              <div class="wizard-card-title">Validate Specification</div>
              <div class="wizard-hint">Runs <code>specify check</code> to verify structure and completeness. Fix issues and re-run.</div>
              <div class="wizard-field-row" style="justify-content:flex-end;">
                <button id="wiz-run-check" class="wizard-btn primary">Run specify check</button>
              </div>
              <div id="wiz-check-output" class="wizard-code" style="min-height:120px;white-space:pre-wrap;"></div>
              <div class="wizard-hint">Done when: no remaining <code>[NEEDS CLARIFICATION]</code> and check passes with no errors.</div>
            </div>
          </div>

          <div class="wizard-grid wizard-tab" data-tab="contracts" style="display:none;">
            <div class="wizard-card">
              <div class="wizard-card-title">Define Contracts</div>
              <div class="wizard-hint">Why: Contract-first (Article IX) drives implementation and tests. Define endpoints/events before writing code.</div>
              <div class="wizard-field-row" style="gap:8px;">
                <span class="wizard-label" style="margin:0;">Target Spec</span>
                <select id="wiz-target-spec-contracts" class="wizard-input" style="height:32px;flex:1"></select>
                <button id="wiz-load-specs-contracts" class="wizard-btn secondary">Refresh</button>
              </div>
              <div id="wiz-endpoints" style="display:flex;flex-direction:column;gap:8px;">
                <!-- rows inserted here -->
              </div>
              <div class="wizard-field-row" style="justify-content:space-between;margin-top:8px;">
                <button id="wiz-add-endpoint" class="wizard-btn secondary">Add Endpoint</button>
                <button id="wiz-scaffold-contracts" class="wizard-btn primary">Scaffold Contracts</button>
              </div>
              <div id="wiz-contracts-result" class="wizard-code" style="margin-top:10px;white-space:pre-wrap;"></div>
              <div class="wizard-hint">Done when: at least one contract file exists under <code>specs/NNN-slug/contracts/</code> with request/response and a test note.</div>
            </div>
          </div>
        </div>
        <div class="wizard-footer">
          <button id="wiz-prev" class="wizard-btn secondary">◀ Back</button>
          <div class="wizard-spacer"></div>
          <button id="wiz-run" class="wizard-btn primary">Run in Terminal</button>
          <button id="wiz-next" class="wizard-btn primary">Next ▶</button>
        </div>
      </div>
    </div>`;
  }

  private switchTab(tab: 'start'|'clarify'|'contracts') {
    if (!this.overlay) return;
    this.currentTab = tab;
    this.overlay.querySelectorAll('.wizard-tab').forEach(el => {
      const t = (el as HTMLElement).dataset.tab;
      (el as HTMLElement).style.display = (t === tab) ? '' : 'none';
    });
    // Footer buttons context
    const runBtn = this.overlay!.querySelector('#wiz-run') as HTMLButtonElement;
    const nextBtn = this.overlay!.querySelector('#wiz-next') as HTMLButtonElement;
    const prevBtn = this.overlay!.querySelector('#wiz-prev') as HTMLButtonElement;
    if (tab === 'start') {
      runBtn.style.display = '';
      runBtn.textContent = 'Create Spec in Terminal';
      nextBtn.disabled = !this.completed.start;
      prevBtn.disabled = true;
    } else if (tab === 'clarify') {
      runBtn.style.display = 'none';
      nextBtn.disabled = !(this.completed.clarify);
      prevBtn.disabled = false;
    } else if (tab === 'contracts') {
      runBtn.style.display = 'none';
      nextBtn.textContent = 'Finish';
      nextBtn.disabled = !(this.completed.contracts);
      prevBtn.disabled = false;
    }
    this.updateStepperUI();
  }

  private updateStepperUI() {
    if (!this.overlay) return;
    const mark = (id: string, done: boolean) => {
      const el = this.overlay!.querySelector(id) as HTMLElement;
      if (!el) return;
      el.classList.toggle('wiz-done', done);
    };
    mark('#wiz-tab-start', this.completed.start);
    mark('#wiz-tab-clarify', this.completed.clarify);
    mark('#wiz-tab-contracts', this.completed.contracts);
  }

  private async findLatestSpecDir(project: string): Promise<string | null> {
    try {
      const specs = await (window as any).fileAPI?.getDirectoryContents?.(`${project}/specs`);
      if (!Array.isArray(specs)) return null;
      const dirs = specs.filter((e: any) => e.type === 'directory' && /^(\d{3})-/.test(e.name));
      if (!dirs.length) return null;
      dirs.sort((a: any, b: any) => parseInt(a.name.slice(0,3)) - parseInt(b.name.slice(0,3)));
      return `${project}/specs/${dirs[dirs.length-1].name}`;
    } catch { return null; }
  }

  private async refreshClarify() {
    try {
      if (!this.overlay) return;
      const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
      const specDir = await this.findLatestSpecDir(project);
      const listEl = this.overlay!.querySelector('#wiz-clarify-list') as HTMLElement;
      if (!specDir) { listEl.textContent = 'No spec directory found.'; return; }
      const specPath = `${specDir}/spec.md`;
      const content = await (window as any).fileAPI?.readFile?.(specPath);
      if (typeof content !== 'string') { listEl.textContent = 'Unable to read spec.md'; return; }
      const matches = content.match(/\[NEEDS CLARIFICATION:[^\]]+\]/g) || [];
      if (!matches.length) {
        listEl.textContent = 'No [NEEDS CLARIFICATION] markers found.';
        this.lastClarifyCount = 0;
      } else {
        listEl.textContent = matches.join('\n');
        this.lastClarifyCount = matches.length;
      }
      this.evaluateClarifyCompletion();
    } catch (e) {
      console.error('[SpecWizard] refreshClarify failed:', e);
    }
  }

  private async removeNeedsClarification() {
    try {
      if (!this.overlay) return;
      const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
      const specDir = this.getSelectedSpec('#wiz-target-spec-clarify', project) || await this.findLatestSpecDir(project);
      if (!specDir) return;
      const specPath = `${specDir}/spec.md`;
      const content = await (window as any).fileAPI?.readFile?.(specPath);
      if (typeof content !== 'string') return;
      try { await (window as any).fileAPI?.writeFile?.(`${specPath}.bak`, content); } catch {}
      const replaced = content.replace(/\[NEEDS CLARIFICATION:[^\]]+\]/g, '').replace(/\n\n\n+/g, '\n\n');
      await (window as any).fileAPI?.writeFile?.(specPath, replaced);
      await this.refreshClarify();
    } catch (e) { console.error(e); }
  }

  private async undoClarify() {
    try {
      if (!this.overlay) return;
      const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
      const specDir = this.getSelectedSpec('#wiz-target-spec-clarify', project) || await this.findLatestSpecDir(project);
      if (!specDir) return;
      const specPath = `${specDir}/spec.md`;
      const bakPath = `${specPath}.bak`;
      const bak = await (window as any).fileAPI?.readFile?.(bakPath);
      if (typeof bak === 'string') {
        await (window as any).fileAPI?.writeFile?.(specPath, bak);
        await this.refreshClarify();
      } else {
        alert('No backup found.');
      }
    } catch (e) { alert('No backup found.'); }
  }

  private async runSpecifyCheck() {
    try {
      if (!this.overlay) return;
      const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
      const outEl = this.overlay!.querySelector('#wiz-check-output') as HTMLElement;
      outEl.textContent = 'Running specify check...';
      const res = await (window as any).electronAPI?.specifyCheck?.(project);
      if (!res?.success) { outEl.textContent = `Failed: ${res?.error || ''}`; this.lastCheckPassed = false; this.evaluateClarifyCompletion(); return; }
      outEl.textContent = (res.stdout || '').trim() || '(no output)';
      this.lastCheckPassed = true;
      this.evaluateClarifyCompletion();
    } catch (e) { console.error(e); }
  }

  private getSelectedSpec(selectId: string, project: string): string | null {
    const sel = this.overlay!.querySelector(selectId) as HTMLSelectElement;
    if (sel && sel.value) return `${project}/specs/${sel.value}`;
    return null;
  }

  private async populateSpecSelects() {
    if (!this.overlay) return;
    const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
    if (!project) return;
    try {
      const specs: string[] = await (window as any).electronAPI?.listSpecs?.(project);
      const fills = ['#wiz-existing-spec', '#wiz-target-spec-clarify', '#wiz-target-spec-contracts'];
      fills.forEach((id) => {
        const el = this.overlay!.querySelector(id) as HTMLSelectElement;
        if (!el) return;
        el.innerHTML = '';
        (specs || []).forEach((name) => {
          const opt = document.createElement('option');
          opt.value = name; opt.textContent = name;
          el.appendChild(opt);
        });
        if (el.options.length > 0) el.selectedIndex = el.options.length - 1;
      });
    } catch {}
  }

  private updateModeUI() {
    if (!this.overlay) return;
    const create = (this.overlay.querySelector('#wiz-mode-create') as HTMLInputElement)?.checked;
    const runBtn = this.overlay!.querySelector('#wiz-run') as HTMLButtonElement;
    const area = this.overlay!.querySelector('#wiz-update-area') as HTMLElement;
    if (create) {
      area.style.display = 'none';
      runBtn.style.display = '';
      runBtn.textContent = 'Create Spec in Terminal';
    } else {
      area.style.display = '';
      runBtn.style.display = 'none';
    }
  }

  private async loadExistingFields() {
    try {
      if (!this.overlay) return;
      const updateMode = (this.overlay.querySelector('#wiz-mode-update') as HTMLInputElement)?.checked;
      if (!updateMode) return;
      const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
      const sel = this.overlay!.querySelector('#wiz-existing-spec') as HTMLSelectElement;
      if (!project || !sel?.value) return;
      const specPath = `${project}/specs/${sel.value}/spec.md`;
      const content = await (window as any).fileAPI?.readFile?.(specPath);
      if (typeof content !== 'string') return;
      const take = (heading: string) => {
        const re = new RegExp(`^## ${heading}\\n([\\s\\S]*?)(\\n## |$)`, 'm');
        const m = content.match(re);
        return m ? m[1].trim() : '';
      };
      const vision = take('Vision');
      const stories = take('Users & Stories');
      const accept = take('Acceptance Criteria');
      (this.overlay!.querySelector('#wiz-vision') as HTMLTextAreaElement).value = vision;
      (this.overlay!.querySelector('#wiz-stories') as HTMLTextAreaElement).value = stories;
      (this.overlay!.querySelector('#wiz-accept') as HTMLTextAreaElement).value = accept;
    } catch (e) {
      // non-fatal
    }
  }

  private evaluateClarifyCompletion() {
    // Clarify considered complete when zero markers and check passed
    const clear = this.lastClarifyCount === 0;
    this.completed.clarify = !!(clear && this.lastCheckPassed);
    if (this.currentTab === 'clarify') this.switchTab('clarify');
  }

  private addEndpointRow() {
    if (!this.overlay) return;
    const container = this.overlay!.querySelector('#wiz-endpoints');
    if (!container) return;
    const row = document.createElement('div');
    row.className = 'wizard-field-row';
    row.innerHTML = `
      <input type="text" placeholder="Name (e.g., list-photos)" class="wizard-input" data-ep="name" style="flex:1;">
      <input type="text" placeholder="Method (GET)" class="wizard-input" data-ep="method" style="width:120px;">
      <input type="text" placeholder="Path (/photos)" class="wizard-input" data-ep="path" style="flex:1;">
      <button class="wizard-btn ghost" aria-label="Remove">×</button>
    `;
    row.querySelector('button')?.addEventListener('click', () => row.remove());
    container.appendChild(row);
  }

  private async scaffoldContracts() {
    try {
      if (!this.overlay) return;
      const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
      const specDirSel = this.getSelectedSpec('#wiz-target-spec-contracts', project);
      const specDirRel = specDirSel ? specDirSel.replace(`${project}/`, '') : undefined;
      const rows = Array.from(this.overlay!.querySelectorAll('#wiz-endpoints .wizard-field-row')) as HTMLElement[];
      const endpoints = rows.map(r => ({
        name: (r.querySelector('[data-ep="name"]') as HTMLInputElement)?.value.trim(),
        method: (r.querySelector('[data-ep="method"]') as HTMLInputElement)?.value.trim(),
        path: (r.querySelector('[data-ep="path"]') as HTMLInputElement)?.value.trim(),
      })).filter(e => e.name || e.path);
      const res = await (window as any).electronAPI?.scaffoldContracts?.({ projectPath: project, specDir: specDirRel, endpoints });
      const out = this.overlay!.querySelector('#wiz-contracts-result') as HTMLElement;
      if (!res?.success) { out.textContent = `Failed: ${res?.error || ''}`; return; }
      this.contractsCreated = (res.created || []).length;
      const createdText = (res.created || []).length ? `Created:\n${(res.created || []).join('\n')}` : '';
      const skippedText = (res.skipped || []).length ? `\nSkipped (exists):\n${(res.skipped || []).join('\n')}` : '';
      out.textContent = `${createdText}${skippedText}`.trim() || 'No changes';
      this.completed.contracts = this.contractsCreated > 0;
      if (this.currentTab === 'contracts') this.switchTab('contracts');
    } catch (e) { console.error(e); }
  }

  private async verifyStartCreated() {
    try {
      const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
      const updateMode = (this.overlay!.querySelector('#wiz-mode-update') as HTMLInputElement)?.checked;
      const specDir = updateMode ? (this.getSelectedSpec('#wiz-existing-spec', project) || await this.findLatestSpecDir(project)) : await this.findLatestSpecDir(project);
      if (!specDir) { alert('No spec found yet. Create the spec from Start step.'); return; }
      const files = await (window as any).fileAPI?.getDirectoryContents?.(specDir);
      const names = (files || []).map((f: any) => f.name);
      const ok = names.includes('spec.md') && names.includes('plan.md') && names.includes('tasks.md');
      this.completed.start = !!ok;
      this.switchTab('start');
      if (ok) alert('Verified: spec/plan/tasks created.'); else alert('Spec directory found, but expected files missing.');
    } catch (e) { console.error(e); }
  }

  private async ensureSpecFiles() {
    try {
      if (!this.overlay) return;
      const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
      const specSelect = this.overlay!.querySelector('#wiz-existing-spec') as HTMLSelectElement;
      const specDir = specSelect?.value;
      if (!project || !specDir) { alert('Select a spec to update.'); return; }
      const res = await (window as any).electronAPI?.ensureSpecFiles?.({ projectPath: project, specDir: `specs/${specDir}` });
      if (!res?.success) { alert('Failed: ' + (res?.error || '')); return; }
      await this.verifyStartCreated();
    } catch (e) { console.error(e); }
  }

  private handleNext() {
    if (this.currentTab === 'start') {
      if (!this.completed.start) { alert('Complete the Start step (create or verify spec files) before continuing.'); return; }
      this.switchTab('clarify');
    } else if (this.currentTab === 'clarify') {
      if (!this.completed.clarify) { alert('Resolve clarifications and pass specify check before continuing.'); return; }
      this.switchTab('contracts');
    } else {
      this.close();
    }
  }

  private handlePrev() {
    if (this.currentTab === 'contracts') this.switchTab('clarify');
    else if (this.currentTab === 'clarify') this.switchTab('start');
  }

  private buildScript(project: string, title: string, slug: string, vision: string, stories: string, accept: string): string {
    // Sanitize multi-line inputs for here-doc
    const esc = (s: string) => (s || '').replace(/\r/g, '');
    const titleSafe = title.replace(/"/g, '\\"');

    return `#!/usr/bin/env bash
set -e
export PATH="$PATH"
PROJECT_DIR=${project ? '"' + project.replace(/"/g,'\\"') + '"' : '"$PWD"'}
cd "$PROJECT_DIR"

echo "🔎 Checking Spec Kit (specify) availability..."
if ! command -v specify >/dev/null 2>&1; then
  echo "❌ Specify CLI not found in PATH. Please install it from AI CLI Tools panel." >&2
  exit 1
fi

if [ ! -d ".specify" ]; then
  echo "📦 Initializing Spec Kit here..."
  specify init --here --script sh || true
fi

mkdir -p specs
last=$(ls -1 specs 2>/dev/null | awk -F '-' '/^[0-9]{3}-/ {print $1}' | sort | tail -1)
if [ -z "$last" ]; then next="001"; else next=$(printf "%03d" $((10#$last + 1))); fi
dest="specs/$next-${slug}"
mkdir -p "$dest/contracts"

cat > "$dest/spec.md" << 'EOF_SPEC'
# ${titleSafe}

## Vision
${esc(vision)}

## Users & Stories
${esc(stories)}

## Acceptance Criteria
${esc(accept)}

## Non‑Functional Requirements
- [NEEDS CLARIFICATION: add NFRs]

EOF_SPEC

cat > "$dest/plan.md" << 'EOF_PLAN'
# Implementation Plan

> IMPORTANT: Keep high‑level. Details go to implementation-details/.

## Architecture & Stack
- [NEEDS CLARIFICATION: stack]

## Constitution Gates (Phase −1)
- [ ] Simplicity (≤3 projects, no future‑proofing)
- [ ] Anti‑Abstraction (prefer framework directly)
- [ ] Integration‑First (contracts + contract tests)

## Contracts to Define
- [NEEDS CLARIFICATION: endpoints/events]

## Data Model Notes
- [NEEDS CLARIFICATION]

EOF_PLAN

cat > "$dest/tasks.md" << 'EOF_TASKS'
# Tasks

- [ ] Define contracts and contract tests [P]
- [ ] Generate skeleton implementation to satisfy contracts [P]
- [ ] Wire acceptance scenarios and NFR checks
- [ ] Review against Constitution gates

EOF_TASKS

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  branch="$next-${slug}-spec"
  echo "🌿 Creating branch $branch and committing files..."
  git checkout -b "$branch" 2>/dev/null || git checkout "$branch" || true
  git add "$dest"
  git commit -m "feat(spec): $next-${slug} ${titleSafe}" || true
fi

echo "🔍 Running specify check (if available)..."
specify check || true

echo "✅ Spec created at $dest"
`;
  }

  private async run() {
    try {
      const project = (this.overlay!.querySelector('#wiz-project') as HTMLInputElement).value.trim();
      const title = (this.overlay!.querySelector('#wiz-title') as HTMLInputElement).value.trim();
      const slug = (this.overlay!.querySelector('#wiz-slug') as HTMLInputElement).value.trim() || this.slugify(title);
      const vision = (this.overlay!.querySelector('#wiz-vision') as HTMLTextAreaElement).value.trim();
      const stories = (this.overlay!.querySelector('#wiz-stories') as HTMLTextAreaElement).value.trim();
      const accept = (this.overlay!.querySelector('#wiz-accept') as HTMLTextAreaElement).value.trim();

      if (!project) { alert('Select a project folder'); return; }
      if (!title) { alert('Enter a title'); return; }

      // Ensure Spec Kit installed (managed)
      try {
        const status = await (window as any).electronAPI?.detectCliTool?.('specify');
        if (!status || !status.installed) {
          const go = confirm('Specify CLI is not installed by Hive. Install it now?');
          if (!go) return;
          const res = await (window as any).electronAPI?.installCliTool?.('specify');
          if (!res?.success) { alert('Install failed: ' + (res?.error || '')); return; }
        }
      } catch {}

      // Expand terminal panel and run the script there
      try { (window as any).expandTTYDTerminal?.(); } catch {}

      const script = this.buildScript(project, title, slug, vision, stories, accept);

      // Create terminal via panel with script content
      const terminal = (window as any).isolatedTerminal;
      if (terminal && typeof terminal.createTerminalTab === 'function') {
        await terminal.createTerminalTab('specify', undefined, undefined, script);
      } else {
        // Fallback: call terminalAPI directly
        const id = `terminal-${Date.now()}`;
        await (window as any).terminalAPI?.createTerminalProcess?.({
          terminalId: id,
          toolId: 'specify',
          cwd: project,
          scriptContent: script
        });
      }

      this.close();
    } catch (e) {
      console.error('[SpecWizard] Failed to run:', e);
      alert('Failed to run Spec‑Kit Wizard. See console for details.');
    }
  }
}

// Expose a singleton open function for easy wiring
(function() {
  const wiz = new SpecWizard();
  (window as any).openSpecWizard = () => wiz.open();
})();
