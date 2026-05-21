# tools/runtime_benchmark

Compares answer quality between two runtimes on the same model:

- `geniex` — this project's CLI.
- `genie-t2t-run` — Qualcomm QAIRT SDK reference runner.

## Workflow

The benchmark splits across two machines:

1. **On a Qualcomm QDC (Snapdragon X Elite) device** — run the prompt
   suite, collect raw answers. See [RUN_ON_QDC.md](RUN_ON_QDC.md).
2. **On the local dev machine, in Claude Code** — score the answers
   against the rubric, emit the final CSV. See
   [SCORE_WITH_CLAUDE.md](SCORE_WITH_CLAUDE.md).

## Files

```
runtime_quality_benchmark.py   # collection script (runs on QDC)
testing_prompts.md             # 100-prompt suite, grouped by category
scoring_rubric.txt             # 0/2/7/10 rubric reference
results/                       # all generated artifacts
RUN_ON_QDC.md                  # how to do step 1
SCORE_WITH_CLAUDE.md           # how to do step 2
```

`results/` already contains finished runs for `Qwen3-4B-Instruct-2507`
and `Llama-v3.2-1B-Instruct` — useful as calibration references.
