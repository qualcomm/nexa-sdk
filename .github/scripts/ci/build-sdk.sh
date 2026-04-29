#!/usr/bin/env bash
# Build the GenieX SDK on Linux. Drives CMakePresets.json rather than passing
# toolchain/options ad-hoc so the CI build stays in sync with the presets
# documented in docs/build.md.
#
# Environment inputs:
#   GENIEX_VERSION      (required)  Version string baked into binaries.
#   PLATFORM            (optional)  linux-arm64 | android-arm64. Default:
#                                   linux-arm64. Selects the CMake preset.
#   INSTALL_PREFIX      (optional)  Default: sdk/pkg-geniex.
#   EXTRA_CMAKE_FLAGS   (optional)  Appended verbatim to `cmake --preset`.

set -euo pipefail

: "${GENIEX_VERSION:?GENIEX_VERSION is required}"
PLATFORM="${PLATFORM:-linux-arm64}"
INSTALL_PREFIX="${INSTALL_PREFIX:-sdk/pkg-geniex}"
EXTRA_CMAKE_FLAGS="${EXTRA_CMAKE_FLAGS:-}"

case "$PLATFORM" in
  linux-arm64)
    PRESET="arm64-linux-snapdragon-release"
    # Linux/arm64 runtime images (GHCR publish target) don't ship with the
    # Hexagon NPU stack; disable the hexagon backend so the build doesn't
    # demand HEXAGON_SDK_ROOT / Hexagon Tools on the ubuntu-latest runner.
    # GENIEX_MODEL_MANAGER is disabled because the Rust crate has no
    # cross-build wiring yet (same as android; tracked as #222).
    EXTRA_CMAKE_FLAGS="$EXTRA_CMAKE_FLAGS -DGGML_HEXAGON=OFF -DGENIEX_MODEL_MANAGER=OFF"
    ;;
  android-arm64)
    PRESET="arm64-android-snapdragon-release"
    EXTRA_CMAKE_FLAGS="$EXTRA_CMAKE_FLAGS -DGENIEX_MODEL_MANAGER=OFF"
    ;;
  *)
    echo "Unsupported PLATFORM: $PLATFORM" >&2
    exit 1
    ;;
esac

BUILD_DIR="sdk/build-${PRESET}"

set -x
# shellcheck disable=SC2086  # EXTRA_CMAKE_FLAGS is intentionally word-split.
cmake -S sdk --preset "$PRESET" \
  -DGENIEX_VERSION="$GENIEX_VERSION" \
  -DGENIEX_TEST=OFF \
  -DGENIEX_DL=ON \
  -DCMAKE_C_COMPILER_LAUNCHER=ccache \
  -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \
  $EXTRA_CMAKE_FLAGS

cmake --build "$BUILD_DIR" -j "$(nproc)"
cmake --install "$BUILD_DIR" --prefix "$INSTALL_PREFIX"
