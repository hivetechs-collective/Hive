const fetch = require('node-fetch');
const sqlite3 = require('sqlite3').verbose();
const path = require('path');
const os = require('os');

async function testOpenRouter() {
  console.log('Starting OpenRouter API test...');
  
  // Get API key from database
  const dbPath = path.join(os.homedir(), '.hive', 'hive-ai.db');
  const db = new sqlite3.Database(dbPath);
  
  const apiKey = await new Promise((resolve, reject) => {
    db.get(
      "SELECT value FROM configurations WHERE key = 'openrouter_api_key'",
      (err, row) => {
        if (err) reject(err);
        else resolve(row?.value || null);
      }
    );
  });
  
  db.close();
  
  if (!apiKey) {
    console.error('No OpenRouter API key found in database');
    return;
  }
  
  console.log('API key found, making test request...');
  
  const requestBody = {
    model: 'meta-llama/llama-3.2-3b-instruct:free',
    messages: [
      {
        role: 'user',
        content: 'Say hello in one word'
      }
    ],
    temperature: 0.7,
    max_tokens: 10,
    stream: false
  };
  
  console.log('Request body:', JSON.stringify(requestBody, null, 2));
  
  try {
    console.log('Sending request to OpenRouter...');
    const startTime = Date.now();
    
    const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${apiKey}`,
        'X-Title': 'Hive Consensus Test',
        'HTTP-Referer': 'https://hivetechs.ai'
      },
      body: JSON.stringify(requestBody),
      timeout: 10000 // 10 second timeout
    });
    
    const elapsed = Date.now() - startTime;
    console.log(`Response received in ${elapsed}ms`);
    console.log('Response status:', response.status);
    console.log('Response headers:', response.headers.raw());
    
    const data = await response.json();
    console.log('Response data:', JSON.stringify(data, null, 2));
    
    if (data.choices && data.choices[0]) {
      console.log('\n✅ SUCCESS! Response:', data.choices[0].message.content);
    }
  } catch (error) {
    console.error('Error calling OpenRouter:', error);
    console.error('Error stack:', error.stack);
  }
}

testOpenRouter().catch(console.error);