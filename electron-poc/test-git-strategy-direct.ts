#!/usr/bin/env ts-node

import Database from 'better-sqlite3';
import fetch from 'node-fetch';
import path from 'path';
import os from 'os';

async function testDirectLLM() {
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
  
  console.log('Testing direct LLM call for git strategy...');
  console.log('Repository:', repoStats);
  console.log('Git Status:', gitStatus);
  
  // Get database path
  const dbPath = path.join(os.homedir(), 'Library', 'Application Support', 'electron-poc', 'hive_unified.db');
  console.log('Database path:', dbPath);
  
  try {
    // Open database
    const db = new Database(dbPath, { readonly: true });
    
    // Get OpenRouter API key from database
    const apiKeyRow = db.prepare(
      "SELECT value FROM configuration WHERE key = 'openrouter_api_key'"
    ).get() as { value: string } | undefined;
    
    if (!apiKeyRow) {
      console.error('No OpenRouter API key found in database');
      process.exit(1);
    }
    
    console.log('Found API key in database (first 10 chars):', apiKeyRow.value.substring(0, 10) + '...');
    
    // Get active profile
    const profileRow = db.prepare(
      "SELECT generator_model FROM consensus_profiles WHERE is_default = 1"
    ).get() as { generator_model: string } | undefined;
    
    let llmModel = 'gpt-4-turbo-preview'; // default
    if (profileRow) {
      llmModel = profileRow.generator_model;
      console.log('Using model from active profile:', llmModel);
    }
    
    db.close();
    
    // Build prompt
    const sizeInMB = parseFloat(repoStats.totalSize) * 1024; // Convert GiB to MB
    const prompt = `Analyze this Git repository and recommend the best push strategy:

Repository: ${sizeInMB}MB, ${repoStats.commitCount} commits
Branch: ${gitStatus.branch} (${gitStatus.hasUpstream ? 'tracked' : 'untracked'})
Status: ${gitStatus.ahead || 0} ahead, ${gitStatus.behind || 0} behind

GitHub limits: 2GB pack size, 2min timeout

Available strategies:
1. Standard Push - Normal push (fails >2GB)
2. Chunked Push - Break into batches (handles any size)
3. Squash Push - Combine commits (reduces size)
4. Force Push - Overwrite remote (dangerous)
5. Fresh Branch - New branch (requires merge)

Recommend ONE strategy with confidence (0-100) and brief reasoning.`;

    console.log('\nSending prompt to OpenRouter...');
    console.log('Prompt:', prompt);
    
    // Call OpenRouter API
    const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${apiKeyRow.value}`,
        'Content-Type': 'application/json',
        'HTTP-Referer': 'https://hivetechs.com',
        'X-Title': 'Hive Consensus'
      },
      body: JSON.stringify({
        model: llmModel,
        messages: [
          {
            role: 'system',
            content: 'You are a Git expert assistant. Provide clear, concise recommendations.'
          },
          {
            role: 'user',
            content: prompt
          }
        ],
        temperature: 0.7,
        max_tokens: 500
      })
    });
    
    const data = await response.json() as any;
    
    if (!response.ok) {
      console.error('OpenRouter API error:', data);
      process.exit(1);
    }
    
    console.log('\nOpenRouter Response:');
    console.log(JSON.stringify(data, null, 2));
    
    if (data.choices?.[0]?.message?.content) {
      console.log('\nLLM Recommendation:');
      console.log(data.choices[0].message.content);
    }
    
  } catch (error) {
    console.error('Error:', error);
  }
  
  process.exit(0);
}

testDirectLLM();