#!/usr/bin/env bash

# Copyright (c) 2026 Eclipse ThreadX contributors
#
# SPDX-License-Identifier: MIT

# Fail on error
set -e

BUILD_TYPE="Debug"
CLEAN=false
REBUILD=false

# Directory resolution
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
TARGET_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$TARGET_DIR/build"

# Print banner
echo "=========================================="
echo " PolarFire SoC Icicle Kit - Build (Bash)  "
echo "=========================================="
echo "Target Dir: $TARGET_DIR"
echo "Build Dir:  $BUILD_DIR"
echo "Build Type: $BUILD_TYPE"
echo ""

# Parse options
while [[ $# -gt 0 ]]; do
    case "$1" in
        --clean)
            CLEAN=true
            shift
            ;;
        --rebuild)
            REBUILD=true
            shift
            ;;
        --release)
            BUILD_TYPE="Release"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--clean] [--rebuild] [--release]"
            exit 1
            ;;
    esac
done

if [ "$CLEAN" = true ] || [ "$REBUILD" = true ]; then
    echo "[INFO] Cleaning build directory..."
    if [ -d "$BUILD_DIR" ]; then
        rm -rf "$BUILD_DIR"
    fi
fi

mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

echo "[INFO] Configuring CMake..."
if command -v ninja &>/dev/null; then
    cmake -G Ninja -DCMAKE_BUILD_TYPE="$BUILD_TYPE" "$TARGET_DIR"
    echo "[INFO] Building with Ninja..."
    ninja
else
    cmake -DCMAKE_BUILD_TYPE="$BUILD_TYPE" "$TARGET_DIR"
    echo "[INFO] Building..."
    cmake --build .
fi

echo ""
echo "=========================================="
echo "[OK] Build completed successfully!"
echo "=========================================="
