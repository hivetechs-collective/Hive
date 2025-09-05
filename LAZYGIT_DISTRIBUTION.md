# LazyGit Distribution & Auto-Update System

## Overview

The Hive Consensus GUI includes an automatic LazyGit distribution and update system that ensures users always have the latest version of LazyGit without manual installation or maintenance.

## How It Works

### 1. **First Launch Detection**
When the GUI starts, it checks for LazyGit in the following order:
- Our managed installation at `~/.hive/tools/lazygit/`
- System-wide installation via `which lazygit`
- If not found, automatically downloads the latest version

### 2. **Automatic Installation**
- Detects the user's platform (macOS x86_64/arm64, Linux x86_64/arm64, Windows x86_64)
- Downloads the appropriate binary from GitHub releases
- Extracts and installs to `~/.hive/tools/lazygit/`
- Sets proper permissions (executable on Unix systems)

### 3. **Daily Update Checks**
- Checks for updates once every 24 hours
- Compares installed version with latest GitHub release
- Downloads and installs updates automatically in the background
- Updates are non-blocking - the app continues using the current version until the update completes

### 4. **Platform Support**
The updater automatically selects the correct binary for:
- **macOS**: Darwin_x86_64 (Intel) or Darwin_arm64 (Apple Silicon)
- **Linux**: Linux_x86_64 or Linux_arm64
- **Windows**: Windows_x86_64

### 5. **Storage & Metadata**
- LazyGit binary: `~/.hive/tools/lazygit/lazygit[.exe]`
- Update metadata: `~/.hive/tools/lazygit_metadata.json`
- Tracks: last check time, installed version, install path

## Benefits for Users

1. **Zero Configuration**: No need to install LazyGit manually
2. **Always Updated**: Automatic daily checks ensure latest features and bug fixes
3. **Cross-Platform**: Works seamlessly on macOS, Linux, and Windows
4. **Offline Friendly**: Uses system LazyGit if available, only downloads when needed
5. **Non-Intrusive**: Updates happen in the background without interrupting work

## Developer Integration

### Using the Updater

```rust
use hive_ai::desktop::git::initialize_lazygit_updater;

// Get LazyGit path (installs/updates if needed)
let lazygit_path = initialize_lazygit_updater().await?;

// Force update to latest version
use hive_ai::desktop::git::force_update_lazygit;
force_update_lazygit().await?;
```

### Architecture

```
LazyGitUpdater
├── Check for updates (24h interval)
├── Download from GitHub releases
├── Extract binary from tar.gz
├── Install to ~/.hive/tools/
└── Update metadata file
```

### Key Components

1. **`LazyGitUpdater`**: Main struct handling all update logic
2. **`initialize_lazygit_updater()`**: Entry point that returns LazyGit path
3. **`force_update_lazygit()`**: Manual update trigger
4. **GitHub API integration**: Fetches latest release information
5. **Platform detection**: Automatic binary selection

## Distribution via R2/Cloudflare

When distributing the Hive GUI:

1. **Initial Bundle**: The GUI itself is lightweight - LazyGit is downloaded on first use
2. **CDN Friendly**: Binary downloads can be cached on Cloudflare's edge network
3. **Bandwidth Efficient**: Only downloads when needed, not with every GUI update
4. **Version Independence**: GUI updates don't require LazyGit redistribution

## Error Handling

The system gracefully handles:
- Network failures (falls back to existing installation)
- GitHub API rate limits (uses cached version)
- Corrupted downloads (retries on next check)
- Permission issues (logs warnings, uses system version)

## Security Considerations

1. **HTTPS Only**: All downloads use secure connections
2. **GitHub Official**: Only downloads from official jesseduffield/lazygit releases
3. **Checksum Verification**: (TODO) Add SHA256 verification for downloads
4. **No Elevation**: Installs to user directory, no admin/sudo required

## Future Enhancements

1. **Checksum Verification**: Add SHA256 verification for downloaded binaries
2. **Mirror Support**: Allow custom download mirrors for enterprise environments
3. **Rollback**: Keep previous version for quick rollback if needed
4. **Progress UI**: Show download progress in the GUI
5. **Proxy Support**: Honor system proxy settings for downloads

## Testing

To test the updater:

```bash
# Remove existing LazyGit installation
rm -rf ~/.hive/tools/lazygit*

# Run the GUI - it should auto-install LazyGit
cargo run --bin hive-consensus

# Force an update check
# (Set last_check to old date in ~/.hive/tools/lazygit_metadata.json)

# Test offline behavior
# (Disconnect network after installation)
```

## Troubleshooting

### LazyGit not installing
- Check `~/.hive/tools/` permissions
- Look for errors in logs about GitHub API
- Verify network connectivity

### Updates not working
- Check `lazygit_metadata.json` for last_check time
- Ensure write permissions in `~/.hive/tools/`
- Check GitHub API rate limits

### Wrong platform binary
- Verify Rust's platform detection: `std::env::consts::OS` and `ARCH`
- Check GitHub release has binary for your platform

---

This auto-update system ensures that Hive GUI users always have the best Git experience with zero maintenance overhead.