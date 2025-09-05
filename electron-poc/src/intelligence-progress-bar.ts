// Intelligence Progress Bar - Shows AI memory retrieval and context building
export interface IntelligenceUpdate {
  phase: 'memory' | 'context' | 'classification' | 'complete' | null;
  progress: number;
  memoryHits?: number;
  contextRelevance?: number;
  message?: string;
}

export class IntelligenceProgressBar {
  private container: HTMLElement;
  private progressFill: HTMLElement;
  private phases: Array<{name: string, icon: string, element: HTMLElement}>;
  private currentPhase: string | null = null;
  private animationTimer: number | null = null;

  constructor() {
    this.container = this.createContainer();
    this.phases = [];
    this.progressFill = null!;
    this.hide();
  }

  private createContainer(): HTMLElement {
    const container = document.createElement('div');
    container.className = 'intelligence-progress-container';
    container.innerHTML = `
      <div class="intelligence-header">
        <div class="intelligence-title">
          <span class="ai-icon">🤖</span>
          <span class="title-text">AI Intelligence Engine</span>
          <span class="memory-badge" style="display: none;"></span>
          <span class="context-badge" style="display: none;"></span>
        </div>
      </div>
      
      <div class="intelligence-phases">
        <div class="phase-segment" data-phase="memory">
          <div class="phase-icon">🧠</div>
          <div class="phase-info">
            <div class="phase-name">Memory Retrieval</div>
            <div class="phase-description">Pending...</div>
          </div>
          <div class="phase-connector">
            <div class="connector-line"></div>
            <div class="connector-arrow">→</div>
          </div>
        </div>
        
        <div class="phase-segment" data-phase="context">
          <div class="phase-icon">🔗</div>
          <div class="phase-info">
            <div class="phase-name">Context Synthesis</div>
            <div class="phase-description">Pending...</div>
          </div>
          <div class="phase-connector">
            <div class="connector-line"></div>
            <div class="connector-arrow">→</div>
          </div>
        </div>
        
        <div class="phase-segment" data-phase="classification">
          <div class="phase-icon">⚡</div>
          <div class="phase-info">
            <div class="phase-name">Classification</div>
            <div class="phase-description">Pending...</div>
          </div>
        </div>
      </div>
      
      <div class="intelligence-progress-track">
        <div class="intelligence-progress-fill">
          <div class="progress-glow"></div>
          <div class="progress-particles">
            <div class="particle particle-1"></div>
            <div class="particle particle-2"></div>
            <div class="particle particle-3"></div>
            <div class="particle particle-4"></div>
            <div class="particle particle-5"></div>
          </div>
        </div>
        
        <div class="neural-network-overlay">
          <svg class="neural-svg" viewBox="0 0 100 20"></svg>
        </div>
      </div>
      
      <div class="intelligence-complete" style="display: none;">
        <span class="complete-icon">✨</span>
        <span class="complete-text">Intelligence gathered - Processing with full context</span>
      </div>
    `;

    // Store references to key elements
    this.progressFill = container.querySelector('.intelligence-progress-fill') as HTMLElement;
    
    // Store phase elements
    const phaseElements = container.querySelectorAll('.phase-segment');
    phaseElements.forEach(el => {
      const phase = el.getAttribute('data-phase');
      if (phase) {
        this.phases.push({
          name: phase,
          icon: (el.querySelector('.phase-icon') as HTMLElement).textContent || '',
          element: el as HTMLElement
        });
      }
    });

    return container;
  }

  public mount(parentElement: HTMLElement): void {
    // Insert at the top of the parent element
    if (parentElement.firstChild) {
      parentElement.insertBefore(this.container, parentElement.firstChild);
    } else {
      parentElement.appendChild(this.container);
    }
  }

  public show(): void {
    this.container.style.display = 'block';
    this.container.classList.add('pulse');
    setTimeout(() => {
      this.container.classList.remove('pulse');
    }, 600);
  }

  public hide(): void {
    this.container.style.display = 'none';
    this.reset();
  }

  public update(update: IntelligenceUpdate): void {
    // Show the container if hidden
    if (this.container.style.display === 'none') {
      this.show();
    }

    // Update progress bar
    if (update.progress !== undefined) {
      this.progressFill.style.width = `${update.progress}%`;
    }

    // Update phase visuals
    if (update.phase) {
      this.updatePhase(update.phase);
      this.updateNeuralAnimation(update.phase);
    }

    // Update badges
    if (update.memoryHits !== undefined && update.memoryHits > 0) {
      const badge = this.container.querySelector('.memory-badge') as HTMLElement;
      badge.textContent = `${update.memoryHits} memories found`;
      badge.style.display = 'inline-block';
    }

    if (update.contextRelevance !== undefined && update.contextRelevance > 0) {
      const badge = this.container.querySelector('.context-badge') as HTMLElement;
      badge.textContent = `${Math.round(update.contextRelevance * 100)}% relevant`;
      badge.style.display = 'inline-block';
    }

    // Handle completion
    if (update.phase === 'complete') {
      const completeDiv = this.container.querySelector('.intelligence-complete') as HTMLElement;
      completeDiv.style.display = 'flex';
      
      // Hide after 2 seconds
      setTimeout(() => {
        this.hide();
      }, 2000);
    }
  }

  private updatePhase(phase: string): void {
    // Clear all phase states
    this.phases.forEach(p => {
      p.element.classList.remove('active', 'completed');
      const desc = p.element.querySelector('.phase-description') as HTMLElement;
      desc.textContent = 'Pending...';
    });

    // Update based on current phase
    switch(phase) {
      case 'memory':
        this.setPhaseActive('memory', 'Searching past conversations...');
        break;
        
      case 'context':
        this.setPhaseCompleted('memory');
        this.setPhaseActive('context', 'Building understanding...');
        break;
        
      case 'classification':
        this.setPhaseCompleted('memory');
        this.setPhaseCompleted('context');
        this.setPhaseActive('classification', 'Determining approach...');
        break;
        
      case 'complete':
        this.phases.forEach(p => this.setPhaseCompleted(p.name));
        break;
    }

    // Update connectors
    this.updateConnectors(phase);
  }

  private setPhaseActive(phaseName: string, description: string): void {
    const phase = this.phases.find(p => p.name === phaseName);
    if (phase) {
      phase.element.classList.add('active');
      const desc = phase.element.querySelector('.phase-description') as HTMLElement;
      desc.textContent = description;
    }
  }

  private setPhaseCompleted(phaseName: string): void {
    const phase = this.phases.find(p => p.name === phaseName);
    if (phase) {
      phase.element.classList.remove('active');
      phase.element.classList.add('completed');
      const desc = phase.element.querySelector('.phase-description') as HTMLElement;
      desc.textContent = '✓ Complete';
    }
  }

  private updateConnectors(phase: string): void {
    const connectors = this.container.querySelectorAll('.phase-connector');
    connectors.forEach((connector, index) => {
      connector.classList.remove('active');
      
      // Activate connectors based on phase
      if (phase === 'context' && index === 0) {
        connector.classList.add('active');
      } else if (phase === 'classification' && index <= 1) {
        connector.classList.add('active');
      } else if (phase === 'complete') {
        connector.classList.add('active');
      }
    });
  }

  private updateNeuralAnimation(phase: string): void {
    const svg = this.container.querySelector('.neural-svg') as SVGElement;
    svg.innerHTML = '';

    switch(phase) {
      case 'memory':
        // Animated neurons for memory retrieval
        svg.innerHTML = `
          <circle class="neuron" cx="10" cy="10" r="2" />
          <circle class="neuron" cx="25" cy="8" r="2" />
          <circle class="neuron" cx="40" cy="12" r="2" />
          <circle class="neuron" cx="55" cy="10" r="2" />
          <circle class="neuron" cx="70" cy="9" r="2" />
          <line class="synapse" x1="10" y1="10" x2="25" y2="8" />
          <line class="synapse" x1="25" y1="8" x2="40" y2="12" />
          <line class="synapse" x1="40" y1="12" x2="55" y2="10" />
          <line class="synapse" x1="55" y1="10" x2="70" y2="9" />
        `;
        break;
        
      case 'context':
        // Weaving threads for context synthesis
        svg.innerHTML = `
          <path class="thread thread-1" d="M 0,10 Q 25,5 50,10 T 100,10" />
          <path class="thread thread-2" d="M 0,8 Q 25,13 50,8 T 100,8" />
          <path class="thread thread-3" d="M 0,12 Q 25,7 50,12 T 100,12" />
        `;
        break;
        
      case 'classification':
        // Lightning bolts for classification
        svg.innerHTML = `
          <path class="lightning" d="M 20,5 L 25,10 L 22,10 L 27,15 L 24,10 L 27,10 Z" />
          <path class="lightning" d="M 50,5 L 55,10 L 52,10 L 57,15 L 54,10 L 57,10 Z" />
          <path class="lightning" d="M 80,5 L 85,10 L 82,10 L 87,15 L 84,10 L 87,10 Z" />
        `;
        break;
    }
  }

  public reset(): void {
    // Reset all phases
    this.phases.forEach(p => {
      p.element.classList.remove('active', 'completed');
      const desc = p.element.querySelector('.phase-description') as HTMLElement;
      desc.textContent = 'Pending...';
    });

    // Reset progress
    this.progressFill.style.width = '0%';

    // Hide badges
    (this.container.querySelector('.memory-badge') as HTMLElement).style.display = 'none';
    (this.container.querySelector('.context-badge') as HTMLElement).style.display = 'none';

    // Hide complete message
    (this.container.querySelector('.intelligence-complete') as HTMLElement).style.display = 'none';

    // Clear animations
    const svg = this.container.querySelector('.neural-svg') as SVGElement;
    svg.innerHTML = '';
  }
}