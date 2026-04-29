"""Merge tars with last-writer-wins dedup on path collisions."""

_TAR_TOOLCHAIN_TYPE = "@tar.bzl//tar/toolchain:type"

def _dedup_tar_impl(ctx):
    bsdtar = ctx.toolchains[_TAR_TOOLCHAIN_TYPE]
    output = ctx.actions.declare_file(ctx.attr.name + ".tar")

    args = ctx.actions.args()
    args.add(bsdtar.tarinfo.binary.path)
    args.add(ctx.executable._gawk.path)
    args.add(output.path)
    args.add_all(ctx.files.tars)

    ctx.actions.run(
        executable = ctx.executable._script,
        inputs = ctx.files.tars,
        outputs = [output],
        tools = [
            bsdtar.default.files,
            ctx.executable._gawk,
        ],
        arguments = [args],
        mnemonic = "DedupTar",
        progress_message = "Dedup-merging tars into %{output}",
    )

    return [DefaultInfo(files = depset([output]))]

dedup_tar = rule(
    implementation = _dedup_tar_impl,
    attrs = {
        "tars": attr.label_list(
            allow_files = True,
            mandatory = True,
        ),
        "_script": attr.label(
            default = "//cli/release/linux:dedup_tar.sh",
            executable = True,
            cfg = "exec",
            allow_single_file = True,
        ),
        "_gawk": attr.label(
            default = "@gawk//:gawk",
            executable = True,
            cfg = "exec",
            allow_single_file = True,
        ),
    },
    toolchains = [_TAR_TOOLCHAIN_TYPE],
)
