#!/bin/bash
set -e

echo "🔐 Code signing Hive for macOS..."

# Auto-detect signing identity if not set
if [ -z "$APPLE_SIGNING_IDENTITY" ]; then
    echo "🔍 Auto-detecting signing identity..."
    
    # Look for Developer ID Application certificate
    DEVELOPER_ID=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | sed 's/.*"\(.*\)".*/\1/' || echo "")
    
    if [ -n "$DEVELOPER_ID" ]; then
        export APPLE_SIGNING_IDENTITY="$DEVELOPER_ID"
        echo "✅ Found signing identity: $APPLE_SIGNING_IDENTITY"
    else
        echo "❌ No Developer ID Application certificate found"
        echo ""
        echo "Your Apple Developer enrollment may still be processing, or you need to:"
        echo "1. Complete Apple Developer Program enrollment"
        echo "2. Create a Developer ID Application certificate"
        echo "3. Download and install the certificate in Keychain"
        echo ""
        echo "To check available certificates:"
        echo "   security find-identity -v -p codesigning"
        echo ""
        echo "📝 Creating unsigned app for now..."
        echo "   (Will work with right-click → Open on macOS)"
        
        # Create ad-hoc signed version that won't show "damaged" error
        if [ -d "dist/Hive.app" ]; then
            echo "🔧 Applying ad-hoc signature to prevent 'damaged' error..."
            codesign --force --deep -s - dist/Hive.app
            echo "✅ Ad-hoc signed - users can right-click → Open to run"
        fi
        exit 0
    fi
fi

# Check if app exists
if [ ! -d "dist/Hive.app" ]; then
    echo "❌ Error: dist/Hive.app not found"
    echo "   Run ./scripts/package-macos.sh first"
    exit 1
fi

# Sign the binary first
echo "📝 Signing binary..."
codesign --force --deep --sign "$APPLE_SIGNING_IDENTITY" \
    --options runtime \
    --entitlements scripts/entitlements.plist \
    --timestamp=none \
    dist/Hive.app/Contents/MacOS/hive 2>&1 | grep -v "Warning: unable to build chain" || true

# Sign the app bundle
echo "📦 Signing app bundle..."
codesign --force --deep --sign "$APPLE_SIGNING_IDENTITY" \
    --options runtime \
    --entitlements scripts/entitlements.plist \
    --timestamp=none \
    dist/Hive.app 2>&1 | grep -v "Warning: unable to build chain" || true

# Verify signature
echo "✅ Verifying signature..."
codesign --verify --deep --strict dist/Hive.app
codesign -dv dist/Hive.app

echo "✅ Code signing complete!"

# Notarization (if credentials available)
if [ -n "$APPLE_ID" ] && [ -n "$APPLE_ID_PASSWORD" ]; then
    echo "📤 Starting notarization process..."
    
    # Create a zip for notarization (more reliable than DMG)
    echo "🗜️ Creating archive for notarization..."
    cd dist
    zip -r "Hive.app.zip" "Hive.app"
    cd ..
    
    # Submit for notarization
    echo "📨 Submitting to Apple for notarization..."
    xcrun notarytool submit "dist/Hive.app.zip" \
        --apple-id "$APPLE_ID" \
        --password "$APPLE_ID_PASSWORD" \
        --team-id "$APPLE_TEAM_ID" \
        --wait
    
    if [ $? -eq 0 ]; then
        echo "✅ Notarization successful!"
        
        # Staple the notarization
        echo "📎 Stapling notarization ticket..."
        xcrun stapler staple "dist/Hive.app"
        
        # Verify stapling
        echo "🔍 Verifying stapled notarization..."
        xcrun stapler validate "dist/Hive.app"
        
        echo "🎉 App is fully signed and notarized!"
    else
        echo "❌ Notarization failed"
        echo "   The app is still signed and will work, but may show warnings"
    fi
    
    # Clean up
    rm -f "dist/Hive.app.zip"
else
    echo "⚠️  Notarization skipped (credentials not set)"
    echo ""
    echo "To enable notarization, set environment variables:"
    echo "   export APPLE_ID='your-apple-id@email.com'"
    echo "   export APPLE_ID_PASSWORD='app-specific-password'"
    echo "   export APPLE_TEAM_ID='your-team-id'"
    echo ""
    echo "The app is signed and will work, but may show warnings until notarized."
fi