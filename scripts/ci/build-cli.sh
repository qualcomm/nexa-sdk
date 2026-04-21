#!/usr/bin/env bash
# Build the GenieX CLI via Bazel (local SDK mode).
#
# Environment inputs:
#   GENIEX_VERSION      (required)  Propagated via --define=VERSION to Bazel.
#   ACTIVATE_MAMBA      (optional)  '1' to activate micromamba env (Linux CI default).

set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib/common.sh
source "$SCRIPT_DIR/../lib/common.sh"

require_env GENIEX_VERSION

if [[ "${ACTIVATE_MAMBA:-0}" == "1" ]]; then
  # shellcheck source=./mamba-activate.sh
  source "$SCRIPT_DIR/mamba-activate.sh"
fi

run bazelisk build //cli:artifact \
  --//sdk:sdk_type=local \
  --define=VERSION="$GENIEX_VERSION"
