# Qwen Code - Critical Implementation Quirks

## ⚠️ CRITICAL: Documentation vs Reality Mismatch

### The Problem
The official Qwen Code documentation shows the command as `qwen-code` everywhere, but the actual NPM package installs the binary as just `qwen`.

### Documentation Says:
```bash
qwen-code auth
qwen-code --version
qwen-code "Your prompt"
```

### Reality Is:
```bash
qwen auth
qwen --version
qwen "Your prompt"
```

## Package Details
- **NPM Package**: `@qwen-code/qwen-code` (correct)
- **Install Command**: `npm install -g @qwen-code/qwen-code@latest` (correct)
- **Binary Installed**: `qwen` (NOT `qwen-code`)
- **Version Output**: Just `0.0.8` (no prefix like other tools)

## Implementation Configuration

```typescript
'qwen-code': {
    id: 'qwen-code',
    name: 'Qwen Code',
    command: 'qwen',  // NOT 'qwen-code'!
    installCommand: 'npm install -g @qwen-code/qwen-code@latest',
    versionCommand: 'qwen --version',  // NOT 'qwen-code --version'
    versionRegex: /(?:qwen\/|v?)(\d+\.\d+\.\d+)/,
    // ... rest
}
```

## How We Discovered This
1. Initially configured based on documentation using `qwen-code` as command
2. Install succeeded but tool showed as "Not Installed"
3. Checked npm global packages - package was installed
4. Examined package.json bin entry - found `"qwen": "dist/index.js"`
5. The binary name doesn't match the package name or documentation

## Lessons Learned
- **Always verify actual binary names** after installation
- **Don't trust documentation blindly** - check package.json
- **Test the actual commands** not just what docs say

## Detection Fix
For detection to work properly:
1. Check for command `qwen` not `qwen-code`
2. Version parsing needs to handle simple output like `0.0.8`
3. Launch command should use `qwen` not `qwen-code`

## UI Refresh Issue
Similar to Gemini CLI, after installation:
- Console shows "installed successfully"
- But UI doesn't refresh to show installed status
- Need to restart app for changes to take effect

## Complete Working Configuration

### Files to Update:
1. `src/shared/types/cli-tools.ts` - Tool registry with correct command
2. `src/index.ts` - Version detection for install/update handlers
3. `src/terminal-ipc-handlers.ts` - Display name "Qwen"
4. `src/renderer.ts` - Dynamic card with FREE badge

### Testing Command:
```bash
# Verify installation
which qwen  # Should show path
qwen --version  # Should show version like "0.0.8"

# NOT these (they won't work):
qwen-code --version  # Command not found
```

## Future Improvements
- Update our local documentation to reflect reality
- Consider filing issue with Qwen Code project about mismatch
- Add automatic binary name detection from package.json