# HiveTechs Consensus Distribution System

## 🎯 Overview

This document describes the complete binary distribution system for HiveTechs Consensus, designed to provide Claude Code-style global installation across all platforms.

## 📁 Directory Structure

```
distribution/
├── build/                  # Cross-platform build scripts
│   ├── macos.sh           # macOS universal binary builder
│   ├── linux.sh           # Linux static binary builder
│   └── windows.ps1        # Windows binary builder
├── installers/            # Platform-specific installer configs
│   ├── macos/
│   │   └── create_pkg.sh  # macOS .pkg installer creator
│   ├── linux/
│   │   └── create_deb.sh  # Debian package creator
│   └── windows/
│       └── hive.nsi       # NSIS installer configuration
├── scripts/               # Universal installation scripts
│   └── install.sh         # Claude Code-style installer
├── ci/                    # CI/CD configurations
│   └── release.yml        # GitHub Actions workflow
├── build_all.sh          # Build orchestration script
└── test_distribution.sh  # Distribution system tests
```

## 🚀 Quick Start

### Full Build (All Platforms)
```bash
# Build everything
./distribution/build_all.sh

# Build specific platforms
./distribution/build_all.sh --targets "macos linux"

# Skip tests for faster builds
./distribution/build_all.sh --skip-tests
```

### Platform-Specific Builds
```bash
# macOS only
./distribution/build/macos.sh

# Linux only
./distribution/build/linux.sh

# Windows (requires PowerShell)
pwsh ./distribution/build/windows.ps1
```

### Test Distribution System
```bash
./distribution/test_distribution.sh
```

## 🔧 Build System Features

### Cross-Platform Support
- **macOS**: Universal binaries (Intel + Apple Silicon)
- **Linux**: Static binaries with musl for maximum compatibility
- **Windows**: Optimized binaries with full installer support

### Build Optimizations
```toml
[profile.production]
opt-level = "z"           # Size optimization
lto = "fat"               # Full link-time optimization
codegen-units = 1         # Single codegen unit
strip = "symbols"         # Strip all symbols
panic = "abort"           # Abort on panic
```

### Security Features
- Checksum verification for all downloads
- Code signing preparation (certificates required)
- Signature validation in auto-updater
- Secure update mechanisms

## 📦 Installation Methods

### 1. Universal Install Script (Recommended)
```bash
curl -fsSL https://hive.ai/install | sh
```

**Features:**
- Automatic platform detection
- Checksum verification
- PATH configuration
- Shell completion installation
- User-friendly progress reporting

### 2. Platform-Specific Installers

#### macOS (.pkg)
- Professional installer with welcome screens
- Automatic PATH configuration
- LaunchAgent for auto-updates
- Proper uninstall support
- Code signing ready

#### Linux (.deb)
- Debian package with dependencies
- Shell completion integration
- Man page installation
- systemd service configuration
- Lintian compliance

#### Windows (.exe)
- NSIS installer with modern UI
- Registry integration
- PowerShell completion
- Start menu shortcuts
- Clean uninstall

### 3. Manual Installation
Download and extract platform archives directly.

## 🔄 Auto-Update System

### Features
- Secure update mechanism with signature verification
- Delta updates for bandwidth efficiency
- Rollback capability
- Background checking
- User notification system

### Configuration
```toml
[auto_update]
enabled = true
check_interval_hours = 24
channel = "stable"  # stable, beta, alpha
verify_signatures = true
backup_enabled = true
```

### Update Process
1. **Check**: Compare local vs remote versions
2. **Download**: Secure download with progress
3. **Verify**: Checksum and signature validation
4. **Backup**: Create recovery backup
5. **Install**: Atomic binary replacement
6. **Verify**: Post-install validation

## 🏗️ Build Artifacts

### Binary Naming Convention
```
hive-{version}-{platform}-{arch}.{ext}

Examples:
- hive-2.0.0-macos-universal.tar.gz
- hive-2.0.0-linux-x86_64.tar.gz
- hive-2.0.0-windows-x64.zip
```

### Installer Naming Convention
```
Platform-specific installer names:
- HiveTechs-Consensus-{version}.pkg (macOS)
- hive-ai_{version}_amd64.deb (Linux)
- HiveTechs-Consensus-Setup.exe (Windows)
```

### Artifact Contents
Each archive includes:
- Optimized binary
- Shell completions (bash, zsh, fish)
- Man page (Unix platforms)
- License and documentation
- Checksums for verification

## ⚙️ GitHub Actions CI/CD

### Workflow Triggers
- **Tags**: Automatic builds on version tags (v*)
- **Manual**: Workflow dispatch with version input
- **Pull Requests**: Build verification (no release)

### Build Matrix
```yaml
Strategy:
  - macOS: x86_64 + aarch64 → Universal
  - Linux: x86_64 + aarch64 (musl static)
  - Windows: x86_64 + i686
```

### Release Process
1. **Build**: Cross-platform compilation
2. **Test**: Quality and performance validation
3. **Package**: Create platform installers
4. **Sign**: Code signing (when certificates available)
5. **Release**: GitHub release with artifacts
6. **Deploy**: Update install script endpoint

## 🔐 Security Considerations

### Code Signing
- **macOS**: Developer ID Application certificate
- **Windows**: Authenticode signing
- **Linux**: GPG signature (planned)

### Verification Chain
1. HTTPS download with certificate validation
2. SHA256 checksum verification
3. Digital signature validation (when available)
4. Binary integrity checks

### Update Security
- Secure update server with HTTPS
- Signature verification before installation
- Atomic updates with rollback capability
- Version pinning and update control

## 📊 Performance Targets

| Metric | Target | Verification |
|--------|--------|--------------|
| **Binary Size** | < 50MB | Archive inspection |
| **Startup Time** | < 100ms | Performance benchmarks |
| **Download Size** | Minimal | Compression optimization |
| **Install Time** | < 30s | Platform testing |
| **Memory Usage** | < 50MB idle | Runtime monitoring |

## 🧪 Testing Strategy

### Distribution Tests
```bash
# System validation
./distribution/test_distribution.sh

# Build verification
./distribution/build_all.sh --skip-tests --targets "linux"

# Installer testing
./distribution/installers/linux/create_deb.sh
```

### Platform Testing
- **Automated**: CI/CD pipeline validation
- **Manual**: Platform-specific installer testing
- **Integration**: End-to-end installation verification

### Quality Gates
- All build scripts pass syntax validation
- Binaries meet size and performance requirements
- Installers work on clean systems
- Auto-update mechanism functions correctly

## 🚦 Release Checklist

### Pre-Release
- [ ] Version bumped in Cargo.toml
- [ ] Changelog updated
- [ ] All tests passing
- [ ] Performance benchmarks acceptable
- [ ] Documentation updated

### Build & Test
- [ ] Multi-platform builds successful
- [ ] All artifacts generated
- [ ] Checksums calculated
- [ ] Installers created
- [ ] Manual testing on target platforms

### Release
- [ ] GitHub release created
- [ ] Artifacts uploaded
- [ ] Install script updated
- [ ] Documentation deployed
- [ ] Release announcement

### Post-Release
- [ ] Monitor download metrics
- [ ] Verify auto-updates work
- [ ] Address any reported issues
- [ ] Update distribution endpoints

## 🛠️ Development Workflow

### Local Development
```bash
# Test distribution system
./distribution/test_distribution.sh

# Build for current platform only
./distribution/build/$(uname -s | tr '[:upper:]' '[:lower:]').sh

# Test universal installer
./distribution/scripts/install.sh --help
```

### Adding New Platforms
1. Create build script in `distribution/build/`
2. Add installer configuration in `distribution/installers/`
3. Update CI/CD workflow matrix
4. Add platform detection to install script
5. Update documentation

### Customizing Builds
Environment variables:
- `HIVE_VERSION`: Override version
- `BUILD_TARGETS`: Space-separated platform list
- `SKIP_TESTS`: Skip test execution
- `PARALLEL_BUILDS`: Enable parallel building

## 📞 Support & Troubleshooting

### Common Issues

#### Build Failures
- Ensure Rust toolchain is up-to-date
- Install platform-specific dependencies
- Check cross-compilation targets

#### Installer Issues
- Verify packaging tools are installed
- Check file permissions
- Validate installer configurations

#### Auto-Update Problems
- Verify network connectivity
- Check certificate validation
- Review update server logs

### Getting Help
- **Documentation**: [GitHub Repository](https://github.com/hivetechs/hive)
- **Issues**: [GitHub Issues](https://github.com/hivetechs/hive/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hivetechs/hive/discussions)
- **Email**: team@hivetechs.com

## 🎯 Future Enhancements

### Planned Features
- [ ] Package manager integration (Homebrew, apt, chocolatey)
- [ ] Incremental/delta updates
- [ ] A/B testing for updates
- [ ] Telemetry and usage analytics
- [ ] Enterprise deployment tools

### Distribution Improvements
- [ ] CDN integration for faster downloads
- [ ] Mirror servers for redundancy
- [ ] Bandwidth-aware update scheduling
- [ ] Offline installer packages

---

## Quick Reference

### Key Commands
```bash
# Full build
./distribution/build_all.sh

# Test system
./distribution/test_distribution.sh

# Install locally
curl -fsSL https://hive.ai/install | sh

# Manual build
cargo build --profile production --features production
```

### Important Files
- `distribution/build_all.sh` - Main build orchestrator
- `distribution/scripts/install.sh` - Universal installer
- `distribution/ci/release.yml` - GitHub Actions workflow
- `src/core/updater.rs` - Auto-update implementation

This distribution system provides a robust, secure, and user-friendly way to distribute HiveTechs Consensus across all platforms, matching the quality and experience of Claude Code.