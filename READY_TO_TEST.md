# ✅ Ready to Test Setup Flow!

I've prepared everything for you to test the new first-time setup experience.

## What I've Done:
1. ✅ Removed the existing config file (`~/.hive/config.toml`)
2. ✅ Built the latest binary with the setup flow
3. ✅ The system is ready for first-time user experience

## To Test:

Run this command in your terminal:
```bash
./target/release/hive tui
```

## What You'll See:

1. **Welcome Screen**:
   - Explains the 2-step setup process
   - Press `Enter` to continue

2. **OpenRouter API Key**:
   - Instructions on getting a key from https://openrouter.ai/keys
   - Enter your key (starts with `sk-or-`)
   - The key will be masked for security
   - Press `Enter` to validate

3. **Hive License Key**:
   - Instructions on getting a license from https://hivetechs.com/account
   - Enter your license key
   - The key will be masked for security
   - Press `Enter` to validate

4. **Success**:
   - Configuration saved message
   - Press `Enter` to start using Hive AI
   - The main TUI will load with working AI consensus

## After Setup:

Your keys will be saved to `~/.hive/config.toml` and you won't need to enter them again. The consensus engine will work with real AI responses instead of demo mode!

## Navigation:
- Use `Backspace` to correct typos
- Use `Left/Right` arrow keys to move cursor
- Press `Ctrl+C` at any time to exit

The setup is designed to be simple and secure, just like Claude Code!