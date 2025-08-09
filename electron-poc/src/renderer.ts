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
  console.log('🔗 Test Rust Connection button clicked!');
  status.textContent = 'Status: Connecting to Rust backend via IPC...';
  status.style.color = '#ffcc00';
  
  try {
    console.log('📡 Making IPC request to backend');
    const result = await (window as any).backendAPI.testConnection();
    console.log('📡 IPC Response:', result);
    
    status.textContent = 'Status: ✅ Connected to Rust backend via IPC!';
    status.style.color = '#4ec9b0';
    
    results.textContent = `Test Connection Success via IPC!\n${JSON.stringify(result, null, 2)}`;
    
    console.log('✅ Day 0 Step 3: Electron connected to Rust backend via IPC!');
  } catch (error) {
    console.error('❌ IPC Connection error:', error);
    status.textContent = 'Status: ❌ Failed to connect to Rust backend';
    status.style.color = '#ff6b6b';
    results.textContent = `Connection failed:\n${error}`;
  }
});

// Test consensus
document.getElementById('test-consensus')?.addEventListener('click', async () => {
  console.log('🧠 Test Consensus button clicked!');
  status.textContent = 'Status: Running consensus engine via IPC...';
  status.style.color = '#ffcc00';
  
  try {
    console.log('📡 Making IPC consensus request');
    const result = await (window as any).backendAPI.runConsensus("What is the capital of France?");
    console.log('📡 Consensus IPC Response:', result);
    
    status.textContent = 'Status: ✅ Consensus completed via IPC!';
    status.style.color = '#4ec9b0';
    
    results.textContent = `Consensus Engine Success via IPC!\n${JSON.stringify(result, null, 2)}`;
    
    console.log('✅ Day 0 Step 4: Real consensus call succeeded via IPC!');
  } catch (error) {
    console.error('❌ Consensus IPC error:', error);
    status.textContent = 'Status: ❌ Consensus failed';
    status.style.color = '#ff6b6b';
    results.textContent = `Consensus failed:\n${error}`;
  }
});

// Auto-test on load
setTimeout(async () => {
  console.log('🔄 Auto-testing Rust connection via IPC...');
  
  try {
    console.log('🌐 Testing if backendAPI is available...');
    if (!(window as any).backendAPI) {
      throw new Error('backendAPI not available');
    }
    
    console.log('📡 Making IPC health check request');
    const health = await (window as any).backendAPI.healthCheck();
    console.log('✅ Backend health check SUCCESS via IPC:', health);
    results.textContent = `Backend is healthy via IPC!\n${JSON.stringify(health, null, 2)}`;
  } catch (error) {
    console.error('❌ Backend not reachable via IPC:', error);
    results.textContent = `Backend not reachable via IPC. Check console for details.\n${error}`;
  }
}, 1000);