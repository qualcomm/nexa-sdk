# Env Setup

1. Install Bazelist
   - On Windows, `winget install -e Bazel-Bazelisk`
   - On Linux, install `bazelisk` through your package manager, for example, on Ubuntu: `sudo apt install bazelisk`

# Build

## SDK

Windows ARM64:

```powershell
$env:BAZEL_VS='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools'
$env:BAZEL_VC='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC'
bazelisk build //sdk/src:libgeniex
```

Output:

```text
bazel-out/arm64_windows-fastbuild/bin/sdk/src/libgeniex.dll
```

Test build:

```powershell
bazelisk build //sdk/tests/src:geniex_test_version
```

## SDK Plugin

Windows ARM64 llama.cpp plugin with OpenCL + Hexagon:

```powershell
$env:BAZEL_VS='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools'
$env:BAZEL_VC='C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC'
bazelisk build `
   '--action_env=HEXAGON_HTP_CERT=<path-to-pfx>' `
   '--action_env=WINDOWS_SDK_BIN=C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0' `
   '--action_env=PYTHON3_EXECUTABLE=<path-to-python.exe>' `
   //sdk/plugins/llama_cpp:geniex_plugin
```

Output:

```text
bazel-out/arm64_windows-fastbuild/bin/sdk/plugins/llama_cpp/geniex_plugin.dll
```

## Notes

- SDK Bazel target name: `//sdk/src:libgeniex`
- CMake is not required for the Bazel SDK build
- llama.cpp plugin Bazel target name: `//sdk/plugins/llama_cpp:geniex_plugin`
