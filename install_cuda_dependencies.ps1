# Windows 10 CUDA 编译环境安装脚本
# 请以管理员身份运行此脚本

Write-Host "========================================" -ForegroundColor Green
Write-Host "CUDA 编译环境自动安装脚本" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""

# 检查管理员权限
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "❌ 请以管理员身份运行此脚本" -ForegroundColor Red
    pause
    exit 1
}

Write-Host "✅ 管理员权限确认" -ForegroundColor Green
Write-Host ""

# 检查当前安装状态
Write-Host "🔍 检查当前安装状态..." -ForegroundColor Yellow

# 检查 NVIDIA 驱动
$nvidiaExists = Test-Path "C:\Windows\System32\nvidia-smi.exe"
if ($nvidiaExists) {
    Write-Host "✅ NVIDIA 驱动: 已安装" -ForegroundColor Green
    try {
        nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader,nounits
    } catch {
        Write-Host "⚠️ NVIDIA 驱动已安装但无法获取详细信息" -ForegroundColor Yellow
    }
} else {
    Write-Host "❌ NVIDIA 驱动: 未安装" -ForegroundColor Red
}

# 检查 Visual Studio
$vsPath = "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
if (Test-Path $vsPath) {
    Write-Host "✅ Visual Studio Build Tools: 已安装" -ForegroundColor Green
} else {
    Write-Host "❌ Visual Studio Build Tools: 未安装" -ForegroundColor Red
}

# 检查 CUDA
$cudaExists = Get-Command nvcc -ErrorAction SilentlyContinue
if ($cudaExists) {
    Write-Host "✅ CUDA Toolkit: 已安装" -ForegroundColor Green
    try {
        nvcc --version
    } catch {
        Write-Host "⚠️ CUDA 已安装但无法获取版本信息" -ForegroundColor Yellow
    }
} else {
    Write-Host "❌ CUDA Toolkit: 未安装" -ForegroundColor Red
}

Write-Host ""

# 创建临时下载目录
$downloadDir = "$env:TEMP\CUDA_Install"
if (!(Test-Path $downloadDir)) {
    New-Item -ItemType Directory -Path $downloadDir -Force | Out-Null
}

Write-Host "📁 临时下载目录: $downloadDir" -ForegroundColor Cyan
Write-Host ""

# 下载链接
$downloads = @{
    "NVIDIA_Driver" = "https://www.nvidia.com/drivers/"
    "VS_BuildTools" = "https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022"
    "CUDA_Toolkit" = "https://developer.nvidia.com/cuda-downloads"
}

# 交互式安装
function Install-Component {
    param(
        [string]$Name,
        [string]$Url,
        [string]$Instructions
    )

    Write-Host "🔧 安装 $Name" -ForegroundColor Yellow
    Write-Host $Instructions -ForegroundColor White
    Write-Host "下载链接: $Url" -ForegroundColor Cyan

    $choice = Read-Host "是否现在安装? (y/n，默认y)"
    if ($choice -eq "" -or $choice -eq "y" -or $choice -eq "Y") {
        Start-Process $Url
        Write-Host "已打开下载页面，请完成安装后继续..." -ForegroundColor Green
        $done = Read-Host "安装完成后按回车继续"
    } else {
        Write-Host "跳过 $Name 安装" -ForegroundColor Yellow
    }
    Write-Host ""
}

# NVIDIA 驱动
if (!$nvidiaExists) {
    Install-Component -Name "NVIDIA 显卡驱动" -Url $downloads["NVIDIA_Driver"] -Instructions @"
1. 访问 NVIDIA 驱动下载页面
2. 选择您的显卡型号
3. 下载最新的 Game Ready 或 Studio 鍑动
4. 运行安装程序
5. 安装完成后验证: nvidia-smi
"@
}

# Visual Studio Build Tools
if (!(Test-Path $vsPath)) {
    Install-Component -Name "Visual Studio Build Tools" -Url $downloads["VS_BuildTools"] -Instructions @"
1. 下载 Build Tools for Visual Studio 2022
2. 运行安装程序
3. 选择工作负载 (必须勾选):
   - C++ build tools
   - Windows 10/11 SDK
   - CMake tools for Visual Studio
4. 完成安装
5. 验证: 打开新的命令提示符，运行 cl
"@
}

# CUDA Toolkit
if (!$cudaExists) {
    Install-Component -Name "CUDA Toolkit" -Url $downloads["CUDA_Toolkit"] -Instructions @"
1. 访问 CUDA Toolkit 下载页面
2. 选择: Windows, x86_64, Version, exe(local)
3. 推荐版本: CUDA 12.0 或 CUDA 11.8
4. 下载并运行安装程序
5. 选择 Express 安装
6. 验证: nvcc --version
"@
}

# 最终验证
Write-Host "🔍 最终验证..." -ForegroundColor Yellow
Write-Host ""

# 重新检查
$finalCheck = @{
    "NVIDIA_Driver" = (Test-Path "C:\Windows\System32\nvidia-smi.exe")
    "Visual_Studio" = (Test-Path $vsPath)
    "CUDA_Toolkit" = (Get-Command nvcc -ErrorAction SilentlyContinue)
}

$allInstalled = $true

foreach ($component in $finalCheck.Keys) {
    if ($finalCheck[$component]) {
        Write-Host "✅ $component : 已安装" -ForegroundColor Green
    } else {
        Write-Host "❌ $component : 未安装" -ForegroundColor Red
        $allInstalled = $false
    }
}

Write-Host ""

if ($allInstalled) {
    Write-Host "🎉 所有组件安装完成！" -ForegroundColor Green
    Write-Host ""
    Write-Host "下一步操作:" -ForegroundColor Cyan
    Write-Host "1. cd C:\Users\Administrator\EchoType\src-tauri" -ForegroundColor White
    Write-Host "2. cargo build --release --features cuda" -ForegroundColor White
    Write-Host "3. cd .." -ForegroundColor White
    Write-Host "4. npm run tauri build" -ForegroundColor White
    Write-Host ""

    # 提供一键编译选项
    $compile = Read-Host "是否现在编译 CUDA 版本的应用程序? (y/n)"
    if ($compile -eq "y" -or $compile -eq "Y") {
        Write-Host "🚀 开始编译..." -ForegroundColor Yellow
        try {
            Set-Location "C:\Users\Administrator\EchoType\src-tauri"
            Write-Host "编译 Rust 代码..." -ForegroundColor Cyan
            cargo build --release --features cuda

            if ($LASTEXITCODE -eq 0) {
                Write-Host "✅ Rust 编译成功！" -ForegroundColor Green
                Write-Host "构建完整应用程序..." -ForegroundColor Cyan
                Set-Location ".."
                npm run tauri build

                if ($LASTEXITCODE -eq 0) {
                    Write-Host "🎉 应用程序构建成功！" -ForegroundColor Green
                    Write-Host "可执行文件位置: C:\Users\Administrator\EchoType\src-tauri\target\release\hello-tauri.exe" -ForegroundColor White
                } else {
                    Write-Host "❌ 应用程序构建失败" -ForegroundColor Red
                }
            } else {
                Write-Host "❌ Rust 编译失败" -ForegroundColor Red
            }
        } catch {
            Write-Host "❌ 编译过程中出现错误: $_" -ForegroundColor Red
        }
    }
} else {
    Write-Host "⚠️ 还有组件未安装完成" -ForegroundColor Yellow
    Write-Host "请完成所有组件的安装后重新运行此脚本进行验证" -ForegroundColor White
}

Write-Host ""
Write-Host "脚本执行完成！" -ForegroundColor Green
pause