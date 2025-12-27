# CUDA Runtime Downloader for Flash-Input
# Downloads CUDA Runtime redistributable components for bundling

param(
    [string]$Version = "12.4.0",  # CUDA version
    [string]$OutputDir = "src-tauri\resources\cuda"
)

$ErrorActionPreference = "Stop"

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "`n========================================" "Cyan"
Write-ColorOutput "  CUDA Runtime Downloader" "Cyan"
Write-ColorOutput "========================================`n" "Cyan"

Write-ColorOutput "CUDA Version: $Version" "Green"
Write-ColorOutput "Output Directory: $OutputDir`n" "Green"

# Create output directory
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    Write-ColorOutput "Created directory: $OutputDir" "Yellow"
}

# CUDA Runtime URLs (from NVIDIA's CUDA redistributable)
$BaseUrl = "https://developer.download.nvidia.com/compute/cuda/redist"

# Runtime components to download
$Components = @(
    @{
        Name = "CUDA Runtime"
        Url = "$BaseUrl/cuda_runtime/$Version/cuda_runtime_12.4.0_windows.exe"
        File = "cuda_runtime.exe"
    },
    @{
        Name = "cuBLAS"
        Url = "$BaseUrl/cublas/$Version/cublas_windows.exe"
        File = "cublas.exe"
    },
    @{
        Name = "cuBLASLt"
        Url = "$BaseUrl/cublas_lt/$Version/cublas_lt_windows.exe"
        File = "cublas_lt.exe"
    }
)

Write-ColorOutput "[Step 1/3] Downloading CUDA Runtime components..." "Cyan"
Write-ColorOutput "This may take a while (files are large)..." "Yellow"`n"

$DownloadedFiles = @()

foreach ($Component in $Components) {
    $OutputPath = Join-Path $OutputDir $Component.File

    if (Test-Path $OutputPath) {
        Write-ColorOutput "  ✓ Already exists: $($Component.Name)" "Gray"
        $DownloadedFiles += $OutputPath
        continue
    }

    Write-ColorOutput "  ↓ Downloading: $($Component.Name)..." "Yellow"

    try {
        # Use TLS 1.2
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

        Invoke-WebRequest -Uri $Component.Url -OutFile $OutputPath -UseBasicParsing
        Write-ColorOutput "  ✓ Downloaded: $($Component.Name)" "Green"
        $DownloadedFiles += $OutputPath
    }
    catch {
        Write-ColorOutput "  ✗ Failed to download: $($Component.Name)" "Red"
        Write-ColorOutput "    Error: $_" "DarkRed"
    }
}

Write-ColorOutput "`n[Step 2/3] Extracting DLLs from installers..." "Cyan"
Write-ColorOutput "This requires 7-Zip or similar tool..." "Yellow"`n"

# Check for 7-Zip
$SevenZip = $null
$SevenZipPaths = @(
    "C:\Program Files\7-Zip\7z.exe",
    "C:\Program Files (x86)\7-Zip\7z.exe",
    "${env:ProgramFiles}\7-Zip\7z.exe",
    "${env:ProgramFiles(x86)}\7-Zip\7z.exe"
)

foreach ($Path in $SevenZipPaths) {
    if (Test-Path $Path) {
        $SevenZip = $Path
        break
    }
}

if ($null -eq $SevenZip) {
    Write-ColorOutput "7-Zip not found. Attempting alternative extraction..." "Yellow"

    # Try using Expand-Archive or just keep the installers
    Write-ColorOutput "Installers downloaded but not extracted." "Yellow"
    Write-ColorOutput "Users can run these installers silently during setup." "Yellow"

    $ExtractedDir = $OutputDir
}
else {
    Write-ColorOutput "Found 7-Zip at: $SevenZip" "Green"

    $ExtractedDir = Join-Path $OutputDir "dlls"
    if (-not (Test-Path $ExtractedDir)) {
        New-Item -ItemType Directory -Path $ExtractedDir -Force | Out-Null
    }

    # Extract DLLs from each installer
    foreach ($Installer in $DownloadedFiles) {
        Write-ColorOutput "  Extracting: $(Split-Path $Installer -Leaf)..." "Yellow"

        # Extract to temp location first
        $TempExtract = Join-Path $env:TEMP "cuda_extract_$(Get-Random)"
        New-Item -ItemType Directory -Path $TempExtract -Force | Out-Null

        try {
            # Extract using 7-Zip (CUDA installers are self-extracting archives)
            & $SevenZip x $Installer -o"$TempExtract" -y | Out-Null

            # Copy relevant DLLs to output directory
            $DllFiles = Get-ChildItem -Path $TempExtract -Recurse -Filter "*.dll" |
                        Where-Object { $_.FullName -like "*cuda*" -or $_.FullName -like "*cublas*" }

            foreach ($Dll in $DllFiles) {
                $DestPath = Join-Path $ExtractedDir $Dll.Name
                if (-not (Test-Path $DestPath)) {
                    Copy-Item -Path $Dll.FullName -Destination $DestPath -Force
                    Write-ColorOutput "    + $($Dll.Name)" "DarkGray"
                }
            }
        }
        catch {
            Write-ColorOutput "    Warning: Failed to extract $(Split-Path $Installer -Leaf)" "Yellow"
        }
        finally {
            # Clean up temp directory
            if (Test-Path $TempExtract) {
                Remove-Item -Path $TempExtract -Recurse -Force
            }
        }
    }
}

Write-ColorOutput "`n[Step 3/3] Creating bundle info..." "Cyan"

# Create info file about the bundled CUDA runtime
$InfoContent = @"
CUDA Runtime Bundle
===================

Version: $Version
Download Date: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

Contents:
$(if (Test-Path (Join-Path $ExtractedDir "*.dll")) {
    Get-ChildItem -Path (Join-Path $ExtractedDir "*.dll") | ForEach-Object { "- $($_.Name)`r`n" }
} else {
    "Installers (not extracted):`r`n"
    $DownloadedFiles | ForEach-Object { "- $(Split-Path $_ -Leaf)`r`n" }
})

Usage:
1. Copy DLLs to the application directory
2. Or run the installers silently during application setup

Note: Users still need to have NVIDIA GPU drivers installed.
"@

$InfoPath = Join-Path $OutputDir "bundle_info.txt"
$InfoContent | Out-File -FilePath $InfoPath -Encoding UTF8

Write-ColorOutput "`n========================================" "Cyan"
Write-ColorOutput "  Download Complete" "Cyan"
Write-ColorOutput "========================================" "Cyan"
Write-ColorOutput "Location: $OutputDir" "White"
Write-ColorOutput "Size: $((Get-ChildItem -Path $OutputDir -Recurse | Measure-Object -Property Length -Sum).Sum / 1MB ToString('0.00')) MB" "White"
Write-ColorOutput "========================================`n" "Cyan"

Write-ColorOutput "Next steps:" "Yellow"
Write-ColorOutput "1. Review the downloaded files in: $OutputDir" "White"
Write-ColorOutput "2. If extracted, copy DLLs to src-tauri\resources\" "White"
Write-ColorOutput "3. Update tauri.conf.json to bundle the DLLs" "White"
Write-ColorOutput "`nOr integrate the installers into your NSIS setup" "White"
