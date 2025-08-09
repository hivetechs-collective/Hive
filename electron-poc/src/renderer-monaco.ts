/**
 * Hive Consensus - Electron Proof of Concept
 * Day 0: Validate Electron + Monaco + Rust architecture
 */

import './index.css';
import * as monaco from 'monaco-editor';

// Create container for Monaco editor
const container = document.createElement('div');
container.id = 'monaco-container';
container.style.width = '100vw';
container.style.height = '100vh';
container.style.display = 'flex';
container.style.flexDirection = 'column';
document.body.appendChild(container);

// Add header
const header = document.createElement('div');
header.style.padding = '10px';
header.style.backgroundColor = '#1e1e1e';
header.style.color = '#cccccc';
header.style.borderBottom = '1px solid #333';
header.innerHTML = `
  <h2 style="margin: 0; font-size: 16px; font-family: monospace;">
    🐝 Hive Consensus - Day 0 Validation
  </h2>
  <div style="margin-top: 10px;">
    <button id="test-rust" style="margin-right: 10px; padding: 5px 10px;">Test Rust Connection</button>
    <button id="test-consensus" style="padding: 5px 10px;">Test Consensus Engine</button>
    <span id="status" style="margin-left: 20px; color: #4ec9b0;"></span>
  </div>
`;
container.appendChild(header);

// Create editor container
const editorContainer = document.createElement('div');
editorContainer.style.flex = '1';
container.appendChild(editorContainer);

// Initialize Monaco Editor
const editor = monaco.editor.create(editorContainer, {
  value: `// Hive Consensus - Electron + Monaco + Rust Backend
// Day 0 Proof of Concept

console.log("✅ Monaco editor loaded successfully!");

// Test 1: Monaco works in Electron
// Test 2: Connect to Rust backend on :8765
// Test 3: Make real consensus call
// Test 4: Display consensus result

async function testRustConnection() {
  try {
    const response = await fetch('http://localhost:8765/test', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify("Hello from Electron")
    });
    const result = await response.json();
    console.log('Rust backend responded:', result);
    return result;
  } catch (error) {
    console.error('Failed to connect to Rust backend:', error);
    return null;
  }
}

async function testConsensus(query) {
  try {
    const response = await fetch('http://localhost:8765/api/consensus', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({ query })
    });
    const result = await response.json();
    console.log('Consensus result:', result);
    return result;
  } catch (error) {
    console.error('Consensus failed:', error);
    return null;
  }
}

// Ready for testing!
`,
  language: 'javascript',
  theme: 'vs-dark',
  fontSize: 14,
  minimap: { enabled: false },
  automaticLayout: true,
  wordWrap: 'on',
});

// Add button handlers
document.getElementById('test-rust')?.addEventListener('click', async () => {
  const status = document.getElementById('status');
  if (status) {
    status.textContent = 'Connecting to Rust backend...';
    status.style.color = '#ffcc00';
  }
  
  try {
    const response = await fetch('http://localhost:8765/test', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify("Hello from Electron")
    });
    const result = await response.json();
    
    if (status) {
      status.textContent = `✅ Rust responded: ${result}`;
      status.style.color = '#4ec9b0';
    }
    
    // Add result to editor
    const currentValue = editor.getValue();
    editor.setValue(currentValue + `\n\n// Rust backend test successful!\n// Response: ${JSON.stringify(result)}`);
  } catch (error) {
    if (status) {
      status.textContent = '❌ Failed to connect to Rust backend (is it running on :8765?)';
      status.style.color = '#ff6b6b';
    }
  }
});

document.getElementById('test-consensus')?.addEventListener('click', async () => {
  const status = document.getElementById('status');
  if (status) {
    status.textContent = 'Running consensus engine...';
    status.style.color = '#ffcc00';
  }
  
  try {
    const response = await fetch('http://localhost:8765/api/consensus', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({ query: "What is the capital of France?" })
    });
    const result = await response.json();
    
    if (status) {
      status.textContent = '✅ Consensus completed!';
      status.style.color = '#4ec9b0';
    }
    
    // Add result to editor
    const currentValue = editor.getValue();
    editor.setValue(currentValue + `\n\n// Consensus engine test successful!\n// Result: ${JSON.stringify(result, null, 2)}`);
  } catch (error) {
    if (status) {
      status.textContent = '❌ Consensus failed (is backend running?)';
      status.style.color = '#ff6b6b';
    }
  }
});

// Log success
console.log('✅ Day 0 Step 1: Electron + Monaco setup complete!');