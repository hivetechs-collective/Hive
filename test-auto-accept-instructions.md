# Testing AI-Enhanced Auto-Accept Features

## How to Test the Auto-Accept Features

1. **Start hive-consensus**:
   ```bash
   ./target/debug/hive-consensus
   ```

2. **Look for the Settings/Preferences Menu**:
   - In the main UI menu bar, look for 'Settings' or 'Preferences'
   - Click on 'Auto-Accept Settings'

3. **Test the Auto-Accept Settings Panel**:
   - **Mode Selection**: Try changing between Manual, Conservative, Balanced, Aggressive, and Plan modes
   - **Risk Settings**: Adjust the risk tolerance slider (0-100%)
   - **AI Trust Level**: Adjust how much you trust AI suggestions (0-100%)
   - **Safety Options**: 
     - Toggle "Auto-backup before operations"
     - Toggle "Require confirmation for deletions"
     - Toggle "Require confirmation for mass updates"
   - **Custom Rules**: 
     - Add file pattern rules (e.g., "*.test.js" → Always Accept)
     - Set different actions for different patterns

4. **Test File Operations with Consensus**:
   - Ask a question that would generate file operations, such as:
     - "Create a new React component called UserProfile"
     - "Add error handling to all API calls"
     - "Refactor the authentication module"

5. **During Consensus Execution, Watch For**:
   - **Operation Preview**: 
     - Syntax-highlighted code showing what will change
     - Side-by-side diff view
     - Confidence scores for each operation
   - **AI Reasoning**: Explanations for why operations are auto-accepted or require confirmation
   - **Progress Indicators**: Real-time status of each operation

6. **Test Keyboard Shortcuts in Approval Interface**:
   - `Y` - Approve current operation
   - `N` - Reject current operation
   - `Shift+A` - Approve all operations
   - `Shift+R` - Reject all operations
   - `Space` - Toggle selection
   - `Arrow keys` - Navigate between operations

7. **Watch the Notification System**:
   - Auto-accepted operations appear as notifications
   - Batch operations show summaries
   - Click on notifications for details

## Testing Different Modes

### Conservative Mode
- Set mode to "Conservative"
- Ask to delete multiple files
- Should require confirmation for all deletions
- Only very high confidence (>90%) operations auto-execute

### Balanced Mode
- Set mode to "Balanced"
- Ask to update several files
- Medium-risk operations require confirmation
- Good balance of automation and safety

### Aggressive Mode
- Set mode to "Aggressive"
- Ask to create multiple new files
- Most operations auto-execute
- Only very high risk operations blocked

### Plan Mode
- Set mode to "Plan"
- Ask to implement a feature
- All operations shown but none auto-execute
- Good for reviewing before execution

## Expected Behaviors

1. **Auto-Accept Decision Display**:
   - Green border/highlight for auto-accepted
   - Yellow border for requiring confirmation
   - Red border for blocked operations

2. **Confidence Scores**:
   - Displayed as percentages
   - Color-coded (green >80%, yellow 50-80%, red <50%)

3. **Risk Indicators**:
   - Visual indicators for operation risk
   - Warnings for dangerous operations

4. **Learning System**:
   - The system learns from your Accept/Reject decisions
   - Over time, it adapts to your preferences

## Troubleshooting

If the UI doesn't start:
- Check the console for error messages
- Make sure all dependencies are installed
- Try running with RUST_LOG=debug for more information

If auto-accept isn't working:
- Check that you have a mode selected other than "Manual"
- Verify your risk tolerance settings
- Check custom rules aren't blocking operations