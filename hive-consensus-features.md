## What to Look for in Hive-Consensus AI-Enhanced Auto-Accept Features

When you run hive-consensus, you should see:

### 1. In the Main UI Menu Bar:
- Look for 'Settings' or 'Preferences' menu
- There should be an 'Auto-Accept Settings' option

### 2. Auto-Accept Settings Panel Features:
- **Mode Selection**: Choose between Manual, Conservative, Balanced, Aggressive, or Plan modes
- **Risk Settings**: Risk tolerance and AI trust level sliders
- **Safety Options**: Toggle for auto-backups, confirmation for deletions/mass updates
- **Custom Rules**: Add file pattern rules for always accept/reject

### 3. When Running Consensus:
- When you ask a question that generates file operations, you should see:
  - Operation preview with syntax highlighting
  - Confidence scores for each operation
  - AI reasoning for auto-accept decisions
  - Progress indicators showing operation status

### 4. Keyboard Shortcuts in Approval Interface:
- Y: Approve current operation
- N: Reject current operation
- Shift+A: Approve all operations
- Shift+R: Reject all operations
- Space: Toggle selection
- Arrow keys: Navigate between operations

### 5. Notification System:
- Auto-accepted operations show notifications
- Summary of batch auto-accepts
- Real-time status updates

The current issue is that the UI has an infinite loop preventing it from starting properly. The features are integrated but need the reactive scope issue fixed first.
