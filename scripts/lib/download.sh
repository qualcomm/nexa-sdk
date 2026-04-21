#!/usr/bin/env bash
# Download helpers. curl preferred, wget fallback. Reusable by user install scripts.

download_file() {
  local url="$1" dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --retry 3 -o "$dest" "$url"
  elif command -v wget >/dev/null 2>&1; then
    wget -q --tries=3 -O "$dest" "$url"
  else
    echo "[ERROR] neither curl nor wget available" >&2
    return 1
  fi
}

verify_sha256() {
  local file="$1" expected="$2"
  local actual
  actual="$(sha256sum "$file" | awk '{print $1}')"
  if [[ "$actual" != "$expected" ]]; then
    echo "[ERROR] sha256 mismatch for $file: expected=$expected actual=$actual" >&2
    return 1
  fi
}
