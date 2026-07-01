#!/usr/bin/env bash
# Cross-compile hamlib for all Android ABIs using the NDK.
# Run once before building with cargo-ndk.
# Prerequisites: autoconf, automake, libtool, Android NDK
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LIB_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
HAMLIB_SRC="$LIB_DIR/hamlib"
PREBUILT_BASE="$LIB_DIR/hamlib-prebuilt/android"
API=28

# Find the NDK — prefer explicit env var, fall back to ANDROID_HOME/ndk/<latest>
if [ -n "${ANDROID_NDK_HOME:-}" ]; then
    NDK="$ANDROID_NDK_HOME"
elif [ -n "${ANDROID_HOME:-}" ] && [ -d "$ANDROID_HOME/ndk" ]; then
    NDK="$ANDROID_HOME/ndk/$(ls -1 "$ANDROID_HOME/ndk" | sort -V | tail -1)"
else
    echo "Error: set ANDROID_NDK_HOME or ANDROID_HOME" >&2
    exit 1
fi

TOOLCHAIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64"
echo "Using NDK: $NDK"

# Bootstrap hamlib (only needed for git checkout, not release tarballs)
if [ ! -f "$HAMLIB_SRC/configure" ]; then
    echo "Bootstrapping hamlib..."
    (cd "$HAMLIB_SRC" && ./bootstrap)
fi

# ABI → GNU cross-compilation target
declare -A TARGETS=(
    ["arm64-v8a"]="aarch64-linux-android"
    ["armeabi-v7a"]="armv7a-linux-androideabi"
    ["x86_64"]="x86_64-linux-android"
    ["x86"]="i686-linux-android"
)

for ABI in "${!TARGETS[@]}"; do
    TARGET="${TARGETS[$ABI]}"
    OUTDIR="$PREBUILT_BASE/$ABI"

    if [ -f "$OUTDIR/lib/libhamlib.a" ]; then
        echo "[$ABI] already built — skipping (delete $OUTDIR to rebuild)"
        continue
    fi

    echo "[$ABI] building hamlib for $TARGET (API $API)..."

    BUILD_DIR="/tmp/hamlib-build-$ABI"
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR" "$OUTDIR"

    (
        cd "$BUILD_DIR"

        export AR="$TOOLCHAIN/bin/llvm-ar"
        export AS="$TOOLCHAIN/bin/llvm-as"
        export CC="$TOOLCHAIN/bin/${TARGET}${API}-clang"
        export CXX="$TOOLCHAIN/bin/${TARGET}${API}-clang++"
        export LD="$TOOLCHAIN/bin/ld"
        export RANLIB="$TOOLCHAIN/bin/llvm-ranlib"
        export STRIP="$TOOLCHAIN/bin/llvm-strip"

        "$HAMLIB_SRC/configure" \
            --host="$TARGET" \
            --prefix="$OUTDIR" \
            --without-libusb \
            --enable-static \
            --disable-shared \
            --quiet

        make -j"$(nproc)" install
    )

    echo "[$ABI] done → $OUTDIR/lib/libhamlib.a"
done

echo "All ABIs built successfully."
