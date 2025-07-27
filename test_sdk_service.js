#!/usr/bin/env node

// Test script to verify the SDK service works
import { spawn } from 'child_process';
import { createInterface } from 'readline';

const service = spawn('node', ['src/consensus/claude_sdk_service.js']);

const rl = createInterface({
  input: service.stdout,
  output: process.stdout,
  terminal: false
});

service.stderr.on('data', (data) => {
  console.error('Service stderr:', data.toString());
});

service.on('error', (err) => {
  console.error('Failed to start service:', err);
  process.exit(1);
});

service.on('exit', (code) => {
  console.log('Service exited with code:', code);
  process.exit(code);
});

// Test ping
const pingRequest = {
  jsonrpc: '2.0',
  id: '1',
  method: 'ping',
  params: {}
};

console.log('Sending ping request...');
service.stdin.write(JSON.stringify(pingRequest) + '\n');

rl.on('line', (line) => {
  console.log('Response:', line);
  
  // Test a real query
  const queryRequest = {
    jsonrpc: '2.0',
    id: '2',
    method: 'query',
    params: {
      prompt: 'Hello, Claude!',
      options: {
        planMode: false,
        autoEdit: true,
        executionMode: 'Direct'
      }
    }
  };
  
  console.log('\nSending query request...');
  service.stdin.write(JSON.stringify(queryRequest) + '\n');
});

// Give it some time then exit
setTimeout(() => {
  console.log('\nTest complete.');
  service.kill();
  process.exit(0);
}, 5000);