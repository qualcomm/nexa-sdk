#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 3 ]]; then
    echo "Usage: $0 <CMakeLists.txt> <toolchain-file> <out-dir>" >&2
    exit 1
fi

cmakelists="$1"
toolchain_file="$2"
out_dir="$3"

if [[ ! -f "$cmakelists" ]]; then
    echo "CMakeLists.txt not found: $cmakelists" >&2
    exit 1
fi

if ! command -v cmake >/dev/null 2>&1; then
    echo "cmake is required but was not found in PATH" >&2
    exit 1
fi

if ! command -v ninja >/dev/null 2>&1; then
    echo "ninja is required but was not found in PATH" >&2
    exit 1
fi

cmakelists="$(cd "$(dirname "$cmakelists")" && pwd)/$(basename "$cmakelists")"
if [[ -f "$toolchain_file" ]]; then
    toolchain_file="$(cd "$(dirname "$toolchain_file")" && pwd)/$(basename "$toolchain_file")"
fi

llama_src="$(cd "$(dirname "$cmakelists")" && pwd)"
short_root="${TMPDIR:-/tmp}/geniex_llama_cpp"
build_dir="$short_root/build"

toolchain_args=()
if [[ "${GENIEX_LLAMA_USE_TOOLCHAIN:-0}" == "1" ]] && [[ -f "$toolchain_file" ]]; then
    toolchain_args=(-DCMAKE_TOOLCHAIN_FILE="$toolchain_file")
fi

rm -rf "$short_root"
mkdir -p "$build_dir"
mkdir -p "$out_dir"

cmake -S "$llama_src" -B "$build_dir" \
    -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    "${toolchain_args[@]}" \
    -DGGML_BLAS=OFF \
    -DGGML_CCACHE=OFF \
    -DGGML_NATIVE=OFF \
    -DGGML_BACKEND_DL=ON \
    -DGGML_OPENMP=OFF \
    -DGGML_OPENCL=OFF \
    -DGGML_HEXAGON=OFF \
    -DLLAMA_BUILD_COMMON=ON \
    -DLLAMA_BUILD_TOOLS=ON \
    -DLLAMA_BUILD_TESTS=OFF \
    -DLLAMA_BUILD_EXAMPLES=OFF \
    -DLLAMA_BUILD_SERVER=OFF \
    -DLLAMA_BUILD_WEBUI=OFF \
    -DLLAMA_CURL=OFF \
    -DLLAMA_OPENSSL=OFF \
    -DBUILD_SHARED_LIBS=ON

cmake --build "$build_dir" --config Release --target common ggml-base ggml ggml-cpu llama mtmd

copy_named_artifact() {
    local artifact="$1"
    local src

    src="$(find "$build_dir" \( -type f -o -type l \) -name "$artifact" | head -n 1 || true)"
    if [[ -z "$src" ]]; then
        echo "Required artifact was not produced: $artifact" >&2
        exit 1
    fi

    cp -f "$src" "$out_dir/$artifact"
}

copy_named_artifact "libcommon.a"
copy_named_artifact "libcpp-httplib.a"
copy_named_artifact "libggml-base.so"
copy_named_artifact "libggml-cpu.so"
copy_named_artifact "libggml.so"
copy_named_artifact "libllama.so"
copy_named_artifact "libmtmd.so"

# The genrule declares both Windows and Linux outputs; create placeholders for non-host artifacts.
: > "$out_dir/common.lib"
: > "$out_dir/cpp-httplib.lib"
: > "$out_dir/ggml-base.dll"
: > "$out_dir/ggml-base.lib"
: > "$out_dir/ggml-cpu.dll"
: > "$out_dir/ggml-cpu.lib"
: > "$out_dir/ggml.dll"
: > "$out_dir/ggml.lib"
: > "$out_dir/ggml-opencl.dll"
: > "$out_dir/ggml-opencl.lib"
: > "$out_dir/llama.dll"
: > "$out_dir/llama.lib"
: > "$out_dir/mtmd.dll"
: > "$out_dir/mtmd.lib"

rm -rf "$short_root"
