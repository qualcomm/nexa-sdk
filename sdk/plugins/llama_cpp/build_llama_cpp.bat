@echo off
setlocal EnableExtensions EnableDelayedExpansion

set "CMAKELISTS=%~f1"
set "TOOLCHAIN_FILE=%~f2"
set "OUT_DIR=%~3"

for %%I in ("%CMAKELISTS%") do set "LLAMA_SRC=%%~dpI"
if "%LLAMA_SRC:~-1%"=="\" set "LLAMA_SRC=%LLAMA_SRC:~0,-1%"

set "SHORT_ROOT=%TEMP%\geniex_llama_cpp"
set "BUILD_DIR=%SHORT_ROOT%\build"
set "VS_ROOT=C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools"
set "CMAKE_EXE=%VS_ROOT%\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"
set "NINJA_EXE=%VS_ROOT%\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe"
set "LLVM_BIN=%VS_ROOT%\VC\Tools\Llvm\x64\bin"
set "PATH=%LLVM_BIN%;%VS_ROOT%\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja;%PATH%"

if "%GGML_HEXAGON%"=="" set "GGML_HEXAGON=ON"
if "%OPENCL_SDK_ROOT%"=="" set "OPENCL_SDK_ROOT=C:\Qualcomm\OpenCL_SDK\2.3.2"
if "%HEXAGON_SDK_ROOT%"=="" set "HEXAGON_SDK_ROOT=C:\Qualcomm\Hexagon_SDK\6.4.0.2"
if "%HEXAGON_TOOLS_ROOT%"=="" set "HEXAGON_TOOLS_ROOT=C:\Qualcomm\Hexagon_SDK\6.4.0.2\tools\HEXAGON_Tools\19.0.04"
if "%PYTHON3_EXECUTABLE%"=="" (
	for /f "delims=" %%I in ('where python.exe 2^>NUL') do (
		if not defined PYTHON3_EXECUTABLE (
			"%%I" -c "import sys" >NUL 2>&1
			if not errorlevel 1 set "PYTHON3_EXECUTABLE=%%I"
		)
	)
)
if "%PYTHON3_EXECUTABLE%"=="" (
	for /f "usebackq delims=" %%I in (`py -3 -c "import sys; print(sys.executable)" 2^>NUL`) do (
		if not defined PYTHON3_EXECUTABLE set "PYTHON3_EXECUTABLE=%%I"
	)
)

if "%OPENCL_SDK_ROOT%"=="" (
	echo OPENCL_SDK_ROOT is required to build the OpenCL backend. >&2
	exit /b 1
)

if "%WINDOWS_SDK_BIN%"=="" (
	echo WINDOWS_SDK_BIN is required to locate Inf2Cat and signtool. >&2
	exit /b 1
)

if "%PYTHON3_EXECUTABLE%"=="" (
	echo PYTHON3_EXECUTABLE is required and was not found in PATH. >&2
	exit /b 1
)

if not exist "%PYTHON3_EXECUTABLE%" (
	echo PYTHON3_EXECUTABLE does not point to a valid Python interpreter: %PYTHON3_EXECUTABLE% >&2
	exit /b 1
)

set "HEXAGON_CMAKE_ARGS=-DGGML_HEXAGON=OFF"
set "HEXAGON_BUILD_TARGETS="

if /I "%GGML_HEXAGON%"=="ON" (
	if "%HEXAGON_SDK_ROOT%"=="" (
		echo HEXAGON_SDK_ROOT is required to build the Hexagon backend. >&2
		exit /b 1
	)

	if "%HEXAGON_TOOLS_ROOT%"=="" (
		echo HEXAGON_TOOLS_ROOT is required to build the Hexagon backend. >&2
		exit /b 1
	)

	if "%HEXAGON_HTP_CERT%"=="" (
		echo HEXAGON_HTP_CERT is required to sign the Hexagon HTP catalog. >&2
		exit /b 1
	)

	if not exist "%HEXAGON_HTP_CERT%" (
		echo HEXAGON_HTP_CERT does not point to a valid certificate: %HEXAGON_HTP_CERT% >&2
		exit /b 1
	)

	set "HEXAGON_CMAKE_ARGS=-DHEXAGON_SDK_ROOT=%HEXAGON_SDK_ROOT% -DHEXAGON_TOOLS_ROOT=%HEXAGON_TOOLS_ROOT% -DGGML_HEXAGON=ON"
	set "HEXAGON_BUILD_TARGETS= ggml-hexagon"
) else (
	echo GGML_HEXAGON is OFF. Building llama.cpp without Hexagon backend.
)

set "HEXAGON_OUT_DIR=%OUT_DIR%"

if exist "%SHORT_ROOT%" rmdir /S /Q "%SHORT_ROOT%"
mkdir "%SHORT_ROOT%"
if errorlevel 1 exit /b 1

set "TMP=%SHORT_ROOT%\tmp"
set "TEMP=%SHORT_ROOT%\tmp"
mkdir "%TMP%"
if errorlevel 1 exit /b 1

if exist "%BUILD_DIR%" rmdir /S /Q "%BUILD_DIR%"

"%CMAKE_EXE%" -S "%LLAMA_SRC%" -B "%BUILD_DIR%" -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_TOOLCHAIN_FILE=%TOOLCHAIN_FILE% -DCMAKE_MAKE_PROGRAM="%NINJA_EXE%" -DCMAKE_PREFIX_PATH="%OPENCL_SDK_ROOT%" -DPython3_EXECUTABLE="%PYTHON3_EXECUTABLE%" !HEXAGON_CMAKE_ARGS! -DPREBUILT_LIB_DIR=windows_aarch64 -DGGML_BLAS=OFF -DGGML_NATIVE=OFF -DGGML_BACKEND_DL=ON -DGGML_OPENMP=OFF -DGGML_OPENCL=ON -DGGML_OPENCL_USE_ADRENO_KERNELS=ON -DLLAMA_BUILD_COMMON=ON -DLLAMA_BUILD_TOOLS=ON -DLLAMA_BUILD_TESTS=OFF -DLLAMA_BUILD_EXAMPLES=OFF -DLLAMA_BUILD_SERVER=OFF -DLLAMA_BUILD_WEBUI=OFF -DLLAMA_CURL=OFF -DLLAMA_OPENSSL=OFF -DBUILD_SHARED_LIBS=ON
if errorlevel 1 exit /b 1

"%CMAKE_EXE%" --build "%BUILD_DIR%" --config Release --target common ggml-base ggml ggml-cpu ggml-opencl!HEXAGON_BUILD_TARGETS! llama mtmd
if errorlevel 1 exit /b 1

copy /Y "%BUILD_DIR%\common\common.lib" "%OUT_DIR%\common.lib" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\vendor\cpp-httplib\cpp-httplib.lib" "%OUT_DIR%\cpp-httplib.lib" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\bin\ggml-base.dll" "%OUT_DIR%\ggml-base.dll" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\ggml\src\ggml-base.lib" "%OUT_DIR%\ggml-base.lib" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\bin\ggml-cpu.dll" "%OUT_DIR%\ggml-cpu.dll" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\ggml\src\ggml-cpu.lib" "%OUT_DIR%\ggml-cpu.lib" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\bin\ggml.dll" "%OUT_DIR%\ggml.dll" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\ggml\src\ggml.lib" "%OUT_DIR%\ggml.lib" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\bin\ggml-opencl.dll" "%OUT_DIR%\ggml-opencl.dll" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\ggml\src\ggml-opencl\ggml-opencl.lib" "%OUT_DIR%\ggml-opencl.lib" >NUL
if errorlevel 1 exit /b 1
if /I "%GGML_HEXAGON%"=="ON" (
	copy /Y "%BUILD_DIR%\bin\ggml-hexagon.dll" "%OUT_DIR%\ggml-hexagon.dll" >NUL
	if errorlevel 1 exit /b 1
	copy /Y "%BUILD_DIR%\ggml\src\ggml-hexagon\ggml-hexagon.lib" "%OUT_DIR%\ggml-hexagon.lib" >NUL
	if errorlevel 1 exit /b 1
	copy /Y "%BUILD_DIR%\ggml\src\ggml-hexagon\libggml-htp-v68.so" "%OUT_DIR%\libggml-htp-v68.so" >NUL
	if errorlevel 1 exit /b 1
	copy /Y "%BUILD_DIR%\ggml\src\ggml-hexagon\libggml-htp-v69.so" "%OUT_DIR%\libggml-htp-v69.so" >NUL
	if errorlevel 1 exit /b 1
	copy /Y "%BUILD_DIR%\ggml\src\ggml-hexagon\libggml-htp-v73.so" "%OUT_DIR%\libggml-htp-v73.so" >NUL
	if errorlevel 1 exit /b 1
	copy /Y "%BUILD_DIR%\ggml\src\ggml-hexagon\libggml-htp-v75.so" "%OUT_DIR%\libggml-htp-v75.so" >NUL
	if errorlevel 1 exit /b 1
	copy /Y "%BUILD_DIR%\ggml\src\ggml-hexagon\libggml-htp-v79.so" "%OUT_DIR%\libggml-htp-v79.so" >NUL
	if errorlevel 1 exit /b 1
	copy /Y "%BUILD_DIR%\ggml\src\ggml-hexagon\libggml-htp-v81.so" "%OUT_DIR%\libggml-htp-v81.so" >NUL
	if errorlevel 1 exit /b 1
	copy /Y "%BUILD_DIR%\ggml\src\ggml-hexagon\libggml-htp.cat" "%OUT_DIR%\libggml-htp.cat" >NUL
	if errorlevel 1 exit /b 1
) else (
	type NUL > "%OUT_DIR%\ggml-hexagon.dll"
	if errorlevel 1 exit /b 1
	type NUL > "%OUT_DIR%\ggml-hexagon.lib"
	if errorlevel 1 exit /b 1
	type NUL > "%OUT_DIR%\libggml-htp-v68.so"
	if errorlevel 1 exit /b 1
	type NUL > "%OUT_DIR%\libggml-htp-v69.so"
	if errorlevel 1 exit /b 1
	type NUL > "%OUT_DIR%\libggml-htp-v73.so"
	if errorlevel 1 exit /b 1
	type NUL > "%OUT_DIR%\libggml-htp-v75.so"
	if errorlevel 1 exit /b 1
	type NUL > "%OUT_DIR%\libggml-htp-v79.so"
	if errorlevel 1 exit /b 1
	type NUL > "%OUT_DIR%\libggml-htp-v81.so"
	if errorlevel 1 exit /b 1
	type NUL > "%OUT_DIR%\libggml-htp.cat"
	if errorlevel 1 exit /b 1
)
copy /Y "%BUILD_DIR%\bin\llama.dll" "%OUT_DIR%\llama.dll" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\src\llama.lib" "%OUT_DIR%\llama.lib" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\bin\mtmd.dll" "%OUT_DIR%\mtmd.dll" >NUL
if errorlevel 1 exit /b 1
copy /Y "%BUILD_DIR%\tools\mtmd\mtmd.lib" "%OUT_DIR%\mtmd.lib" >NUL
if errorlevel 1 exit /b 1

rmdir /S /Q "%SHORT_ROOT%"
