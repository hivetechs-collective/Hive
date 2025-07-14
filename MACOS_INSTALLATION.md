# macOS Installation Instructions

## Security Notice

The Hive IDE app is currently not signed with an Apple Developer certificate. This means macOS will show a security warning when you first try to open it. This is normal and expected.

## Installation Steps

### Method 1: Right-Click to Open (Recommended)

1. Download `hive-macos-arm64.app.tar.gz` from the downloads page
2. Double-click the `.tar.gz` file to extract it
3. **Right-click** on `Hive.app` and select **"Open"**
4. In the security dialog, click **"Open"** to bypass Gatekeeper
5. The app will now be trusted and you can open it normally in the future

### Method 2: Using Terminal

1. Download and extract the app as above
2. Open Terminal
3. Run the following command (replace path with your actual path):
   ```bash
   xattr -cr /path/to/Hive.app
   ```
4. You can now open the app normally

### Method 3: System Preferences (macOS Ventura and earlier)

1. Try to open the app normally (it will be blocked)
2. Open **System Preferences** → **Security & Privacy**
3. In the **General** tab, you'll see a message about Hive being blocked
4. Click **"Open Anyway"**
5. Confirm in the next dialog

### Method 4: System Settings (macOS Sonoma and later)

1. Try to open the app normally (it will be blocked)
2. Open **System Settings** → **Privacy & Security**
3. Scroll down to see the blocked app message
4. Click **"Open Anyway"**
5. Enter your password and confirm

## Why This Happens

Apple requires all apps distributed outside the Mac App Store to be:
1. Code-signed with a valid Apple Developer certificate ($99/year)
2. Notarized by Apple (automated security scan)

As an open-source project, we currently distribute unsigned builds. This doesn't mean the app is unsafe - it just means Apple hasn't verified it.

## Verifying Download Integrity

To ensure your download hasn't been tampered with, verify the SHA-256 checksum:

```bash
shasum -a 256 hive-macos-arm64.app.tar.gz
```

Compare the output with the checksum provided on our downloads page.

## Future Plans

We're working on getting an Apple Developer certificate to properly sign future releases, which will eliminate these security warnings.