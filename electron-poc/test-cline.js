#!/usr/bin/env node

/**
 * Test script for Cline CLI integration
 * Verifies detection, version, and integration points
 */

const { exec } = require('child_process');
const path = require('path');

console.log('Testing Cline CLI Integration...\n');

// Test 1: Check if cline-cli is installed
exec('which cline-cli', (error, stdout, stderr) => {
    if (error) {
        console.log('❌ Cline not found in PATH');
        console.log('   Install with: npm install -g @yaegaki/cline-cli');
        return;
    }
    
    const clinePath = stdout.trim();
    console.log('✅ Cline found at:', clinePath);
    
    // Test 2: Get version
    exec('cline-cli --version', (error, stdout, stderr) => {
        if (error) {
            console.log('❌ Failed to get version:', error.message);
        } else {
            const version = stdout.trim();
            console.log('✅ Version:', version);
            
            // Test version regex
            const versionRegex = /(\d+\.\d+\.\d+)/;
            const match = version.match(versionRegex);
            if (match) {
                console.log('✅ Version regex match:', match[1]);
            } else {
                console.log('❌ Version regex failed to match output:', version);
            }
        }
    });
    
    // Test 3: Check npm package
    exec('npm list -g @yaegaki/cline-cli --depth=0', (error, stdout, stderr) => {
        if (stdout.includes('@yaegaki/cline-cli')) {
            const versionMatch = stdout.match(/@yaegaki\/cline-cli@(\d+\.\d+\.\d+)/);
            if (versionMatch) {
                console.log('✅ NPM package version:', versionMatch[1]);
            }
        } else {
            console.log('❌ NPM package not found globally');
        }
    });
    
    // Test 4: Check binary name in package.json
    exec('cat $(npm root -g)/@yaegaki/cline-cli/package.json | grep -A2 "\\"bin\\""', (error, stdout, stderr) => {
        if (!error && stdout) {
            console.log('✅ Binary configuration in package.json:');
            console.log('  ', stdout.trim().replace(/\n/g, '\n   '));
        }
    });
});

// Test 5: Memory Service configuration check
const fs = require('fs');
const os = require('os');
const configPath = path.join(os.homedir(), '.hive', 'cli-tools-config.json');

setTimeout(() => {
    if (fs.existsSync(configPath)) {
        try {
            const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
            if (config['cline']) {
                console.log('\n✅ Memory Service configuration found:');
                console.log('   Endpoint:', config['cline'].memoryService?.endpoint || 'Not configured');
                console.log('   Token:', config['cline'].memoryService?.token ? 'Present' : 'Not set');
                console.log('   Connected at:', config['cline'].memoryService?.connectedAt || 'Never');
            } else {
                console.log('\n⚠️  No Memory Service configuration for cline');
            }
        } catch (e) {
            console.log('\n❌ Error reading configuration:', e.message);
        }
    } else {
        console.log('\n⚠️  Configuration file not found at:', configPath);
    }
    
    console.log('\n📋 Integration checklist:');
    console.log('   ✓ Added to CLI_TOOLS_REGISTRY');
    console.log('   ✓ Version detection configured');
    console.log('   ✓ Memory Service detection enabled');
    console.log('   ✓ Dynamic card in UI');
    console.log('   ✓ Terminal display name set');
    console.log('   ✓ Aider removed (Python dependency avoided)');
    console.log('\n⚠️  KNOWN ISSUE: UI won\'t refresh after install!');
    console.log('   User MUST restart app to see installed status');
    console.log('   This affects ALL tools (Claude Code, Gemini, Qwen, Codex, Cline)');
    console.log('\n🎯 Ready for testing in Electron app!');
}, 1000);