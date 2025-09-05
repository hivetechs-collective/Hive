// Neural Network Animation for Startup Screen
class StartupNeuralNetwork {
    constructor(canvas) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.width = 350;
        this.height = 350;
        this.canvas.width = this.width;
        this.canvas.height = this.height;
        
        // Animation properties
        this.animationFrame = null;
        this.progress = 0;
        this.targetProgress = 0;
        
        // Neural network structure
        this.layers = [
            { neurons: 3, x: 0.2 },
            { neurons: 4, x: 0.4 },
            { neurons: 4, x: 0.6 },
            { neurons: 3, x: 0.8 }
        ];
        
        // Calculate neuron positions
        this.neurons = [];
        this.connections = [];
        this.initializeNetwork();
        
        // Start animation
        this.animate();
    }
    
    initializeNetwork() {
        // Create neurons
        this.layers.forEach((layer, layerIndex) => {
            const x = layer.x * this.width;
            const spacing = this.height / (layer.neurons + 1);
            
            for (let i = 0; i < layer.neurons; i++) {
                const y = spacing * (i + 1);
                this.neurons.push({
                    x,
                    y,
                    layer: layerIndex,
                    index: i,
                    activation: 0,
                    pulse: 0
                });
            }
        });
        
        // Create connections between adjacent layers
        this.neurons.forEach(neuron => {
            const nextLayerNeurons = this.neurons.filter(n => n.layer === neuron.layer + 1);
            nextLayerNeurons.forEach(nextNeuron => {
                this.connections.push({
                    from: neuron,
                    to: nextNeuron,
                    strength: Math.random() * 0.5 + 0.5,
                    pulse: 0
                });
            });
        });
    }
    
    updateProgress(percent) {
        this.targetProgress = percent / 100;
    }
    
    animate() {
        this.animationFrame = requestAnimationFrame(() => this.animate());
        
        // Smooth progress transition
        const progressDiff = this.targetProgress - this.progress;
        this.progress += progressDiff * 0.1;
        
        // Clear canvas
        this.ctx.fillStyle = '#0E1414';
        this.ctx.fillRect(0, 0, this.width, this.height);
        
        // Update and draw connections
        this.drawConnections();
        
        // Update and draw neurons
        this.drawNeurons();
    }
    
    drawConnections() {
        const time = Date.now() * 0.001;
        
        this.connections.forEach((conn, index) => {
            const connectionProgress = index / this.connections.length;
            const isActive = connectionProgress <= this.progress;
            
            // Calculate pulse effect
            if (isActive) {
                conn.pulse = Math.sin(time * 3 + index * 0.5) * 0.3 + 0.7;
            } else {
                conn.pulse *= 0.95; // Fade out
            }
            
            // Draw connection
            this.ctx.beginPath();
            this.ctx.moveTo(conn.from.x, conn.from.y);
            
            // Add slight curve to connections
            const midX = (conn.from.x + conn.to.x) / 2;
            const midY = (conn.from.y + conn.to.y) / 2;
            const curveOffset = (Math.sin(time + index) * 10);
            
            this.ctx.quadraticCurveTo(
                midX + curveOffset,
                midY,
                conn.to.x,
                conn.to.y
            );
            
            // Set style based on activation
            const opacity = isActive ? conn.pulse * 0.6 : 0.1;
            this.ctx.strokeStyle = `rgba(255, 193, 7, ${opacity})`;
            this.ctx.lineWidth = isActive ? 2 : 1;
            this.ctx.stroke();
            
            // Draw pulse along active connections
            if (isActive && Math.random() > 0.98) {
                this.drawPulsePacket(conn, time);
            }
        });
    }
    
    drawPulsePacket(conn, time) {
        const t = (time * 0.3) % 1;
        const x = conn.from.x + (conn.to.x - conn.from.x) * t;
        const y = conn.from.y + (conn.to.y - conn.from.y) * t;
        
        this.ctx.beginPath();
        this.ctx.arc(x, y, 3, 0, Math.PI * 2);
        this.ctx.fillStyle = 'rgba(255, 193, 7, 0.8)';
        this.ctx.fill();
    }
    
    drawNeurons() {
        const time = Date.now() * 0.001;
        
        this.neurons.forEach((neuron, index) => {
            const neuronProgress = index / this.neurons.length;
            const isActive = neuronProgress <= this.progress;
            
            // Update activation
            if (isActive) {
                neuron.activation = Math.sin(time * 2 + index * 0.3) * 0.3 + 0.7;
            } else {
                neuron.activation *= 0.95; // Fade out
            }
            
            // Draw outer glow for active neurons
            if (isActive) {
                const glowRadius = 15 + Math.sin(time * 3 + index) * 5;
                const gradient = this.ctx.createRadialGradient(
                    neuron.x, neuron.y, 0,
                    neuron.x, neuron.y, glowRadius
                );
                gradient.addColorStop(0, `rgba(255, 193, 7, ${neuron.activation * 0.3})`);
                gradient.addColorStop(1, 'rgba(255, 193, 7, 0)');
                
                this.ctx.beginPath();
                this.ctx.arc(neuron.x, neuron.y, glowRadius, 0, Math.PI * 2);
                this.ctx.fillStyle = gradient;
                this.ctx.fill();
            }
            
            // Draw neuron circle
            this.ctx.beginPath();
            this.ctx.arc(neuron.x, neuron.y, 8, 0, Math.PI * 2);
            
            // Fill based on activation
            if (isActive) {
                const fillGradient = this.ctx.createRadialGradient(
                    neuron.x - 2, neuron.y - 2, 0,
                    neuron.x, neuron.y, 8
                );
                fillGradient.addColorStop(0, '#FFD54F');
                fillGradient.addColorStop(1, '#FFC107');
                this.ctx.fillStyle = fillGradient;
                this.ctx.fill();
            } else {
                this.ctx.fillStyle = '#1A1F21';
                this.ctx.fill();
                this.ctx.strokeStyle = 'rgba(255, 193, 7, 0.2)';
                this.ctx.lineWidth = 1;
                this.ctx.stroke();
            }
            
            // Draw center dot for active neurons
            if (isActive) {
                this.ctx.beginPath();
                this.ctx.arc(neuron.x, neuron.y, 3, 0, Math.PI * 2);
                this.ctx.fillStyle = '#FFFFFF';
                this.ctx.fill();
            }
        });
    }
    
    destroy() {
        if (this.animationFrame) {
            cancelAnimationFrame(this.animationFrame);
        }
    }
}

// Initialize neural network when page loads
let neuralNetwork = null;
const progressBar = document.getElementById('progress');
const statusMessage = document.getElementById('status-message');
const errorContainer = document.getElementById('error-container');
const errorMessage = document.getElementById('error-message');

document.addEventListener('DOMContentLoaded', () => {
    const canvas = document.getElementById('neural-canvas');
    neuralNetwork = new StartupNeuralNetwork(canvas);
});

// Listen for progress updates from main process
if (window.electronAPI) {
    window.electronAPI.onStartupProgress((event, data) => {
        // Update progress bar
        progressBar.style.width = `${data.percent}%`;
        
        // Update status text with fade effect
        const statusEl = document.getElementById('status');
        statusEl.classList.add('changing');
        setTimeout(() => {
            statusMessage.textContent = data.status;
            statusEl.classList.remove('changing');
        }, 150);
        
        // Update neural network animation
        if (neuralNetwork) {
            neuralNetwork.updateProgress(data.percent);
        }
    });
    
    window.electronAPI.onStartupError((event, error) => {
        // Show error state
        errorContainer.classList.add('visible');
        errorMessage.textContent = error.message || 'An unexpected error occurred';
        
        // Stop neural animation
        if (neuralNetwork) {
            neuralNetwork.destroy();
        }
    });
}