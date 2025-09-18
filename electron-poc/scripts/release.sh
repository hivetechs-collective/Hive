#!/usr/bin/env bash
# Release script for Hive Consensus
# Usage: ./scripts/release.sh [stable|beta|channel]
set -euo pipefail

CHANNEL=${1:-stable}
DEBUG_PORT=${PLAYWRIGHT_REMOTE_DEBUG_PORT:-61323}

if [ "${CI:-""}" = "true" ]; then
  WORKDIR="$PWD/electron-poc"
else
  WORKDIR="/Users/veronelazio/Developer/Private/hive/electron-poc"
fi

if [ ! -d "$WORKDIR" ]; then
  echo "❌ Expected workdir $WORKDIR does not exist" >&2
  exit 1
fi

cd "$WORKDIR"

printf "🏗  Building Hive Consensus for release (%s)…\n" "$CHANNEL"
PLAYWRIGHT_E2E=1 \
PLAYWRIGHT_REMOTE_DEBUG_PORT=$DEBUG_PORT \
PLAYWRIGHT_RUN_TESTS=1 \
npm run build:complete

printf "☁️  Uploading artifacts to Cloudflare R2 (%s)…\n" "$CHANNEL"
./scripts/upload-to-r2.sh "$CHANNEL"

printf "✅ Release pipeline finished: channel=%s\n" "$CHANNEL"
