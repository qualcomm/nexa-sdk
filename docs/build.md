# Build

This repository uses Bazel and Bazelisk for SDK and plugin builds.

## Prerequisites

Install Bazelisk:

- Windows: `winget install -e Bazel-Bazelisk`
- Linux: install `bazelisk` from your package manager

Windows ARM64 builds also require:

- Visual Studio Build Tools 2022 or newer
- LLVM toolchain bundled with Visual Studio Build Tools
- Windows SDK
- Python 3

## Recommended Local Workflow

For day-to-day Windows ARM64 development, use the local wrapper script instead of typing long `bazelisk build` commands:

```powershell
.\scripts\build_llama_cpp_local.ps1
```

Build the llama.cpp plugin with Hexagon enabled:

```powershell
.\scripts\build_llama_cpp_local.ps1 -Hexagon
```

The wrapper script sets the local environment expected by Bazel and selects the right build config.

## Bazel Configs

The main configs are defined in [.bazelrc](c:/Users/mengshen/workspace/github.qualcomm.com/qcom-it-nexa-ai/geniex/.bazelrc):

- `sdk_debug`: enables `GENIEX_DEBUG`
- `llama_cpp_hexagon`: enables `GGML_HEXAGON`
- `llama_cpp_no_hexagon`: disables `GGML_HEXAGON`

An optional local override file [.bazelrc.user](c:/Users/mengshen/workspace/github.qualcomm.com/qcom-it-nexa-ai/geniex/.bazelrc.user) is imported automatically. It is intended for machine-specific environment forwarding.

## Common Builds

Build the SDK shared library:

```powershell
$env:BAZEL_VS='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools'
$env:BAZEL_VC='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC'
bazelisk build //sdk/src:geniex
```

Build the SDK with debug logging enabled:

```powershell
$env:BAZEL_VS='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools'
$env:BAZEL_VC='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC'
bazelisk build --config=sdk_debug //sdk/src:geniex
```

Build a lightweight SDK test target:

```powershell
bazelisk build --config=sdk_debug //sdk/tests/src:geniex_test_version
```

Build the llama.cpp plugin without Hexagon:

```powershell
bazelisk build --config=llama_cpp_no_hexagon //sdk/plugins/llama_cpp:geniex_plugin
```

Build the llama.cpp plugin with Hexagon:

```powershell
$env:HEXAGON_HTP_CERT='<path-to-pfx>'
$env:WINDOWS_SDK_BIN='C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0'
$env:PYTHON3_EXECUTABLE='<path-to-python.exe>'
bazelisk build --config=llama_cpp_hexagon //sdk/plugins/llama_cpp:geniex_plugin
```

## Raw Windows ARM64 Environment

If you do not use the wrapper script, these environment variables are the ones most commonly required:

```powershell
$env:BAZEL_VS='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools'
$env:BAZEL_VC='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC'
$env:WINDOWS_SDK_BIN='C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0'
$env:PYTHON3_EXECUTABLE='C:\Users\<user>\AppData\Local\Programs\Python\Python313-arm64\python.exe'
```

Hexagon builds additionally require:

```powershell
$env:HEXAGON_HTP_CERT='<path-to-pfx>'
```

## Outputs

SDK shared library:

```text
bazel-out/arm64_windows-fastbuild/bin/sdk/src/geniex.dll
```

llama.cpp plugin:

```text
bazel-out/arm64_windows-fastbuild/bin/sdk/plugins/llama_cpp/geniex_plugin.dll
```

## Notes

- The Bazel target for the SDK shared library is `//sdk/src:geniex`
- The Bazel target for the llama.cpp plugin is `//sdk/plugins/llama_cpp:geniex_plugin`
- CMake is not required for the SDK Bazel build
- `llama_cpp_no_hexagon` is the recommended default when you only need CPU or OpenCL validation on Windows
