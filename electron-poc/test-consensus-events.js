const { ipcRenderer } = require('electron');

// Test consensus with event monitoring
async function testConsensusEvents() {
  console.log('Starting consensus event test...');
  
  // Listen for WebSocket messages (consensus events)
  ipcRenderer.on('websocket-message', (event, message) => {
    try {
      const data = JSON.parse(message);
      console.log('📨 Event received:', {
        type: data.type,
        stage: data.stage,
        model: data.model,
        message: data.message,
        tokens: data.tokens
      });
    } catch (e) {
      console.log('📨 Raw message:', message);
    }
  });
  
  console.log('Sending consensus request...');
  
  try {
    // Trigger consensus
    const result = await ipcRenderer.invoke('backend-consensus', 'What is JavaScript?');
    console.log('✅ Consensus result:', result);
  } catch (error) {
    console.error('❌ Error:', error);
  }
}

// Run test from renderer console
if (typeof window !== 'undefined') {
  window.testConsensusEvents = testConsensusEvents;
  console.log('Test ready! Run: testConsensusEvents()');
}