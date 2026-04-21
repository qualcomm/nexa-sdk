#!/usr/bin/env bash
# Shared helpers for CI and user-facing install scripts.
# Must stay dependency-free: only POSIX tools + bash builtins.

set -euo pipefail

log_info()  { printf '[INFO]  %s\n' "$*"; }
log_warn()  { printf '[WARN]  %s\n' "$*" >&2; }
log_error() { printf '[ERROR] %s\n' "$*" >&2; }
log_die()   { log_error "$*"; exit 1; }

require_env() {
  local name="$1"
  if [[ -z "${!name:-}" ]]; then
    log_die "Environment variable '$name' is required"
  fi
}

# Echo commands before running them (parity with `set -x` but tidier output).
run() {
  printf '+ %s\n' "$*"
  "$@"
}
