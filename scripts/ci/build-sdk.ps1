#Requires -Version 5.1
<#
Build the GenieX SDK on Windows ARM64.

Environment inputs (must be pre-populated by setup-vcvars composite action):
  CC, CXX                 Full paths to clang.exe / clang++.exe.
  CMAKE, NINJA            Full paths to VS-bundled cmake / ninja.
  TOOLCHAIN_FILE          Full path to arm64-windows-llvm.cmake.

Additional environment inputs:
  GENIEX_VERSION          (required)  Version string baked into binaries.
  BUILD_TYPE              (optional)  Default: Release.
  BUILD_DIR               (optional)  Default: sdk/build-windows-arm64.
  INSTALL_PREFIX          (optional)  Default: sdk/pkg-geniex.
  EXTRA_CMAKE_FLAGS       (optional)  Appended verbatim to `cmake -B`.
#>

$ErrorActionPreference = 'Stop'

function Require-Env([string]$name) {
  if (-not (Test-Path "Env:$name") -or [string]::IsNullOrEmpty((Get-Item "Env:$name").Value)) {
    throw "Environment variable '$name' is required"
  }
}

Require-Env 'GENIEX_VERSION'
Require-Env 'CC'
Require-Env 'CXX'
Require-Env 'CMAKE'
Require-Env 'TOOLCHAIN_FILE'

$BuildType      = if ($env:BUILD_TYPE)      { $env:BUILD_TYPE }      else { 'Release' }
$BuildDir       = if ($env:BUILD_DIR)       { $env:BUILD_DIR }       else { 'sdk/build-windows-arm64' }
$InstallPrefix  = if ($env:INSTALL_PREFIX)  { $env:INSTALL_PREFIX }  else { 'sdk/pkg-geniex' }
$ExtraFlags     = if ($env:EXTRA_CMAKE_FLAGS) { $env:EXTRA_CMAKE_FLAGS -split '\s+' | Where-Object { $_ } } else { @() }

# Fresh build dir (avoid stale cmake cache binding to a previous generator / compiler).
Remove-Item -Recurse -Force $BuildDir -ErrorAction SilentlyContinue

$cmakeArgs = @(
  '-B', $BuildDir,
  '-G', 'Ninja',
  '-S', 'sdk',
  '--log-level=VERBOSE',
  "-DCMAKE_MAKE_PROGRAM=$env:NINJA",
  "-DCMAKE_C_COMPILER=$env:CC",
  "-DCMAKE_CXX_COMPILER=$env:CXX",
  "-DCMAKE_TOOLCHAIN_FILE=$env:TOOLCHAIN_FILE",
  "-DCMAKE_BUILD_TYPE=$BuildType",
  "-DGENIEX_VERSION=$env:GENIEX_VERSION",
  '-DGENIEX_TEST=OFF',
  '-DGENIEX_DEBUG=OFF',
  '-DGENIEX_DL=ON',
  '-DGENIEX_PLUGIN_LLAMA_CPP=ON',
  '-DGENIEX_PLUGIN_QAIRT=OFF',
  '-DGGML_OPENCL=OFF',
  '-DGGML_HEXAGON=OFF',
  '-DPREBUILT_LIB_DIR=windows_aarch64'
) + $ExtraFlags

Write-Host "+ $env:CMAKE $($cmakeArgs -join ' ')"
& $env:CMAKE @cmakeArgs
if ($LASTEXITCODE -ne 0) { throw "cmake configure failed ($LASTEXITCODE)" }

& $env:CMAKE --build $BuildDir -j 8
if ($LASTEXITCODE -ne 0) { throw "cmake build failed ($LASTEXITCODE)" }

& $env:CMAKE --install $BuildDir --prefix $InstallPrefix
if ($LASTEXITCODE -ne 0) { throw "cmake install failed ($LASTEXITCODE)" }
