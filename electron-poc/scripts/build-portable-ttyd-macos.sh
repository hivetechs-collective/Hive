#!/usr/bin/env bash
set -euo pipefail

# Build a portable (pluginless) ttyd for macOS (arm64) that does not rely on Homebrew Cellar plugin scanning.
# Result: places the binary (and needed dylibs if any) under electron-poc/vendor/ttyd/darwin-arm64/.
#
# Requirements: Xcode Command Line Tools, Homebrew, cmake, pkg-config
#
# What this does:
# - Builds libwebsockets with plugins disabled and libuv enabled
# - Builds ttyd against that libwebsockets
# - Copies the resulting ttyd (and libwebsockets.dylib if shared) into vendor path
# - The app’s build script will prefer this vendor binary and bundle any vendor libs
#
# Usage:
#   cd electron-poc
#   ./scripts/build-portable-ttyd-macos.sh

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
VENDOR_DIR="$ROOT_DIR/vendor/ttyd/darwin-arm64"
BUILD_DIR="$ROOT_DIR/vendor/.build-ttyd"

echo "📦 Preparing build directories"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR" "$VENDOR_DIR"

echo "🔧 Ensuring build dependencies (brew)"
command -v brew >/dev/null 2>&1 || { echo "Homebrew is required" >&2; exit 1; }
brew list --versions cmake >/dev/null 2>&1 || brew install cmake
brew list --versions pkg-config >/dev/null 2>&1 || brew install pkg-config
brew list --versions libuv >/dev/null 2>&1 || brew install libuv
brew list --versions openssl@3 >/dev/null 2>&1 || brew install openssl@3

OPENSSL_ROOT=$(brew --prefix openssl@3)
LIBUV_ROOT=$(brew --prefix libuv)

pushd "$BUILD_DIR" >/dev/null

echo "⬇️  Cloning libwebsockets"
git clone --depth 1 -b v4.4-stable https://github.com/warmcat/libwebsockets.git

echo "🏗️  Building libwebsockets (plugins OFF, libuv ON)"
mkdir -p libwebsockets/build && cd libwebsockets/build
cmake .. \
  -DCMAKE_BUILD_TYPE=Release \
  -DLWS_WITH_PLUGINS=OFF \
  -DLWS_WITH_EVLIB_PLUGINS=OFF \
  -DLWS_WITH_LIBUV=ON \
  -DLWS_STATIC_PIC=ON \
  -DLWS_WITH_SHARED=ON \
  -DLWS_WITH_STATIC=OFF \
  -DLWS_WITH_TESTAPPS=OFF \
  -DLWS_WITH_MINIMAL_EXAMPLES=OFF \
  -DLWS_WITHOUT_TESTAPPS=ON \
  -DLWS_WITHOUT_TEST_SERVER=ON \
  -DLWS_WITHOUT_TEST_CLIENT=ON \
  -DLWS_WITHOUT_TEST_PING=ON \
  -DOPENSSL_ROOT_DIR="$OPENSSL_ROOT" \
  -DOPENSSL_LIBRARIES="$OPENSSL_ROOT/lib" \
  -DCMAKE_OSX_ARCHITECTURES=arm64
cmake --build . --parallel
cmake --install . --prefix "$BUILD_DIR/prefix"
cd ../..

echo "⬇️  Cloning ttyd"
git clone --depth 1 https://github.com/tsl0922/ttyd.git

echo "🏗️  Building ttyd against built libwebsockets"
mkdir -p ttyd/build && cd ttyd/build
PKG_CONFIG_PATH="$BUILD_DIR/prefix/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
LDFLAGS="-L$BUILD_DIR/prefix/lib ${LDFLAGS:-}" \
CPPFLAGS="-I$BUILD_DIR/prefix/include ${CPPFLAGS:-}" \
cmake .. \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_PREFIX_PATH="$BUILD_DIR/prefix" \
  -DLibwebsockets_DIR="$BUILD_DIR/prefix/lib/cmake/libwebsockets" \
  -DCMAKE_OSX_ARCHITECTURES=arm64 \
  -DOPENSSL_ROOT_DIR="$OPENSSL_ROOT"
cmake --build . --parallel

echo "📥 Copying artifacts to vendor directory"
TTYD_BIN="$(pwd)/ttyd"
if [[ ! -x "$TTYD_BIN" ]]; then
  echo "Failed to build ttyd binary" >&2; exit 1
fi
cp -f "$TTYD_BIN" "$VENDOR_DIR/ttyd"
chmod 755 "$VENDOR_DIR/ttyd"

# For static build, no dylibs are required. If a shared lib was still produced, vendor it.
LWS_DYLIB=$(find "$BUILD_DIR/prefix/lib" -maxdepth 1 -name 'libwebsockets*.dylib' | head -n1 || true)
if [[ -f "$LWS_DYLIB" ]]; then
  mkdir -p "$VENDOR_DIR/lib"
  cp -f "$LWS_DYLIB" "$VENDOR_DIR/lib/"
  echo "Vendored $(basename "$LWS_DYLIB")"
fi

popd >/dev/null
echo "✅ Portable ttyd ready at: $VENDOR_DIR/ttyd"
echo "   If libwebsockets.dylib was built, it is placed under: $VENDOR_DIR/lib"
echo "   Next: cd electron-poc && npm run build:complete to test locally"
