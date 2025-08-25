#!/usr/bin/env node

const { GitStrategyService } = require('./dist/git-strategy-service/index.js');

async function test() {
  const service = new GitStrategyService();
  
  // Test repository stats
  const repoStats = {
    totalSize: '11.07 GiB',
    fileCount: 45921,
    commitCount: 5233,
    largestFile: '524.8 MiB'
  };
  
  // Test git status
  const gitStatus = {
    branch: 'fix/cpu-performance-improvements', 
    hasUpstream: true,
    ahead: 0,
    behind: 0
  };
  
  console.log('Testing GitStrategyService...');
  console.log('Repository:', repoStats);
  console.log('Git Status:', gitStatus);
  
  try {
    const result = await service.handleRequest({ repoStats, gitStatus });
    console.log('\nResult:', JSON.stringify(result, null, 2));
  } catch (error) {
    console.error('Error:', error);
  }
  
  process.exit(0);
}

test();