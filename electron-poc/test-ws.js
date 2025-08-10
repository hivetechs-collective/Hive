#\!/usr/bin/env node

const WebSocket = require('ws');

console.log('Connecting to WebSocket...');
const ws = new WebSocket('ws://localhost:8765/ws');

ws.on('open', () => {
    console.log('✅ WebSocket connected');
    
    // Send a consensus request
    const request = {
        type: 'start_consensus',
        query: 'What is the capital of France?',
        profile: 'Free Also'
    };
    
    console.log('📤 Sending request:', request);
    ws.send(JSON.stringify(request));
});

ws.on('message', (data) => {
    try {
        const msg = JSON.parse(data.toString());
        console.log('📥 Received:', msg);
        
        // Log specific message types with more detail
        if (msg.type === 'profile_loaded') {
            console.log('   ✨ Profile loaded:', msg.name, 'Models:', msg.models);
        } else if (msg.type === 'stage_started') {
            console.log('   🚀 Stage started:', msg.stage, 'Model:', msg.model);
        } else if (msg.type === 'stream_chunk') {
            console.log('   📝 Chunk:', msg.chunk.substring(0, 50) + '...');
        } else if (msg.type === 'consensus_complete') {
            console.log('   ✅ COMPLETE\! Result:', msg.result);
            console.log('   📊 Tokens:', msg.total_tokens, 'Cost:', msg.total_cost);
            ws.close();
        }
    } catch (e) {
        console.log('📥 Raw message:', data.toString());
    }
});

ws.on('error', (err) => {
    console.error('❌ WebSocket error:', err);
});

ws.on('close', () => {
    console.log('🔌 WebSocket disconnected');
    process.exit(0);
});

// Timeout after 30 seconds
setTimeout(() => {
    console.log('⏱️ Timeout - closing connection');
    ws.close();
    process.exit(1);
}, 30000);
EOF < /dev/null