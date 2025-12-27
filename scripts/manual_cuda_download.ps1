# CUDA DLL Manual Setup Guide
# Use this script to verify manually downloaded DLLs

$ErrorActionPreference = "Stop"

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "`n========================================" "Cyan"
Write-ColorOutput "  CUDA DLL Manual Setup" "Cyan"
Write-ColorOutput "========================================`n" "Cyan"

$OutputDir = "src-tauri\resources\cuda"

# Create directory
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

Write-ColorOutput "CUDA DLLs should be placed in:" "Yellow"
Write-ColorOutput "  $OutputDir`n" "White"

# Check if DLLs already exist
$ExistingDlls = Get-ChildItem -Path $OutputDir -Filter "*.dll" -ErrorAction SilentlyContinue

if ($ExistingDlls) {
    Write-ColorOutput "Found existing DLLs:" "Green"
    $ExistingDlls | ForEach-Object {
        $Size = $_.Length / 1MB
        Write-ColorOutput "  ✓ $($_.Name) ($($Size.ToString('0.00')) MB)" "Gray"
    }
    Write-ColorOutput "`nTotal: $($ExistingDlls.Count) DLL(s)`n" "Green"
} else {
    Write-ColorOutput "No CUDA DLLs found yet.`n" "Yellow"
}

Write-ColorOutput "========================================" "Cyan"
Write-ColorOutput "  Manual Download Instructions" "Cyan"
Write-ColorOutput "========================================`n" "Cyan"

Write-ColorOutput "Option 1: From CUDA Toolkit Installation" "Yellow"
Write-ColorOutput "1. Download CUDA Toolkit from:" "White"
Write-ColorOutput "   https://developer.nvidia.com/cuda-downloads" "Gray"
Write-ColorOutput "2. Install the toolkit (local disk)" "White"
Write-ColorOutput "3. Copy these files from C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4\bin:" "White"
Write-ColorOutput "   - cudart64_*.dll" "Gray"
Write-ColorOutput "   - cublas64_*.dll" "Gray"
Write-ColorOutput "   - cublasLt64_*.dll" "Gray"
Write-ColorOutput "4. Paste to: $OutputDir`n" "White"

Write-ColorOutput "Option 2: Extract from CUDA Installer" "Yellow"
Write-ColorOutput "1. Download CUDA Toolkit exe (local)" "White"
Write-ColorOutput "2. Run: cuda_12.4.0_windows.exe -extract `<path>`" "Gray"
Write-ColorOutput "3. Find and copy DLLs from extracted folder`n" "White"

Write-ColorOutput "Option 3: From Existing CUDA Installation" "Yellow"
Write-ColorOutput "If you already have CUDA installed, run:" "White"
Write-ColorOutput "`n  .\scripts\copy_local_cuda_dlls.ps1`n" "Gray"

Write-ColorOutput "========================================" "Cyan"
Write-ColorOutput "  Required DLLs" "Cyan"
Write-ColorOutput "========================================`n" "Cyan"

Write-ColorOutput "Minimum required:" "Yellow"
Write-ColorOutput "  - cudart64_*.dll (CUDA Runtime)" "White"
Write-ColorOutput "  - cublas64_*.dll (CUDA BLAS)" "White"
Write-ColorOutput "`nOptional but recommended:" "Yellow"
Write-ColorOutput "  - cublasLt64_*.dll (CUDA BLAS LT)" "White"
Write-ColorOutput "  - cufft64_*.dll (CUDA FFT)" "White"
Write-ColorOutput "`nAfter copying DLLs, run:" "Yellow"
Write-ColorOutput "  npm run release:cuda`n" "White"

Write-ColorOutput "Press Enter to exit..." "Yellow"
Read-Host
