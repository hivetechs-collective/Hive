#!/usr/bin/env node

const sqlite3 = require('sqlite3').verbose();
const path = require('path');
const os = require('os');

// Get the database path
const dbPath = path.join(os.homedir(), 'Library/Application Support/electron-poc/hive_unified.db');

console.log('Database path:', dbPath);

const db = new sqlite3.Database(dbPath, (err) => {
  if (err) {
    console.error('Error opening database:', err);
    return;
  }
  
  console.log('Connected to database');
  
  // Check tables
  db.all("SELECT name FROM sqlite_master WHERE type='table'", (err, tables) => {
    if (err) {
      console.error('Error getting tables:', err);
      return;
    }
    console.log('\nTables in database:', tables.map(t => t.name));
    
    // Get all configuration values
    db.all('SELECT * FROM configuration', (err, configs) => {
      if (err) {
        console.error('Error getting configuration:', err);
        return;
      }
      
      console.log('\nConfiguration values:');
      configs.forEach(config => {
        const value = config.key.includes('key') ? 
          config.value.substring(0, 6) + '...' + config.value.substring(config.value.length - 4) : 
          config.value;
        console.log(`  ${config.key}: ${value}`);
      });
      
      // Check for API keys
      db.get("SELECT value FROM configuration WHERE key = 'openrouter_api_key'", (err, openrouterKey) => {
        db.get("SELECT value FROM configuration WHERE key = 'hive_license_key'", (err2, hiveKey) => {
          
          if (!openrouterKey) {
            console.log('\nAdding test OpenRouter key...');
            db.run("INSERT INTO configuration (key, value) VALUES (?, ?)",
              ['openrouter_api_key', 'sk-or-v1-1234567890abcdefghijklmnopqrstuvwxyz'],
              (err) => {
                if (err) console.error('Error adding OpenRouter key:', err);
                else console.log('Test OpenRouter key added');
              }
            );
          }
          
          if (!hiveKey) {
            console.log('Adding test Hive key...');
            db.run("INSERT INTO configuration (key, value) VALUES (?, ?)",
              ['hive_license_key', 'hive-xxxx-yyyy-zzzz-1234'],
              (err) => {
                if (err) console.error('Error adding Hive key:', err);
                else console.log('Test Hive key added');
                
                // Close database
                db.close((err) => {
                  if (err) console.error('Error closing database:', err);
                  else console.log('\nDatabase test complete!');
                });
              }
            );
          } else {
            // Close database
            db.close((err) => {
              if (err) console.error('Error closing database:', err);
              else console.log('\nDatabase test complete!');
            });
          }
        });
      });
    });
  });
});