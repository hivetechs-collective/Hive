import React, { useState, useEffect } from 'react';
import './IntelligenceProgressBar.css';

interface IntelligencePhase {
  name: string;
  icon: string;
  description: string;
  startPercent: number;
  endPercent: number;
  active: boolean;
  completed: boolean;
}

interface IntelligenceProgressBarProps {
  phase?: 'memory' | 'context' | 'classification' | 'complete' | null;
  progress?: number;
  memoryHits?: number;
  contextRelevance?: number;
  visible?: boolean;
}

export const IntelligenceProgressBar: React.FC<IntelligenceProgressBarProps> = ({
  phase = null,
  progress = 0,
  memoryHits = 0,
  contextRelevance = 0,
  visible = false
}) => {
  const [animatedProgress, setAnimatedProgress] = useState(0);
  const [pulseAnimation, setPulseAnimation] = useState(false);
  
  const phases: IntelligencePhase[] = [
    {
      name: 'Memory Retrieval',
      icon: '🧠',
      description: 'Searching past conversations...',
      startPercent: 0,
      endPercent: 33,
      active: phase === 'memory',
      completed: progress > 33 || ['context', 'classification', 'complete'].includes(phase || '')
    },
    {
      name: 'Context Synthesis',
      icon: '🔗',
      description: 'Building understanding...',
      startPercent: 33,
      endPercent: 66,
      active: phase === 'context',
      completed: progress > 66 || ['classification', 'complete'].includes(phase || '')
    },
    {
      name: 'Classification',
      icon: '⚡',
      description: 'Determining approach...',
      startPercent: 66,
      endPercent: 100,
      active: phase === 'classification',
      completed: phase === 'complete'
    }
  ];

  useEffect(() => {
    // Smooth animation for progress
    const timer = setTimeout(() => {
      setAnimatedProgress(progress);
    }, 100);
    
    // Pulse effect when phase changes
    if (phase) {
      setPulseAnimation(true);
      setTimeout(() => setPulseAnimation(false), 600);
    }
    
    return () => clearTimeout(timer);
  }, [progress, phase]);

  if (!visible) return null;

  return (
    <div className={`intelligence-progress-container ${pulseAnimation ? 'pulse' : ''}`}>
      <div className="intelligence-header">
        <div className="intelligence-title">
          <span className="ai-icon">🤖</span>
          <span className="title-text">AI Intelligence Engine</span>
          {memoryHits > 0 && (
            <span className="memory-badge">
              {memoryHits} memories found
            </span>
          )}
          {contextRelevance > 0 && (
            <span className="context-badge">
              {Math.round(contextRelevance * 100)}% relevant
            </span>
          )}
        </div>
      </div>
      
      <div className="intelligence-phases">
        {phases.map((p, index) => (
          <div 
            key={p.name}
            className={`phase-segment ${p.active ? 'active' : ''} ${p.completed ? 'completed' : ''}`}
          >
            <div className="phase-icon">{p.icon}</div>
            <div className="phase-info">
              <div className="phase-name">{p.name}</div>
              <div className="phase-description">
                {p.active ? p.description : p.completed ? '✓ Complete' : 'Pending...'}
              </div>
            </div>
            {index < phases.length - 1 && (
              <div className={`phase-connector ${phases[index + 1].active || phases[index + 1].completed ? 'active' : ''}`}>
                <div className="connector-line"></div>
                <div className="connector-arrow">→</div>
              </div>
            )}
          </div>
        ))}
      </div>
      
      <div className="intelligence-progress-track">
        <div 
          className="intelligence-progress-fill"
          style={{ width: `${animatedProgress}%` }}
        >
          <div className="progress-glow"></div>
          <div className="progress-particles">
            {[...Array(5)].map((_, i) => (
              <div key={i} className={`particle particle-${i + 1}`}></div>
            ))}
          </div>
        </div>
        
        {/* Neural network animation overlay */}
        <div className="neural-network-overlay">
          <svg className="neural-svg" viewBox="0 0 100 20">
            {phase === 'memory' && (
              <>
                {/* Animated neurons for memory retrieval */}
                <circle className="neuron" cx="10" cy="10" r="2" />
                <circle className="neuron" cx="25" cy="8" r="2" />
                <circle className="neuron" cx="40" cy="12" r="2" />
                <line className="synapse" x1="10" y1="10" x2="25" y2="8" />
                <line className="synapse" x1="25" y1="8" x2="40" y2="12" />
              </>
            )}
            {phase === 'context' && (
              <>
                {/* Weaving threads for context synthesis */}
                <path className="thread thread-1" d="M 0,10 Q 25,5 50,10 T 100,10" />
                <path className="thread thread-2" d="M 0,8 Q 25,13 50,8 T 100,8" />
                <path className="thread thread-3" d="M 0,12 Q 25,7 50,12 T 100,12" />
              </>
            )}
            {phase === 'classification' && (
              <>
                {/* Lightning bolts for classification */}
                <path className="lightning" d="M 30,5 L 35,10 L 32,10 L 37,15 L 34,10 L 37,10 Z" />
                <path className="lightning" d="M 60,5 L 65,10 L 62,10 L 67,15 L 64,10 L 67,10 Z" />
              </>
            )}
          </svg>
        </div>
      </div>
      
      {phase === 'complete' && (
        <div className="intelligence-complete">
          <span className="complete-icon">✨</span>
          <span className="complete-text">Intelligence gathered - Processing with full context</span>
        </div>
      )}
    </div>
  );
};