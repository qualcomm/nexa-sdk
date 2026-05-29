"""Minimal repro of the wrong geniex answer to qwen3-4b-2507 prompt #1.

Runs TWO paths back-to-back against the same loaded model so you can compare:

  PATH A (default benchmark path)
      apply_chat_template([system, user]) -> generate()
      The pybind's apply_chat_template currently prepends '<|endoftext|>' to
      the formatted string on turn 1 (BOS-token support landed in the qairt
      submodule and is in pybind 0.2.1rc2 but NOT in CLI v0.2.2). For
      Qwen3-Instruct-2507 the tokenizer config sets add_bos_token=false, so
      this prepend is out-of-distribution -> model answers about a "riddle"
      instead of gravity.

  PATH B (bypass)
      Hand-built chatML string identical to what chatMLTemplate / the CLI /
      genie-t2t-run feed the model. NO <|endoftext|> prefix.
      If the BOS-prepend is the bug, this path should answer correctly.

Both paths call model.reset() first so they start from a fresh KV cache.

Run:
    python repro_gravity.py
    python repro_gravity.py --device-map npu     # force qairt:NPU explicitly
    python repro_gravity.py --path a             # only run path A
    python repro_gravity.py --path b             # only run path B
"""
import argparse

from geniex import AutoModelForCausalLM

MODEL = "qualcomm/Qwen3-4B-Instruct-2507"
SYSTEM_PROMPT = "You are a helpful AI assistant."
QUESTION = "What is gravity?"

# Hand-built chatML — matches geniex-qairt/core/src/pipeline/chat_template.cpp
# chatMLTemplate() with system_prompt set, enable_thinking=false. This is what
# the CLI feeds the model and what genie-t2t-run also produces.
HANDBUILT_CHATML = (
    f"<|im_start|>system\n{SYSTEM_PROMPT}<|im_end|>\n"
    f"<|im_start|>user\n{QUESTION}<|im_end|>\n"
    "<|im_start|>assistant\n<think>\n\n</think>\n\n"
)


def run_path_a(model, max_tokens: int) -> None:
    print("\n" + "=" * 70)
    print("PATH A: apply_chat_template (current benchmark path)")
    print("=" * 70)
    model.reset()
    formatted = model.tokenizer.apply_chat_template(
        [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": QUESTION},
        ],
        add_generation_prompt=True,
    )
    print("\n--- formatted prompt (from apply_chat_template) ---")
    print(repr(formatted))
    has_eot = formatted.startswith("<|endoftext|>")
    print(f"--- starts with <|endoftext|>? {has_eot} ---")

    out = model.generate(formatted, max_new_tokens=max_tokens)
    print("\n--- thinking ---")
    print(out.thinking or "(none)")
    print("\n--- answer ---")
    print(out.text or "(empty)")


def run_path_b(model, max_tokens: int) -> None:
    print("\n" + "=" * 70)
    print("PATH B: hand-built chatML (bypass apply_chat_template)")
    print("=" * 70)
    model.reset()
    print("\n--- formatted prompt (hand-built) ---")
    print(repr(HANDBUILT_CHATML))

    out = model.generate(HANDBUILT_CHATML, max_new_tokens=max_tokens)
    print("\n--- thinking ---")
    print(out.thinking or "(none)")
    print("\n--- answer ---")
    print(out.text or "(empty)")


def run_path_c(model, max_tokens: int) -> None:
    """Same payload as path A but with enable_thinking=True — matches what
    `geniex infer` sends by default (the Go CLI defaults --think to true,
    while the pybind tokenizer.apply_chat_template defaults enable_thinking
    to False). With enable_thinking=True, chatMLTemplate skips the trailing
    '<think>\\n\\n</think>\\n\\n' block."""
    print("\n" + "=" * 70)
    print("PATH C: apply_chat_template + enable_thinking=True (CLI default)")
    print("=" * 70)
    model.reset()
    formatted = model.tokenizer.apply_chat_template(
        [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": QUESTION},
        ],
        add_generation_prompt=True,
        enable_thinking=True,
    )
    print("\n--- formatted prompt (enable_thinking=True) ---")
    print(repr(formatted))
    has_eot = formatted.startswith("<|endoftext|>")
    has_think = "<think>" in formatted
    print(f"--- starts with <|endoftext|>? {has_eot} ---")
    print(f"--- contains <think>? {has_think} ---")

    out = model.generate(formatted, max_new_tokens=max_tokens)
    print("\n--- thinking ---")
    print(out.thinking or "(none)")
    print("\n--- answer ---")
    print(out.text or "(empty)")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default=MODEL)
    ap.add_argument("--device-map", default="auto")
    ap.add_argument("--max-tokens", type=int, default=512)
    ap.add_argument(
        "--path",
        choices=["a", "b", "c", "all"],
        default="all",
        help="Which path(s) to run (default: all)",
    )
    args = ap.parse_args()

    print(f"Loading {args.model} (device_map={args.device_map})...", flush=True)
    model = AutoModelForCausalLM.from_pretrained(args.model, device_map=args.device_map)
    try:
        if args.path in ("a", "all"):
            run_path_a(model, args.max_tokens)
        if args.path in ("b", "all"):
            run_path_b(model, args.max_tokens)
        if args.path in ("c", "all"):
            run_path_c(model, args.max_tokens)
    finally:
        model.close()


if __name__ == "__main__":
    main()
