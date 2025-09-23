#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 2 ]]; then
  cat <<'USAGE'
Usage: scripts/sign-notarize-macos.sh <path-to-app> <output-dmg>

Environment variables:
  SIGN_ID          Developer ID Application identity (required)
  NOTARY_PROFILE   notarytool keychain profile name (default: HiveNotaryProfile)
  VOLUME_NAME      DMG volume name (default: derived from app bundle)
  ENTITLEMENTS_PATH optional override path for entitlements plist
USAGE
  exit 64
fi

APP_INPUT_PATH=$1
DMG_OUTPUT_PATH=$2

if [[ -z "${SIGN_ID:-}" ]]; then
  echo "SIGN_ID environment variable is required." >&2
  exit 64
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)

NOTARY_PROFILE=${NOTARY_PROFILE:-HiveNotaryProfile}
ENTITLEMENTS=${ENTITLEMENTS_PATH:-$REPO_ROOT/scripts/entitlements.plist}

if [[ ! -f "$ENTITLEMENTS" ]]; then
  echo "Entitlements file not found at $ENTITLEMENTS" >&2
  exit 66
fi

normalize_path() {
  python3 -c 'import pathlib, sys; print(pathlib.Path(sys.argv[1]).expanduser().resolve())' "$1"
}

APP_PATH=$(normalize_path "$APP_INPUT_PATH")
DMG_PATH=$(normalize_path "$DMG_OUTPUT_PATH")

if [[ ! -d "$APP_PATH" ]]; then
  echo "App bundle not found at $APP_PATH" >&2
  exit 66
fi

APP_NAME=$(basename "$APP_PATH")
APP_DISPLAY_NAME=${APP_NAME%.app}
VOLUME_NAME=${VOLUME_NAME:-$APP_DISPLAY_NAME}

echo "🔐 Signing app bundle: $APP_PATH"

# Sign every Mach-O binary (executables, dylibs, native modules)
echo "🔍 Scanning for Mach-O binaries..."
find "$APP_PATH" -type f -print0 |
  while IFS= read -r -d '' file; do
    if [[ "$file" == *".framework/"* || "$file" == *".app/"* ]]; then
      continue
    fi
    if file "$file" | grep -q 'Mach-O'; then
      echo "  • codesign $(basename "$file")"
      codesign --force --options runtime --timestamp \
        --sign "$SIGN_ID" "$file"
    fi
  done

APP_DISPLAY_NAME=${APP_DISPLAY_NAME:-${APP_NAME%.app}}
APP_MAIN_BINARY="$APP_PATH/Contents/MacOS/$APP_DISPLAY_NAME"
if [[ -f "$APP_MAIN_BINARY" ]]; then
  echo "  • sealing main binary $(basename \"$APP_MAIN_BINARY\")"
  codesign --force --options runtime --timestamp \
    --sign "$SIGN_ID" "$APP_MAIN_BINARY"
fi

# Sign frameworks and helper apps at the directory level
if [[ -d "$APP_PATH/Contents/Frameworks" ]]; then
  find "$APP_PATH/Contents/Frameworks" -maxdepth 1 -type d \( -name '*.framework' -o -name '*.app' \) -print0 |
    while IFS= read -r -d '' bundle; do
      echo "  • sealing $(basename "$bundle")"
      if [[ "$bundle" == *.framework ]]; then
        FRAMEWORK_BASENAME=$(basename "$bundle" .framework)
        FRAMEWORK_VERSION_DIR="$bundle/Versions/A"
        FRAMEWORK_VERSION_BINARY="$FRAMEWORK_VERSION_DIR/$FRAMEWORK_BASENAME"

        if [[ -f "$FRAMEWORK_VERSION_BINARY" ]]; then
          codesign --force --options runtime --timestamp \
            --sign "$SIGN_ID" "$FRAMEWORK_VERSION_BINARY"
        fi

        if [[ -d "$FRAMEWORK_VERSION_DIR" ]]; then
          codesign --force --options runtime --timestamp \
            --sign "$SIGN_ID" "$FRAMEWORK_VERSION_DIR"
        fi

        # Skip signing the root directory explicitly; version directories carry the signature.
        continue
      fi

      codesign --force --options runtime --timestamp \
        --sign "$SIGN_ID" "$bundle"
    done
fi

# Sign Plugins (if any)
if [[ -d "$APP_PATH/Contents/PlugIns" ]]; then
  find "$APP_PATH/Contents/PlugIns" -maxdepth 1 -type d -name '*.plugin' -print0 |
    while IFS= read -r -d '' plugin; do
      echo "  • sealing plugin $(basename "$plugin")"
      codesign --force --options runtime --timestamp \
        --sign "$SIGN_ID" "$plugin"
    done
fi

# Sign the app bundle with entitlements
codesign --force --options runtime --timestamp \
  --entitlements "$ENTITLEMENTS" \
  --sign "$SIGN_ID" "$APP_PATH"

echo "📦 Building notarized DMG"

DMG_DIR=$(dirname "$DMG_PATH")
mkdir -p "$DMG_DIR"

STAGING_DIR=$(mktemp -d)
trap 'rm -rf "$STAGING_DIR"' EXIT

rsync -a "$APP_PATH" "$STAGING_DIR/"

hdiutil create -volname "$VOLUME_NAME" \
  -srcfolder "$STAGING_DIR" -ov -format UDZO "$DMG_PATH"

codesign --force --sign "$SIGN_ID" --timestamp "$DMG_PATH"

echo "📨 Submitting DMG for notarization..."
SUBMISSION_INFO=$(mktemp)
if ! xcrun notarytool submit "$DMG_PATH" --keychain-profile "$NOTARY_PROFILE" --wait | tee "$SUBMISSION_INFO"; then
  echo "Notarization submission failed" >&2
  if grep -Eq '^[[:space:]]*id:' "$SUBMISSION_INFO"; then
    SUBMISSION_ID=$(grep -E '^[[:space:]]*id:' "$SUBMISSION_INFO" | awk '{print $2}')
    if [[ -n "$SUBMISSION_ID" ]]; then
      echo "Fetching notarization log for submission $SUBMISSION_ID"
      xcrun notarytool log "$SUBMISSION_ID" --keychain-profile "$NOTARY_PROFILE" || true
    fi
  fi
  exit 1
fi

SUBMISSION_ID=$(grep -E '^[[:space:]]*id:' "$SUBMISSION_INFO" | awk '{print $2}')
if grep -Eq '^[[:space:]]*status:[[:space:]]+Invalid' "$SUBMISSION_INFO"; then
  echo "Notarization returned Invalid" >&2
  if [[ -n "$SUBMISSION_ID" ]]; then
    echo "Fetching notarization log for submission $SUBMISSION_ID"
    xcrun notarytool log "$SUBMISSION_ID" --keychain-profile "$NOTARY_PROFILE" || true
  fi
  exit 1
fi

echo "📎 Stapling tickets..."
xcrun stapler staple "$APP_PATH"
xcrun stapler staple "$DMG_PATH"

echo "🔍 Verifying Gatekeeper assessment"
spctl --assess --type exec --verbose "$APP_PATH"

echo "🔢 DMG checksum"
shasum -a 256 "$DMG_PATH"

echo "✅ Signing and notarization complete"
