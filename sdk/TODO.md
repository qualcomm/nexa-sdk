1. `CMAKE_SYSTEMP_PROCESSOR` is not valid on upstream `tokenizer.cpp`. Pass `arm64` but only `ARM64` and `aarch64` are valid.
2. For `genniex_qairt`, need to update `CMAKE_CURRENT_SOURCE_DIR` keyword at `https://github.com/qcom-it-nexa-ai/qnn-run/pull/132`
3. `genniex_qairt` repo needs an api to get htp versions
4. `geniex_qairt` will panic when destructing, `geniex-cli` will print error message after `CTRL-D`
