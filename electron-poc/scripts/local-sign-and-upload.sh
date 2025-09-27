#!/usr/bin/env bash

set -euo pipefail

# One-shot local signer + uploader with embedded values used in our release flow.
#
# Usage:
#   ./scripts/local-sign-and-upload.sh [stable|beta] [optional:/path/to/Hive\ Consensus.dmg]
#
# Behavior:
#   - Detects the DMG under out/make if not provided
#   - Signs + notarizes the DMG using our standard identity/profile
#   - Uploads to R2 via Wrangler to the given channel (default: stable)

CHANNEL=${1:-stable}
DMG_ARG=${2:-}

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ELECTRON_DIR=$(cd "$SCRIPT_DIR/.." && pwd)

DMG_PATH=""
if [[ -n "$DMG_ARG" ]]; then
  DMG_PATH=$(python3 - <<'PY'
import pathlib,sys
print(pathlib.Path(sys.argv[1]).expanduser().resolve())
PY
"$DMG_ARG")
else
  DMG_PATH=$(ls "$ELECTRON_DIR"/out/make/*.dmg 2>/dev/null | head -n 1 || true)
fi

if [[ -z "$DMG_PATH" || ! -f "$DMG_PATH" ]]; then
  echo "❌ DMG not found." >&2
  echo "   Expected at $ELECTRON_DIR/out/make/*.dmg or pass an explicit path as arg 2." >&2
  echo "   Build first with: npm run build:complete" >&2
  exit 1
fi

echo "📦 Using DMG: $DMG_PATH"
echo "📺 Channel:  $CHANNEL"

# Prereqs
command -v xcrun >/dev/null 2>&1 || { echo "❌ xcrun not found (install Xcode Command Line Tools)" >&2; exit 1; }
command -v wrangler >/dev/null 2>&1 || { echo "❌ wrangler not found. Install via: npm i -g wrangler && wrangler login" >&2; exit 1; }

# Embedded values (match CI and local docs)
export SIGN_ID="Developer ID Application: HiveTechs Collective LLC (FWBLB27H52)"
export NOTARY_PROFILE="HiveNotaryProfile"

echo "🔐 Signing + Notarizing with:"
echo "    SIGN_ID=$SIGN_ID"
echo "    NOTARY_PROFILE=$NOTARY_PROFILE"

pushd "$ELECTRON_DIR" >/dev/null

# Run the local wrapper and pass the DMG explicitly to be exact
npm run sign:notarize:local -- "$DMG_PATH"

echo "☁️  Uploading DMG to R2 via Wrangler ($CHANNEL)…"
"$SCRIPT_DIR"/upload-dmg-to-r2.sh "$CHANNEL"

echo "✅ Local sign + notarize + upload complete"
popd >/dev/null

