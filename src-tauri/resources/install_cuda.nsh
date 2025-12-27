; NSIS script to include CUDA Runtime installation
; Add this to your main NSIS script

!define CUDA_RUNTIME_VERSION "12.4.0"
!define CUDA_RUNTIME_URL "https://developer.download.nvidia.com/compute/cuda/redist/cuda_runtime/${CUDA_RUNTIME_VERSION}/cuda_runtime_${CUDA_RUNTIME_VERSION}_windows.exe"

Var CUDAInstallerPath
Var CUDAInstalled

; Function to download and install CUDA Runtime
Function DownloadAndInstallCUDA
    ; Check if CUDA is already installed
    ReadRegDWORD $0 HKLM "SOFTWARE\NVIDIA Corporation\CUDA" "Version"
    ${If} $0 != ""
        ; CUDA already installed
        StrCpy $CUDAInstalled 1
        Return
    ${EndIf}

    ; Ask user if they want to install CUDA Runtime
    MessageBox MB_YESNO "This version includes CUDA acceleration.$\n$\nDo you want to install CUDA Runtime?$\n$\nRequired for GPU acceleration (NVIDIA GPU only)." IDYES install_cuda
    Return

    install_cuda:
        DetailPrint "Downloading CUDA Runtime..."
        nsExec::ExecToLog '"$WINDIR\System32\curl.exe" -L -o "$TEMP\cuda_runtime.exe" "${CUDA_RUNTIME_URL}"'
        Pop $0

        ${If} $0 == "0"
            DetailPrint "Installing CUDA Runtime (this may take a few minutes)..."
            nsExec::ExecToLog '"$TEMP\cuda_runtime.exe" -s'
            Pop $0

            ${If} $0 == "0"
                DetailPrint "CUDA Runtime installed successfully"
                StrCpy $CUDAInstalled 1
            ${Else}
                MessageBox MB_OK "CUDA Runtime installation failed. You can install it later."
            ${EndIf}

            Delete "$TEMP\cuda_runtime.exe"
        ${Else}
            MessageBox MB_OK "Failed to download CUDA Runtime. You can download it manually from NVIDIA website."
        ${EndIf}
FunctionEnd
