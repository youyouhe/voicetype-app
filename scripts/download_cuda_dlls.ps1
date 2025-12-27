# CUDA DLL Extractor for Flash-Input
# Downloads and extracts CUDA Runtime DLLs for bundling

$ErrorActionPreference = "Stop"

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "`n========================================" "Cyan"
Write-ColorOutput "  CUDA DLL Extractor" "Cyan"
Write-ColorOutput "========================================`n" "Cyan"

# Configuration
$CudaVersion = "12.4.0"
$OutputDir = "src-tauri\resources\cuda"
$TempDir = Join-Path $env:TEMP "cuda_extract_$(Get-Random)"

Write-ColorOutput "CUDA Version: $CudaVersion" "Green"
Write-ColorOutput "Output Directory: $OutputDir`n" "Green"

# Create directories
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

try {
    Write-ColorOutput "[Step 1/3] Downloading CUDA Runtime installer..." "Cyan"
    Write-ColorOutput "Size: ~200 MB - This may take a while...`n" "Yellow"

    $InstallerUrl = "https://developer.download.nvidia.com/compute/cuda/redist/cuda_runtime/$CudaVersion/cuda_runtime_${CudaVersion}_windows.exe"
    $InstallerPath = Join-Path $TempDir "cuda_runtime.exe"

    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

    Write-ColorOutput "Downloading from: $InstallerUrl`n" "Gray"

    # Download the file
    Invoke-WebRequest -Uri $InstallerUrl -OutFile $InstallerPath -UseBasicParsing

    $FileSize = (Get-Item $InstallerPath).Length / 1MB
    Write-ColorOutput "  Downloaded: $($FileSize.ToString('0.00')) MB`n" "Green"

    Write-ColorOutput "[Step 2/3] Extracting DLLs..." "Cyan"

    # Extract using 7-Zip if available
    $SevenZip = $null
    @("C:\Program Files\7-Zip\7z.exe", "C:\Program Files (x86)\7-Zip\7z.exe") | ForEach-Object {
        if (Test-Path $_) { $SevenZip = $_ }
    }

    if ($SevenZip) {
        Write-ColorOutput "Using 7-Zip to extract..." "Yellow"
        & $SevenZip x $InstallerPath -o"$TempDir\extracted" -y | Out-Null
    } else {
        Write-ColorOutput "7-Zip not found. Running installer to extract..." "Yellow"
        Write-ColorOutput "The installer will launch - close it after extraction starts.`n" "Yellow"

        # Run installer with extract switch
        Start-Process -FilePath $InstallerPath -ArgumentList "-s", "-tempdir=$TempDir\extracted" -Wait
    }

    Write-ColorOutput "`n[Step 3/3] Copying CUDA DLLs..." "Cyan"

    # Essential CUDA DLLs needed by whisper.cpp
    $RequiredDlls = @(
        "cudart64_",
        "cublas64_",
        "cublasLt64_",
        "cufft64_",
        "cufftw64_"
    )

    $CopiedDlls = 0

    # Find and copy DLLs
    Get-ChildItem -Path $TempDir -Recurse -Filter "*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
        $DllName = $_.Name

        # Check if it's one of the required CUDA DLLs
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
                Write-ColorOutput "  + $DllName" "Gray"
                $CopiedDlls++
            }
        }
    }

    if ($CopiedDlls -eq 0) {
        Write-ColorOutput "`n⚠️  No DLLs found. Manual extraction may be needed." "Yellow"
        Write-ColorOutput "`nManual extraction instructions:" "Yellow"
        Write-ColorOutput "1. Download: $InstallerUrl" "White"
        Write-ColorOutput "2. Extract the installer using 7-Zip or similar tool" "White"
        Write-ColorOutput "3. Copy these DLLs to: $OutputDir" "White"
        Write-ColorOutput "   - cudart64_*.dll" "White"
        Write-ColorOutput "   - cublas64_*.dll" "White"
        Write-ColorOutput "   - cublasLt64_*.dll" "White"
        Write-ColorOutput "   - cufft64_*.dll" "White"
    } else {
        Write-ColorOutput "`n✓ Copied $CopiedDlls DLL(s) to: $OutputDir`n" "Green"

        # Create info file
        $InfoFile = Join-Path $OutputDir "README.txt"
        $DllList = Get-ChildItem -Path $OutputDir -Filter "*.dll" | ForEach-Object { $_.Name }
        $DllListStr = $DllList -join "`n  - "

        $InfoContent = "CUDA Runtime DLLs`n"
        $InfoContent += "=================`n`n"
        $InfoContent += "Version: $CudaVersion`n"
        $InfoContent += "Extracted: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')`n`n"
        $InfoContent += "Required DLLs:`n  - $DllListStr`n`n"
        $InfoContent += "These DLLs will be bundled with the application.`n"
        $InfoContent += "Users with NVIDIA GPUs will get automatic CUDA acceleration.`n`n"
        $InfoContent += "Note: Users still need NVIDIA GPU drivers installed."

        $InfoContent | Out-File -FilePath $InfoFile -Encoding UTF8

        $TotalSize = (Get-ChildItem -Path $OutputDir -Filter "*.dll" | Measure-Object -Property Length -Sum).Sum / 1MB
        Write-ColorOutput "Total DLL size: $($TotalSize.ToString('0.00')) MB`n" "Green"

        Write-ColorOutput "========================================" "Cyan"
        Write-ColorOutput "  Extraction Complete" "Cyan"
        Write-ColorOutput "========================================" "Cyan"
        Write-ColorOutput "Location: $OutputDir" "White"
        Write-ColorOutput "========================================`n" "Cyan"
    }

} finally {
    # Cleanup
    if (Test-Path $TempDir) {
        Remove-Item -Path $TempDir -Recurse -Force
        Write-ColorOutput "Cleaned up temporary files`n" "Gray"
    }
}

Write-ColorOutput "Next steps:" "Yellow"
Write-ColorOutput "1. Verify DLLs in: $OutputDir" "White"
Write-ColorOutput "2. Run: npm run release:cuda" "White"
Write-ColorOutput "`nPress Enter to exit..." "Yellow"
Read-Host
