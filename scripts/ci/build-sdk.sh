#!/usr/bin/env bash
# Build the GenieX SDK on Linux (native or cross to linux-arm64).
#
# Environment inputs:
#   GENIEX_VERSION      (required)  Version string baked into binaries.
#   BUILD_TYPE          (optional)  CMake build type, default: Release.
#   TOOLCHAIN_FILE      (optional)  Default: sdk/cmake/arm64-linux-gnu.cmake.
#   BUILD_DIR           (optional)  Default: sdk/build.
#   INSTALL_PREFIX      (optional)  Default: sdk/pkg-geniex.
#   EXTRA_CMAKE_FLAGS   (optional)  Appended verbatim to `cmake -B`.
#   ACTIVATE_MAMBA      (optional)  '1' to activate micromamba env (CI default).

set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib/common.sh
source "$SCRIPT_DIR/../lib/common.sh"

require_env GENIEX_VERSION

BUILD_TYPE="${BUILD_TYPE:-Release}"
TOOLCHAIN_FILE="${TOOLCHAIN_FILE:-sdk/cmake/arm64-linux-gnu.cmake}"
BUILD_DIR="${BUILD_DIR:-sdk/build}"
INSTALL_PREFIX="${INSTALL_PREFIX:-sdk/pkg-geniex}"
EXTRA_CMAKE_FLAGS="${EXTRA_CMAKE_FLAGS:-}"

if [[ "${ACTIVATE_MAMBA:-1}" == "1" ]]; then
  # shellcheck source=./mamba-activate.sh
  source "$SCRIPT_DIR/mamba-activate.sh"
fi

# shellcheck disable=SC2086  # EXTRA_CMAKE_FLAGS is intentionally word-split.
run cmake -B "$BUILD_DIR" -S sdk \
  -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
  -DCMAKE_TOOLCHAIN_FILE="$TOOLCHAIN_FILE" \
  -DGENIEX_VERSION="$GENIEX_VERSION" \
  -DGENIEX_TEST=OFF \
  -DGENIEX_DEBUG=OFF \
  -DGENIEX_DL=ON \
  -DGENIEX_PLUGIN_LLAMA_CPP=ON \
  -DGENIEX_PLUGIN_QAIRT=ON \
  -DGGML_OPENCL=OFF \
  $EXTRA_CMAKE_FLAGS

run cmake --build "$BUILD_DIR" -j
run cmake --install "$BUILD_DIR" --prefix "$INSTALL_PREFIX"
