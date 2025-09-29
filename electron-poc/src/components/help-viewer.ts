/**
 * Integrated Help Viewer Component
 * Professional, developer-focused documentation viewer
 * Matches Claude Code CLI aesthetic with dark theme and monospace fonts
 */

export interface HelpSection {
  id: string;
  title: string;
  icon?: string;
  content: string;
}

export class HelpViewer {
  private container: HTMLElement | null = null;
  private currentSection: string = 'getting-started';
  
  private sections: HelpSection[] = [
    {
      id: 'getting-started',
      title: 'Getting Started',
      icon: `<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M14.25 1H1.75C1.336 1 1 1.336 1 1.75v12.5c0 .414.336.75.75.75h12.5c.414 0 .75-.336.75-.75V1.75c0-.414-.336-.75-.75-.75zM14 14H2V3h12v11z"/>
        <path d="M5.5 7L7 8.5v3.25a.75.75 0 001.5 0V8.5L10 7l-2.25-3L5.5 7z"/>
      </svg>`,
      content: `
        <h2>Getting Started with Hive Memory</h2>
        
        <div class="hero-section">
          <h3>Quick Start for AI CLI Tools</h3>
          <p>When you open your AI CLI tool (Claude, Gemini, Grok, etc.), simply copy and paste this command:</p>
          
          <div class="copy-block">
            <div class="copy-header">
              <span>Copy this command to get started</span>
              <button class="copy-button" data-copy-text="Read ~/.MEMORY.md to understand how to access my memory and context system through the database at ~/.hive-ai.db">
                📋 Copy
              </button>
            </div>
            <pre><code>Read ~/.MEMORY.md to understand how to access my memory and context system through the database at ~/.hive-ai.db</code></pre>
          </div>
          
          <p class="success-message">✨ That's it! Your AI tool now has access to your entire memory system.</p>
        </div>
        
        <h3>What This Does</h3>
        <p>This single command gives your AI tool:</p>
        <ul>
          <li>✅ Access to all your past conversations and context</li>
          <li>✅ Understanding of your preferences and patterns</li>
          <li>✅ Ability to query your unified knowledge base</li>
          <li>✅ Same memory capabilities as Hive Consensus</li>
        </ul>
        
        <h3>How It Works</h3>
        <div class="info-box">
          <p>When you install CLI tools through Hive, we create two symbolic links:</p>
          <ol>
            <li><code>~/.MEMORY.md</code> - A guide that teaches AI tools how to use your memory</li>
            <li><code>~/.hive-ai.db</code> - Direct access to your unified database</li>
          </ol>
          <p>The AI reads the guide and immediately understands how to query your knowledge base!</p>
        </div>
        
        <h3>Daily Workflow</h3>
        <div class="workflow-steps">
          <div class="workflow-step">
            <span class="step-number">1</span>
            <div class="step-content">
              <strong>Install CLI Tool</strong>
              <p>Use Hive to install your preferred AI CLI tool</p>
            </div>
          </div>
          <div class="workflow-step">
            <span class="step-number">2</span>
            <div class="step-content">
              <strong>Launch Tool</strong>
              <p>Open the tool from Hive or terminal</p>
            </div>
          </div>
          <div class="workflow-step">
            <span class="step-number">3</span>
            <div class="step-content">
              <strong>Paste Command</strong>
              <p>Copy the command above and paste it</p>
            </div>
          </div>
          <div class="workflow-step">
            <span class="step-number">4</span>
            <div class="step-content">
              <strong>Start Working</strong>
              <p>Your AI now has full context!</p>
            </div>
          </div>
        </div>
        
        <h3>Example Queries After Setup</h3>
        <div class="code-block">
          <div class="code-header">Once your AI has read the memory guide, you can ask:</div>
          <pre><code>"What did we work on yesterday?"
"Search the database for authentication examples"
"What's my preferred testing framework?"
"Find that solution we used for the API error last week"</code></pre>
        </div>
        
        <h3>Supported CLI Tools</h3>
        <p>This memory system works with all major AI CLI tools:</p>
        <div class="tool-grid">
          <div class="tool-card">Claude Code</div>
          <div class="tool-card">Gemini CLI</div>
          <div class="tool-card">GitHub Copilot CLI</div>
          <div class="tool-card">Grok CLI</div>
          <div class="tool-card">aichat</div>
          <div class="tool-card">mods</div>
          <div class="tool-card">Cursor</div>
        </div>
      `
    },
    {
      id: 'ai-workflows',
      title: 'AI Workflows',
      icon: `<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M2 3h12v2H2zM2 7h8v2H2zM2 11h12v2H2z"/></svg>`,
      content: `
        <h2>AI Workflows for Developers</h2>
        <p>Use these fast, repeatable workflows to accelerate common dev tasks.</p>

        <h3>Code Navigation</h3>
        <div class="code-block">
          <div class="code-header">Quickly locate code and context</div>
          <pre><code>"Open the repo and list key modules"
"Summarize the architecture from MASTER_ARCHITECTURE.md"
"Find all usages of fetchProjectConfig()"</code></pre>
        </div>

        <h3>Bug Triaging</h3>
        <div class="code-block">
          <div class="code-header">Drive from symptom to root cause</div>
          <pre><code>"Search logs for the error signature"
"Trace the error path through the code"
"Suggest a minimal safe fix with tests"</code></pre>
        </div>

        <h3>Refactor & Tests</h3>
        <div class="code-block">
          <div class="code-header">Iterate safely</div>
          <pre><code>"Propose a refactor plan for X"
"Add focused tests around Y"
"Show diff-only patch for review"</code></pre>
        </div>

        <h3>Memory & DB</h3>
        <p>Tell your AI tool to read: <code>~/.MEMORY.md</code> for unified memory and <code>~/.hive-ai.db</code> for direct queries.</p>
      `
    },
    {
      id: 'whats-new',
      title: "What's New",
      icon: `<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1l2 4 4 .5-3 3 .8 4.5L8 11l-3.8 2 1-4.5-3-3L6 5z"/></svg>`,
      content: `
        <h2>Hive v{{VERSION}} — Highlights</h2>
        <ul>
          <li><strong>Spec‑Kit Wizard (End‑to‑End)</strong>: Guided flow from idea → spec → validate → contracts. Required steps are gated and verified.</li>
          <li><strong>Update Existing Spec</strong>: Safely load existing Vision, Stories, and Acceptance Criteria; create missing files without overwriting.</li>
          <li><strong>“Design” Start Here</strong>: Distinct left‑bar button under AI CLI Tools to kick off Spec‑Kit quickly.</li>
          <li><strong>Responsive Left Pane</strong>: Icons scale on small screens; hover tooltips restored for all items.</li>
          <li><strong>Terminal Stability</strong>: Wizard temp script persists across reconnect; no more missing‑file errors.</li>
        </ul>

        <h3>Quick Actions</h3>
        <div class="code-block">
          <div class="code-header">Start with Spec‑Kit</div>
          <pre><code>Left bar → DESIGN (Spec‑Kit Wizard)
Create a spec → Clarify & Validate → Contracts</code></pre>
        </div>

        <h3>Recent Maintenance Releases</h3>
        <ul>
          <li><strong>v1.8.494</strong>: Activity bar polish; Wizard UX refinements.</li>
          <li><strong>v1.8.493</strong>: “Start Here” visual treatment; layout adjustments.</li>
        </ul>
      `
    },
    {
      id: 'memory-query',
      title: 'Memory Query',
      icon: `<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 1a7 7 0 100 14A7 7 0 008 1zm0 1.5a5.5 5.5 0 110 11 5.5 5.5 0 010-11z"/>
        <path d="M8 4.5a.5.5 0 00-.5.5v3.5a.5.5 0 00.146.354l2 2a.5.5 0 00.708-.708L8.5 8.293V5a.5.5 0 00-.5-.5z"/>
        <circle cx="8" cy="8" r="1"/>
      </svg>`,
      content: `
        <h2>Memory Query Guide</h2>
        
        <h3>Overview</h3>
        <p>The Hive Consensus memory system provides intelligent context retrieval for all your AI interactions. It maintains a unified database of all conversations, learnings, and insights.</p>
        
        <h3>Query Commands</h3>
        <div class="code-block">
          <div class="code-header">Direct Database Queries (work across all CLI tools)</div>
          <pre><code># Ask your AI tool to query the database:
"Query the local database for Python examples"
"Check ~/.hive-ai.db for what we've worked on"
"Search the memory database for authentication solutions"</code></pre>
        </div>
        
        <h3>Natural Language Queries</h3>
        <p>Ask your AI tools to query the database with phrases like:</p>
        <ul>
          <li><code>Query ~/.hive-ai.db for recent work</code></li>
          <li><code>Check the database for Python examples</code></li>
          <li><code>Search memory for authentication solutions</code></li>
          <li><code>What's in the recent_work view?</code></li>
        </ul>
        
        <h3>How the Memory System Works</h3>
        <div class="info-box">
          <p>When you install CLI tools through Hive, we create two important symlinks:</p>
          <ol>
            <li><code>~/.hive-ai.db</code> - Links to your unified database</li>
            <li><code>~/.MEMORY.md</code> - Complete guide for AI tools</li>
          </ol>
          
          <div class="code-block">
            <div class="code-header">Starting your day with AI tools</div>
            <pre><code># Tell your AI tool to read the memory guide:
"Read ~/.MEMORY.md for context"

# This gives it access to the same memory system as Consensus!</code></pre>
          </div>
        </div>
      `
    },
    {
      id: 'database-access',
      title: 'Database Access',
      icon: `<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M8 1C5.79 1 4 1.895 4 3v10c0 1.105 1.79 2 4 2s4-.895 4-2V3c0-1.105-1.79-2-4-2zm0 1c1.66 0 3 .672 3 1.5S9.66 5 8 5 5 4.328 5 3.5 6.34 2 8 2zm3 11.5c0 .828-1.34 1.5-3 1.5s-3-.672-3-1.5V11.2c.859.492 1.914.8 3 .8s2.141-.308 3-.8v2.3zm0-4c0 .828-1.34 1.5-3 1.5s-3-.672-3-1.5V7.2c.859.492 1.914.8 3 .8s2.141-.308 3-.8v2.3z"/>
      </svg>`,
      content: `
        <h2>Database Access Guide</h2>
        
        <h3>Unified Database Location</h3>
        <p>All data is stored in: <code>~/.hive/hive-ai.db</code></p>
        <p>This SQLite database contains all conversations, memory, settings, and analytics.</p>
        
        <h3>Direct Query Examples</h3>
        <div class="code-block">
          <div class="code-header">For AI CLI Tools (Claude, Gemini, etc.)</div>
          <pre><code># When user asks to query the database, run:
sqlite3 ~/.hive-ai.db "SELECT * FROM recent_work LIMIT 10"
sqlite3 ~/.hive-ai.db "SELECT * FROM solutions WHERE content LIKE '%auth%' LIMIT 5"
sqlite3 ~/.hive-ai.db "SELECT * FROM messages ORDER BY timestamp DESC LIMIT 5"</code></pre>
        </div>
        
        <div class="code-block">
          <div class="code-header">Available Views for Querying</div>
          <pre><code># List all views
sqlite3 ~/.hive-ai.db ".tables"

# Query specific views
sqlite3 ~/.hive-ai.db "SELECT * FROM recent_work LIMIT 10"
sqlite3 ~/.hive-ai.db "SELECT * FROM solutions LIMIT 10"
sqlite3 ~/.hive-ai.db "SELECT * FROM code_examples LIMIT 10"
sqlite3 ~/.hive-ai.db "SELECT * FROM patterns LIMIT 10"</code></pre>
        </div>
        
        <div class="code-block">
          <div class="code-header">Context-Aware Queries</div>
          <pre><code># Find messages about specific topics
sqlite3 ~/.hive-ai.db "SELECT * FROM messages WHERE content LIKE '%authentication%' LIMIT 10"

# Get recent assistant responses
sqlite3 ~/.hive-ai.db "SELECT substr(content, 1, 200), timestamp FROM messages WHERE role = 'assistant' ORDER BY timestamp DESC LIMIT 5"</code></pre>
        </div>
        
        <h3>Enhanced Memory Views (Consensus-Level Context)</h3>
        <p>These views replicate the Hive Consensus engine's intelligent memory system:</p>
        
        <h4>🧠 Temporal Memory Layers</h4>
        <ul>
          <li><code>memory_recent</code> - Last 2 hours (conversation continuity)</li>
          <li><code>memory_today</code> - Last 24 hours (recent context)</li>
          <li><code>memory_week</code> - Last 7 days (pattern recognition)</li>
          <li><code>memory_semantic</code> - All-time (thematic relevance)</li>
        </ul>
        
        <h4>🎯 Context Building Views</h4>
        <ul>
          <li><code>memory_patterns</code> - Recurring code patterns & solutions</li>
          <li><code>memory_preferences</code> - User technology preferences</li>
          <li><code>memory_themes</code> - Thematic conversation clusters</li>
          <li><code>memory_solutions_enhanced</code> - Problem-solution pairs</li>
          <li><code>memory_context_full</code> - Combined layered memory</li>
          <li><code>memory_context_summary</code> - Quick statistics overview</li>
        </ul>
        
        <h4>📚 Legacy Views</h4>
        <ul>
          <li><code>recent_work</code> - Last 100 assistant responses</li>
          <li><code>solutions</code> - Basic solution filtering</li>
          <li><code>code_examples</code> - Code snippet extraction</li>
          <li><code>patterns</code> - Simple pattern matching</li>
        </ul>
        
        <h3>Database Statistics</h3>
        <div class="code-block">
          <pre><code># Check memory size
sqlite3 ~/.hive-ai.db "SELECT COUNT(*) as total_messages FROM messages"

# View usage over time
sqlite3 ~/.hive-ai.db "SELECT DATE(timestamp) as day, COUNT(*) as count FROM messages GROUP BY day ORDER BY day DESC LIMIT 7"</code></pre>
        </div>
      `
    },
    {
      id: 'cli-integration',
      title: 'CLI Tools',
      icon: `<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M1.5 1h13a.5.5 0 01.5.5v10a.5.5 0 01-.5.5h-13a.5.5 0 01-.5-.5v-10a.5.5 0 01.5-.5zm.5 1v9h12V2H2z"/>
        <path d="M5.854 5.854a.5.5 0 10-.708-.708l-2 2a.5.5 0 000 .708l2 2a.5.5 0 00.708-.708L4.207 7.5l1.647-1.646zM10.146 5.854a.5.5 0 01.708-.708l2 2a.5.5 0 010 .708l-2 2a.5.5 0 01-.708-.708L11.793 7.5l-1.647-1.646z"/>
        <path d="M9.5 13h-3a.5.5 0 000 1h3a.5.5 0 000-1z"/>
      </svg>`,
      content: `
        <h2>CLI Tools Integration</h2>
        
        <h3>Supported Tools</h3>
        <ul>
          <li><strong>Claude Code</strong> - Full memory integration via Direct API</li>
          <li><strong>Grok CLI</strong> - Morph Fast Apply with memory context</li>
          <li><strong>Gemini CLI</strong> - Unified command support</li>
          <li><strong>aichat</strong> - Multiple model access with memory</li>
          <li><strong>mods</strong> - Streaming responses with context</li>
          <li><strong>Cursor Composer</strong> - IDE integration</li>
        </ul>
        
        <h3>Direct Database Access</h3>
        <div class="code-block">
          <div class="code-header">How CLI tools access memory</div>
          <pre><code># Tools can directly query the database via symlink:
sqlite3 ~/.hive-ai.db ".tables"  # List all tables and views
sqlite3 ~/.hive-ai.db "SELECT * FROM messages WHERE content LIKE '%error%' LIMIT 5"</code></pre>
        </div>
        
        <h3>Installation & Configuration</h3>
        <p>When you install CLI tools through Hive Consensus:</p>
        <ul>
          <li>Database symlink created at <code>~/.hive-ai.db</code></li>
          <li>Memory guide symlink created at <code>~/.MEMORY.md</code></li>
          <li>Points to your unified database at <code>~/.hive/hive-ai.db</code></li>
          <li>All tools can query using standard SQLite commands</li>
          <li>No additional configuration needed</li>
        </ul>
        
        <h3>Daily Workflow with AI Tools</h3>
        <div class="info-box">
          <p><strong>Step 1:</strong> Install your preferred CLI tool through Hive</p>
          <p><strong>Step 2:</strong> Launch the tool from Hive</p>
          <p><strong>Step 3:</strong> Tell your AI: <code>"Read ~/.MEMORY.md for context"</code></p>
          <p><strong>Result:</strong> Your AI now has full access to your memory system!</p>
        </div>
        
        <h3>Supported Query Examples</h3>
        <div class="code-block">
          <div class="code-header">Tell your AI tools to run queries like</div>
          <pre><code># View recent work
sqlite3 ~/.hive-ai.db "SELECT * FROM recent_work LIMIT 10"

# Search for solutions
sqlite3 ~/.hive-ai.db "SELECT * FROM solutions WHERE content LIKE '%error%'"

# Get code examples
sqlite3 ~/.hive-ai.db "SELECT * FROM code_examples LIMIT 5"</code></pre>
        </div>
      `
    },
    {
      id: 'shortcuts',
      title: 'Shortcuts',
      icon: `<svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M14 5H2a1 1 0 00-1 1v4a1 1 0 001 1h12a1 1 0 001-1V6a1 1 0 00-1-1zM2 4h12a2 2 0 012 2v4a2 2 0 01-2 2H2a2 2 0 01-2-2V6a2 2 0 012-2z"/>
        <path d="M3 6.5h2v1H3zm3 0h1v1H6zm2 0h1v1H8zm2 0h1v1h-1zm2 0h1v1h-1zM3 8.5h10v1H3z"/>
      </svg>`,
      content: `
        <h2>Keyboard Shortcuts</h2>
        
        <h3>Application Control</h3>
        <div class="shortcut-grid">
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + R</kbd>
            <span>Reload application</span>
          </div>
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + Shift + R</kbd>
            <span>Force reload</span>
          </div>
          <div class="shortcut-item">
            <kbd>F12</kbd>
            <span>Toggle Developer Tools</span>
          </div>
          <div class="shortcut-item">
            <kbd>F11</kbd>
            <span>Toggle Fullscreen</span>
          </div>
        </div>
        
        <h3>Navigation</h3>
        <div class="shortcut-grid">
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + P</kbd>
            <span>Go to File</span>
          </div>
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + G</kbd>
            <span>Go to Line</span>
          </div>
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + /</kbd>
            <span>Show this help</span>
          </div>
        </div>
        
        <h3>View Controls</h3>
        <div class="shortcut-grid">
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + 0</kbd>
            <span>Reset Zoom</span>
          </div>
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + Plus</kbd>
            <span>Zoom In</span>
          </div>
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + Minus</kbd>
            <span>Zoom Out</span>
          </div>
        </div>
        
        <h3>Window Management</h3>
        <div class="shortcut-grid">
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + M</kbd>
            <span>Minimize Window</span>
          </div>
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + W</kbd>
            <span>Close Window</span>
          </div>
          <div class="shortcut-item">
            <kbd>Cmd/Ctrl + Q</kbd>
            <span>Quit Application (macOS)</span>
          </div>
        </div>
      `
    }
  ];

  mount(container: HTMLElement): void {
    this.container = container;
    this.render();
    this.attachEventListeners();
  }

  unmount(): void {
    if (this.container) {
      this.container.innerHTML = '';
    }
    this.container = null;
  }

  private render(): void {
    if (!this.container) return;

    // Add styles if not already present
    if (!document.getElementById('help-viewer-styles')) {
      const style = document.createElement('style');
      style.id = 'help-viewer-styles';
      style.textContent = this.getStyles();
      document.head.appendChild(style);
    }

    this.container.innerHTML = `
      <div class="help-viewer">
        <div class="help-sidebar">
          <div class="help-sidebar-header">
            <h3>Documentation</h3>
          </div>
          <nav class="help-nav">
            ${this.sections.map(section => `
              <button class="help-nav-item ${section.id === this.currentSection ? 'active' : ''}" 
                      data-section="${section.id}">
                <span class="help-nav-icon">${section.icon || ''}</span>
                <span class="help-nav-title">${section.title}</span>
              </button>
            `).join('')}
          </nav>
        </div>
        <div class="help-content">
          <div class="help-content-inner">
            ${this.renderSection(this.currentSection)}
          </div>
        </div>
      </div>
    `;

    // Post-render dynamic version injection for What's New
    this.updateWhatsNewVersion();
  }

  private renderSection(sectionId: string): string {
    const section = this.sections.find(s => s.id === sectionId);
    if (!section) return '<p>Section not found</p>';
    return section.content;
  }

  private attachEventListeners(): void {
    if (!this.container) return;

    // Navigation click handlers
    this.container.querySelectorAll('.help-nav-item').forEach(item => {
      item.addEventListener('click', (e) => {
        const button = e.currentTarget as HTMLElement;
        const sectionId = button.dataset.section;
        if (sectionId) {
          this.navigateToSection(sectionId);
        }
      });
    });

    // Copy button handlers
    this.container.querySelectorAll('.copy-button').forEach(button => {
      button.addEventListener('click', (e) => {
        const btn = e.currentTarget as HTMLElement;
        const textToCopy = btn.dataset.copyText;
        if (textToCopy) {
          navigator.clipboard.writeText(textToCopy).then(() => {
            const originalText = btn.innerHTML;
            btn.innerHTML = '✅ Copied!';
            btn.style.background = '#4ec9b0';
            setTimeout(() => {
              btn.innerHTML = originalText;
              btn.style.background = '';
            }, 2000);
          }).catch(err => {
            console.error('Failed to copy text: ', err);
          });
        }
      });
    });
  }

  navigateToSection(sectionId: string): void {
    this.currentSection = sectionId;
    
    if (!this.container) return;

    // Update active nav item
    this.container.querySelectorAll('.help-nav-item').forEach(item => {
      item.classList.toggle('active', item.getAttribute('data-section') === sectionId);
    });

    // Update content
    const contentArea = this.container.querySelector('.help-content-inner');
    if (contentArea) {
      contentArea.innerHTML = this.renderSection(sectionId);
      this.updateWhatsNewVersion();
      
      // Re-attach event listeners for any interactive elements in the new content
      contentArea.querySelectorAll('.copy-button').forEach(button => {
        button.addEventListener('click', (e) => {
          const btn = e.currentTarget as HTMLElement;
          const textToCopy = btn.dataset.copyText;
          if (textToCopy) {
            navigator.clipboard.writeText(textToCopy).then(() => {
              const originalText = btn.innerHTML;
              btn.innerHTML = '✅ Copied!';
              btn.style.background = '#4ec9b0';
              setTimeout(() => {
                btn.innerHTML = originalText;
                btn.style.background = '';
              }, 2000);
            }).catch(err => {
              console.error('Failed to copy text: ', err);
            });
          }
        });
      });
    }
  }

  private async updateWhatsNewVersion() {
    try {
      if (this.currentSection !== 'whats-new') return;
      const version = await (window as any).electronAPI?.getVersion?.();
      if (!version) return;
      const contentArea = this.container?.querySelector('.help-content-inner');
      if (!contentArea) return;
      const h2 = contentArea.querySelector('h2');
      if (h2 && /^Hive v/.test(h2.textContent || '')) {
        h2.textContent = `Hive v${version} — Highlights`;
      } else if (h2) {
        // Fallback: set explicitly
        h2.textContent = `Hive v${version} — Highlights`;
      }

      // Also refresh the "Recent Maintenance Releases" list dynamically
      this.updateRecentMaintenanceList(version);
    } catch {}
  }

  private updateRecentMaintenanceList(currentVersion: string) {
    try {
      const contentArea = this.container?.querySelector('.help-content-inner');
      if (!contentArea) return;
      // Find the Recent Maintenance header
      const headers = Array.from(contentArea.querySelectorAll('h3')) as HTMLHeadingElement[];
      const recentHeader = headers.find(h => /Recent Maintenance Releases/i.test(h.textContent || ''));
      if (!recentHeader) return;
      // Expect the following sibling to be the UL we want to replace
      const ul = recentHeader.nextElementSibling as HTMLUListElement | null;
      if (!ul || ul.tagName.toLowerCase() !== 'ul') return;

      // Compute two previous patch versions as a simple, robust default
      const m = currentVersion.trim().match(/^(\d+)\.(\d+)\.(\d+)$/);
      if (!m) return;
      const major = parseInt(m[1], 10);
      const minor = parseInt(m[2], 10);
      const patch = parseInt(m[3], 10);

      const items: Array<{ v: string; text: string }> = [];
      if (patch > 0) items.push({ v: `${major}.${minor}.${patch - 1}`, text: 'Maintenance: UI polish and UX refinements.' });
      if (patch > 1) items.push({ v: `${major}.${minor}.${patch - 2}`, text: 'Maintenance: Security and stability fixes.' });

      // Render list items; fall back to a generic note if no previous versions
      if (items.length === 0) {
        ul.innerHTML = '<li><strong>No prior patch releases</strong>: This is the first build for this train.</li>';
      } else {
        ul.innerHTML = items
          .map(i => `<li><strong>v${i.v}</strong>: ${i.text}</li>`) 
          .join('');
      }
    } catch {}
  }

  private getStyles(): string {
    return `
      .help-viewer {
        display: flex;
        height: 100%;
        background: #1e1e1e;
        color: #cccccc;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'SF Pro Text', 'SF Pro Display', system-ui, sans-serif;
        font-size: 13px;
        line-height: 1.5;
        overflow: hidden;
      }

      .help-sidebar {
        width: 240px;
        background: #252526;
        border-right: 1px solid #3c3c3c;
        display: flex;
        flex-direction: column;
        flex-shrink: 0;
      }

      .help-sidebar-header {
        padding: 16px;
        border-bottom: 1px solid #3c3c3c;
        flex-shrink: 0;
      }

      .help-sidebar-header h3 {
        margin: 0;
        font-size: 14px;
        font-weight: 500;
        color: #e1e1e1;
      }

      .help-nav {
        flex: 1;
        overflow-y: auto;
        padding: 8px 0;
      }

      .help-nav-item {
        display: flex;
        align-items: center;
        width: 100%;
        padding: 8px 16px;
        background: transparent;
        border: none;
        color: #cccccc;
        cursor: pointer;
        text-align: left;
        transition: background-color 0.1s;
      }

      .help-nav-item:hover {
        background: #2a2d2e;
      }

      .help-nav-item.active {
        background: #094771;
        color: #ffffff;
      }

      .help-nav-icon {
        margin-right: 8px;
        font-size: 16px;
      }

      .help-nav-title {
        font-size: 13px;
      }

      .help-content {
        flex: 1;
        overflow-y: scroll; /* Force scrollbar to always be visible */
        overflow-x: hidden;
        background: #1e1e1e;
        position: relative;
        height: 100%;
      }

      .help-content-inner {
        padding: 32px;
        max-width: 900px;
        padding-bottom: 100px; /* Extra padding at bottom for comfortable scrolling */
      }

      .help-content h2 {
        margin: 0 0 24px 0;
        font-size: 24px;
        font-weight: 300;
        color: #e1e1e1;
        border-bottom: 1px solid #3c3c3c;
        padding-bottom: 8px;
      }

      .help-content h3 {
        margin: 24px 0 12px 0;
        font-size: 18px;
        font-weight: 400;
        color: #e1e1e1;
      }

      .help-content h4 {
        margin: 16px 0 8px 0;
        font-size: 14px;
        font-weight: 500;
        color: #e1e1e1;
      }

      .help-content p {
        margin: 0 0 12px 0;
        line-height: 1.6;
      }

      .help-content ul {
        margin: 0 0 16px 0;
        padding-left: 24px;
      }

      .help-content ol {
        margin: 0 0 16px 0;
        padding-left: 24px;
      }

      .help-content li {
        margin: 4px 0;
        line-height: 1.6;
      }

      .help-content code {
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 12px;
        background: #2d2d30;
        padding: 2px 4px;
        border-radius: 3px;
        color: #ce9178;
      }

      .code-block {
        margin: 16px 0;
        background: #1a1a1a;
        border: 1px solid #3c3c3c;
        border-radius: 4px;
        overflow: hidden;
      }

      .code-header {
        padding: 8px 12px;
        background: #2d2d30;
        border-bottom: 1px solid #3c3c3c;
        font-size: 12px;
        color: #969696;
      }

      .code-block pre {
        margin: 0;
        padding: 12px;
        overflow-x: auto;
      }

      .code-block code {
        display: block;
        background: transparent;
        padding: 0;
        color: #d4d4d4;
        line-height: 1.5;
      }

      .info-box {
        margin: 16px 0;
        padding: 16px;
        background: #252526;
        border: 1px solid #007acc;
        border-radius: 4px;
      }

      .info-box p {
        margin: 0 0 8px 0;
      }

      .info-box ol {
        margin: 8px 0;
      }

      .shortcut-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
        gap: 12px;
        margin: 16px 0;
      }

      .shortcut-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 8px;
        background: #252526;
        border-radius: 4px;
      }

      .shortcut-item kbd {
        display: inline-block;
        padding: 3px 6px;
        background: #3c3c3c;
        border: 1px solid #464647;
        border-radius: 3px;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 11px;
        color: #e1e1e1;
        box-shadow: 0 1px 0 rgba(0,0,0,0.2);
        white-space: nowrap;
      }

      .shortcut-item span {
        color: #969696;
        font-size: 12px;
      }

      /* Getting Started specific styles */
      .hero-section {
        background: #252526;
        border: 1px solid #007acc;
        border-radius: 8px;
        padding: 24px;
        margin: 24px 0;
      }

      .copy-block {
        background: #1a1a1a;
        border: 1px solid #3c3c3c;
        border-radius: 6px;
        margin: 16px 0;
        overflow: hidden;
      }

      .copy-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 12px;
        background: #2d2d30;
        border-bottom: 1px solid #3c3c3c;
        font-size: 12px;
        color: #969696;
      }

      .copy-button {
        background: #007acc;
        color: white;
        border: none;
        padding: 4px 12px;
        border-radius: 4px;
        cursor: pointer;
        font-size: 12px;
        transition: background 0.2s;
      }

      .copy-button:hover {
        background: #1177bb;
      }

      .copy-button:active {
        background: #005a9e;
      }

      .success-message {
        color: #4ec9b0;
        font-size: 14px;
        margin-top: 16px;
        font-weight: 500;
      }

      .workflow-steps {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 16px;
        margin: 24px 0;
      }

      .workflow-step {
        display: flex;
        align-items: flex-start;
        gap: 12px;
        padding: 16px;
        background: #252526;
        border-radius: 6px;
        border: 1px solid #3c3c3c;
      }

      .step-number {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 28px;
        height: 28px;
        background: #007acc;
        color: white;
        border-radius: 50%;
        font-weight: bold;
        font-size: 14px;
        flex-shrink: 0;
      }

      .step-content strong {
        display: block;
        color: #e1e1e1;
        margin-bottom: 4px;
      }

      .step-content p {
        margin: 0;
        font-size: 12px;
        color: #969696;
      }

      .tool-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
        gap: 12px;
        margin: 16px 0;
      }

      .tool-card {
        padding: 12px;
        background: #2d2d30;
        border: 1px solid #3c3c3c;
        border-radius: 4px;
        text-align: center;
        font-size: 12px;
        color: #e1e1e1;
      }

      /* Scrollbar styling - Always visible */
      .help-nav::-webkit-scrollbar,
      .help-content::-webkit-scrollbar {
        width: 14px;
        height: 14px;
      }

      .help-nav::-webkit-scrollbar-track,
      .help-content::-webkit-scrollbar-track {
        background: #1e1e1e;
        border-left: 1px solid #3c3c3c;
      }

      .help-nav::-webkit-scrollbar-thumb,
      .help-content::-webkit-scrollbar-thumb {
        background: #424242;
        border-radius: 7px;
        border: 3px solid #1e1e1e;
      }

      .help-nav::-webkit-scrollbar-thumb:hover,
      .help-content::-webkit-scrollbar-thumb:hover {
        background: #4a4a4a;
      }

      .help-nav::-webkit-scrollbar-corner,
      .help-content::-webkit-scrollbar-corner {
        background: #1e1e1e;
      }

      /* Force scrollbar visibility */
      .help-content {
        height: 100%;
        max-height: 100%;
        overflow-y: scroll !important; /* Force scrollbar to always show */
      }
      
      .help-nav {
        overflow-y: overlay; /* Use overlay to prevent layout shift */
      }
    `;
  }
}

// Export singleton instance
export const helpViewer = new HelpViewer();
