#!/usr/bin/env node

// Test slash commands through the simple integration

const { spawn } = require('child_process');

async function testCommand(command, args = []) {
    console.log(`\n🧪 Testing: claude ${command} ${args.join(' ')}`);
    
    return new Promise((resolve) => {
        const child = spawn('claude', [command, ...args]);
        let output = '';
        let error = '';
        
        child.stdout.on('data', (data) => {
            output += data.toString();
        });
        
        child.stderr.on('data', (data) => {
            error += data.toString();
        });
        
        child.on('exit', (code) => {
            console.log(`Exit code: ${code}`);
            if (output) console.log(`Output:\n${output}`);
            if (error) console.log(`Error:\n${error}`);
            resolve({ code, output, error });
        });
    });
}

async function runTests() {
    console.log('🚀 Testing Claude Code slash command integration\n');
    
    // Test basic commands
    await testCommand('help');
    await testCommand('--version');
    
    // Test slash command style (what the integration sends)
    console.log('\n📝 Testing how slash commands would be processed:');
    console.log('When user types: /help');
    console.log('Integration sends: claude help');
    
    console.log('\nWhen user types: /login');
    console.log('Integration sends: claude login');
    
    console.log('\nWhen user types: /context');
    console.log('Integration sends: claude context');
}

runTests().catch(console.error);