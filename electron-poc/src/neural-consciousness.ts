// Neural Consciousness Awakening - Advanced AI Visualization
// A living, breathing representation of AI intelligence processing

import hiveLogo from './Hive-Logo-small.jpg';

export interface ConsciousnessPhase {
  name: 'awakening' | 'memory' | 'synthesis' | 'classification' | 'generator' | 'refiner' | 'validator' | 'curator' | 'completion';
  progress: number;
  neurons: number;
  connections: number;
  energy: number;
  description?: string;
}

export class NeuralConsciousness {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private animationId: number | null = null;
  private currentPhase: ConsciousnessPhase;
  private neurons: Neuron[] = [];
  private connections: Connection[] = [];
  private particles: Particle[] = [];
  private centerCore: Core;
  private time: number = 0;
  private enabled: boolean = false;
  
  constructor() {
    this.container = this.createContainer();
    this.canvas = this.container.querySelector('.neural-canvas') as HTMLCanvasElement;
    this.ctx = this.canvas.getContext('2d')!;
    
    this.currentPhase = {
      name: 'awakening',
      progress: 0,
      neurons: 0,
      connections: 0,
      energy: 0
    };
    
    this.centerCore = new Core(0, 0);
    this.setupCanvas();
    this.initializeNeuralNetwork();
  }
  
  private createContainer(): HTMLElement {
    const container = document.createElement('div');
    container.className = 'neural-consciousness-container';
    container.innerHTML = `
      <div class="consciousness-overlay">
        <canvas class="neural-canvas"></canvas>
        
        <!-- Hive Logo in center -->
        <div class="hive-logo-center">
          <img src="${hiveLogo}" alt="HiveTechs Logo" class="hive-logo-img" />
        </div>
        
        <div class="consciousness-info">
          <div class="phase-indicator">
            <span class="phase-status">Neural Core</span>
          </div>
          
          <div class="thought-stream">
            <div class="thought-bubble memory-thought" style="display: none;">
              <span class="thought-text">Searching memories...</span>
            </div>
            <div class="thought-bubble synthesis-thought" style="display: none;">
              <span class="thought-text">Building context...</span>
            </div>
            <div class="thought-bubble classification-thought" style="display: none;">
              <span class="thought-text">Routing decision...</span>
            </div>
            <div class="thought-bubble generator-thought" style="display: none;">
              <span class="thought-text">Creating response...</span>
            </div>
            <div class="thought-bubble refiner-thought" style="display: none;">
              <span class="thought-text">Refining quality...</span>
            </div>
            <div class="thought-bubble validator-thought" style="display: none;">
              <span class="thought-text">Verifying accuracy...</span>
            </div>
            <div class="thought-bubble curator-thought" style="display: none;">
              <span class="thought-text">Final polish...</span>
            </div>
          </div>
        </div>
      </div>
    `;
    
    // Container is now visible by default
    container.style.display = 'block';
    return container;
  }
  
  private setupCanvas(): void {
    const resize = () => {
      // Fixed circular dimensions
      this.canvas.width = 180;
      this.canvas.height = 180;
      this.centerCore.x = 90;
      this.centerCore.y = 90;
    };
    
    window.addEventListener('resize', resize);
    resize();
  }
  
  private initializeNeuralNetwork(): void {
    // Create initial neurons in a circular pattern
    const neuronCount = 30; // Original neuron count
    const radius = 60; // Radius for proper display
    
    for (let i = 0; i < neuronCount; i++) {
      const angle = (i / neuronCount) * Math.PI * 2;
      const x = this.centerCore.x + Math.cos(angle) * radius * (0.5 + Math.random() * 0.5);
      const y = this.centerCore.y + Math.sin(angle) * radius * (0.5 + Math.random() * 0.5);
      
      this.neurons.push(new Neuron(x, y, Math.random() * 2 + 1));
    }
    
    // Create some initial connections
    for (let i = 0; i < 20; i++) {
      const n1 = this.neurons[Math.floor(Math.random() * this.neurons.length)];
      const n2 = this.neurons[Math.floor(Math.random() * this.neurons.length)];
      if (n1 !== n2) {
        this.connections.push(new Connection(n1, n2));
      }
    }
  }
  
  public getContainer(): HTMLElement {
    return this.container;
  }
  
  public mount(parentElement: HTMLElement): void {
    // Mount to the left sidebar as the first element (replacing logo)
    const sidebar = document.getElementById('left-sidebar');
    if (sidebar) {
      sidebar.insertBefore(this.container, sidebar.firstChild);
    } else {
      // Fallback to parent element
      parentElement.appendChild(this.container);
    }
    
    // Always visible, start in idle mode
    this.container.style.display = 'block';
    this.container.classList.add('consciousness-idle');
    
    // Start idle animation
    this.animate();
    this.startIdleAnimation();
  }
  
  public enable(featureFlag: boolean = true): void {
    this.enabled = featureFlag;
    console.log(`Neural Consciousness ${this.enabled ? 'enabled' : 'disabled'}`);
  }
  
  public startIdleAnimation(): void {
    // Gentle idle animation for neurons
    this.neurons.forEach((neuron, i) => {
      neuron.activate(0.2 + Math.random() * 0.3);
      // Add orbital motion properties
      (neuron as any).orbitAngle = (i / this.neurons.length) * Math.PI * 2;
      (neuron as any).orbitRadius = 40 + Math.random() * 30;
      (neuron as any).orbitSpeed = 0.001 + Math.random() * 0.002;
      (neuron as any).baseX = this.centerCore.x;
      (neuron as any).baseY = this.centerCore.y;
    });
    
    // Weak but visible connections
    this.connections.forEach(conn => {
      conn.strength = 0.1 + Math.random() * 0.15;
    });
    
    // Gentle pulsing core energy
    this.centerCore.energy = 0.2;
  }
  
  public async show(): Promise<void> {
    if (!this.enabled) return;
    
    // Transition from idle to awakening
    this.container.classList.remove('consciousness-idle');
    this.container.classList.add('consciousness-awakening');
    
    // Begin awakening sequence
    await this.awaken();
  }
  
  public hide(): void {
    // Don't actually hide, just return to idle state
    this.container.classList.remove('consciousness-awakening');
    this.container.classList.add('consciousness-idle');
    
    // Restore the Hive logo in center
    const logoCenter = this.container.querySelector('.hive-logo-center') as HTMLElement;
    if (logoCenter) {
      logoCenter.innerHTML = `<img src="${hiveLogo}" alt="HiveTechs Logo" class="hive-logo-img" />`;
    }
    
    // Reset phase indicator to idle
    const statusElement = this.container.querySelector('.phase-status') as HTMLElement;
    if (statusElement) statusElement.textContent = 'Neural Core';
    
    // Reset to idle animation (but keep neurons active)
    this.resetToIdle();
  }
  
  public async showCompletion(): Promise<void> {
    if (!this.enabled) return;
    
    const statusElement = this.container.querySelector('.phase-status') as HTMLElement;
    
    // Update to completion phase
    this.container.setAttribute('data-phase', 'completion');
    this.currentPhase.name = 'completion';
    if (statusElement) statusElement.textContent = 'Complete ✨';
    
    // Hide all thought bubbles
    this.container.querySelectorAll('.thought-bubble').forEach(el => {
      (el as HTMLElement).style.display = 'none';
    });
    
    // Grand finale animation
    await this.animateCompletionPhase();
  }
  
  private async animateCompletionPhase(): Promise<void> {
    // Grand finale: explosion of understanding
    return new Promise(resolve => {
      const duration = 3000;
      const startTime = Date.now();
      
      const completionAnimation = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        // Massive energy burst from core
        if (progress < 0.3) {
          // Initial explosion
          for (let i = 0; i < 5; i++) {
            const angle = Math.random() * Math.PI * 2;
            const distance = 50 + Math.random() * 200;
            const x = this.centerCore.x + Math.cos(angle) * distance;
            const y = this.centerCore.y + Math.sin(angle) * distance;
            
            // Multi-colored celebration particles
            const colors = ['#ff6b6b', '#4ecdc4', '#45b7d1', '#f9ca24', '#ffd700', '#00ff88'];
            const color = colors[Math.floor(Math.random() * colors.length)];
            this.particles.push(new Particle(
              this.centerCore.x,
              this.centerCore.y,
              x,
              y,
              color
            ));
          }
          
          // Flash the core
          this.centerCore.energy = 1;
          this.centerCore.flash();
        }
        
        // Middle phase: golden convergence
        if (progress > 0.3 && progress < 0.7) {
          // All neurons pulse in sync
          this.neurons.forEach(neuron => {
            neuron.activate(0.5 + Math.sin(elapsed * 0.01) * 0.5);
          });
          
          // Golden particles spiral inward
          const spiralAngle = elapsed * 0.005;
          const spiralRadius = 150 * (1 - (progress - 0.3) / 0.4);
          const x = this.centerCore.x + Math.cos(spiralAngle) * spiralRadius;
          const y = this.centerCore.y + Math.sin(spiralAngle) * spiralRadius;
          this.particles.push(new Particle(x, y, this.centerCore.x, this.centerCore.y, '#ffd700'));
        }
        
        // Final phase: peaceful fade
        if (progress > 0.7) {
          const fadeProgress = (progress - 0.7) / 0.3;
          
          // Neurons slowly deactivate
          this.neurons.forEach(neuron => {
            neuron.activate((1 - fadeProgress) * 0.5);
          });
          
          // Connections fade
          this.connections.forEach(conn => {
            conn.strength = (1 - fadeProgress) * 0.5;
          });
          
          // Core energy fades
          this.centerCore.energy = (1 - fadeProgress) * 0.5;
        }
        
        this.currentPhase.energy = 100 * (1 - progress * 0.3);
        
        if (progress < 1) {
          requestAnimationFrame(completionAnimation);
        } else {
          resolve();
        }
      };
      
      completionAnimation();
    });
  }
  
  private async awaken(): Promise<void> {
    // Gradual awakening animation
    const duration = 1500;
    const startTime = Date.now();
    
    const awakeningAnimation = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);
      
      this.currentPhase.energy = progress * 30;
      this.centerCore.energy = progress;
      
      // Gradually activate neurons
      const activeNeurons = Math.floor(progress * this.neurons.length);
      for (let i = 0; i < activeNeurons; i++) {
        this.neurons[i].activate(0.5 + progress * 0.5);
      }
      
      if (progress < 1) {
        requestAnimationFrame(awakeningAnimation);
      }
    };
    
    awakeningAnimation();
  }
  
  public async updatePhase(phase: 'memory' | 'synthesis' | 'classification' | 'generator' | 'refiner' | 'validator' | 'curator'): Promise<void> {
    if (!this.enabled) return;
    
    // Update container data attribute for CSS effects
    this.container.setAttribute('data-phase', phase);
    
    // Update logo center to show processing stage
    this.updateLogoCenter(phase);
    
    // Excite neurons for this specific stage
    this.exciteNeuronsForStage(phase);
    
    switch (phase) {
      case 'memory':
        this.currentPhase.name = 'memory';
        await this.animateMemoryPhase();
        break;
        
      case 'synthesis':
        this.currentPhase.name = 'synthesis';
        await this.animateSynthesisPhase();
        break;
        
      case 'classification':
        this.currentPhase.name = 'classification';
        await this.animateClassificationPhase();
        break;
        
      case 'generator':
        this.currentPhase.name = 'generator';
        await this.animateGeneratorPhase();
        break;
        
      case 'refiner':
        this.currentPhase.name = 'refiner';
        await this.animateRefinerPhase();
        break;
        
      case 'validator':
        this.currentPhase.name = 'validator';
        await this.animateValidatorPhase();
        break;
        
      case 'curator':
        this.currentPhase.name = 'curator';
        await this.animateCuratorPhase();
        break;
    }
  }
  
  private updateLogoCenter(phase: string): void {
    const logoCenter = this.container.querySelector('.hive-logo-center') as HTMLElement;
    if (!logoCenter) return;
    
    const stageIcons: { [key: string]: string } = {
      'memory': '🧠',
      'synthesis': '🔗', 
      'classification': '🎯',
      'generator': '✨',
      'refiner': '💎',
      'validator': '✅',
      'curator': '🎨'
    };
    
    const stageNames: { [key: string]: string } = {
      'memory': 'Memory',
      'synthesis': 'Context', 
      'classification': 'Route',
      'generator': 'Generate',
      'refiner': 'Refine',
      'validator': 'Validate',
      'curator': 'Curate'
    };
    
    // Replace logo with processing indicator
    logoCenter.innerHTML = `
      <div class="processing-indicator">
        <div class="stage-icon">${stageIcons[phase]}</div>
        <div class="stage-name">${stageNames[phase]}</div>
      </div>
    `;
  }
  
  private exciteNeuronsForStage(stage: string): void {
    // Define stage-specific neuron behavior
    const stageConfig: { [key: string]: { color: string, speed: number, intensity: number, pattern: string } } = {
      'memory': { color: '#64c8ff', speed: 0.003, intensity: 0.7, pattern: 'convergent' },
      'synthesis': { color: '#a78bfa', speed: 0.004, intensity: 0.8, pattern: 'weaving' },
      'classification': { color: '#fbbf24', speed: 0.008, intensity: 0.9, pattern: 'lightning' },
      'generator': { color: '#ff6b6b', speed: 0.006, intensity: 1.0, pattern: 'explosive' },
      'refiner': { color: '#ffd700', speed: 0.002, intensity: 0.8, pattern: 'crystalline' },
      'validator': { color: '#00ff88', speed: 0.005, intensity: 0.9, pattern: 'scanning' },
      'curator': { color: '#ffd700', speed: 0.003, intensity: 1.0, pattern: 'convergent' }
    };
    
    const config = stageConfig[stage];
    if (!config) return;
    
    // Update all neurons for this stage
    this.neurons.forEach((neuron, i) => {
      const n = neuron as any;
      
      // Set stage-specific properties
      n.stageColor = config.color;
      n.stageSpeed = config.speed;
      n.stageIntensity = config.intensity;
      n.stagePattern = config.pattern;
      
      // Activate neurons with stage-specific intensity
      neuron.activate(config.intensity);
      
      // Update orbital properties for stage pattern
      switch (config.pattern) {
        case 'convergent':
          n.orbitRadius *= 0.8; // Move closer to center
          n.orbitSpeed = config.speed;
          break;
        case 'explosive':
          n.orbitRadius *= 1.5; // Move further from center
          n.orbitSpeed = config.speed * 2;
          break;
        case 'lightning':
          n.orbitSpeed = config.speed * 3; // Very fast movement
          break;
        case 'weaving':
          n.orbitSpeed = config.speed;
          n.weavingPhase = Math.random() * Math.PI * 2;
          break;
        case 'crystalline':
          n.orbitSpeed = config.speed * 0.5; // Slow, precise movement
          break;
        case 'scanning':
          n.scanPhase = i / this.neurons.length * Math.PI * 2;
          n.orbitSpeed = config.speed;
          break;
      }
    });
  }
  
  private async animateMemoryPhase(): Promise<void> {
    // Memory retrieval: neurons pulse and gather
    return new Promise(resolve => {
      const duration = 2000;
      const startTime = Date.now();
      
      const memoryAnimation = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        // Neurons converge toward center
        this.neurons.forEach(neuron => {
          neuron.x += (this.centerCore.x - neuron.x) * 0.01 * progress;
          neuron.y += (this.centerCore.y - neuron.y) * 0.01 * progress;
          neuron.activate(0.3 + progress * 0.7);
        });
        
        // Create memory particles
        if (Math.random() < 0.1) {
          const angle = Math.random() * Math.PI * 2;
          const distance = 200;
          const x = this.centerCore.x + Math.cos(angle) * distance;
          const y = this.centerCore.y + Math.sin(angle) * distance;
          this.particles.push(new Particle(x, y, this.centerCore.x, this.centerCore.y, '#64c8ff'));
        }
        
        this.currentPhase.energy = 30 + progress * 20;
        this.currentPhase.neurons = Math.floor(50 + progress * 50);
        
        if (progress < 1) {
          requestAnimationFrame(memoryAnimation);
        } else {
          resolve();
        }
      };
      
      memoryAnimation();
    });
  }
  
  private async animateSynthesisPhase(): Promise<void> {
    // Context synthesis: connections form and strengthen
    return new Promise(resolve => {
      const duration = 2000;
      const startTime = Date.now();
      
      const synthesisAnimation = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        // Create new connections
        if (this.connections.length < 100 && Math.random() < 0.1 * progress) {
          const n1 = this.neurons[Math.floor(Math.random() * this.neurons.length)];
          const n2 = this.neurons[Math.floor(Math.random() * this.neurons.length)];
          if (n1 !== n2) {
            this.connections.push(new Connection(n1, n2));
          }
        }
        
        // Strengthen connections
        this.connections.forEach(conn => {
          conn.strength = Math.min(1, conn.strength + 0.01 * progress);
        });
        
        this.currentPhase.energy = 50 + progress * 25;
        this.currentPhase.connections = this.connections.length;
        
        if (progress < 1) {
          requestAnimationFrame(synthesisAnimation);
        } else {
          resolve();
        }
      };
      
      synthesisAnimation();
    });
  }
  
  private async animateClassificationPhase(): Promise<void> {
    // Classification: lightning-fast decision spark
    return new Promise(resolve => {
      const duration = 1000;
      const startTime = Date.now();
      
      const classificationAnimation = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        // Lightning effect from core
        if (progress < 0.5 && Math.random() < 0.2) {
          const angle = Math.random() * Math.PI * 2;
          const distance = 150 * progress;
          const x = this.centerCore.x + Math.cos(angle) * distance;
          const y = this.centerCore.y + Math.sin(angle) * distance;
          this.particles.push(new Particle(
            this.centerCore.x, 
            this.centerCore.y, 
            x, 
            y, 
            '#fbbf24'
          ));
        }
        
        // Core energy surge
        this.centerCore.energy = 0.5 + progress * 0.5;
        this.currentPhase.energy = 75 + progress * 25;
        
        if (progress < 1) {
          requestAnimationFrame(classificationAnimation);
        } else {
          // Final flash
          this.centerCore.flash();
          setTimeout(() => resolve(), 500);
        }
      };
      
      classificationAnimation();
    });
  }
  
  private async animateGeneratorPhase(): Promise<void> {
    // Creative Genesis: explosive creativity, rainbow particles
    return new Promise(resolve => {
      const duration = 3000;
      const startTime = Date.now();
      
      const generatorAnimation = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        // Neurons explode outward in creative bursts
        if (Math.random() < 0.2 * progress) {
          const angle = Math.random() * Math.PI * 2;
          const distance = 50 + Math.random() * 100;
          const x = this.centerCore.x + Math.cos(angle) * distance;
          const y = this.centerCore.y + Math.sin(angle) * distance;
          
          this.neurons.push(new Neuron(x, y, Math.random() * 4 + 2));
          
          // Rainbow particles for creativity
          const colors = ['#ff6b6b', '#4ecdc4', '#45b7d1', '#f9ca24', '#f0932b', '#eb4d4b', '#6ab04c'];
          const color = colors[Math.floor(Math.random() * colors.length)];
          this.particles.push(new Particle(
            this.centerCore.x, 
            this.centerCore.y, 
            x, 
            y, 
            color
          ));
        }
        
        // Rapidly forming connections
        if (this.connections.length < 200 && Math.random() < 0.3) {
          const n1 = this.neurons[Math.floor(Math.random() * this.neurons.length)];
          const n2 = this.neurons[Math.floor(Math.random() * this.neurons.length)];
          if (n1 && n2 && n1 !== n2) {
            this.connections.push(new Connection(n1, n2));
          }
        }
        
        // Pulsing creative energy
        this.centerCore.energy = 0.7 + Math.sin(elapsed * 0.005) * 0.3;
        this.currentPhase.energy = 60 + progress * 20;
        this.currentPhase.neurons = this.neurons.length;
        
        if (progress < 1) {
          requestAnimationFrame(generatorAnimation);
        } else {
          resolve();
        }
      };
      
      generatorAnimation();
    });
  }
  
  private async animateRefinerPhase(): Promise<void> {
    // Crystalline Refinement: organization, golden threads
    return new Promise(resolve => {
      const duration = 2500;
      const startTime = Date.now();
      
      const refinerAnimation = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        // Neurons organize into geometric patterns
        const targetRadius = 60; // Fixed for 150px height
        this.neurons.forEach((neuron, i) => {
          const angle = (i / this.neurons.length) * Math.PI * 2;
          const targetX = this.centerCore.x + Math.cos(angle) * targetRadius;
          const targetY = this.centerCore.y + Math.sin(angle) * targetRadius;
          
          neuron.x += (targetX - neuron.x) * 0.02 * progress;
          neuron.y += (targetY - neuron.y) * 0.02 * progress;
          neuron.activate(0.6 + progress * 0.4);
        });
        
        // Golden refinement particles
        if (Math.random() < 0.1) {
          const angle = Math.random() * Math.PI * 2;
          const r = Math.random() * 150;
          const x = this.centerCore.x + Math.cos(angle) * r;
          const y = this.centerCore.y + Math.sin(angle) * r;
          this.particles.push(new Particle(x, y, this.centerCore.x, this.centerCore.y, '#ffd700'));
        }
        
        // Strengthen best connections
        this.connections.forEach(conn => {
          if (conn.strength > 0.5) {
            conn.strength = Math.min(1, conn.strength + 0.02 * progress);
          }
        });
        
        this.currentPhase.energy = 80 + progress * 10;
        
        if (progress < 1) {
          requestAnimationFrame(refinerAnimation);
        } else {
          resolve();
        }
      };
      
      refinerAnimation();
    });
  }
  
  private async animateValidatorPhase(): Promise<void> {
    // Truth Verification: scanning beams, verification pulses
    return new Promise(resolve => {
      const duration = 2000;
      const startTime = Date.now();
      let scanAngle = 0;
      
      const validatorAnimation = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        // Scanning beam effect
        scanAngle += 0.05;
        const scanX = this.centerCore.x + Math.cos(scanAngle) * 200;
        const scanY = this.centerCore.y + Math.sin(scanAngle) * 200;
        
        // Create scanning particles
        if (Math.random() < 0.3) {
          this.particles.push(new Particle(
            this.centerCore.x,
            this.centerCore.y,
            scanX,
            scanY,
            '#00ff00'
          ));
        }
        
        // Verification pulses from core
        if (progress > 0.3 && Math.random() < 0.1) {
          // Create verification wave
          for (let i = 0; i < 8; i++) {
            const angle = (i / 8) * Math.PI * 2;
            const x = this.centerCore.x + Math.cos(angle) * 50;
            const y = this.centerCore.y + Math.sin(angle) * 50;
            this.particles.push(new Particle(
              this.centerCore.x,
              this.centerCore.y,
              x,
              y,
              '#00ff88'
            ));
          }
        }
        
        // Flash validated neurons
        this.neurons.forEach(neuron => {
          if (Math.random() < 0.01 * progress) {
            neuron.activate(1);
          }
        });
        
        this.currentPhase.energy = 90 + progress * 5;
        
        if (progress < 1) {
          requestAnimationFrame(validatorAnimation);
        } else {
          resolve();
        }
      };
      
      validatorAnimation();
    });
  }
  
  private async animateCuratorPhase(): Promise<void> {
    // Final Polish: convergence, golden crystallization
    return new Promise(resolve => {
      const duration = 2000;
      const startTime = Date.now();
      
      const curatorAnimation = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        
        // All neurons converge toward center
        this.neurons.forEach(neuron => {
          const distToCenter = Math.sqrt(
            Math.pow(neuron.x - this.centerCore.x, 2) + 
            Math.pow(neuron.y - this.centerCore.y, 2)
          );
          
          if (distToCenter > 50) {
            neuron.x += (this.centerCore.x - neuron.x) * 0.03 * progress;
            neuron.y += (this.centerCore.y - neuron.y) * 0.03 * progress;
          }
          
          neuron.activate(0.8 + progress * 0.2);
        });
        
        // Golden crystallization particles
        if (Math.random() < 0.2) {
          const angle = Math.random() * Math.PI * 2;
          const distance = 200 * (1 - progress);
          const x = this.centerCore.x + Math.cos(angle) * distance;
          const y = this.centerCore.y + Math.sin(angle) * distance;
          this.particles.push(new Particle(x, y, this.centerCore.x, this.centerCore.y, '#ffd700'));
        }
        
        // All connections turn golden
        this.connections.forEach(conn => {
          conn.strength = 0.8 + progress * 0.2;
        });
        
        // Core reaches maximum energy
        this.centerCore.energy = 0.5 + progress * 0.5;
        this.currentPhase.energy = 95 + progress * 5;
        
        if (progress < 1) {
          requestAnimationFrame(curatorAnimation);
        } else {
          // Final completion flash
          this.centerCore.flash();
          setTimeout(() => resolve(), 500);
        }
      };
      
      curatorAnimation();
    });
  }
  
  public animate(): void {
    if (!this.ctx) return;
    
    // Clear canvas with slight fade effect
    this.ctx.fillStyle = 'rgba(15, 15, 35, 0.1)';
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    
    // Update time
    this.time += 0.01;
    
    // Draw connections
    this.connections.forEach(conn => conn.draw(this.ctx));
    
    // Draw and update particles
    this.particles = this.particles.filter(particle => {
      particle.update();
      particle.draw(this.ctx);
      return particle.life > 0;
    });
    
    // Draw neurons with continuous orbital motion and stage-specific behavior
    this.neurons.forEach(neuron => {
      const n = neuron as any;
      
      // Always apply orbital motion - different patterns for different stages
      if (n.orbitAngle !== undefined) {
        n.orbitAngle += n.orbitSpeed || 0.001;
        
        // Base orbital motion
        let newX = n.baseX + Math.cos(n.orbitAngle) * n.orbitRadius;
        let newY = n.baseY + Math.sin(n.orbitAngle) * n.orbitRadius;
        
        // Apply stage-specific movement patterns
        if (n.stagePattern) {
          switch (n.stagePattern) {
            case 'weaving':
              // Figure-8 weaving pattern
              if (n.weavingPhase !== undefined) {
                newX += Math.sin(n.orbitAngle * 2 + n.weavingPhase) * 15;
                newY += Math.cos(n.orbitAngle * 3 + n.weavingPhase) * 10;
              }
              break;
            
            case 'lightning':
              // Erratic, jagged movement
              if (Math.random() < 0.1) {
                newX += (Math.random() - 0.5) * 20;
                newY += (Math.random() - 0.5) * 20;
              }
              break;
            
            case 'scanning':
              // Synchronized scanning sweep
              if (n.scanPhase !== undefined) {
                const sweepAngle = this.time * 0.5 + n.scanPhase;
                newX += Math.cos(sweepAngle) * 10;
                newY += Math.sin(sweepAngle) * 10;
              }
              break;
            
            case 'crystalline':
              // Perfect geometric orbits
              newX = n.baseX + Math.cos(n.orbitAngle) * n.orbitRadius;
              newY = n.baseY + Math.sin(n.orbitAngle) * n.orbitRadius;
              break;
          }
        }
        
        neuron.x = newX;
        neuron.y = newY;
      }
      
      // Update activation based on state
      if (this.container.classList.contains('consciousness-idle')) {
        // Gentle pulsing for idle state
        neuron.activation = 0.2 + Math.sin(this.time * 2 + (n.orbitAngle || 0)) * 0.15;
        
        // Spawn idle particles occasionally
        if (Math.random() < 0.002) {
          const angle = Math.random() * Math.PI * 2;
          const startRadius = 20 + Math.random() * 30;
          const endRadius = 60 + Math.random() * 40;
          const startX = this.centerCore.x + Math.cos(angle) * startRadius;
          const startY = this.centerCore.y + Math.sin(angle) * startRadius;
          const endX = this.centerCore.x + Math.cos(angle) * endRadius;
          const endY = this.centerCore.y + Math.sin(angle) * endRadius;
          
          this.particles.push(new Particle(startX, startY, endX, endY, '#64c8ff'));
        }
      }
      
      // Set stage-specific color if available
      if (n.stageColor) {
        (neuron as any).currentColor = n.stageColor;
      } else {
        (neuron as any).currentColor = '#64c8ff'; // Default blue
      }
      
      neuron.update(this.time);
      neuron.draw(this.ctx);
    });
    
    // No longer drawing the core circle - using Hive logo instead
    
    // Continue animation
    this.animationId = requestAnimationFrame(() => this.animate());
  }
  
  private updateStats(): void {
    // Stats removed for compact view - neurons and connections are visual only
  }
  
  private resetToIdle(): void {
    this.currentPhase = {
      name: 'awakening',
      progress: 0,
      neurons: 0,
      connections: 0,
      energy: 0
    };
    
    // Don't deactivate neurons completely - return to idle state
    this.startIdleAnimation();
    this.particles = [];
    this.centerCore.energy = 0.2; // Keep some idle energy
  }
  
  private reset(): void {
    this.resetToIdle();
  }
}

// Neural network components
class Neuron {
  public x: number;
  public y: number;
  public radius: number;
  public activation: number = 0;
  private targetActivation: number = 0;
  private pulsePhase: number = 0;
  
  constructor(x: number, y: number, radius: number = 3) {
    this.x = x;
    this.y = y;
    this.radius = radius;
  }
  
  activate(level: number): void {
    this.targetActivation = level;
  }
  
  deactivate(): void {
    this.targetActivation = 0;
  }
  
  update(time: number): void {
    // Smooth activation transition
    this.activation += (this.targetActivation - this.activation) * 0.1;
    
    // Pulsing effect
    this.pulsePhase = Math.sin(time * 2 + this.x * 0.01) * 0.5 + 0.5;
  }
  
  draw(ctx: CanvasRenderingContext2D): void {
    const intensity = this.activation * (0.7 + this.pulsePhase * 0.3);
    const size = Math.max(0.1, this.radius * (1 + intensity * 0.5));
    
    // Get current color (stage-specific or default blue)
    const currentColor = (this as any).currentColor || '#64c8ff';
    
    // Parse color for gradient
    const colorMap: { [key: string]: { r: number, g: number, b: number } } = {
      '#64c8ff': { r: 100, g: 200, b: 255 }, // Blue
      '#a78bfa': { r: 167, g: 139, b: 250 }, // Purple
      '#fbbf24': { r: 251, g: 191, b: 36 },  // Yellow
      '#ff6b6b': { r: 255, g: 107, b: 107 }, // Red
      '#ffd700': { r: 255, g: 215, b: 0 },   // Gold
      '#00ff88': { r: 0, g: 255, b: 136 }    // Green
    };
    
    const color = colorMap[currentColor] || colorMap['#64c8ff'];
    
    // Stage-specific glow effect
    const glowRadius = Math.max(0.1, size * 4);
    const gradient = ctx.createRadialGradient(this.x, this.y, 0, this.x, this.y, glowRadius);
    gradient.addColorStop(0, `rgba(${color.r}, ${color.g}, ${color.b}, ${intensity * 1.2})`);
    gradient.addColorStop(0.3, `rgba(${color.r}, ${color.g}, ${color.b}, ${intensity * 0.8})`);
    gradient.addColorStop(0.6, `rgba(${color.r}, ${color.g}, ${color.b}, ${intensity * 0.4})`);
    gradient.addColorStop(1, `rgba(${color.r}, ${color.g}, ${color.b}, 0)`);
    
    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(this.x, this.y, glowRadius, 0, Math.PI * 2);
    ctx.fill();
    
    // Bright core with stage-specific color
    const coreR = Math.min(255, color.r + 50);
    const coreG = Math.min(255, color.g + 50);
    const coreB = Math.min(255, color.b + 50);
    ctx.fillStyle = `rgba(${coreR}, ${coreG}, ${coreB}, ${intensity * 1.2})`;
    ctx.beginPath();
    ctx.arc(this.x, this.y, size, 0, Math.PI * 2);
    ctx.fill();
  }
}

class Connection {
  public n1: Neuron;
  public n2: Neuron;
  public strength: number = 0.1;
  private pulsePosition: number = 0;
  
  constructor(n1: Neuron, n2: Neuron) {
    this.n1 = n1;
    this.n2 = n2;
  }
  
  draw(ctx: CanvasRenderingContext2D): void {
    const activity = (this.n1.activation + this.n2.activation) / 2;
    if (activity < 0.1) return;
    
    ctx.strokeStyle = `rgba(167, 139, 250, ${this.strength * activity * 0.5})`;
    ctx.lineWidth = this.strength * 2;
    ctx.beginPath();
    ctx.moveTo(this.n1.x, this.n1.y);
    ctx.lineTo(this.n2.x, this.n2.y);
    ctx.stroke();
    
    // Signal pulse
    if (activity > 0.5) {
      this.pulsePosition = (this.pulsePosition + 0.02) % 1;
      const px = this.n1.x + (this.n2.x - this.n1.x) * this.pulsePosition;
      const py = this.n1.y + (this.n2.y - this.n1.y) * this.pulsePosition;
      
      ctx.fillStyle = `rgba(255, 255, 255, ${activity})`;
      ctx.beginPath();
      ctx.arc(px, py, 2, 0, Math.PI * 2);
      ctx.fill();
    }
  }
}

class Particle {
  public x: number;
  public y: number;
  public targetX: number;
  public targetY: number;
  public life: number = 1;
  public color: string;
  private speed: number = 0.02;
  
  constructor(x: number, y: number, targetX: number, targetY: number, color: string) {
    this.x = x;
    this.y = y;
    this.targetX = targetX;
    this.targetY = targetY;
    this.color = color;
  }
  
  update(): void {
    this.x += (this.targetX - this.x) * this.speed;
    this.y += (this.targetY - this.y) * this.speed;
    this.life -= 0.01;  // Slower fade
  }
  
  draw(ctx: CanvasRenderingContext2D): void {
    // Ensure radius is never negative
    const radius = Math.max(0, 2 * this.life);
    if (radius <= 0) return;
    
    ctx.fillStyle = this.color.replace(')', `, ${this.life})`).replace('rgb', 'rgba');
    ctx.beginPath();
    ctx.arc(this.x, this.y, radius, 0, Math.PI * 2);
    ctx.fill();
  }
}

class Core {
  public x: number;
  public y: number;
  public energy: number = 0;
  private flashIntensity: number = 0;
  
  constructor(x: number, y: number) {
    this.x = x;
    this.y = y;
  }
  
  update(time: number): void {
    this.flashIntensity *= 0.95;
  }
  
  flash(): void {
    this.flashIntensity = 1;
  }
  
  draw(ctx: CanvasRenderingContext2D): void {
    const size = Math.max(1, 10 + this.energy * 20);
    const glow = Math.max(1, size * (2 + this.flashIntensity * 3));
    
    // Outer glow
    const gradient = ctx.createRadialGradient(this.x, this.y, 0, this.x, this.y, glow);
    gradient.addColorStop(0, `rgba(100, 200, 255, ${this.energy * 0.8})`);
    gradient.addColorStop(0.3, `rgba(167, 139, 250, ${this.energy * 0.4})`);
    gradient.addColorStop(0.6, `rgba(251, 191, 36, ${this.energy * 0.2})`);
    gradient.addColorStop(1, 'rgba(100, 200, 255, 0)');
    
    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(this.x, this.y, glow, 0, Math.PI * 2);
    ctx.fill();
    
    // Inner core
    ctx.fillStyle = `rgba(255, 255, 255, ${this.energy + this.flashIntensity})`;
    ctx.beginPath();
    ctx.arc(this.x, this.y, size, 0, Math.PI * 2);
    ctx.fill();
  }
}
