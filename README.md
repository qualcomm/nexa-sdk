# Geniex

WIP, see [build.md](./docs/build.md) and [run.md](./docs/run.md).

## Directory Structure

- All libraries and tools intended for external release are placed in the repository root
- Dependencies are managed through the build system rather than by directory layout
- Use a single unified build system to avoid complexity caused by different subprojects using different build tools

## Bazel

:# Code Structure

```
.
├── .github/                  # GitHub workflows and configs
│   ├── actions/              # reusable actions, like env setup and s3 config
│   │   ├── env.yml
│   │   └── s3.yml
│   ├── scripts/              # reusable scripts used by workflows, like create github release
│   │   └── release.js
│   └── workflows/            # github actions workflows, it's simple because we use bazel for everything
│       ├── build.yml
│       ├── lint.yml
│       └── test.yml
│
├── .vscode/                  # VS Code debug settings, AI agent rules, LLM prompts
│
├── sdk/                      # C API layer (entry for CLI and bindings)
│   ├── include/              # public C API headers
│   ├── libs/                 # resource .so files (QAIRT, Hexagon, utility libs)
│   ├── src/                  # C API source, plugin loading, common utilities
│   └── BUILD.bazel
│
├── third-party/              # third-party dependencies
│   ├── geniex-proc/          # preprocessing and postprocessing repo
│   ├── geniex-qairt/         # core runtime
│   ├── llama.cpp/            # public version, not Qualcomm internal
│   ├── pybind11/             # for Python binding
│   └── jni/                  # for Java binding
│
├── bindings/                 # language bindings and packaging
│   ├── python/               # pybind11 code and setup.py for Python package
│   ├── android/              # JNI code and Maven files for Java package
│   └── docker/               # Dockerfile and scripts for Docker build/release
│
├── cli/                      # command-line interface
│   ├── main.go               # CLI entry point
│   ├── server/               # CLI server components
│   ├── go.mod
│   └── go.sum
│
├── docs/                     # documentation (C API, CLI, Python, Maven, Docker)
│
├── scripts/                  # build, release, signing, file upload/download scripts
│
├── tests/                    # unit/integration tests for C API, Python, Java
│   ├── qdc/                  # QDC device connection/testing scripts/configs
│   ├── include/              # test headers
│   └── src/                  # test source code
│
├── BUILD.bazel               # root BUILD file
├── MODULE.bazel              # root MODULE file
├── MODULE.bazel.lock
├── LICENSE
└── README.md
```

## Native dependency matrix

Each native SDK gates a specific plugin/backend. They're all independent, so you can pick the subset that fits your hardware and goals.

| Dep | Enabled by | Plugin affected | Hardware target | Bundled in repo? |
|---|---|---|---|---|
| **Hexagon SDK** | `-DGGML_HEXAGON=ON` (snapdragon preset) | `llama_cpp` only | Snapdragon NPU (HTP), via `ggml-hexagon` DSP skels | No — external install; `HEXAGON_SDK_ROOT` / `HEXAGON_TOOLS_ROOT` required |
| **OpenCL SDK** | `-DGGML_OPENCL=ON` (snapdragon preset) | `llama_cpp` only | Adreno GPU | No — external; `OPENCL_SDK_ROOT` needed for headers + `OpenCL.lib`. Runtime ICD ships with the Snapdragon GPU driver |
| **QAIRT / QNN** | `-DGENIEX_PLUGIN_QAIRT=ON` | `qairt` plugin only | Snapdragon NPU (HTP), via Qualcomm's QNN runtime | **Yes** — `third-party/geniex-qairt/third-party/{windows,android,linux-gcc11.2}/` bundles `QnnHtp.dll`, `Genie.dll`, HTP skels. An externally installed QAIRT is **not** required for building |
| *(none)* | `-DGGML_HEXAGON=OFF -DGGML_OPENCL=OFF` | `llama_cpp` | CPU | Always works |

The `llama_cpp` and `qairt` plugins can both drive the NPU, but via **two separate user-space stacks** (ggml-hexagon skels vs. QNN) that consume **different model formats** (GGUF vs. QAIRT `.bin` shards). They are not substitutes for each other.

Minimal Snapdragon build that still exercises the NPU: disable `GGML_HEXAGON` and `GGML_OPENCL`, keep `GENIEX_PLUGIN_QAIRT=ON`. llama.cpp runs CPU-only; the QAIRT plugin drives the NPU through its bundled libs.

## Notes

- All tutorials, cookbook and sample apps are in a separate repo [geniex-app](https://github.com/geniex-app).
