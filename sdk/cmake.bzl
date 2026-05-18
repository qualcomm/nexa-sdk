"""Bazel rule that drives the SDK's CMake build via `cmake --preset`.

The action is `local + no-sandbox + no-cache` so the per-preset CMake
build directory lives in the source tree (`sdk/build-<preset>/`) and
ninja's incremental state survives across Bazel invocations. Hermeticity
is traded for the same edit-rebuild loop the manual `cmake --build`
workflow has.

Outputs (under `bazel-bin/sdk/cmake_out/`):

    libgeniex.so, geniex.h, llama_cpp/, qairt/, bin/

The output path is preset-independent on purpose; the *build* directory
still varies by preset so cross-compile targets keep separate ninja
state.
"""

def _cmake_preset_build_impl(ctx):
    preset = ctx.attr.preset
    out_dir = "cmake_out"

    out_lib = ctx.actions.declare_file(out_dir + "/libgeniex.so")
    out_hdr = ctx.actions.declare_file(out_dir + "/geniex.h")
    out_llama = ctx.actions.declare_directory(out_dir + "/llama_cpp")
    out_qairt = ctx.actions.declare_directory(out_dir + "/qairt")
    out_bin = ctx.actions.declare_directory(out_dir + "/bin")
    outputs = [out_lib, out_hdr, out_llama, out_qairt, out_bin]

    ctx.actions.run_shell(
        outputs = outputs,
        inputs = ctx.files.srcs,
        command = r"""
set -euo pipefail
PRESET="$1"
OUT_LIB="$PWD/$2"
OUT_HDR="$PWD/$3"
OUT_LLAMA="$PWD/$4"
OUT_QAIRT="$PWD/$5"
OUT_BIN="$PWD/$6"

# Operate on the real source tree (followed through Bazel's execroot
# symlink) so build-<preset>/ persists between actions for ninja.
SDK_DIR="$(realpath sdk)"
cd "$SDK_DIR"

cmake --preset "$PRESET" >&2
cmake --build "build-$PRESET" -j >&2

STAGE="$SDK_DIR/build-$PRESET/.bazel-stage"
rm -rf "$STAGE"
cmake --install "build-$PRESET" --prefix "$STAGE" >&2

cp "$STAGE/lib/libgeniex.so" "$OUT_LIB"
cp "$STAGE/include/geniex.h" "$OUT_HDR"
if [ -d "$STAGE/lib/llama_cpp" ]; then cp -r "$STAGE/lib/llama_cpp/." "$OUT_LLAMA/"; fi
if [ -d "$STAGE/lib/qairt" ]; then cp -r "$STAGE/lib/qairt/." "$OUT_QAIRT/"; fi
if [ -d "$STAGE/bin" ]; then cp -r "$STAGE/bin/." "$OUT_BIN/"; fi
""",
        arguments = [
            preset,
            out_lib.path,
            out_hdr.path,
            out_llama.path,
            out_qairt.path,
            out_bin.path,
        ],
        execution_requirements = {
            "no-sandbox": "1",
            "local": "1",
            "no-cache": "1",
            # cargo (model-manager) fetches crates on first build.
            "requires-network": "1",
        },
        progress_message = "CMake preset build (%s)" % preset,
        mnemonic = "CMakePresetBuild",
        use_default_shell_env = True,
    )

    return [
        DefaultInfo(files = depset(outputs)),
        OutputGroupInfo(
            shared_library = depset([out_lib]),
            header = depset([out_hdr]),
            plugins = depset([out_llama, out_qairt]),
            bin = depset([out_bin]),
        ),
    ]

cmake_preset_build = rule(
    implementation = _cmake_preset_build_impl,
    attrs = {
        "preset": attr.string(mandatory = True),
        "srcs": attr.label_list(allow_files = True),
    },
)

def _select_output_group_impl(ctx):
    files = ctx.attr.target[OutputGroupInfo][ctx.attr.group]
    return [DefaultInfo(files = files)]

select_output_group = rule(
    implementation = _select_output_group_impl,
    attrs = {
        "target": attr.label(mandatory = True),
        "group": attr.string(mandatory = True),
    },
)
