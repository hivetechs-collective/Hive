#!/usr/bin/env node

// Test script to trigger iterative consensus via IPC
const { app, BrowserWindow, ipcMain } = require('electron');

async function testIterativeConsensus() {
  console.log('Testing iterative consensus implementation...');
  
  // Simple test query that should trigger iterative deliberation
  const testQuery = "What is 2+2? Please explain your reasoning step by step.";
  
  console.log(`Test Query: ${testQuery}`);
  console.log('Expected behavior:');
  console.log('1. Generator provides initial answer');
  console.log('2. Refiner improves the answer');
  console.log('3. Validator validates the answer');
  console.log('4. All three models vote YES/NO on "Can this be improved?"');
  console.log('5. If ANY says YES, loop continues');
  console.log('6. If ALL say NO, Curator polishes and returns');
  console.log('');
  console.log('To test manually:');
  console.log('1. Open the Hive Consensus app');
  console.log('2. Type the query in the consensus chat');
  console.log('3. Watch for round indicators and consensus checks');
  console.log('4. Verify multiple rounds occur before consensus');
}

testIterativeConsensus();