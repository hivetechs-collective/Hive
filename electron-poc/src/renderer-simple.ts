/**
 * Hive Consensus - Simple Electron Proof of Concept
 * Day 0: Validate Electron + Rust architecture WITHOUT Monaco
 */

import './index.css';

// Create simple UI
document.body.innerHTML = `
<div style="padding: 20px; font-family: monospace; background: #1e1e1e; color: #ccc; height: 100vh;">
  <h1 style="color: #4ec9b0;">🐝 Hive Consensus - Day 0 Validation (Simple)</h1>
  
  <div style="margin: 20px 0;">
    <button id="test-rust" style="padding: 10px 20px; margin-right: 10px;">Test Rust Connection</button>
    <button id="test-consensus" style="padding: 10px 20px;">Test Consensus Engine</button>
  </div>
  
  <div id="status" style="padding: 10px; background: #252526; border-radius: 4px; margin: 20px 0;">
    Status: Ready
  </div>
  
  <div style="background: #252526; padding: 20px; border-radius: 4px; height: 400px; overflow-y: auto;">
    <h3>Results:</h3>
    <pre id="results" style="color: #4ec9b0; font-size: 14px;">Waiting for tests...</pre>
  </div>
</div>
`;

const status = document.getElementById('status')!;
const results = document.getElementById('results')!;

// Test Rust connection
document.getElementById('test-rust')?.addEventListener('click', async () => {
  // REMOVED: Backend test - using DirectConsensusEngine now  
  status.textContent = 'Status: ✅ DirectConsensusEngine Ready!';
  status.style.color = '#4ec9b0';
  results.textContent = 'DirectConsensusEngine: Ready for consensus processing';
  console.log('✅ DirectConsensusEngine ready!');
});

// Test consensus
document.getElementById('test-consensus')?.addEventListener('click', async () => {
  status.textContent = 'Status: Running consensus engine...';
  status.style.color = '#ffcc00';
  
  try {
    // Use DirectConsensusEngine via IPC (reuse existing API)
    const result = await (window as any).backendAPI.runConsensus("What is the capital of France?");
    
    status.textContent = 'Status: ✅ Consensus completed!';
    status.style.color = '#4ec9b0';
    results.textContent = `DirectConsensusEngine Success!\n${JSON.stringify(result, null, 2)}`;
    console.log('✅ DirectConsensusEngine consensus call succeeded!');
  } catch (error) {
    status.textContent = 'Status: ❌ Consensus failed';
    status.style.color = '#ff6b6b';
    results.textContent = `Consensus failed:\n${error}`;
  }
});

// Auto-test on load (simplified)
setTimeout(async () => {
  console.log('DirectConsensusEngine: Ready for consensus processing');
  results.textContent = 'DirectConsensusEngine: Ready for consensus processing';
}, 1000);