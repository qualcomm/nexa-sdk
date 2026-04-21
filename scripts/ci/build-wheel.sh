#!/usr/bin/env bash
# Build the Python wheel for the geniex bindings.
# Assumes the SDK has already been installed to SDK_PKG_DIR.
#
# Environment inputs:
#   SDK_PKG_DIR         (optional)  Default: sdk/pkg-geniex.
#   DIST_DIR            (optional)  Default: dist.
#   ACTIVATE_MAMBA      (optional)  '1' to activate micromamba env (CI default).

set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib/common.sh
source "$SCRIPT_DIR/../lib/common.sh"

SDK_PKG_DIR="${SDK_PKG_DIR:-sdk/pkg-geniex}"
DIST_DIR="${DIST_DIR:-dist}"
BINDING_DIR="bindings/python"
LIB_STAGE="$BINDING_DIR/geniex/lib"

if [[ "${ACTIVATE_MAMBA:-1}" == "1" ]]; then
  # shellcheck source=./mamba-activate.sh
  source "$SCRIPT_DIR/mamba-activate.sh"
fi

if [[ ! -d "$SDK_PKG_DIR/lib" ]]; then
  log_die "SDK libs not found at $SDK_PKG_DIR/lib — build the SDK first"
fi

run rm -rf "$LIB_STAGE"
run cp -r "$SDK_PKG_DIR/lib" "$LIB_STAGE"

run pip install --quiet build
run python -m build --wheel "$BINDING_DIR" -o "$DIST_DIR"
