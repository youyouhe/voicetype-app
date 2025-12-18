# Windows 10 专业版 CUDA 编译环境设置指南

## 当前系统状态

✅ **已安装组件**:
- Git (2.52.0)
- Rust (1.91.1)
- Windows 10 专业版

❌ **需要安装的组件**:
- NVIDIA 显卡驱动
- Visual Studio Build Tools
- CUDA Toolkit

## 📥 下载和安装步骤

### 步骤1：安装 NVIDIA 显卡驱动

**下载地址**: https://www.nvidia.com/drivers/

**安装步骤**:
1. 打开下载链接
2. 选择产品类型: GeForce
3. 产品系列: 根据您的显卡选择（如 GeForce RTX 30 Series）
4. 产品型号: 具体显卡型号（如 GeForce RTX 3080）
5. 操作系统: Windows 10 64-bit
6. 下载驱动并安装

**验证安装**:
```cmd
nvidia-smi
```

### 步骤2：安装 Visual Studio Build Tools

**下载地址**: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

**安装步骤**:
1. 下载 "Build Tools for Visual Studio 2022"
2. 运行安装程序
3. 选择工作负载（必须勾选）:
   - ✅ **C++ build tools** - 核心编译环境
   - ✅ **Windows 10/11 SDK** - 系统开发包
   - ✅ **CMake tools for Visual Studio** - 构建工具
4. 点击安装

**验证安装**:
```cmd
cl
```
应该显示 Microsoft C++ 编译器版本信息

### 步骤3：安装 CUDA Toolkit

**下载地址**: https://developer.nvidia.com/cuda-downloads

**选择配置**:
- Operating System: Windows
- Architecture: x86_64
- Version: Windows 11 或 Windows 10
- Installer Type: exe (local)

**推荐版本**:
- **CUDA 12.0** (最新，功能最全)
- **CUDA 11.8** (稳定，兼容性好)

**安装步骤**:
1. 下载 CUDA Toolkit
2. 运行安装程序
3. 选择 Express（推荐）或 Custom 安装
4. 等待安装完成

**验证安装**:
```cmd
nvcc --version
```

## 🔧 环境变量检查

安装完成后，检查系统环境变量：

```cmd
# 检查 CUDA 相关环境变量
echo %CUDA_PATH%
echo %PATH%

# 应该包含 CUDA 路径，例如：
# CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0
# PATH=...;C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.0\bin;...
```

## 🧪 编译环境验证

### 方法1：使用提供的脚本
```cmd
cd C:\Users\Administrator\EchoType
verify_cuda.bat
```

### 方法2：手动验证
创建测试文件 `test_cuda.cpp`:
```cpp
#include <iostream>
#include <cuda_runtime.h>

int main() {
    int deviceCount;
    cudaError_t error = cudaGetDeviceCount(&deviceCount);

    if (error == cudaSuccess) {
        std::cout << "CUDA 设备数量: " << deviceCount << std::endl;

        for (int i = 0; i < deviceCount; i++) {
            cudaDeviceProp prop;
            cudaGetDeviceProperties(&prop, i);
            std::cout << "设备 " << i << ": " << prop.name << std::endl;
            std::cout << "  计算能力: " << prop.major << "." << prop.minor << std::endl;
            std::cout << "  内存: " << prop.totalGlobalMem / (1024*1024) << " MB" << std::endl;
        }
    } else {
        std::cout << "CUDA 初始化失败: " << cudaGetErrorString(error) << std::endl;
    }

    return 0;
}
```

编译测试：
```cmd
nvcc test_cuda.cpp -o test_cuda.exe
.\test_cuda.exe
```

## 🚀 编译我们的应用程序

环境准备好后，编译 CUDA 版本的 EchoType：

### 步骤1：更新 Cargo.toml
确保 `src-tauri/Cargo.toml` 包含 CUDA 特性：
```toml
whisper-rs = { version = "0.13", features = ["cuda"] }
```

### 步骤2：编译 Rust 版本
```cmd
cd C:\Users\Administrator\EchoType\src-tauri
cargo build --release --features cuda
```

### 步骤3：构建完整应用
```cmd
cd C:\Users\Administrator\EchoType\src
npm run build
cd ..
npm run tauri build
```

## 📋 完整安装清单

### 必需组件
- [ ] NVIDIA 显卡驱动 (470.x+)
- [ ] Visual Studio Build Tools 2022
- [ ] CUDA Toolkit (11.8+ 或 12.x)

### 验证命令
```cmd
# 验证驱动
nvidia-smi

# 验证编译器
cl

# 验证 CUDA
nvcc --version

# 验证 Rust
rustc --version

# 验证 Git
git --version
```

## 🛠️ 故障排除

### 问题1：cl 命令未找到
**解决方案**:
1. 安装 Visual Studio Build Tools
2. 或运行 `vcvarsall.bat` 设置环境变量

### 问题2：nvcc 命令未找到
**解决方案**:
1. 重新安装 CUDA Toolkit
2. 手动添加 CUDA bin 目录到 PATH

### 问题3：CUDA 编译失败
**解决方案**:
1. 检查驱动版本兼容性
2. 更新 Visual Studio
3. 清理并重新编译

### 问题4：链接错误
**解决方案**:
1. 确保 CUDA 安装完整
2. 检查 Visual Studio 版本兼容性
3. 以管理员身份运行编译

## 💡 快速安装命令

如果您想要自动化安装：

```cmd
# 一键运行安装向导
C:\Users\Administrator\EchoType\setup_cuda_build_env.bat
```

这个脚本会引导您完成所有必要的安装步骤。

## ⏱️ 预期安装时间

- NVIDIA 驱动: 5-10分钟
- Visual Studio Build Tools: 20-30分钟
- CUDA Toolkit: 15-25分钟
- 总计: 约40-60分钟

---

**完成后您就可以编译 CUDA 加速版本的 EchoType 了！**