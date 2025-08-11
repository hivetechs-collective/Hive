#!/usr/bin/env node

const WebSocket = require('ws');

// Connect to the backend WebSocket
const ws = new WebSocket('ws://localhost:8765/ws');

ws.on('open', () => {
    console.log('Connected to backend WebSocket');
    
    // Test 1: Simple math - should use Direct mode
    console.log('\n📊 Test 1: Simple Math Question');
    console.log('Sending: "what is 2 + 2?"');
    ws.send(JSON.stringify({
        type: 'start_consensus',
        query: 'what is 2 + 2?'
    }));
});

ws.on('message', (data) => {
    try {
        const msg = JSON.parse(data.toString());
        
        if (msg.type === 'a_i_helper_decision') {
            console.log(`\n🎯 AI Helper Routing Decision:`);
            console.log(`   Mode: ${msg.direct_mode ? 'DIRECT (Fast)' : 'CONSENSUS (Full Pipeline)'}`);
            console.log(`   Reason: ${msg.reason}`);
        } else if (msg.type === 'stage_start') {
            console.log(`\n🎯 Stage Started: ${msg.stage}`);
            console.log(`   Model: ${msg.model}`);
        } else if (msg.type === 'token') {
            process.stdout.write(msg.content);
        } else if (msg.type === 'stage_complete') {
            console.log(`\n✅ Stage Complete: ${msg.stage}`);
            console.log(`   Tokens: ${msg.tokens}, Cost: $${msg.cost}`);
        } else if (msg.type === 'consensus_complete') {
            console.log('\n🏁 Consensus Complete!');
            console.log(`   Total Tokens: ${msg.total_tokens}`);
            console.log(`   Total Cost: $${msg.total_cost}`);
            console.log(`   Stages Used: ${msg.stages_used}`);
            
            // After first test, run second test
            if (!ws.secondTestDone) {
                ws.secondTestDone = true;
                setTimeout(() => {
                    console.log('\n\n📊 Test 2: Complex Question');
                    console.log('Sending: "design a full electron app that can use multiple llms"');
                    ws.send(JSON.stringify({
                        type: 'start_consensus',
                        query: 'design a full electron app that can use multiple llms'
                    }));
                }, 2000);
            } else {
                // Both tests done, close connection
                console.log('\n\n✅ All tests complete!');
                ws.close();
                process.exit(0);
            }
        } else if (msg.type === 'error') {
            console.error('\n❌ Error:', msg.message);
            if (msg.details) {
                console.error('Details:', msg.details);
            }
        }
    } catch (e) {
        console.error('Failed to parse message:', e);
    }
});

ws.on('error', (err) => {
    console.error('WebSocket error:', err);
});

ws.on('close', () => {
    console.log('\nWebSocket connection closed');
});