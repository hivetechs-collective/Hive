#!/usr/bin/env bash

set -euo pipefail

# Local helper to sign + notarize the DMG produced by `npm run build:complete`.
#
# Usage:
#   npm run sign:notarize:local
# or
#   SIGN_ID="Developer ID Application: ..." NOTARY_PROFILE=HiveNotaryProfile npm run sign:notarize:local
#
# Optional arg: path to an existing DMG to sign instead of auto-detecting
#   ./scripts/sign-notarize-local.sh /path/to/Hive\ Consensus.dmg

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ELECTRON_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
REPO_ROOT=$(cd "$ELECTRON_DIR/.." && pwd)

DMG_PATH_ARG=${1:-}

if [[ -n "$DMG_PATH_ARG" ]]; then
  DMG_PATH=$(python3 - <<'PY'
import pathlib,sys
print(pathlib.Path(sys.argv[1]).expanduser().resolve())
PY
"$DMG_PATH_ARG")
else
  # Auto-detect DMG built by Electron Forge
  DMG_PATH=$(ls "$ELECTRON_DIR"/out/make/*.dmg 2>/dev/null | head -n 1 || true)
fi

if [[ -z "${DMG_PATH:-}" || ! -f "$DMG_PATH" ]]; then
  echo "❌ No DMG found. Expected at $ELECTRON_DIR/out/make/*.dmg or pass path as arg." >&2
  echo "   Build first with: npm run build:complete" >&2
  exit 1
fi

echo "📦 Target DMG: $DMG_PATH"

# Try to reuse an extracted .app if present in out/*-darwin-*/
APP_PATH=$(ls -d "$ELECTRON_DIR"/out/*-darwin-*/"Hive Consensus.app" 2>/dev/null | head -n 1 || true)
MOUNT_DIR=""
APP_TMP_DIR=""

if [[ -z "${APP_PATH:-}" ]]; then
  echo "📂 Extracting app bundle from DMG…"
  MOUNT_DIR=$(mktemp -d)
  APP_TMP_DIR=$(mktemp -d)
  trap '(
    [[ -n "$MOUNT_DIR" && -d "$MOUNT_DIR" ]] && hdiutil detach "$MOUNT_DIR" >/dev/null 2>&1 || true
    [[ -n "$MOUNT_DIR" ]] && rm -rf "$MOUNT_DIR"
    [[ -n "$APP_TMP_DIR" ]] && rm -rf "$APP_TMP_DIR"
  )' EXIT

  hdiutil attach "$DMG_PATH" -mountpoint "$MOUNT_DIR" -nobrowse
  if [[ ! -d "$MOUNT_DIR/Hive Consensus.app" ]]; then
    echo "❌ Could not find 'Hive Consensus.app' inside DMG" >&2
    exit 1
  fi
  ditto "$MOUNT_DIR/Hive Consensus.app" "$APP_TMP_DIR/Hive Consensus.app"
  hdiutil detach "$MOUNT_DIR"
  APP_PATH="$APP_TMP_DIR/Hive Consensus.app"
fi

if [[ ! -d "$APP_PATH" ]]; then
  echo "❌ App bundle path not found: $APP_PATH" >&2
  exit 1
fi

# Provide sensible defaults; can be overridden by caller
export SIGN_ID="${SIGN_ID:-Developer ID Application: HiveTechs Collective LLC (FWBLB27H52)}"
export NOTARY_PROFILE="${NOTARY_PROFILE:-HiveNotaryProfile}"

echo "🔐 Using signing identity: $SIGN_ID"
echo "🔑 Using notary profile:  $NOTARY_PROFILE"

# Delegate actual deep-sign + notarize to the shared script at repo root
"$REPO_ROOT"/scripts/sign-notarize-macos.sh "$APP_PATH" "$DMG_PATH"

echo "✅ DMG signed, notarized and stapled: $DMG_PATH"

