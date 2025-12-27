# VoiceType

AI Voice Assistant with local Whisper support and CUDA acceleration.

## Features

- **Local Speech Recognition** - Uses Whisper models for offline speech-to-text
- **CUDA Acceleration** - GPU-accelerated inference for faster processing
- **Multi-language Support** - Chinese and English interface
- **Global Hotkeys** - System-wide keyboard shortcuts for quick access
- **Model Management** - Download and manage Whisper models
- **Driver Compatibility Check** - Automatic NVIDIA driver version verification

## Requirements

### Hardware
- NVIDIA GPU with CUDA 11.8 support
- Minimum 4GB GPU memory (8GB recommended)

### Software
- Windows 10/11 (64-bit)
- **NVIDIA Driver Version 522.xx or higher** (for CUDA 11.8)
- Node.js 18+ (for development only)
- Rust 1.70+ (for development only)
- Visual Studio 2022 with C++ build tools
- CUDA Toolkit 11.8 (optional, for local development)

## Quick Start

### 1. Download Pre-built Release

Download the latest release from [Releases](https://github.com/youyouhe/voicetype-app/releases) and extract the archive.

**Important:** Make sure you have NVIDIA GPU drivers version 522.xx or higher installed. The application will check your driver version on startup and warn you if it's too old.

### 2. Download Whisper Model

1. Run `voicetype.exe`
2. Go to Settings → Whisper Models
3. Click "Download" next to the model you want (recommend: `large-v3-turbo` for best performance)
4. Wait for download to complete (~1.5GB)
5. Click "Use" to activate the model

### 3. Start Using

- Press the global hotkey (default: `Ctrl+Shift+Space`) to start recording
- Speak clearly into your microphone
- Press the hotkey again to stop
- Text will be automatically typed at your cursor position

## Building from Source

### Prerequisites

1. Install Visual Studio 2022 with C++ build tools
2. Install CUDA Toolkit 11.8 from [NVIDIA](https://developer.nvidia.com/cuda-11-8-0-download-archive)
3. Install Rust from [rustup.rs](https://rustup.rs/)
4. Install Node.js 18+ from [nodejs.org](https://nodejs.org/)

### Build on Windows

**Option 1: Quick Build (CUDA 11.8)**

```batch
.\build_package_cuda_11.8.bat
```

This will:
- Build the frontend
- Compile the Rust backend with CUDA support
- Create a portable package in `src-tauri\target\release\portable\`

**Option 2: Manual Build**

```batch
# Install frontend dependencies
cd src
npm install
npm run build
cd ..

# Build Rust backend with CUDA feature
cd src-tauri
cargo build --release --features cuda
```

### Build Script

The `build_package_cuda_11.8.bat` script handles the entire build process:

- Sets up CUDA 11.8 environment
- Builds frontend with Vite
- Compiles Rust backend with CUDA support
- Creates portable package with all required DLLs

**Required DLLs** (automatically included in portable package):
- `cublas64_11.dll`
- `cublasLt64_11.dll`
- `cudart64_110.dll`
- `cufft64_10.dll`

## Project Structure

```
voicetype-app/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/        # UI components
│   ├── i18n/             # Internationalization
│   └── services/         # API services
├── src-tauri/            # Backend (Rust)
│   ├── src/
│   │   ├── voice_assistant/    # Voice assistant logic
│   │   ├── commands/           # Tauri commands
│   │   └── database.rs         # Database
│   └── resources/              # CUDA DLLs
└── build_package_cuda_11.8.bat # Build script
```

## Troubleshooting

### Driver Version Too Old

If you see the warning "CUDA 驱动版本警告", update your NVIDIA GPU driver:

1. Go to [NVIDIA Driver Downloads](https://www.nvidia.com/Download/index.aspx)
2. Select your GPU model
3. Download and install the latest driver (version 522.xx or higher)
4. Restart your computer

### CUDA DLL Not Found

If you get "DLL not found" errors:
1. Make sure all CUDA DLLs are in the same directory as `voicetype.exe`
2. Use the portable package which includes all required DLLs

### Model Not Working

If speech recognition fails:
1. Check that a model is downloaded and activated
2. Verify your NVIDIA driver is up to date
3. Try a different model (e.g., switch from `large-v3-turbo` to `large-v2`)

## Development

```bash
# Install frontend dependencies
cd src && npm install

# Start dev server
npm run tauri dev

# Build release
npm run tauri build
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
