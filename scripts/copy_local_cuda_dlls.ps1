# Copy CUDA DLLs from Local Installation
# If you already have CUDA installed, use this script

$ErrorActionPreference = "Stop"

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "`n========================================" "Cyan"
Write-ColorOutput "  Copy CUDA DLLs from Local Install" "Cyan"
Write-ColorOutput "========================================`n" "Cyan"

$OutputDir = "src-tauri\resources\cuda"
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# Common CUDA installation paths
$CudaPaths = @(
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.8\bin",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4\bin",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.3\bin",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.2\bin",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.1\bin",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0\bin",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v11.8\bin",
    "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\"
)

# Find CUDA installation
$FoundPath = $null
foreach ($Path in $CudaPaths) {
    if (Test-Path $Path) {
        $FoundPath = $Path
        break
    }
}

if ($null -eq $FoundPath) {
    Write-ColorOutput "⚠️  CUDA installation not found!" "Red"
    Write-ColorOutput "`nSearched paths:" "Yellow"
    $CudaPaths | ForEach-Object { Write-ColorOutput "  - $_" "Gray" }
    Write-ColorOutput "`nPlease install CUDA Toolkit or manually copy DLLs." "Yellow"
    Write-ColorOutput "`nDownload from: https://developer.nvidia.com/cuda-downloads`n" "White"
    Read-Host
    exit 1
}

Write-ColorOutput "✓ Found CUDA at: $FoundPath`n" "Green"

# Required DLLs
$RequiredDlls = @(
    "cudart64_",
    "cublas64_",
    "cublasLt64_",
    "cufft64_"
)

$CopiedCount = 0

Write-ColorOutput "Copying CUDA DLLs...`n" "Cyan"

# Find and copy DLLs
Get-ChildItem -Path $FoundPath -Filter "*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
    $DllName = $_.Name

    # Check if it matches required pattern
    $IsRequired = $false
    foreach ($Pattern in $RequiredDlls) {
        if ($DllName -like "$Pattern*") {
            $IsRequired = $true
            break
        }
    }

    if ($IsRequired) {
        $DestPath = Join-Path $OutputDir $DllName
        if (-not (Test-Path $DestPath)) {
            Copy-Item -Path $_.FullName -Destination $DestPath -Force
            $Size = $_.Length / 1MB
            Write-ColorOutput "  ✓ Copied: $DllName ($($Size.ToString('0.00')) MB)" "Gray"
            $CopiedCount++
        }
    }
}

Write-ColorOutput "`n✓ Copied $CopiedCount DLL(s) to: $OutputDir`n" "Green"

# Create info file
$InfoFile = Join-Path $OutputDir "README.txt"
$DllList = Get-ChildItem -Path $OutputDir -Filter "*.dll" | ForEach-Object { $_.Name }
$DllListStr = $DllList -join "`n  - "

$InfoContent = "CUDA Runtime DLLs`n"
$InfoContent += "=================`n`n"
$InfoContent += "Source: $FoundPath`n"
$InfoContent += "Copied: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')`n`n"
$InfoContent += "Required DLLs:`n  - $DllListStr`n`n"
$InfoContent += "These DLLs will be bundled with the application.`n"
$InfoContent += "Users with NVIDIA GPUs will get automatic CUDA acceleration.`n`n"
$InfoContent += "Note: Users still need NVIDIA GPU drivers installed."

$InfoContent | Out-File -FilePath $InfoFile -Encoding UTF8

$TotalSize = (Get-ChildItem -Path $OutputDir -Filter "*.dll" | Measure-Object -Property Length -Sum).Sum / 1MB
Write-ColorOutput "Total DLL size: $($TotalSize.ToString('0.00')) MB`n" "Green"

Write-ColorOutput "========================================" "Cyan"
Write-ColorOutput "  Copy Complete" "Cyan"
Write-ColorOutput "========================================" "Cyan"
Write-ColorOutput "Location: $OutputDir" "White"
Write-ColorOutput "========================================`n" "Cyan"

Write-ColorOutput "Next step:" "Yellow"
Write-ColorOutput "  npm run release:cuda`n" "White"

Write-ColorOutput "Press Enter to exit..." "Yellow"
Read-Host
