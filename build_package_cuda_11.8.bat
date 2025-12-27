@echo off
echo ========================================
echo Building CUDA 11.8 Release Package
echo ========================================

REM Set CUDA 11.8 environment (force mode)
set CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8
set PATH=%CUDA_PATH%\bin;%USERPROFILE%\.cargo\bin;C:\Windows\system32;C:\Windows;%PATH%
set CUDACXX=%CUDA_PATH%\bin\nvcc.exe
set CUDA_BIN_PATH=%CUDA_PATH%\bin

REM Create nvcc wrapper with -allow-unsupported-compiler
set NVCC_WRAPPER=%TEMP%\nvcc_wrapper.bat
echo @echo off > "%NVCC_WRAPPER%"
echo "%CUDA_PATH%\bin\nvcc.exe" %%* --allow-unsupported-compiler >> "%NVCC_WRAPPER%"
set "CUDACXX=%NVCC_WRAPPER%"
set "CUDA_NVCC_EXECUTABLE=%NVCC_WRAPPER%"

echo CUDA_PATH: %CUDA_PATH%
echo Using nvcc wrapper: %CUDACXX%
echo.

echo [1/5] Building frontend...
cd /d D:\EchoType\src
call npm run build
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Frontend build failed!
    del "%NVCC_WRAPPER%" 2>nul
    pause
    exit /b 1
)

echo.
echo [2/5] Building Tauri bundle with CUDA 11.8...
cd /d D:\EchoType
set TAURI_BUNDLE_CARGO_FLAGS=--features cuda
set CUDAFLAGS=--allow-unsupported-compiler
set CMAKE_CUDA_FLAGS=--allow-unsupported-compiler

REM Initialize VS environment
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

call npm run tauri build
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Tauri build failed!
    del "%NVCC_WRAPPER%" 2>nul
    pause
    exit /b 1
)

echo.
echo [3/5] Creating portable package with CUDA DLLs...
set PORTABLE_DIR=src-tauri\target\release\portable
if exist "%PORTABLE_DIR%" rd /s /q "%PORTABLE_DIR%"
mkdir "%PORTABLE_DIR%"

REM Copy exe and DLLs
copy /Y "src-tauri\target\release\voicetype.exe" "%PORTABLE_DIR%\" >nul
for %%f in (src-tauri\resources\cuda\*.dll) do (
    echo   + %%~nxf
    copy /Y "%%f" "%PORTABLE_DIR%\" >nul
)

REM Create README
echo VoiceType v0.1.0 (CUDA 11.8 Edition) > "%PORTABLE_DIR%\README.txt"
echo. >> "%PORTABLE_DIR%\README.txt"
echo AI Voice Assistant with CUDA-accelerated Whisper support >> "%PORTABLE_DIR%\README.txt"
echo. >> "%PORTABLE_DIR%\README.txt"
echo Requirements: >> "%PORTABLE_DIR%\README.txt"
echo - Windows 10/11 (64-bit) >> "%PORTABLE_DIR%\README.txt"
echo - NVIDIA GPU with CUDA 11.8 support >> "%PORTABLE_DIR%\README.txt"
echo - NVIDIA GPU Drivers installed >> "%PORTABLE_DIR%\README.txt"
echo. >> "%PORTABLE_DIR%\README.txt"
echo Just run voicetype.exe - no installation needed! >> "%PORTABLE_DIR%\README.txt"

echo Created portable package: %PORTABLE_DIR%

echo.
echo [4/5] Adding CUDA DLLs to NSIS installer...
set NSIS_DIR=src-tauri\target\release\bundle\nsis
if exist "%NSIS_DIR%" (
    REM Copy DLLs to NSIS directory for installer to include
    for %%f in (src-tauri\resources\cuda\*.dll) do (
        echo   + %%~nxf to NSIS
        copy /Y "%%f" "%NSIS_DIR%\" >nul
    )

    REM Modify NSIS script to include DLLs
    set NSIS_SCRIPT=%NSIS_DIR%\nsis-installer.nsi
    if exist "%NSIS_SCRIPT%" (
        echo   Updating NSIS script...
        powershell -Command "$content = Get-Content '%NSIS_SCRIPT%' -Raw; $dllFiles = Get-ChildItem 'src-tauri\resources\cuda\*.dll' | ForEach-Object { $_.Name }; $dllSection = ''; foreach ($dll in $dllFiles) { $dllSection += 'File ``' + $dll + '```' + \"`n\" }; $content = $content -replace 'SetOutPath ``\$INSTDIR```.*?Section', 'SetOutPath ``$INSTDIR````n' + $dllSection + 'Section'; $content | Set-Content '%NSIS_SCRIPT%'"
    )
)

echo.
echo [5/5] Build artifacts:
if exist "%PORTABLE_DIR%" (
    echo   Portable: %PORTABLE_DIR%
)
if exist "src-tauri\target\release\bundle\msi\*.msi" (
    for %%f in (src-tauri\target\release\bundle\msi\*.msi) do echo   MSI: %%f
)
if exist "src-tauri\target\release\bundle\nsis\*.exe" (
    for %%f in (src-tauri\target\release\bundle\nsis\*.exe) do echo   NSIS: %%f
)

echo.
echo ========================================
echo Package completed!
echo ========================================
echo.
echo Use the portable version for immediate use,
echo or the NSIS installer for system-wide installation.
echo ========================================

REM Cleanup
del "%NVCC_WRAPPER%" 2>nul

pause
