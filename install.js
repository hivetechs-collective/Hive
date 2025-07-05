#!/usr/bin/env node
// Hive AI NPM Package Installer
// Installs the Rust binary globally and sets up shell completions

const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

const BINARY_NAME = process.platform === 'win32' ? 'hive.exe' : 'hive';
const PACKAGE_DIR = __dirname;
const BINARY_PATH = path.join(PACKAGE_DIR, 'bin', BINARY_NAME);

// Platform-specific installation
const platform = os.platform();
const arch = os.arch();

console.log('🐝 Installing Hive AI globally...');
console.log(`Platform: ${platform}-${arch}`);
console.log(`Package: ${PACKAGE_DIR}`);

// Check if binary exists
if (!fs.existsSync(BINARY_PATH)) {
    console.error('❌ Binary not found in package');
    console.error(`Expected: ${BINARY_PATH}`);
    
    // List available files for debugging
    const binDir = path.join(PACKAGE_DIR, 'bin');
    if (fs.existsSync(binDir)) {
        console.log('Available files in bin/:');
        fs.readdirSync(binDir).forEach(file => {
            console.log(`  - ${file}`);
        });
    } else {
        console.error('Bin directory does not exist');
    }
    
    process.exit(1);
}

// Platform-specific installation
try {
    if (platform === 'win32') {
        installWindows();
    } else {
        installUnix();
    }
    
    // Test installation
    try {
        const version = execSync(`${BINARY_NAME} --version`, { stdio: 'pipe' }).toString();
        console.log('✅ Hive AI installed successfully!');
        console.log(`   Version: ${version.trim()}`);
        console.log('   Run "hive" to get started');
        
        // Install shell completions
        try {
            execSync(`${BINARY_NAME} install completions`, { stdio: 'pipe' });
            console.log('✅ Shell completions installed');
        } catch (error) {
            console.log('⚠️  Shell completions not installed (optional)');
        }
        
    } catch (error) {
        console.log('⚠️  Installation completed but verification failed');
        console.log('   You may need to restart your terminal or update your PATH');
        console.log(`   Try running: ${BINARY_NAME} --version`);
    }
} catch (error) {
    console.error('❌ Installation failed:', error.message);
    
    // Provide troubleshooting information
    console.log('\n🔧 Troubleshooting:');
    console.log('1. Make sure you have permission to write to system directories');
    console.log('2. Try running with elevated privileges (sudo on Unix, Administrator on Windows)');
    console.log('3. Check if antivirus software is blocking the installation');
    console.log('4. Manually copy the binary to a directory in your PATH');
    
    process.exit(1);
}

function installWindows() {
    console.log('📦 Installing on Windows...');
    
    // Windows installation logic
    const programFiles = process.env.ProgramFiles || 'C:\\Program Files';
    const installDir = path.join(programFiles, 'Hive');
    const targetPath = path.join(installDir, BINARY_NAME);
    
    // Create directory
    if (!fs.existsSync(installDir)) {
        try {
            fs.mkdirSync(installDir, { recursive: true });
        } catch (error) {
            console.log('⚠️  Could not create installation directory, trying user directory...');
            
            // Fallback to user directory
            const userDir = path.join(os.homedir(), 'AppData', 'Local', 'Hive');
            fs.mkdirSync(userDir, { recursive: true });
            const userTargetPath = path.join(userDir, BINARY_NAME);
            fs.copyFileSync(BINARY_PATH, userTargetPath);
            
            console.log(`✅ Installed to user directory: ${userTargetPath}`);
            console.log('⚠️  You may need to add this directory to your PATH manually');
            return;
        }
    }
    
    // Copy binary
    fs.copyFileSync(BINARY_PATH, targetPath);
    
    // Add to PATH (requires admin privileges)
    try {
        execSync(`setx PATH "%PATH%;${installDir}" /M`, { stdio: 'inherit' });
        console.log('✅ Added Hive to system PATH');
    } catch (error) {
        console.log('⚠️  Could not add to system PATH automatically');
        console.log(`   Please add ${installDir} to your PATH manually`);
        console.log('   Or run the installer as Administrator');
    }
    
    console.log(`✅ Binary installed: ${targetPath}`);
}

function installUnix() {
    console.log(`📦 Installing on ${platform}...`);
    
    // Unix installation logic
    const possibleDirs = [
        '/usr/local/bin',
        '/usr/bin',
        path.join(os.homedir(), '.local', 'bin'),
        path.join(os.homedir(), 'bin')
    ];
    
    let installDir = null;
    let targetPath = null;
    
    // Try to find a writable directory
    for (const dir of possibleDirs) {
        try {
            // Test write permissions
            const testFile = path.join(dir, '.hive_test');
            fs.writeFileSync(testFile, 'test');
            fs.unlinkSync(testFile);
            
            installDir = dir;
            targetPath = path.join(dir, BINARY_NAME);
            break;
        } catch (error) {
            // Directory not writable, try next
            continue;
        }
    }
    
    if (!installDir) {
        // Try with sudo for system directories
        try {
            console.log('🔐 Requesting elevated privileges...');
            execSync(`sudo cp "${BINARY_PATH}" "/usr/local/bin/${BINARY_NAME}"`, { stdio: 'inherit' });
            execSync(`sudo chmod +x "/usr/local/bin/${BINARY_NAME}"`, { stdio: 'inherit' });
            targetPath = `/usr/local/bin/${BINARY_NAME}`;
            console.log(`✅ Binary installed: ${targetPath}`);
            return;
        } catch (sudoError) {
            throw new Error('Failed to install with elevated privileges');
        }
    }
    
    // Create directory if it doesn't exist
    if (!fs.existsSync(installDir)) {
        fs.mkdirSync(installDir, { recursive: true });
    }
    
    // Copy binary
    fs.copyFileSync(BINARY_PATH, targetPath);
    
    // Make executable
    fs.chmodSync(targetPath, 0o755);
    
    console.log(`✅ Binary installed: ${targetPath}`);
    
    // Check if directory is in PATH
    const currentPath = process.env.PATH || '';
    if (!currentPath.includes(installDir)) {
        console.log(`⚠️  ${installDir} is not in your PATH`);
        console.log(`   Add this line to your shell configuration file:`);
        console.log(`   export PATH="${installDir}:$PATH"`);
    }
}

// Handle process signals
process.on('SIGINT', () => {
    console.log('\n❌ Installation cancelled by user');
    process.exit(1);
});

process.on('SIGTERM', () => {
    console.log('\n❌ Installation terminated');
    process.exit(1);
});