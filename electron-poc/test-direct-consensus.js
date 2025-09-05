// Test DirectConsensusEngine directly
const { app } = require('electron');
const path = require('path');
const sqlite3 = require('sqlite3').verbose();

// Initialize database
const dbPath = path.join(require('os').homedir(), '.hive', 'hive-ai.db');
const db = new sqlite3.Database(dbPath);

// Import DirectConsensusEngine
const { DirectConsensusEngine } = require('./src/consensus/DirectConsensusEngine');

async function testConsensus() {
  console.log('Testing DirectConsensusEngine...');
  console.log('Database path:', dbPath);
  
  try {
    // Create engine
    const engine = new DirectConsensusEngine(db);
    
    // Listen for events
    engine.on('stage-update', (data) => {
      console.log('📊 Stage Update:', data);
    });
    
    engine.on('stage-complete', (data) => {
      console.log('✅ Stage Complete:', data);
    });
    
    engine.on('error', (error) => {
      console.error('❌ Error:', error);
    });
    
    // Test request
    const request = {
      query: 'What is JavaScript?',
      context: 'Test context',
      memories: [],
      requestId: 'test-' + Date.now()
    };
    
    console.log('Sending request:', request);
    
    // Process consensus
    const result = await engine.processConsensus(request);
    
    console.log('✅ Result:', result);
  } catch (error) {
    console.error('❌ Test failed:', error);
    console.error('Stack:', error.stack);
  }
  
  // Close database
  db.close();
  process.exit(0);
}

// Run test
testConsensus();