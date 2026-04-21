#!/usr/bin/env bash
# Source this file (not execute) to activate the CI micromamba environment.
# Idempotent. Assumes micromamba is installed at ~/.local/bin (matches setup-deps).

export PATH="${HOME}/.local/bin:${PATH}"
export MAMBA_ROOT_PREFIX="${MAMBA_ROOT_PREFIX:-${HOME}/micromamba}"
eval "$(micromamba shell hook --shell bash --root-prefix "$MAMBA_ROOT_PREFIX")"
micromamba activate
