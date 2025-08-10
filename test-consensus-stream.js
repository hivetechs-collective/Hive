const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:8765/ws');

ws.on('open', () => {
    console.log('✅ WebSocket connected');
    
    // Start consensus
    const message = {
        type: 'start_consensus',
        query: 'What is the capital of France?',
        profile: 'Free Also'
    };
    
    console.log('Sending:', message);
    ws.send(JSON.stringify(message));
});

ws.on('message', (data) => {
    const msg = JSON.parse(data.toString());
    console.log('\n📨 Message received:');
    console.log('  Type:', msg.type);
    
    if (msg.type === 'stream_chunk') {
        console.log('  Stage:', msg.stage);
        console.log('  Chunk:', msg.chunk.substring(0, 50) + '...');
    } else if (msg.type === 'consensus_complete') {
        console.log('  Result:', msg.result);
        console.log('  Tokens:', msg.total_tokens);
        console.log('  Cost: $', msg.total_cost);
        ws.close();
    } else {
        console.log('  Data:', JSON.stringify(msg, null, 2));
    }
});

ws.on('error', (err) => {
    console.error('❌ WebSocket error:', err);
});

ws.on('close', () => {
    console.log('🔌 WebSocket closed');
    process.exit(0);
});

// Timeout after 30 seconds
setTimeout(() => {
    console.log('⏰ Timeout - closing connection');
    ws.close();
    process.exit(1);
}, 30000);