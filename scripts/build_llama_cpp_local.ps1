param(
    [switch]$Hexagon
)

$ErrorActionPreference = 'Stop'

$env:BAZEL_VS = 'C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools'
$env:BAZEL_VC = 'C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC'
$env:WINDOWS_SDK_BIN = 'C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0'
$env:PYTHON3_EXECUTABLE = 'C:\Users\mengshen\AppData\Local\Programs\Python\Python313-arm64\python.exe'

$config = if ($Hexagon) { 'llama_cpp_local_hexagon' } else { 'llama_cpp_local' }

bazelisk build --config=$config //sdk/plugins/llama_cpp:geniex_plugin