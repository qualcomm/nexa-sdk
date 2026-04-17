# Python Bindings — Build & Run

## Prerequisites

- Python 3.10+
- CMake 3.20+
- C++ compiler (GCC / Clang / MSVC)

---

## 1. Build the native library

```bash
cd sdk
cmake --preset default          # configure (native Linux x86_64)
cmake --build build-default --parallel $(nproc)
```

> **Other platforms:**
> - ARM64 Linux:  `--preset arm64-linux-snapdragon-release`
> - ARM64 Windows: `--preset arm64-windows-snapdragon-release`
> - Android:      `--preset arm64-android-snapdragon-release`

---

## 2. Set environment variables

```bash
export SDK_BUILD=sdk/build-default

export GENIEX_LIB_PATH=$SDK_BUILD/src/libgeniex.so
export GENIEX_PLUGIN_PATH=$SDK_BUILD/plugins
export LD_LIBRARY_PATH=$SDK_BUILD/bin:$SDK_BUILD/plugins/llama_cpp
```

> On **Windows** replace `LD_LIBRARY_PATH` with `PATH`, and `libgeniex.so` with `geniex.dll`.

---

## 3. Run the example

```bash
python bindings/python/examples/llm_basic.py \
  --model /path/to/model.gguf \
  --prompt "Name the capital of France." \
  --device cpu
```

**Streaming:**

```bash
python bindings/python/examples/llm_basic.py \
  --model /path/to/model.gguf \
  --prompt "Explain gravity in one sentence." \
  --stream
```

---

## 4. Use in your own script

No installation needed — set the env vars above and point `sys.path` at this directory:

```python
import sys
sys.path.insert(0, "bindings/python")

from geniex import AutoModelForCausalLM

model = AutoModelForCausalLM.from_pretrained("/path/to/model.gguf", device_map="cpu")

text = model.tokenizer.apply_chat_template(
    [{"role": "user", "content": "Hello!"}],
    tokenize=False,
    add_generation_prompt=True,
)

output = model.generate(text, max_new_tokens=256)
print(output.text)
model.close()
```

---

## 5. (Optional) Bazel build

```bash
bazel build //bindings/python:geniex_py \
  --//:sdk_type=local
```
