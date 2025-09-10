# CLI Tools Symlink Strategy (v1.8.272+)

## Overview
Starting with version 1.8.272, Hive Consensus has improved the user experience for CLI tools by moving symlink creation from the launch phase to the installation phase. This eliminates repetitive folder selection dialogs and creates a seamless workflow.

## Previous Behavior (v1.8.271 and earlier)
- User clicks "Launch" button for a CLI tool
- Folder selection dialog appears
- User selects a working directory
- Symlink to `.hive-ai.db` is created in that directory
- CLI tool launches in that directory
- **Issue**: This happened EVERY time a CLI tool was launched

## New Behavior (v1.8.272+)
- User clicks "Install" button for a CLI tool
- Tool is installed
- Symlinks to `.hive-ai.db` are automatically created in common directories
- User clicks "Launch" button
- CLI tool launches immediately in the appropriate directory
- **No folder selection dialog required**

## Symlink Locations
When a CLI tool is installed, symlinks to `~/.hive/hive-ai.db` are created in:
- `~/` (Home directory)
- `~/Desktop`
- `~/Documents`
- `~/Downloads`
- `~/Developer` (if it exists)

## How It Works

### Installation Phase
```javascript
// In cli-tools-install handler
const actualDbPath = path.join(os.homedir(), '.hive', 'hive-ai.db');
const commonDirs = [
  os.homedir(),
  path.join(os.homedir(), 'Desktop'),
  path.join(os.homedir(), 'Documents'),
  path.join(os.homedir(), 'Downloads')
];

// Create symlinks in each directory
for (const dir of commonDirs) {
  const symlinkPath = path.join(dir, '.hive-ai.db');
  if (!fs.existsSync(symlinkPath)) {
    fs.symlinkSync(actualDbPath, symlinkPath);
  }
}
```

### Launch Phase
```javascript
// In cli-tool-launch handler
// No folder selection - just launch
const selectedPath = projectPath || os.homedir();
// Launch the CLI tool directly
```

## Using with AI CLI Tools

Once installed, AI CLI tools can access the Hive database from any common directory:

### Example with Claude
```bash
cd ~/Desktop
# .hive-ai.db symlink already exists here
claude "Show me recent work from .hive-ai.db"
```

### Example with Gemini
```bash
cd ~/Developer/my-project
# .hive-ai.db symlink already exists here
gemini "Query .hive-ai.db for debugging patterns"
```

## Database Views Available
The `.hive-ai.db` symlink provides access to these views:
- `recent_work` - Recent coding activities
- `solutions` - Solved problems and solutions
- `patterns` - Common patterns and best practices
- `debugging` - Debugging sessions and fixes
- `errors_fixed` - Error resolutions
- `code_examples` - Code snippets and examples
- `project_context` - Project-specific context

## Benefits
1. **No Repetitive Dialogs**: Install once, use everywhere
2. **Consistent Access**: Database available in all common working directories
3. **Simplified UX**: Launch buttons work instantly
4. **Dynamic Detection**: Automatically includes user-specific directories like `~/Developer`
5. **Backward Compatible**: Existing symlinks are preserved

## Troubleshooting

### Symlink Not Found
If a CLI tool can't find `.hive-ai.db`:
1. Check if the tool is properly installed
2. Verify symlink exists: `ls -la ~/.hive-ai.db`
3. Reinstall the CLI tool to recreate symlinks

### Manual Symlink Creation
If needed, you can manually create a symlink:
```bash
ln -s ~/.hive/hive-ai.db /path/to/your/directory/.hive-ai.db
```

### Checking Symlink Locations
To see all `.hive-ai.db` symlinks:
```bash
find ~ -name ".hive-ai.db" -type l 2>/dev/null
```

## Technical Details
- **Implementation**: Modified IPC handlers in `src/index.ts`
- **Version**: Introduced in v1.8.272
- **Commit**: `refactor(cli-tools): move symlink creation from launch to installation`
- **Files Modified**: 
  - `src/index.ts` - IPC handlers
  - `MASTER_ARCHITECTURE.md` - Documentation update

## Future Enhancements
- User-configurable symlink directories
- Automatic symlink updates when new directories are created
- Integration with project templates
- Smart directory detection based on usage patterns