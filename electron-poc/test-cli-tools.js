#!/usr/bin/env node

/**
 * Test script for CLI Tools functionality
 */

const { exec } = require('child_process');
const util = require('util');
const execPromise = util.promisify(exec);

async function testCliToolDetection() {
  console.log('Testing CLI Tool Detection...\n');
  
  const tools = [
    { id: 'claude-code', command: 'claude --version' },
    { id: 'gemini-cli', command: 'gemini --version' },
    { id: 'qwen-code', command: 'qwen --version' },
    { id: 'aider', command: 'aider --version' }
  ];
  
  for (const tool of tools) {
    try {
      const { stdout } = await execPromise(tool.command);
      console.log(`✅ ${tool.id}: INSTALLED`);
      console.log(`   Version: ${stdout.trim()}`);
    } catch (error) {
      console.log(`❌ ${tool.id}: NOT INSTALLED`);
    }
  }
}

async function testMemoryServiceAPI() {
  console.log('\n\nTesting Memory Service API...\n');
  
  try {
    // Test health endpoint
    const healthResponse = await fetch('http://localhost:3457/health');
    const health = await healthResponse.json();
    console.log('✅ Memory Service Health Check:', health);
    
    // Test stats endpoint
    const statsResponse = await fetch('http://localhost:3457/api/v1/memory/stats');
    const stats = await statsResponse.json();
    console.log('✅ Memory Service Stats:', stats);
    
    // Test tools endpoint
    const toolsResponse = await fetch('http://localhost:3457/api/v1/memory/tools');
    const tools = await toolsResponse.json();
    console.log('✅ Connected Tools:', tools);
    
  } catch (error) {
    console.log('❌ Memory Service API Error:', error.message);
  }
}

async function testMemoryServiceRegistration() {
  console.log('\n\nTesting Memory Service Registration...\n');
  
  try {
    // Register a test tool
    const registerResponse = await fetch('http://localhost:3457/api/v1/memory/register', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        toolName: 'test-cli-tool'
      })
    });
    
    const registration = await registerResponse.json();
    console.log('✅ Tool Registration:', registration);
    
    // Test query with the token
    if (registration.token) {
      const queryResponse = await fetch('http://localhost:3457/api/v1/memory/query', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${registration.token}`,
          'X-Client-Name': 'test-cli-tool'
        },
        body: JSON.stringify({
          client: 'test-cli-tool',
          context: {
            file: 'test.js',
            line: 1
          },
          query: 'test query',
          options: {
            limit: 5
          }
        })
      });
      
      const queryResult = await queryResponse.json();
      console.log('✅ Memory Query Result:', queryResult);
    }
    
  } catch (error) {
    console.log('❌ Registration Error:', error.message);
  }
}

async function main() {
  console.log('========================================');
  console.log('     CLI Tools Integration Test');
  console.log('========================================\n');
  
  await testCliToolDetection();
  await testMemoryServiceAPI();
  await testMemoryServiceRegistration();
  
  console.log('\n========================================');
  console.log('     Test Complete');
  console.log('========================================');
}

main().catch(console.error);