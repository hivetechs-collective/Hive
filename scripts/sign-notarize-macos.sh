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
# Build a reusable base codesign command with optional keychain
declare -a CS_BASE
CS_BASE=(codesign --force --options runtime --timestamp)
if [[ -n "${HIVE_SIGNING_KEYCHAIN:-}" ]]; then
  CS_BASE+=(--keychain "$HIVE_SIGNING_KEYCHAIN")
fi

IDENTITY_SEARCH_ARGS=(-v -p codesigning)
if [[ -n "${HIVE_SIGNING_KEYCHAIN:-}" ]]; then
  IDENTITY_SEARCH_ARGS+=("$HIVE_SIGNING_KEYCHAIN")
fi

SIGN_IDENTITY=$(security find-identity "${IDENTITY_SEARCH_ARGS[@]}" | awk -F '"' -v wanted="$SIGN_ID" '($2 == wanted) {print $2}')
if [[ -z "$SIGN_IDENTITY" ]]; then
  echo "Developer ID identity '$SIGN_ID' not found in code-signing identities" >&2
  security find-identity "${IDENTITY_SEARCH_ARGS[@]}" >&2 || true
  exit 1
fi
echo "Using signing identity: $SIGN_IDENTITY"

if [[ ! -f "$ENTITLEMENTS" ]]; then
  echo "Entitlements file not found at $ENTITLEMENTS" >&2
  exit 66
fi

normalize_path() {
  python3 -c 'import pathlib, sys; print(pathlib.Path(sys.argv[1]).expanduser().resolve())' "$1"
}

verify_or_warn() {
  local target="$1"
  [[ -e "$target" ]] || return
  if ! codesign --verify --strict "$target" 2>&1 | sed 's/^/    /'; then
    echo "⚠️ codesign verification reported issues for $target" >&2
  fi
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
while IFS= read -r -d '' file; do
    if [[ "$file" == *.framework/* ]]; then
      framework_dir="${file%%.framework/*}.framework"
      framework_name=$(basename "$framework_dir" .framework)
      if [[ "$file" == "$framework_dir/$framework_name" && "$file" != *"/Versions/"* ]]; then
        continue
      fi
    fi
    if file "$file" | grep -q 'Mach-O'; then
      echo "  • codesign $(basename "$file")"
      "${CS_BASE[@]}" --sign "$SIGN_ID" "$file"
    fi
done < <(find "$APP_PATH" -type f -print0)

APP_DISPLAY_NAME=${APP_DISPLAY_NAME:-${APP_NAME%.app}}
APP_MAIN_BINARY="$APP_PATH/Contents/MacOS/$APP_DISPLAY_NAME"
if [[ -f "$APP_MAIN_BINARY" ]]; then
  echo "  • sealing main binary $(basename \"$APP_MAIN_BINARY\")"
  "${CS_BASE[@]}" --sign "$SIGN_ID" "$APP_MAIN_BINARY"
fi

# Sign frameworks and helper apps at the directory level
if [[ -d "$APP_PATH/Contents/Frameworks" ]]; then
  while IFS= read -r -d '' bundle; do
      echo "  • sealing $(basename "$bundle")"
      if [[ "$bundle" == *.framework ]]; then
        FRAMEWORK_BASENAME=$(basename "$bundle" .framework)
        FRAMEWORK_VERSION_DIR="$bundle/Versions/A"
        FRAMEWORK_VERSION_BINARY="$FRAMEWORK_VERSION_DIR/$FRAMEWORK_BASENAME"

        if [[ -f "$FRAMEWORK_VERSION_BINARY" ]]; then
          "${CS_BASE[@]}" --sign "$SIGN_ID" "$FRAMEWORK_VERSION_BINARY"
        fi

        if [[ -d "$FRAMEWORK_VERSION_DIR" ]]; then
          "${CS_BASE[@]}" --deep --sign "$SIGN_ID" "$FRAMEWORK_VERSION_DIR"
        else
          "${CS_BASE[@]}" --sign "$SIGN_ID" "$bundle"
        fi
      else
        "${CS_BASE[@]}" --sign "$SIGN_ID" "$bundle"
      fi
    done < <(find "$APP_PATH/Contents/Frameworks" -maxdepth 1 -type d \( -name '*.framework' -o -name '*.app' \) -print0)
fi

# Sign Plugins (if any)
if [[ -d "$APP_PATH/Contents/PlugIns" ]]; then
  while IFS= read -r -d '' plugin; do
      echo "  • sealing plugin $(basename "$plugin")"
      "${CS_BASE[@]}" --sign "$SIGN_ID" "$plugin"
    done < <(find "$APP_PATH/Contents/PlugIns" -maxdepth 1 -type d -name '*.plugin' -print0)
fi

# Ensure key embedded executables inherit required entitlements
echo "🔏 Applying entitlements to embedded executables (node/ttyd/git)"
EMBED_BASE="$APP_PATH/Contents/Resources/app.asar.unpacked/.webpack/main"
EMBED_EXECUTABLES=(
  "$EMBED_BASE/binaries/node"
  "$EMBED_BASE/binaries/ttyd"
  "$EMBED_BASE/binaries/git-bundle/bin/git"
)
for exe in "${EMBED_EXECUTABLES[@]}"; do
  if [[ -f "$exe" ]] && file "$exe" | grep -q 'Mach-O'; then
    echo "  • entitlements for $(basename "$exe")"
    "${CS_BASE[@]}" --entitlements "$ENTITLEMENTS" --sign "$SIGN_ID" "$exe"
    (codesign -d --entitlements :- "$exe" 2>/dev/null || true) | sed 's/^/      /'
  fi
 done
# Sign the app bundle with entitlements
"${CS_BASE[@]}" --entitlements "$ENTITLEMENTS" --sign "$SIGN_ID" "$APP_PATH"

echo "🧪 Verifying code signatures"
verify_or_warn "$APP_PATH/Contents/MacOS/$APP_DISPLAY_NAME"

if [[ -d "$APP_PATH/Contents/Frameworks" ]]; then
  find "$APP_PATH/Contents/Frameworks" -maxdepth 1 -type d \( -name '*.framework' -o -name '*.app' \) -print0 |
    while IFS= read -r -d '' bundle; do
      if [[ "$bundle" == *.framework ]]; then
        FRAMEWORK_BASENAME=$(basename "$bundle" .framework)
        FRAMEWORK_BINARY="$bundle/Versions/A/$FRAMEWORK_BASENAME"
        if [[ -f "$FRAMEWORK_BINARY" ]]; then
          verify_or_warn "$FRAMEWORK_BINARY"
        fi
      else
        HELPER_NAME=$(basename "$bundle" .app)
        HELPER_BINARY="$bundle/Contents/MacOS/$HELPER_NAME"
        if [[ -f "$HELPER_BINARY" ]]; then
          verify_or_warn "$HELPER_BINARY"
        fi
      fi
    done
fi

codesign -dv --verbose=4 "$APP_PATH" | sed 's/^/    /'

echo "📦 Building notarized DMG"

DMG_DIR=$(dirname "$DMG_PATH")
mkdir -p "$DMG_DIR"

STAGING_DIR=$(mktemp -d)
trap 'rm -rf "$STAGING_DIR"' EXIT

rsync -a "$APP_PATH" "$STAGING_DIR/"

hdiutil create -volname "$VOLUME_NAME" \
  -srcfolder "$STAGING_DIR" -ov -format UDZO "$DMG_PATH"

"${CS_BASE[@]}" --sign "$SIGN_ID" "$DMG_PATH"

echo "📨 Submitting DMG for notarization..."
SUBMISSION_INFO=$(mktemp)
if ! xcrun notarytool submit "$DMG_PATH" --keychain-profile "$NOTARY_PROFILE" --wait | tee "$SUBMISSION_INFO"; then
  echo "Notarization submission failed" >&2
  if grep -Eq '^[[:space:]]*id:' "$SUBMISSION_INFO"; then
    SUBMISSION_ID=$(grep -E '^[[:space:]]*id:' "$SUBMISSION_INFO" | head -n1 | awk '{print $2}' | tr -d '[:space:]')
    if [[ -n "$SUBMISSION_ID" ]]; then
      echo "Fetching notarization log for submission $SUBMISSION_ID"
      NOTARY_LOG=$(mktemp)
      if xcrun notarytool log "$SUBMISSION_ID" "$NOTARY_LOG" --keychain-profile "$NOTARY_PROFILE" --output-format json; then
        cat "$NOTARY_LOG"
      fi
      rm -f "$NOTARY_LOG"
    fi
  fi
  exit 1
fi

SUBMISSION_ID=$(grep -E '^[[:space:]]*id:' "$SUBMISSION_INFO" | head -n1 | awk '{print $2}' | tr -d '[:space:]')
if grep -Eq '^[[:space:]]*status:[[:space:]]+Invalid' "$SUBMISSION_INFO"; then
  echo "Notarization returned Invalid" >&2
  if [[ -n "$SUBMISSION_ID" ]]; then
    echo "Fetching notarization log for submission $SUBMISSION_ID"
    NOTARY_LOG=$(mktemp)
    if xcrun notarytool log "$SUBMISSION_ID" "$NOTARY_LOG" --keychain-profile "$NOTARY_PROFILE" --output-format json; then
      cat "$NOTARY_LOG"
    fi
    rm -f "$NOTARY_LOG"
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
