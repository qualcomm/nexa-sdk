#!/usr/bin/env bash
# Usage: dedup_tar.sh <bsdtar> <gawk> <output.tar> <input1.tar> [...]
# Relies only on the explicitly-passed binaries + bash built-ins so it
# works under Bazel's empty-env sandbox.
set -o pipefail -o errexit

bsdtar="$1"
gawk="$2"
output="$3"
shift 3

mtree="${output}.mtree"
: > "$mtree"

usrmerge=(
    -s ',^\./bin/,./usr/bin/,'
    -s ',^\./sbin/,./usr/sbin/,'
    -s ',^\./lib/,./usr/lib/,'
    -s ',^\./bin$,./usr/bin,'
    -s ',^\./sbin$,./usr/sbin,'
    -s ',^\./lib$,./usr/lib,'
)

for f in "$@"; do
    "$bsdtar" -tf "$f" \
        | "$gawk" '
            { sub(/^\.\/bin\//,  "./usr/bin/")
              sub(/^\.\/sbin\//, "./usr/sbin/")
              sub(/^\.\/lib\//,  "./usr/lib/")
              sub(/^\.\/bin$/,   "./usr/bin")
              sub(/^\.\/sbin$/,  "./usr/sbin")
              sub(/^\.\/lib$/,   "./usr/lib")
              print
            }' >> "$mtree"
done

inputs=()
for f in "$@"; do
    inputs+=("@$f")
done

"$bsdtar" --confirmation -P -s ',^\([^/.]\),./\1,' \
    "${usrmerge[@]}" \
    --create --file "$output" \
    "${inputs[@]}" \
    2< <("$gawk" '
function normalize(p) {
    if (p == "") return "/"
    if (p ~ /^(\/|\.\/)/) { return p }
    return "./" p
}
{
    key = normalize($1)
    count[key]++
    files[NR] = $1
    keys[NR] = key
}
END {
    ORS=""
    for (i = 1; i <= NR; i++) {
        seen[keys[i]]++
        keep = "n"
        if (count[keys[i]] == seen[keys[i]]) { keep = "y" }
        for (j = 0; j < 31; j++) print keep
        fflush()
    }
}' "$mtree")
