// Neural Consciousness Awakening - Advanced AI Visualization
// A living, breathing representation of AI intelligence processing

export interface ConsciousnessPhase {
  name: 'awakening' | 'memory' | 'synthesis' | 'classification' | 'execution';
  progress: number;
  neurons: number;
  connections: number;
  energy: number;
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
        
        <div class="consciousness-info">
          <div class="phase-indicator">
            <span class="phase-icon">🧠</span>
            <span class="phase-name">Initializing...</span>
          </div>
          
          <div class="neural-stats">
            <div class="stat-item">
              <span class="stat-label">Neurons:</span>
              <span class="stat-value neurons-count">0</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Synapses:</span>
              <span class="stat-value connections-count">0</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">Energy:</span>
              <span class="stat-value energy-level">0%</span>
            </div>
          </div>
          
          <div class="thought-stream">
            <div class="thought-bubble memory-thought" style="display: none;">
              <span class="thought-icon">💭</span>
              <span class="thought-text">Accessing memory banks...</span>
            </div>
            <div class="thought-bubble synthesis-thought" style="display: none;">
              <span class="thought-icon">🔮</span>
              <span class="thought-text">Synthesizing context...</span>
            </div>
            <div class="thought-bubble classification-thought" style="display: none;">
              <span class="thought-icon">⚡</span>
              <span class="thought-text">Analyzing complexity...</span>
            </div>
          </div>
        </div>
      </div>
    `;
    
    // Initially hidden
    container.style.display = 'none';
    return container;
  }
  
  private setupCanvas(): void {
    const resize = () => {
      const rect = this.container.getBoundingClientRect();
      this.canvas.width = rect.width;
      this.canvas.height = rect.height;
      this.centerCore.x = this.canvas.width / 2;
      this.centerCore.y = this.canvas.height / 2;
    };
    
    window.addEventListener('resize', resize);
    resize();
  }
  
  private initializeNeuralNetwork(): void {
    // Create initial neurons in a circular pattern
    const neuronCount = 50;
    const radius = Math.min(this.canvas.width, this.canvas.height) * 0.3;
    
    for (let i = 0; i < neuronCount; i++) {
      const angle = (i / neuronCount) * Math.PI * 2;
      const x = this.centerCore.x + Math.cos(angle) * radius * (0.5 + Math.random() * 0.5);
      const y = this.centerCore.y + Math.sin(angle) * radius * (0.5 + Math.random() * 0.5);
      
      this.neurons.push(new Neuron(x, y, Math.random() * 3 + 2));
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
  
  public mount(parentElement: HTMLElement): void {
    // Mount as an overlay at the top of the chat area
    parentElement.parentElement?.insertBefore(this.container, parentElement);
  }
  
  public enable(featureFlag: boolean = true): void {
    this.enabled = featureFlag;
    console.log(`Neural Consciousness ${this.enabled ? 'enabled' : 'disabled'}`);
  }
  
  public async show(): Promise<void> {
    if (!this.enabled) return;
    
    this.container.style.display = 'block';
    this.container.classList.add('consciousness-awakening');
    
    // Start the animation
    this.animate();
    
    // Begin awakening sequence
    await this.awaken();
  }
  
  public hide(): void {
    this.container.style.display = 'none';
    this.container.classList.remove('consciousness-awakening');
    
    if (this.animationId) {
      cancelAnimationFrame(this.animationId);
      this.animationId = null;
    }
    
    this.reset();
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
  
  public async updatePhase(phase: 'memory' | 'synthesis' | 'classification'): Promise<void> {
    if (!this.enabled) return;
    
    const phaseElement = this.container.querySelector('.phase-name') as HTMLElement;
    const phaseIcon = this.container.querySelector('.phase-icon') as HTMLElement;
    
    switch (phase) {
      case 'memory':
        this.currentPhase.name = 'memory';
        phaseElement.textContent = 'Memory Retrieval';
        phaseIcon.textContent = '🧠';
        this.showThought('memory');
        await this.animateMemoryPhase();
        break;
        
      case 'synthesis':
        this.currentPhase.name = 'synthesis';
        phaseElement.textContent = 'Context Synthesis';
        phaseIcon.textContent = '🔗';
        this.showThought('synthesis');
        await this.animateSynthesisPhase();
        break;
        
      case 'classification':
        this.currentPhase.name = 'classification';
        phaseElement.textContent = 'Classification';
        phaseIcon.textContent = '⚡';
        this.showThought('classification');
        await this.animateClassificationPhase();
        break;
    }
    
    this.updateStats();
  }
  
  private showThought(type: 'memory' | 'synthesis' | 'classification'): void {
    // Hide all thoughts
    this.container.querySelectorAll('.thought-bubble').forEach(el => {
      (el as HTMLElement).style.display = 'none';
    });
    
    // Show specific thought
    const thought = this.container.querySelector(`.${type}-thought`) as HTMLElement;
    if (thought) {
      thought.style.display = 'flex';
      thought.classList.add('thought-appear');
    }
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
  
  private animate(): void {
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
    
    // Draw neurons
    this.neurons.forEach(neuron => {
      neuron.update(this.time);
      neuron.draw(this.ctx);
    });
    
    // Draw central core
    this.centerCore.update(this.time);
    this.centerCore.draw(this.ctx);
    
    // Continue animation
    this.animationId = requestAnimationFrame(() => this.animate());
  }
  
  private updateStats(): void {
    const neuronsEl = this.container.querySelector('.neurons-count') as HTMLElement;
    const connectionsEl = this.container.querySelector('.connections-count') as HTMLElement;
    const energyEl = this.container.querySelector('.energy-level') as HTMLElement;
    
    if (neuronsEl) neuronsEl.textContent = this.currentPhase.neurons.toString();
    if (connectionsEl) connectionsEl.textContent = this.currentPhase.connections.toString();
    if (energyEl) energyEl.textContent = Math.round(this.currentPhase.energy) + '%';
  }
  
  private reset(): void {
    this.currentPhase = {
      name: 'awakening',
      progress: 0,
      neurons: 0,
      connections: 0,
      energy: 0
    };
    
    this.neurons.forEach(n => n.deactivate());
    this.connections.forEach(c => c.strength = 0.1);
    this.particles = [];
    this.centerCore.energy = 0;
    this.time = 0;
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
    const size = this.radius * (1 + intensity * 0.5);
    
    // Glow effect
    const gradient = ctx.createRadialGradient(this.x, this.y, 0, this.x, this.y, size * 3);
    gradient.addColorStop(0, `rgba(100, 200, 255, ${intensity})`);
    gradient.addColorStop(0.5, `rgba(100, 200, 255, ${intensity * 0.3})`);
    gradient.addColorStop(1, 'rgba(100, 200, 255, 0)');
    
    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(this.x, this.y, size * 3, 0, Math.PI * 2);
    ctx.fill();
    
    // Core
    ctx.fillStyle = `rgba(255, 255, 255, ${intensity})`;
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
  private speed: number = 0.05;
  
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
    this.life -= 0.02;
  }
  
  draw(ctx: CanvasRenderingContext2D): void {
    ctx.fillStyle = this.color.replace(')', `, ${this.life})`).replace('rgb', 'rgba');
    ctx.beginPath();
    ctx.arc(this.x, this.y, 2 * this.life, 0, Math.PI * 2);
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
    const size = 10 + this.energy * 20;
    const glow = size * (2 + this.flashIntensity * 3);
    
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