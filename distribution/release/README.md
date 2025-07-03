# HiveTechs Consensus - Production Release Artifacts

This directory contains production-ready binary releases for all supported platforms.

## Release Structure

```
release/
├── checksums.txt          # SHA256 checksums for all artifacts
├── signatures/            # Digital signatures for verification
├── binaries/              # Platform binaries
│   ├── hive-macos-universal
│   ├── hive-linux-x86_64
│   ├── hive-linux-aarch64
│   ├── hive-windows-x86_64.exe
│   └── hive-windows-i686.exe
├── installers/            # Platform installers
│   ├── hive-installer.pkg      # macOS
│   ├── hive_*.deb             # Linux
│   └── hive-installer.exe     # Windows
└── packages/              # Package manager artifacts
    ├── homebrew/
    ├── chocolatey/
    └── apt/
```

## Installation Methods

### 1. Universal Installer (Recommended)
```bash
curl -fsSL https://hive.ai/install | sh
```

### 2. Platform-Specific Downloads
- **macOS**: Download `hive-installer.pkg`
- **Linux**: Download appropriate `.deb` package
- **Windows**: Download `hive-installer.exe`

### 3. Package Managers
- **macOS**: `brew install hivetechs/tap/hive`
- **Linux**: `sudo apt install hive-ai`
- **Windows**: `choco install hive-ai`

## Security

All artifacts are:
- ✅ Digitally signed
- ✅ SHA256 checksums provided
- ✅ GPG signatures available
- ✅ Distributed over HTTPS only

## Performance

- **Binary Size**: < 50MB (all platforms)
- **Startup Time**: < 100ms cold start
- **Memory Usage**: < 25MB idle
- **Installation**: < 30 seconds