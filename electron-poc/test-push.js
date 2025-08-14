#!/usr/bin/env node

// Test script to debug push operation

const { GitExecutor } = require('./dist/git-executor.js');
const { GitManagerV2 } = require('./dist/git-manager-v2.js');

async function testPush() {
  console.log('=== Testing Git Push Operation ===\n');
  
  const repoPath = '/Users/veronelazio/Developer/Private/hive';
  const manager = new GitManagerV2(repoPath);
  
  console.log('1. Getting status...');
  const status = await manager.getStatus();
  console.log(`   Branch: ${status.branch}`);
  console.log(`   Ahead: ${status.ahead}`);
  console.log(`   Behind: ${status.behind}`);
  console.log(`   Has upstream: ${status.hasUpstream}\n`);
  
  if (status.ahead > 0) {
    console.log('2. Attempting push...');
    try {
      await manager.push();
      console.log('   Push completed successfully!\n');
      
      console.log('3. Getting status after push...');
      const newStatus = await manager.getStatus();
      console.log(`   Ahead: ${newStatus.ahead}`);
      console.log(`   Behind: ${newStatus.behind}`);
    } catch (error) {
      console.error('   Push failed:', error.message);
      console.error('   Full error:', error);
    }
  } else {
    console.log('2. Nothing to push (ahead = 0)');
  }
}

testPush().catch(console.error);