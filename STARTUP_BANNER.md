# Hive AI Startup Banner

## Command Behavior

Just like Claude Code, when a user types just `hive` in the terminal, it will display a beautiful startup banner with system information and quick start guide.

## Banner Output

```
  ╭─────────────────────────────────────────────────────────────╮
  │  🐝 HiveTechs Consensus                                    │
  ╰─────────────────────────────────────────────────────────────╯

  Version: 2.0.0
  Config:  ✓ Configured
  Memory:  ✓ 142 conversations
  Profile: Balanced
  Models:  323 available

  Quick Start:
    hive ask "What does this code do?"
    hive analyze .
    hive plan "Add user authentication"
    hive interactive                    # Start interactive mode

  Documentation:
    hive --help                         # Show all commands
    hive <command> --help              # Command-specific help
    https://docs.hivetechs.com         # Full documentation

  System Status:
    Internet: ✓ Connected
    AI Models: ✓ Available
    Memory: 25.0 MB

```

## Banner States

### First Time User (Not Configured)
```
  ╭─────────────────────────────────────────────────────────────╮
  │  🐝 HiveTechs Consensus                                    │
  ╰─────────────────────────────────────────────────────────────╯

  Version: 2.0.0
  Config:  ⚠ Not configured
  Memory:  ⚠ Not initialized

  Setup Required:
    hive init                          # Initialize configuration
    hive index                         # Build project index

  Quick Start:
    hive ask "What does this code do?"
    hive analyze .
    hive plan "Add user authentication"
    hive interactive                    # Start interactive mode

  Documentation:
    hive --help                         # Show all commands
    hive <command> --help              # Command-specific help
    https://docs.hivetechs.com         # Full documentation

  System Status:
    Internet: ✓ Connected
    AI Models: ✓ Available
    Memory: 25.0 MB

```

### Offline Mode
```
  ╭─────────────────────────────────────────────────────────────╮
  │  🐝 HiveTechs Consensus                                    │
  ╰─────────────────────────────────────────────────────────────╯

  Version: 2.0.0
  Config:  ✓ Configured
  Memory:  ✓ 142 conversations
  Profile: Balanced
  Models:  323 available

  Quick Start:
    hive ask "What does this code do?"
    hive analyze .
    hive plan "Add user authentication"
    hive interactive                    # Start interactive mode

  Documentation:
    hive --help                         # Show all commands
    hive <command> --help              # Command-specific help
    https://docs.hivetechs.com         # Full documentation

  System Status:
    Internet: ✗ Offline
    AI Models: ✗ Unavailable
    Memory: 25.0 MB

```

## Implementation Details

### Trigger Behavior
```bash
# Shows banner (like Claude Code)
$ hive

# Executes command directly (no banner)
$ hive ask "Hello world"
$ hive --help
$ hive analyze .
```

### Color Scheme
- **Banner Border**: Cyan (`style().cyan()`)
- **Title**: Bold Yellow (`style().bold().yellow()`)
- **Labels**: Dim (`style().dim()`)
- **Success Values**: Green (`style().green()`)
- **Warning Values**: Yellow (`style().yellow()`)
- **Error Values**: Red (`style().red()`)
- **Commands**: Cyan (`style().cyan()`)
- **Comments**: Dim (`style().dim()`)
- **Links**: Blue Underlined (`style().blue().underlined()`)

### Dynamic Information
The banner displays real-time information:

1. **Version**: From `Cargo.toml` version
2. **Configuration Status**: Checks for `~/.hive/config.toml`
3. **Memory/Database**: Counts conversations in local database
4. **Consensus Profile**: Reads from config (Speed/Balanced/Cost/Elite)
5. **Model Count**: Available models from OpenRouter
6. **Internet Status**: Connectivity check to OpenRouter API
7. **API Status**: Model availability check
8. **Memory Usage**: Current process memory consumption

### Screen Clearing
Like Claude Code, the banner clears the terminal screen first for a clean presentation, then displays the formatted information.

### Professional Presentation
The banner provides:
- **Immediate Status**: User can see system health at a glance
- **Quick Commands**: Most common operations readily available
- **Setup Guidance**: Clear next steps for new users
- **Documentation Links**: Easy access to help resources
- **Brand Identity**: Consistent HiveTechs Consensus branding

This creates the same professional, polished experience that developers love about Claude Code, while showcasing Hive AI's unique consensus-powered capabilities.