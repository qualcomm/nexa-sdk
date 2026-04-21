#!/usr/bin/env bash
# Package a build output into geniex-${KIND}-${PLATFORM}.zip at the repo root.
#
# Environment inputs:
#   KIND        (required)  sdk | cli | wheel
#   PLATFORM    (required)  e.g. linux-arm64, windows-arm64, android-arm64
#   OUT_DIR     (optional)  Destination directory for the archive. Default: repo root.

set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../lib/common.sh
source "$SCRIPT_DIR/../lib/common.sh"

require_env KIND
require_env PLATFORM

OUT_DIR="${OUT_DIR:-.}"
mkdir -p "$OUT_DIR"

archive="$(cd "$OUT_DIR" && pwd)/geniex-${KIND}-${PLATFORM}.zip"
rm -f "$archive"

case "$KIND" in
  sdk)
    (cd sdk/pkg-geniex && run 7z a "$archive" ./*)
    ;;
  cli)
    # Assemble a runnable bundle: cli binary + SDK libs.
    stage="dist/cli-${PLATFORM}"
    rm -rf "$stage"
    mkdir -p "$stage"
    if [[ -f cli/build/geniex ]]; then
      cp cli/build/geniex "$stage/"
    elif [[ -f cli/build/geniex.exe ]]; then
      cp cli/build/geniex.exe "$stage/"
    else
      log_die "CLI binary not found under cli/build/"
    fi
    cp -r sdk/pkg-geniex/lib "$stage/lib"
    (cd dist && run 7z a "$archive" "cli-${PLATFORM}/")
    ;;
  wheel)
    # Wheels are already in dist/*.whl — just repackage as a zip for uniform release flow.
    shopt -s nullglob
    wheels=(dist/*.whl)
    shopt -u nullglob
    if [[ ${#wheels[@]} -eq 0 ]]; then
      log_die "No .whl files found under dist/"
    fi
    run 7z a "$archive" "${wheels[@]}"
    ;;
  *)
    log_die "Unknown KIND='$KIND' (expected: sdk|cli|wheel)"
    ;;
esac

log_info "Packaged $archive"
