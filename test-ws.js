const WebSocket = require('ws');

console.log('Testing WebSocket connection to backend...');

// Test the simple echo endpoint first
const testWs = new WebSocket('ws://localhost:8765/ws-test');

testWs.on('open', () => {
  console.log('✅ Test WebSocket connected!');
  testWs.send('Hello from test client');
});

testWs.on('message', (data) => {
  console.log('Received:', data.toString());
  testWs.close();
  
  // Now test the main consensus WebSocket
  console.log('\nTesting main consensus WebSocket...');
  const mainWs = new WebSocket('ws://localhost:8765/ws');
  
  mainWs.on('open', () => {
    console.log('✅ Main WebSocket connected!');
    mainWs.send(JSON.stringify({
      type: 'start_consensus',
      query: 'What is 1+1?',
      profile: 'Free Also'
    }));
  });
  
  mainWs.on('message', (data) => {
    const msg = JSON.parse(data.toString());
    console.log('Message:', msg);
  });
  
  mainWs.on('error', (err) => {
    console.error('Main WebSocket error:', err);
  });
  
  mainWs.on('close', () => {
    console.log('Main WebSocket closed');
    process.exit(0);
  });
});

testWs.on('error', (err) => {
  console.error('Test WebSocket error:', err);
  process.exit(1);
});

setTimeout(() => {
  console.log('Timeout - exiting');
  process.exit(1);
}, 10000);