#!/usr/bin/env bash
# OS / arch detection for CI and user-facing install scripts.
# Emits normalized identifiers used throughout scripts/ci/* and artifact names.

detect_os() {
  local uname_s
  uname_s="$(uname -s)"
  case "$uname_s" in
    Linux*)  echo "linux" ;;
    Darwin*) echo "macos" ;;
    MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
    *) echo "unknown" ;;
  esac
}

detect_arch() {
  local uname_m
  uname_m="$(uname -m)"
  case "$uname_m" in
    aarch64|arm64) echo "arm64" ;;
    x86_64|amd64)  echo "amd64" ;;
    *) echo "$uname_m" ;;
  esac
}

# Normalized platform id used by CI (e.g. linux-arm64, windows-arm64, android-arm64).
detect_platform() {
  printf '%s-%s\n' "$(detect_os)" "$(detect_arch)"
}
