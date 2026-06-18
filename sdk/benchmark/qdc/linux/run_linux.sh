#!/bin/bash
# geniex-bench entry script for QDC Linux IoT (BASH framework).
#
# QDC extracts the artifact zip to /data/local/tmp/TestContent/ and runs this
# via /bin/bash. Anything under /data/local/tmp/QDC_logs/ is auto-uploaded.
# run_qdc_jobs.py substitutes:
#   {MODELS}  → `name|plugin|csv_devices|model_id|vlm|image` lines
#   {CHIPSET} → AI Hub chipset slug (e.g. qualcomm-qcs9075)
# Each cell's column-4 model_id is resolved on the device by the model-
# manager C API (multi-connection HTTPS, byte-range resume) on first
# reference; the cached copy is reused across the ctx sweep.
#
# We sweep ctx in {512, 1024, 4096} per cell to align with test-llama.cpp's
# PERFORMANCE SESSION; geniex-bench prefills exactly N=ctx random token ids
# (mirrors llama-bench `pp{N}`) so each ctx is also the prefill length.

set +e
umask 022

LOG=/data/local/tmp/QDC_logs
OUT=$LOG/results
MM_CACHE=/data/local/tmp/geniex-cache
TC=/data/local/tmp/TestContent
BUNDLE=$TC/pkg-geniex

mkdir -p "$LOG" "$OUT" "$MM_CACHE"
exec > "$LOG/script.log" 2>&1
date -u
uname -a

cd "$BUNDLE" || { echo "FATAL: missing $BUNDLE"; exit 1; }
chmod +x bin/* 2>/dev/null
export LD_LIBRARY_PATH="$BUNDLE/lib:$BUNDLE/lib/llama_cpp:$BUNDLE/lib/qairt:$LD_LIBRARY_PATH"
export GENIEX_PLUGIN_PATH="$BUNDLE/lib"

IMG=$TC/test.png

TSV512=/data/local/tmp/matrix-512.tsv
TSV1024=/data/local/tmp/matrix-1024.tsv
TSV4096=/data/local/tmp/matrix-4096.tsv
: > "$TSV512" > "$TSV1024" > "$TSV4096"

while IFS='|' read -r name plugin devs model_id vlm image; do
  [ -z "$name" ] && continue
  echo "=== plan $name id=$model_id ==="
  imgpath=""
  [ "$image" = "1" ] && imgpath="$IMG"
  IFS=','
  for d in $devs; do
    for ctx in 512 1024 4096; do
      tsv_var="TSV$ctx"
      # Columns 5/6 (tokenizer/mmproj) intentionally blank: the model
      # manager fills both from the resolved manifest.
      printf '%s-%s-%s-c%s\t%s\t%s\t%s\t\t\t%s\t%s\n' \
        "$name" "$plugin" "$d" "$ctx" "$plugin" "$d" "$model_id" "$imgpath" "$vlm" \
        >> "${!tsv_var}"
    done
  done
  IFS='|'
done <<'EOF'
{MODELS}
EOF

for ctx in 512 1024 4096; do
  tsv_var="TSV$ctx"
  tsv="${!tsv_var}"
  echo "=== matrix ctx=$ctx ==="
  cat "$tsv"
  ./bin/geniex-bench --matrix-file "$tsv" --output-json-dir "$OUT" -r 3 \
    -c "$ctx" -p "$ctx" \
    --mm-data-dir "$MM_CACHE" --chipset "{CHIPSET}"
  echo "rc=$?  ($(ls "$OUT" | wc -l) cell json files so far)"
done
echo "=== done ==="
exit 0
