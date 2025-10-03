# Hive Consensus Distribution Strategy

## Distribution Channels

### 1. Direct Download (via R2)
- **macOS**: DMG installer with code signing
- **Windows**: NSIS installer with auto-update
- **Linux**: AppImage, Snap, and .deb packages

### 2. Package Managers
- **npm**: `npm install -g @hivetechs/consensus` (launches electron app)
- **Homebrew (Cask, macOS)**: `brew install --cask hivetechs-collective/tap/hive-consensus`
- **Chocolatey** (Windows): `choco install hive-consensus`
- **Snap Store** (Linux): `snap install hive-consensus`

### 3. Auto-Update System
- Uses electron-updater with R2 as the update server
- Automatic background updates
- Delta updates for faster downloads

## Current Package Locations

After running `npm run package`:
- **macOS (ARM64)**: `out/Hive Consensus-darwin-arm64/`
- **macOS (Intel)**: `out/Hive Consensus-darwin-x64/` (when built)
- **Windows**: `out/Hive Consensus-win32-x64/` (when built)
- **Linux**: `out/Hive Consensus-linux-x64/` (when built)

## Distribution Files

After running `npm run make`:
- **macOS**: `out/make/zip/darwin/arm64/Hive Consensus-darwin-arm64-1.0.0.zip`
- **Windows**: `out/make/squirrel.windows/x64/` (Setup.exe)
- **Linux**: `out/make/deb/x64/` (.deb package)

## R2 Upload Structure

```
r2-bucket/
├── releases/
│   ├── latest/
│   │   ├── mac/
│   │   │   ├── Hive-Consensus.dmg
│   │   │   └── Hive-Consensus.dmg.blockmap
│   │   ├── win/
│   │   │   ├── Hive-Consensus-Setup.exe
│   │   │   └── latest.yml
│   │   └── linux/
│   │       ├── Hive-Consensus.AppImage
│   │       └── latest-linux.yml
│   └── v1.0.0/
│       └── [same structure]
└── update/
    ├── mac/
    │   └── latest-mac.yml
    ├── win/
    │   └── latest.yml
    └── linux/
        └── latest-linux.yml
```

## NPM Distribution

The app can be distributed via npm as a global package that downloads and installs the appropriate binary:

```json
{
  "name": "@hivetechs/consensus",
  "bin": {
    "hive-consensus": "./bin/hive-consensus.js"
  },
  "scripts": {
    "postinstall": "node scripts/install.js"
  }
}
```

The install script downloads the appropriate binary from R2 based on the platform.
