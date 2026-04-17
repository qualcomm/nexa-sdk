"""Minimal geniex LLM example.

Run from the repo root:

    SDK_DIR=sdk/build-default
    export GENIEX_LIB_PATH=$SDK_DIR/src/libgeniex.so
    export GENIEX_PLUGIN_PATH=$SDK_DIR/plugins
    export LD_LIBRARY_PATH=$SDK_DIR/bin:$SDK_DIR/plugins/llama_cpp
    python bindings/python/examples/llm_basic.py --model /path/to/model.gguf

Example with a local Qwen3 model:

    python bindings/python/examples/llm_basic.py \\
        --model ~/.cache/nexa.ai/nexa_sdk/models/ggml-org/Qwen3-1.7B-GGUF/Qwen3-1.7B-Q4_K_M.gguf \\
        --prompt "Explain gravity in one sentence." --stream
"""

import argparse
import sys
import os

# Allow running directly from the repo without `pip install`
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from geniex import AutoModelForCausalLM


def main():
    parser = argparse.ArgumentParser(description='geniex minimal LLM example')
    parser.add_argument('--model', required=True, help='Path to GGUF model file or HF repo id')
    parser.add_argument('--prompt', default='Give me a short introduction to large language models.', help='User prompt')
    parser.add_argument('--max-tokens', type=int, default=256)
    parser.add_argument('--stream', action='store_true', help='Stream output token by token')
    parser.add_argument('--device', default='auto', help='device_map: auto | cpu | plugin:device')
    args = parser.parse_args()

    print(f'Loading model: {args.model}')
    model = AutoModelForCausalLM.from_pretrained(
        args.model,
        device_map=args.device,
        n_ctx=2048,
    )
    print('Model loaded.\n')

    messages = [{'role': 'user', 'content': args.prompt}]
    text = model.tokenizer.apply_chat_template(
        messages,
        tokenize=False,
        add_generation_prompt=True,
        enable_thinking=False,   # set True for Qwen3 thinking/reasoning mode
    )

    print(f'Prompt: {args.prompt}\n')
    print('Response:')

    if args.stream:
        streamer = model.generate(text, max_new_tokens=args.max_tokens, stream=True)
        for chunk in streamer:
            print(chunk, end='', flush=True)
        print()
        if streamer.output:
            p = streamer.output.profile
            print(f'\n[decode: {p.decode_speed:.1f} tok/s, {p.generated_tokens} tokens]')
    else:
        output = model.generate(text, max_new_tokens=args.max_tokens)
        if output.thinking:
            print(f'<thinking>\n{output.thinking}\n</thinking>\n')
        print(output.text)
        p = output.profile
        print(f'\n[decode: {p.decode_speed:.1f} tok/s, {p.generated_tokens} tokens, stop: {p.stop_reason}]')

    model.close()


if __name__ == '__main__':
    main()
