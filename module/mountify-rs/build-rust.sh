#!/bin/bash
# build-rust.sh - Cross-compile mountify Rust core for Android
# Requires: Android NDK, Rust with Android targets installed
#
# Usage:
#   export ANDROID_NDK_HOME=/path/to/android-ndk
#   ./build-rust.sh [target]
#
# Targets: arm64 (default), arm, x86, x86_64

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Detect NDK
NDK="${ANDROID_NDK_HOME:-${ANDROID_HOME:-$HOME/Android/Sdk}/ndk}"
if [ ! -d "$NDK" ]; then
    # Try to find the latest NDK
    NDK=$(ls -d "$NDK"*/ 2>/dev/null | sort -V | tail -1 || true)
fi

if [ ! -d "$NDK" ]; then
    echo "Error: Android NDK not found."
    echo "Set ANDROID_NDK_HOME or install NDK via sdkmanager."
    exit 1
fi

NDK=$(cd "$NDK" && pwd)
echo "Using NDK: $NDK"

# Add NDK toolchain to PATH
TOOLCHAIN="$NDK/toolchains/llvm/prebuilt"
case "$(uname -s)" in
    Linux)  TOOLCHAIN="$TOOLCHAIN/linux-x86_64/bin" ;;
    Darwin) TOOLCHAIN="$TOOLCHAIN/darwin-x86_64/bin" ;;
    MINGW*|MSYS*) TOOLCHAIN="$TOOLCHAIN/windows-x86_64/bin" ;;
    *)      echo "Unsupported OS"; exit 1 ;;
esac

export PATH="$TOOLCHAIN:$PATH"

# Install Rust targets if missing
rustup target list --installed 2>/dev/null | grep -q "aarch64-linux-android" || rustup target add aarch64-linux-android
rustup target list --installed 2>/dev/null | grep -q "armv7-linux-androideabi" || rustup target add armv7-linux-androideabi

TARGET="${1:-arm64}"

case "$TARGET" in
    arm64|aarch64)
        RUST_TARGET="aarch64-linux-android"
        ;;
    arm|armv7)
        RUST_TARGET="armv7-linux-androideabi"
        ;;
    x86)
        RUST_TARGET="i686-linux-android"
        ;;
    x86_64)
        RUST_TARGET="x86_64-linux-android"
        ;;
    *)
        echo "Unknown target: $TARGET"
        echo "Usage: $0 [arm64|arm|x86|x86_64]"
        exit 1
        ;;
esac

echo "Building for $RUST_TARGET..."

# Configure linker via environment
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="aarch64-linux-android21-clang"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="armv7a-linux-androideabi21-clang"
export CARGO_TARGET_I686_LINUX_ANDROID_LINKER="i686-linux-android21-clang"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="x86_64-linux-android21-clang"

# Build release
RUSTFLAGS="-C link-arg=-fuse-ld=lld" cargo build --release --target "$RUST_TARGET"

echo ""
echo "Build complete!"
echo "Binary: target/$RUST_TARGET/release/mountify"
file "target/$RUST_TARGET/release/mountify"
