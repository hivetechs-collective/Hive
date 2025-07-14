# macOS Installation Guide - Hive IDE

## Quick Install Steps

1. **Download** the DMG file from hivetechs.io/downloads
2. **Mount** the DMG by double-clicking it
3. **Drag** Hive.app to the Applications folder
4. **Right-click** on Hive.app in Applications
5. **Select "Open"** from the context menu
6. **Click "Open"** in the security dialog

## What to Expect

### First Launch
- macOS will show: "Apple could not verify 'Hive' is free of malware..."
- This is normal for apps from independent developers
- The right-click → Open method bypasses this safely

### Security Dialog
```
"Hive" is an app downloaded from the internet.
Are you sure you want to open it?

[Cancel] [Open]
```
- Click **"Open"** to proceed

### After First Launch
- Subsequent launches work normally (double-click)
- No more security warnings
- App is permanently trusted by macOS

## Why This Happens

- **Apple's Gatekeeper** requires apps to be notarized
- **We're waiting** for Apple Developer certificate approval (48 hours)
- **Ad-hoc signing** prevents "damaged app" errors but not Gatekeeper
- **This is standard** for new independent developers

## When Certificates Arrive

Once Apple approves our Developer certificates:
- **No security warnings** at all
- **Double-click to install** normally
- **Automatic updates** will work seamlessly
- **Full notarization** like VS Code, Docker, etc.

## Alternative: System Preferences Method

If right-click doesn't work:

1. Try to open Hive normally (will fail)
2. Go to **System Preferences** → **Security & Privacy**
3. Click **"Open Anyway"** next to the Hive message
4. Confirm in the dialog

## Need Help?

- **Support**: support@hivetechs.io
- **Documentation**: hivetechs.io/documentation
- **This is temporary** - full signing coming soon!

---

*This guide will be updated once Apple Developer certificates are approved.*