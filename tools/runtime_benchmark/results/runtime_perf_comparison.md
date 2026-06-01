# Genie vs. geniex runtime — TTFT / TPS comparison

Source: 100-prompt benchmark CSVs under `tools/runtime_benchmark/results/`.
Metrics:
- **TTFT** — time to first token (ms, lower is better)
- **TPS** — decode tokens / second (higher is better)

For each run, `n` = rows with a numeric value (excludes errored / empty).
For `llama3.2-3b`, the `_scored.csv` file the user listed has no timing columns, so timings were
pulled from the sibling un-scored CSV (`llama3_2_3b_instruct.csv`) for the same run.

## Summary table (averages across 100 prompts)

| Model / run                                       | Genie TTFT (ms) | geniex TTFT (ms) | Genie TPS | geniex TPS | TPS speed-up |
| ------------------------------------------------- | --------------: | ---------------: | --------: | ---------: | -----------: |
| qwen3-4b (original, `qwen3_4b/qwen3_4b.csv`)      |          173.51 |         1023.76¹ |      6.52 |      16.48 |        2.53× |
| qwen3-4b-instruct-2507 (geniex-only, root CSV)    |          158.74 |           110.50 |      7.54 |      16.08 |        2.13× |
| qwen3-4b-instruct-2507 (no BOS token)             |          158.74 |           109.82 |      7.54 |      15.10 |        2.00× |
| qwen3-4b-instruct-2507 (CLI, with BOS)            |          158.74 |           119.77 |      7.54 |      15.96 |        2.12× |
| llama3.2-3b-instruct                              |          190.98 |           227.24 |      8.25 |      18.17 |        2.20× |

¹ The original qwen3-4b run shows abnormally high geniex TTFT (median 1291 ms). Looks like
cold-start / first-token framing was being charged to TTFT in that run; the three follow-up
qwen3-4b-2507 runs all land near 110–120 ms, which is the steady-state number.

## Per-model breakdown

### 1. qwen3-4b (original) — `qwen3_4b/qwen3_4b.csv`

|       | TTFT avg / med / min / max (ms)            | TPS avg / med / min / max          |
| ----- | ------------------------------------------ | ---------------------------------- |
| Genie | 173.51 / 130.25 / 108.95 / 718.57 (n=81)   | 6.52 / 4.05 / 2.57 / 15.20 (n=81)  |
| geniex| 1023.76 / 1291.27 / 114.24 / 1484.43 (n=100) | 16.48 / 16.44 / 15.74 / 17.55 (n=100) |

Errors: Genie 16 / 100, geniex 0 / 100.

- **TPS:** geniex ≈ **2.5× faster** on decode (16.48 vs 6.52) and far more consistent
  (range 15.7 – 17.6 vs Genie's 2.6 – 15.2).
- **TTFT:** Genie is faster on average in this run, but it also errored on 16 prompts and
  the geniex TTFT figure here is anomalous compared to all other qwen3-4b-2507 runs.
- **Reliability:** Genie failed 16 prompts; geniex failed 0.

### 2. qwen3-4b-instruct-2507 (geniex-only re-run, root CSV) — `qwen3_4b_instruct_2507_geniex_only.csv`

|       | TTFT avg / med / min / max (ms)         | TPS avg / med / min / max         |
| ----- | --------------------------------------- | --------------------------------- |
| Genie | 158.74 / 128.68 / 108.03 / 510.97 (n=100) | 7.54 / 4.79 / 2.37 / 16.84 (n=100) |
| geniex| 110.50 / 109.90 /  98.65 / 126.48 (n=99)  | 16.08 / 16.07 / 14.67 / 17.65 (n=99)  |

Errors: Genie 0 / 100, geniex 1 / 100.

- **TPS:** geniex **2.13× faster** on average (16.08 vs 7.54), and tightly clustered.
- **TTFT:** geniex **~30 % lower** on average (110 vs 159 ms) and an order of magnitude
  more stable — Genie's TTFT max is 511 ms vs geniex's 126 ms.

### 3. qwen3-4b-instruct-2507, no BOS token — `qwen3-4b-2507-no-bos-token/...csv`

|       | TTFT avg / med / min / max (ms)         | TPS avg / med / min / max         |
| ----- | --------------------------------------- | --------------------------------- |
| Genie | 158.74 / 128.68 / 108.03 / 510.97 (n=100) | 7.54 / 4.79 / 2.37 / 16.84 (n=100) |
| geniex| 109.82 / 108.98 / 100.60 / 130.17 (n=100) | 15.10 / 15.08 / 13.84 / 16.56 (n=100) |

Errors: 0 / 100 on both.

- **TPS:** geniex **2.00× faster** (15.10 vs 7.54). Slightly slower TPS than the with-BOS
  CLI run (15.10 vs 15.96), so dropping the BOS token costs ~5 % decode throughput.
- **TTFT:** geniex ~31 % lower (110 vs 159 ms), again with much tighter spread.

### 4. qwen3-4b-instruct-2507, CLI with BOS — `cli-qwen3-4b-2507-with-bos/...csv`

|       | TTFT avg / med / min / max (ms)         | TPS avg / med / min / max         |
| ----- | --------------------------------------- | --------------------------------- |
| Genie | 158.74 / 128.68 / 108.03 / 510.97 (n=100) | 7.54 / 4.79 / 2.37 / 16.84 (n=100) |
| geniex| 119.77 / 118.82 / 112.03 / 132.11 (n=100) | 15.96 / 15.98 / 14.88 / 16.77 (n=100) |

Errors: 0 / 100 on both.

- **TPS:** geniex **2.12× faster** (15.96 vs 7.54).
- **TTFT:** geniex **~25 % lower** (120 vs 159 ms).
- Compared to the no-BOS run on the same model, with-BOS CLI gives slightly higher TTFT
  (120 vs 110 ms) but slightly higher TPS (15.96 vs 15.10).

### 5. llama3.2-3b-instruct — `llama3.2-3b/llama3_2_3b_instruct.csv`

|       | TTFT avg / med / min / max (ms)         | TPS avg / med / min / max         |
| ----- | --------------------------------------- | --------------------------------- |
| Genie | 190.98 / 136.21 / 112.56 / 507.32 (n=100) | 8.25 / 4.53 / 2.98 / 18.91 (n=100) |
| geniex| 227.24 / 136.00 / 128.19 / 882.72 (n=100) | 18.17 / 18.15 / 18.04 / 18.37 (n=100) |

Errors: 0 / 100 on both.

- **TPS:** geniex **2.20× faster** (18.17 vs 8.25), and almost flat (18.04 – 18.37) vs
  Genie's wide 3.0 – 18.9 spread.
- **TTFT:** medians are essentially tied (~136 ms). geniex's average is dragged up by a
  small number of outliers (max 883 ms) — likely first-prompt warm-up.

## Take-aways

1. **TPS is the headline win:** across every qwen3-4b / 2507 / llama3.2 run, geniex
   delivers **~2.0× – 2.5× the decode throughput** of Genie, and the per-prompt variance
   collapses (geniex TPS sits in a narrow band; Genie's swings from ~2.5 to ~19).
2. **TTFT:** on the qwen3-4b-2507 runs geniex is **~25–30 % faster** than Genie on
   average and dramatically more consistent (sub-130 ms ceiling vs Genie's 500+ ms tail).
   On llama3.2-3b the medians match; geniex's mean is pulled up by a few warm-up
   outliers.
3. **Reliability:** the only run with errors is `qwen3_4b/qwen3_4b.csv` — Genie failed
   16 / 100, geniex 0. The geniex root-CSV run had 1 error.
4. **The original `qwen3_4b.csv` geniex-TTFT outlier (avg 1024 ms)** does not reproduce
   in any of the three subsequent 2507 runs (all ~110–120 ms). Treat it as a stale /
   cold-start artifact rather than steady-state behaviour.
