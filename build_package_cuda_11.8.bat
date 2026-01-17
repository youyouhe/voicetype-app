@echo off
setlocal EnableDelayedExpansion

REM ========================================
REM Command-line Arguments
REM ========================================
set "CLEAN_DEPS=0"
set "SKIP_DEPS_CHECK=0"

:parse_args
if "%~1"=="--clean" set "CLEAN_DEPS=1"
if "%~1"=="--skip-deps" set "SKIP_DEPS_CHECK=1"
if "%~1"=="-c" set "CLEAN_DEPS=1"
if "%~1"=="-s" set "SKIP_DEPS_CHECK=1"
shift
if not "%~1"=="" goto parse_args

REM ========================================
REM Colors for Windows CMD
REM ========================================
set "INFO=[INFO]"
set "SUCCESS=[OK]"
set "WARNING=[WARN]"
set "ERROR=[ERROR]"
set "REPAIR=[REPAIR]"

echo ========================================
echo Building CUDA 11.8 Release Package
echo ========================================
echo.

REM Get project root directory (where this script is located)
set PROJECT_ROOT=%~dp0
set PROJECT_ROOT=%PROJECT_ROOT:~0,-1%

echo Project Root: %PROJECT_ROOT%

if %CLEAN_DEPS%==1 (
    echo %WARNING% Clean mode: will reinstall all dependencies
)
if %SKIP_DEPS_CHECK%==1 (
    echo %WARNING% Skipping dependency check
)
echo.

REM ========================================
REM Prerequisites Check
REM ========================================
echo Checking prerequisites...

REM Check Node.js
where node >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo %ERROR% Node.js not found in PATH!
    echo Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)
for /f "tokens=*" %%i in ('node --version') do set NODE_VERSION=%%i
echo %SUCCESS% Node.js: %NODE_VERSION%

REM Check npm
where npm >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo %ERROR% npm not found in PATH!
    pause
    exit /b 1
)
for /f "tokens=*" %%i in ('npm --version') do set NPM_VERSION=%%i
echo %SUCCESS% npm: %NPM_VERSION%

REM Check Cargo
where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo %ERROR% Cargo not found in PATH!
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)
for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VERSION=%%i
echo %SUCCESS% %CARGO_VERSION%

REM Check CUDA 11.8
set CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8
if not exist "%CUDA_PATH%\bin\nvcc.exe" (
    echo %ERROR% CUDA 11.8 not found at: %CUDA_PATH%
    echo Please install CUDA 11.8 from NVIDIA
    pause
    exit /b 1
)
echo %SUCCESS% CUDA 11.8 found

REM Check Visual Studio Build Tools
set VS_PATH=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat
if not exist "%VS_PATH%" (
    echo %WARNING% Visual Studio 2022 BuildTools not found at default location
    echo Looking for alternative VS installations...

    REM Try to find vswhere
    set "VSWHERE=C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
    if exist "%VSWHERE%" (
        for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do (
            set "VS_PATH=%%i\VC\Auxiliary\Build\vcvars64.bat"
        )
        if exist "!VS_PATH!" (
            echo %SUCCESS% Found VS at: !VS_PATH!
        ) else (
            echo %ERROR% Visual Studio with C++ build tools not found!
            echo Please install Visual Studio 2022 with C++ build tools
            pause
            exit /b 1
        )
    ) else (
        echo %ERROR% Visual Studio not found and vswhere.exe missing!
        pause
        exit /b 1
    )
) else (
    echo %SUCCESS% Visual Studio 2022 BuildTools found
)

echo.
echo ========================================

REM ========================================
REM Environment Setup
REM ========================================
echo.
echo Setting up build environment...

REM Set CUDA 11.8 environment (force mode)
set PATH=%CUDA_PATH%\bin;%USERPROFILE%\.cargo\bin;C:\Windows\system32;C:\Windows;%PATH%
set CUDACXX=%CUDA_PATH%\bin\nvcc.exe
set CUDA_BIN_PATH=%CUDA_PATH%\bin

REM Force CMake to use CUDA 11.8
set CMAKE_PREFIX_PATH=%CUDA_PATH%
set CUDA_TOOLKIT_ROOT_DIR=%CUDA_PATH%
set CUDAToolkit_ROOT=%CUDA_PATH%

REM Create nvcc wrapper with -allow-unsupported-compiler
set NVCC_WRAPPER=%TEMP%\nvcc_wrapper.bat
echo @echo off > "%NVCC_WRAPPER%"
echo "%CUDA_PATH%\bin\nvcc.exe" %%* --allow-unsupported-compiler >> "%NVCC_WRAPPER%"
set "CUDACXX=%NVCC_WRAPPER%"
set "CUDA_NVCC_EXECUTABLE=%NVCC_WRAPPER%"

echo Using CUDA: %CUDA_PATH%
echo Using nvcc wrapper: %CUDACXX%
echo.

REM ========================================
REM Step 0: Install/Repair Dependencies
REM ========================================
echo [0/6] Checking and installing dependencies...
cd /d "%PROJECT_ROOT%\src"

REM Clean mode: remove everything and reinstall
if %CLEAN_DEPS%==1 (
    echo %WARNING% Clean mode enabled...
    if exist "node_modules" (
        echo Removing node_modules...
        rd /s /q "node_modules" 2>nul
    )
    if exist "package-lock.json" (
        echo Removing package-lock.json...
        del /q "package-lock.json" 2>nul
    )
    echo.
)

REM Function to check if key dependencies exist
:check_deps
if %SKIP_DEPS_CHECK%==1 goto deps_ok

if not exist "node_modules" goto install_needed

REM Check for critical dependencies
set "MISSING_DEPS=0"
if not exist "node_modules\d3-format" (
    set "MISSING_DEPS=1"
    echo %WARNING% d3-format is missing
)
if not exist "node_modules\react" (
    set "MISSING_DEPS=1"
    echo %WARNING% react is missing
)
if not exist "node_modules\vite" (
    set "MISSING_DEPS=1"
    echo %WARNING% vite is missing
)

if !MISSING_DEPS!==1 goto install_needed
goto deps_ok

:install_needed
echo.
echo %REPAIR% Installing dependencies...
call npm install
if %ERRORLEVEL% NEQ 0 (
    echo %ERROR% npm install failed!
    echo.
    echo Try running manually:
    echo   cd /d "%PROJECT_ROOT%\src"
    echo   npm install
    echo.
    del "%NVCC_WRAPPER%" 2>nul
    pause
    exit /b 1
)

REM Verify d3-format was installed
if not exist "node_modules\d3-format" (
    echo %ERROR% d3-format still missing after npm install!
    echo %INFO% Running additional install...
    call npm install d3-format --save
    if %ERRORLEVEL% NEQ 0 (
        echo %ERROR% Failed to install d3-format!
        del "%NVCC_WRAPPER%" 2>nul
        pause
        exit /b 1
    )
)
echo %SUCCESS% Dependencies installed
echo.

:deps_ok
if %SKIP_DEPS_CHECK%==0 (
    echo %SUCCESS% All dependencies found
)
echo.

REM ========================================
REM Step 1: Copy CUDA DLLs
REM ========================================
echo [1/6] Copying CUDA 11.8 DLLs...
set CUDA_DLL_DIR=%PROJECT_ROOT%\src-tauri\resources\cuda
if exist "%CUDA_DLL_DIR%" rd /s /q "%CUDA_DLL_DIR%"
mkdir "%CUDA_DLL_DIR%"

copy /Y "%CUDA_PATH%\bin\cublas64_*.dll" "%CUDA_DLL_DIR%\" >nul 2>&1
copy /Y "%CUDA_PATH%\bin\cublasLt64_*.dll" "%CUDA_DLL_DIR%\" >nul 2>&1
copy /Y "%CUDA_PATH%\bin\cudart64_*.dll" "%CUDA_DLL_DIR%\" >nul 2>&1
copy /Y "%CUDA_PATH%\bin\cufft64_*.dll" "%CUDA_DLL_DIR%\" >nul 2>&1

echo %SUCCESS% Copied CUDA 11.8 DLLs to: %CUDA_DLL_DIR%
dir "%CUDA_DLL_DIR%" /B
echo.

REM ========================================
REM Step 2: Build Frontend
REM ========================================
echo [2/6] Building frontend...
cd /d "%PROJECT_ROOT%\src"
call npm run build
if %ERRORLEVEL% NEQ 0 (
    call :frontend_build_failed
    del "%NVCC_WRAPPER%" 2>nul
    pause
    exit /b 1
)
echo %SUCCESS% Frontend build completed
echo.

REM ========================================
REM Step 3: Build Tauri Bundle
REM ========================================
echo [3/6] Building Tauri bundle with CUDA 11.8...
cd /d "%PROJECT_ROOT%"
set TAURI_BUNDLE_CARGO_FLAGS=--features cuda
set CUDAFLAGS=--allow-unsupported-compiler
set CMAKE_CUDA_FLAGS=--allow-unsupported-compiler

REM Initialize VS environment
echo Initializing Visual Studio environment...
call "%VS_PATH%"
if %ERRORLEVEL% NEQ 0 (
    echo %ERROR% Failed to initialize Visual Studio environment!
    del "%NVCC_WRAPPER%" 2>nul
    pause
    exit /b 1
)

echo Starting Tauri build (this may take a while)...
call npm run tauri build
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo %ERROR% Tauri build failed!
    echo.
    echo Troubleshooting tips:
    echo 1. Ensure CUDA 11.8 is properly installed
    echo 2. Check that Visual Studio C++ build tools are installed
    echo 3. Verify NVIDIA GPU drivers are up to date
    echo 4. Try running 'cargo clean' in src-tauri directory and rebuild
    echo.
    del "%NVCC_WRAPPER%" 2>nul
    pause
    exit /b 1
)
echo %SUCCESS% Tauri build completed
echo.

REM ========================================
REM Step 4: Create Portable Package
REM ========================================
echo [4/6] Creating portable package with CUDA DLLs...
set PORTABLE_DIR=%PROJECT_ROOT%\src-tauri\target\release\portable
if exist "%PORTABLE_DIR%" rd /s /q "%PORTABLE_DIR%"
mkdir "%PORTABLE_DIR%"

REM Check if exe exists
if not exist "%PROJECT_ROOT%\src-tauri\target\release\voicetype.exe" (
    echo %ERROR% voicetype.exe not found in target/release!
    echo Build may have failed - check the output above for errors
    del "%NVCC_WRAPPER%" 2>nul
    pause
    exit /b 1
)

REM Copy exe and DLLs
copy /Y "%PROJECT_ROOT%\src-tauri\target\release\voicetype.exe" "%PORTABLE_DIR%\" >nul
for %%f in (%PROJECT_ROOT%\src-tauri\resources\cuda\*.dll) do (
    echo   + %%~nxf
    copy /Y "%%f" "%PORTABLE_DIR%\" >nul
)

REM Create README
echo VoiceType v0.2.0 (CUDA 11.8 Edition) > "%PORTABLE_DIR%\README.txt"
echo. >> "%PORTABLE_DIR%\README.txt"
echo AI Voice Assistant with CUDA-accelerated Whisper support >> "%PORTABLE_DIR%\README.txt"
echo. >> "%PORTABLE_DIR%\README.txt"
echo Requirements: >> "%PORTABLE_DIR%\README.txt"
echo - Windows 10/11 (64-bit) >> "%PORTABLE_DIR%\README.txt"
echo - NVIDIA GPU with CUDA 11.8 support >> "%PORTABLE_DIR%\README.txt"
echo - NVIDIA GPU Drivers installed >> "%PORTABLE_DIR%\README.txt"
echo. >> "%PORTABLE_DIR%\README.txt"
echo Just run voicetype.exe - no installation needed! >> "%PORTABLE_DIR%\README.txt"

echo %SUCCESS% Created portable package: %PORTABLE_DIR%
echo.

REM ========================================
REM Step 5: Add CUDA DLLs to NSIS Installer
REM ========================================
echo [5/6] Adding CUDA DLLs to NSIS installer...
set NSIS_DIR=%PROJECT_ROOT%\src-tauri\target\release\bundle\nsis
if exist "%NSIS_DIR%" (
    REM Copy DLLs to NSIS directory for installer to include
    for %%f in (%PROJECT_ROOT%\src-tauri\resources\cuda\*.dll) do (
        echo   + %%~nxf to NSIS
        copy /Y "%%f" "%NSIS_DIR%\" >nul
    )

    REM Modify NSIS script to include DLLs
    set NSIS_SCRIPT=%NSIS_DIR%\nsis-installer.nsi
    if exist "%NSIS_SCRIPT%" (
        echo   Updating NSIS script...
        powershell -Command "$content = Get-Content '%NSIS_SCRIPT%' -Raw; $dllFiles = Get-ChildItem '%PROJECT_ROOT%\src-tauri\resources\cuda\*.dll' | ForEach-Object { $_.Name }; $dllSection = ''; foreach ($dll in $dllFiles) { $dllSection += 'File ``' + $dll + '```' + \"`n\" }; $content = $content -replace 'SetOutPath ``\$INSTDIR```.*?Section', 'SetOutPath ``$INSTDIR````n' + $dllSection + 'Section'; $content | Set-Content '%NSIS_SCRIPT%'"
    )
    echo %SUCCESS% NSIS installer updated
) else (
    echo %WARNING% NSIS directory not found, skipping installer update
)

echo.

REM ========================================
REM Step 6: Build Summary
REM ========================================
echo [6/6] Build artifacts:
echo ----------------------------------------
if exist "%PORTABLE_DIR%" (
    echo   Portable: %PORTABLE_DIR%
)
if exist "%PROJECT_ROOT%\src-tauri\target\release\bundle\msi\*.msi" (
    for %%f in (%PROJECT_ROOT%\src-tauri\target\release\bundle\msi\*.msi) do echo   MSI: %%f
)
if exist "%PROJECT_ROOT%\src-tauri\target\release\bundle\nsis\*.exe" (
    for %%f in (%PROJECT_ROOT%\src-tauri\target\release\bundle\nsis\*.exe) do echo   NSIS: %%f
)

echo.
echo ========================================
echo %SUCCESS% Package completed successfully!
echo ========================================
echo.
echo Build outputs:
echo   - Portable package: %PORTABLE_DIR%
echo   - Installer bundles: %PROJECT_ROOT%\src-tauri\target\release\bundle\
echo.
echo Use the portable version for immediate use,
echo or the NSIS installer for system-wide installation.
echo ========================================

REM Cleanup
del "%NVCC_WRAPPER%" 2>nul

echo.
pause
goto :eof

REM ========================================
REM Subroutine: Frontend Build Failed Repair
REM ========================================
:frontend_build_failed
echo.
echo %ERROR% Frontend build failed!
echo.
echo %REPAIR% Attempting automatic repair...
echo.

REM Try to install missing d3-format
echo Step 1: Installing d3-format...
call npm install d3-format --save
if %ERRORLEVEL% NEQ 0 (
    echo %ERROR% Failed to install d3-format!
    goto :frontend_repair_failed
)

REM Try rebuilding
echo.
echo Step 2: Rebuilding frontend...
call npm run build
if %ERRORLEVEL% EQU 0 (
    echo %SUCCESS% Repair successful! Build completed.
    goto :eof
)

:frontend_repair_failed
echo.
echo %ERROR% Automatic repair failed!
echo.
echo Troubleshooting tips:
echo 1. Try deleting node_modules and package-lock.json:
echo    cd /d "%PROJECT_ROOT%\src"
echo    rd /s /q node_modules
echo    del package-lock.json
echo    npm install
echo.
echo 2. Or run this script with --clean flag:
echo    build_package_cuda_11.8.bat --clean
echo.
echo 3. Run 'npm run build' separately to see detailed error messages
echo.
exit /b 1
