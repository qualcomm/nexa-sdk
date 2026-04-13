# Plugin Architecture

This repository only keeps two production backends:

- `cpu_gpu` (llama.cpp plugin)
- `npu` (QNN plugin)

## Directory Layout

- `plugins/llama_cpp`: implementation of the `cpu_gpu` backend
- `plugins/qnn`: implementation of the `npu` backend

All other historical backends were removed from this repository.
