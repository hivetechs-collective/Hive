#!/usr/bin/env node

/**
 * Migration runner for Memory-Context enhancement
 * Run this to update the database schema for the complete memory loop
 */

import * as sqlite3 from 'sqlite3';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';

const dbPath = path.join(os.homedir(), '.hive', 'hive-ai.db');
const migrationPath = path.join(__dirname, 'memory-context-enhancement.sql');

console.log('🔄 Running Memory-Context Enhancement Migration...');
console.log(`📁 Database: ${dbPath}`);

// Check if database exists
if (!fs.existsSync(dbPath)) {
  console.error('❌ Database not found at:', dbPath);
  process.exit(1);
}

// Check if migration file exists
if (!fs.existsSync(migrationPath)) {
  console.error('❌ Migration file not found at:', migrationPath);
  process.exit(1);
}

// Read migration SQL
const migrationSQL = fs.readFileSync(migrationPath, 'utf-8');

// Connect to database
const db = new sqlite3.Database(dbPath, (err) => {
  if (err) {
    console.error('❌ Failed to connect to database:', err);
    process.exit(1);
  }
  console.log('✅ Connected to database');
});

// Run migration
db.serialize(() => {
  // Split migration into individual statements
  const statements = migrationSQL
    .split(';')
    .filter(stmt => stmt.trim().length > 0)
    .map(stmt => stmt.trim() + ';');
  
  let completed = 0;
  const total = statements.length;
  
  statements.forEach((statement, index) => {
    // Skip comments
    if (statement.startsWith('--')) {
      completed++;
      return;
    }
    
    // Extract operation type for logging
    const operation = statement.match(/^(CREATE|ALTER|INSERT|DROP|UPDATE)/i)?.[1] || 'EXECUTE';
    const target = statement.match(/(TABLE|INDEX|VIEW|TRIGGER)\s+(?:IF\s+NOT\s+EXISTS\s+)?(\w+)/i)?.[2] || '';
    
    db.run(statement, (err) => {
      if (err) {
        // Check if it's a column already exists error (expected for ALTER TABLE)
        if (err.message.includes('duplicate column name')) {
          console.log(`⏭️  Skipping ${operation} ${target} - column already exists`);
        } else {
          console.error(`❌ Failed to ${operation} ${target}:`, err.message);
        }
      } else {
        console.log(`✅ ${operation} ${target || 'statement'} completed`);
      }
      
      completed++;
      if (completed === total) {
        // Verify the migration
        verifyMigration(db);
      }
    });
  });
});

function verifyMigration(db: sqlite3.Database) {
  console.log('\n🔍 Verifying migration...');
  
  // Check if messages table has new columns
  db.all("PRAGMA table_info(messages)", (err, columns: any[]) => {
    if (err) {
      console.error('❌ Failed to verify messages table:', err);
      return;
    }
    
    const columnNames = columns.map(col => col.name);
    const requiredColumns = ['tokens_used', 'cost', 'consensus_path', 'parent_message_id'];
    const hasAllColumns = requiredColumns.every(col => columnNames.includes(col));
    
    if (hasAllColumns) {
      console.log('✅ Messages table has all required columns');
    } else {
      console.log('⚠️  Some columns may be missing from messages table');
    }
  });
  
  // Check if memory_context_logs table exists
  db.get("SELECT name FROM sqlite_master WHERE type='table' AND name='memory_context_logs'", (err, row) => {
    if (err) {
      console.error('❌ Failed to verify memory_context_logs table:', err);
      return;
    }
    
    if (row) {
      console.log('✅ memory_context_logs table exists');
    } else {
      console.log('❌ memory_context_logs table not created');
    }
  });
  
  // Check indexes
  db.all("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_messages_%'", (err, indexes: any[]) => {
    if (err) {
      console.error('❌ Failed to verify indexes:', err);
      return;
    }
    
    console.log(`✅ Created ${indexes.length} indexes for optimized memory retrieval`);
    
    // Close database
    db.close((err) => {
      if (err) {
        console.error('❌ Error closing database:', err);
      } else {
        console.log('\n✨ Migration completed successfully!');
        console.log('🔄 The Memory-Context-Consensus loop is now ready');
      }
    });
  });
}

// Handle errors
process.on('unhandledRejection', (reason, promise) => {
  console.error('❌ Unhandled Rejection:', reason);
  process.exit(1);
});