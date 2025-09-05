#!/usr/bin/env node
// Hive AI NPM Package Uninstaller
// Removes the Rust binary and cleans up shell completions

const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

const BINARY_NAME = process.platform === 'win32' ? 'hive.exe' : 'hive';
const platform = os.platform();

console.log('🗑️  Uninstalling Hive AI...');

// Try to run cleanup via the binary first
try {
    execSync(`${BINARY_NAME} install uninstall --keep-config`, { stdio: 'pipe' });
    console.log('✅ Hive AI uninstalled via binary');
    return;
} catch (error) {
    // Binary might not be accessible, continue with manual cleanup
    console.log('📦 Binary not accessible, performing manual cleanup...');
}

try {
    if (platform === 'win32') {
        uninstallWindows();
    } else {
        uninstallUnix();
    }
    
    console.log('✅ Hive AI uninstalled successfully!');
    console.log('   Configuration and cache preserved in ~/.hive/');
    console.log('   To remove all data: rm -rf ~/.hive/');
} catch (error) {
    console.error('❌ Uninstallation failed:', error.message);
    console.log('\n🔧 Manual cleanup may be required:');
    console.log('1. Remove binary from your PATH directories');
    console.log('2. Remove shell completions');
    console.log('3. Optionally remove ~/.hive/ directory');
    process.exit(1);
}

function uninstallWindows() {
    console.log('📦 Uninstalling from Windows...');
    
    const possiblePaths = [
        path.join(process.env.ProgramFiles || 'C:\\Program Files', 'Hive', BINARY_NAME),
        path.join(os.homedir(), 'AppData', 'Local', 'Hive', BINARY_NAME)
    ];
    
    let found = false;
    
    for (const binaryPath of possiblePaths) {
        if (fs.existsSync(binaryPath)) {
            try {
                fs.unlinkSync(binaryPath);
                console.log(`✅ Removed: ${binaryPath}`);
                found = true;
                
                // Try to remove directory if empty
                const dir = path.dirname(binaryPath);
                try {
                    fs.rmdirSync(dir);
                    console.log(`✅ Removed directory: ${dir}`);
                } catch (error) {
                    // Directory not empty or protected
                }
            } catch (error) {
                console.log(`⚠️  Could not remove: ${binaryPath}`);
            }
        }
    }
    
    if (!found) {
        console.log('ℹ️  Binary not found in standard locations');
    }
    
    // Clean up PATH (best effort)
    try {
        const programFiles = process.env.ProgramFiles || 'C:\\Program Files';
        const installDir = path.join(programFiles, 'Hive');
        
        // This might not work without admin privileges
        execSync(`setx PATH "%PATH:${installDir};=%"`, { stdio: 'pipe' });
        console.log('✅ Removed from system PATH');
    } catch (error) {
        console.log('⚠️  Could not remove from PATH automatically');
    }
}

function uninstallUnix() {
    console.log(`📦 Uninstalling from ${platform}...`);
    
    const possiblePaths = [
        `/usr/local/bin/${BINARY_NAME}`,
        `/usr/bin/${BINARY_NAME}`,
        path.join(os.homedir(), '.local', 'bin', BINARY_NAME),
        path.join(os.homedir(), 'bin', BINARY_NAME)
    ];
    
    let found = false;
    
    for (const binaryPath of possiblePaths) {
        if (fs.existsSync(binaryPath)) {
            try {
                fs.unlinkSync(binaryPath);
                console.log(`✅ Removed: ${binaryPath}`);
                found = true;
            } catch (error) {
                // Try with sudo for system directories
                if (binaryPath.startsWith('/usr/')) {
                    try {
                        execSync(`sudo rm "${binaryPath}"`, { stdio: 'inherit' });
                        console.log(`✅ Removed with sudo: ${binaryPath}`);
                        found = true;
                    } catch (sudoError) {
                        console.log(`⚠️  Could not remove: ${binaryPath}`);
                    }
                } else {
                    console.log(`⚠️  Could not remove: ${binaryPath}`);
                }
            }
        }
    }
    
    if (!found) {
        console.log('ℹ️  Binary not found in standard locations');
    }
    
    // Clean up shell completions
    cleanupCompletions();
}

function cleanupCompletions() {
    console.log('🧹 Cleaning up shell completions...');
    
    const completionPaths = [
        // Bash
        '/usr/local/share/bash-completion/completions/hive',
        '/usr/share/bash-completion/completions/hive',
        '/etc/bash_completion.d/hive',
        path.join(os.homedir(), '.bash_completion.d', 'hive'),
        
        // Zsh
        '/usr/local/share/zsh/site-functions/_hive',
        '/usr/share/zsh/site-functions/_hive',
        '/usr/share/zsh/vendor-completions/_hive',
        path.join(os.homedir(), '.zsh', 'completions', '_hive'),
        
        // Fish
        '/usr/local/share/fish/completions/hive.fish',
        '/usr/share/fish/completions/hive.fish',
        path.join(os.homedir(), '.config', 'fish', 'completions', 'hive.fish'),
    ];
    
    let cleanedCount = 0;
    
    for (const completionPath of completionPaths) {
        if (fs.existsSync(completionPath)) {
            try {
                fs.unlinkSync(completionPath);
                console.log(`✅ Removed completion: ${path.basename(completionPath)}`);
                cleanedCount++;
            } catch (error) {
                // Try with sudo for system paths
                if (completionPath.startsWith('/usr/')) {
                    try {
                        execSync(`sudo rm "${completionPath}"`, { stdio: 'pipe' });
                        console.log(`✅ Removed completion with sudo: ${path.basename(completionPath)}`);
                        cleanedCount++;
                    } catch (sudoError) {
                        // Ignore - completion file might be protected
                    }
                }
            }
        }
    }
    
    if (cleanedCount > 0) {
        console.log(`✅ Cleaned up ${cleanedCount} completion file(s)`);
    } else {
        console.log('ℹ️  No completion files found');
    }
}

// Handle process signals
process.on('SIGINT', () => {
    console.log('\n❌ Uninstallation cancelled by user');
    process.exit(1);
});

process.on('SIGTERM', () => {
    console.log('\n❌ Uninstallation terminated');
    process.exit(1);
});