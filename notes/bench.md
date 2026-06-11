# Geniex Bench

The Geniex Bench pipeline benchmarks `geniex-bench` on real Qualcomm hardware via QDC
(Qualcomm Device Cloud) and reports results as a GitHub Actions step summary.

```
build-sdk  -->  load-models  -->  bench (device x model matrix)  -->  aggregate (per device)
```

## 1. Benchmark artifact compilation & upload

The SDK is cross-compiled for 3 platforms via `_build-sdk.yml`, producing a
`geniex-bench` binary for each:

| Platform       | Target Device                       | Binary             | Test Framework     |
|----------------|-------------------------------------|--------------------|--------------------|
| Linux ARM64    | QCS9075M (IoT)                      | `geniex-bench`     | Bash               |
| Windows ARM64  | SC8380XP / SC8480XP (Snapdragon X)  | `geniex-bench.exe` | PowerShell         |
| Android ARM64  | SM8750 / SM8850 (phone)             | `geniex-bench`     | Appium + pytest    |

Each platform gets a zip artifact assembled by `build_*_artifact()` in
`sdk/benchmark/qdc/run_qdc_jobs.py`:

- **Linux / Windows** — pre-built SDK package + entry script
  (`run_linux.sh` or `run_windows.ps1`) + sample prompts + VLM test image.
- **Android** — SDK package + pytest suite (`test_bench.py`, `utils.py`) +
  matrix rows + prompts.

The model matrix is defined in `sdk/benchmark/qdc/bench-models.json`
(currently 19 models across `llama_cpp` and `qairt` plugins).

## 2. QDC job execution

QDC interaction lives in `sdk/benchmark/qdc/_qdc.py`:

| Function             | Purpose                                                                 |
|----------------------|-------------------------------------------------------------------------|
| `make_client`        | Creates an authenticated QDC API client (`QDC_API_KEY`, app=`geniex-ci`)|
| `resolve_target`     | Maps chipset name (e.g. `SM8850`) to a QDC target ID                    |
| `submit_and_wait`    | Uploads artifact zip, submits job, polls every 30 s until terminal state. Quota-aware retry with exponential backoff (30 s base, 1 hr budget) |

On-device, all platforms perform the same work:

1. Build 3 TSV matrix files (context sizes 512, 1024, 4096) from the model rows.
2. Invoke `geniex-bench --matrix-file <tsv> --output-json-dir <out> --chipset <chip>`.
3. The benchmark binary (`sdk/benchmark/benchmark.c`) runs each cell:
   1 warmup + 3 measured repetitions, writing a per-cell JSON with aggregated
   stats (median / stdev / min / max for TTFT, prefill tok/s, decode tok/s).
4. Results land in `QDC_logs/results/` which QDC auto-collects.

## 3. Result collection & bench report rendering

Orchestrated by `bench.yml` in 4 jobs:

1. **build-sdk** — cross-compile binaries for all 3 platforms.
2. **load-models** — parse `bench-models.json`, emit the matrix + device list.
3. **bench** — one job per (device, model) pair: submit to QDC, wait, download
   per-cell JSON.  Uploaded as artifact `bench-cells-{device}-{model}`.
4. **aggregate** — one job per device: download all matching cell artifacts, call
   `render_aggregate()` which flattens cells, builds a markdown table, and writes
   it to `$GITHUB_STEP_SUMMARY`.

Example output:

```
## QDC Bench -- SM8850

| Model       | Backend   | Device | Ctx | ngl | Test       | TTFT (ms) | Prefill (tok/s) | Decode (tok/s) |
|-------------|-----------|--------|----:|----:|------------|----------:|----------------:|---------------:|
| Qwen3-0.6B | llama_cpp | cpu    | 512 |   - | pp42+tg128 | 49.8 +/-2.4 | 102.1 +/-6.2 | 60.9 +/-2.3    |
```

## Key design choices

- **Matrix-driven** — model x device pairs run in parallel on QDC.
- **Platform isolation** — differences are confined to artifact-building and entry
  scripts; the on-device benchmark binary and JSON schema are shared.
- **Common per-cell JSON schema** — every platform produces the same v2 schema,
  so the aggregator renders uniformly regardless of origin.

## Downloading geniex-bench

Standalone `geniex-bench` archives (binary + runtime libs) are published to S3
on each release. Use the **latest** URLs to always get the current stable build,
or pin to a specific version tag.

### Latest stable (always points to the newest non-prerelease)

| Platform      | URL |
|---------------|-----|
| Linux ARM64   | https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex/geniex-bench-linux-arm64.tar.gz |
| Android ARM64 | https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex/geniex-bench-android-arm64.tar.gz |
| Windows ARM64 | https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex/geniex-bench-windows-arm64.zip |

### Versioned (pinned to a specific release)

Replace `<tag>` with the release tag (e.g. `v1.2.3`):

| Platform      | URL |
|---------------|-----|
| Linux ARM64   | `https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex/geniex-bench-linux-arm64-<tag>.tar.gz` |
| Android ARM64 | `https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex/geniex-bench-android-arm64-<tag>.tar.gz` |
| Windows ARM64 | `https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex/geniex-bench-windows-arm64-<tag>.zip` |

### Quick start

```bash
# Linux / Android (same binary format, different target device)
curl -fsSL https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex/geniex-bench-linux-arm64.tar.gz | tar xz
./geniex-bench-linux-arm64-*/bin/geniex-bench --help

# Windows (PowerShell)
Invoke-WebRequest https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-geniex/geniex-bench-windows-arm64.zip -OutFile bench.zip
Expand-Archive bench.zip -DestinationPath .
.\geniex-bench-windows-arm64-*\bin\geniex-bench.exe --help
```

Each archive contains `bin/geniex-bench` (or `.exe`) plus `lib/` with all
required runtime shared libraries (libgeniex, llama_cpp plugin, qairt plugin).
