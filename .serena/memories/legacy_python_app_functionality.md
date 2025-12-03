# Legacy Python App 功能描述文档 (完整版)

## 项目概述
这是一个**语音输入助手**（Voice Assistant），支持**热键触发录音**、**实时语音转录**（保持原文）和**语音翻译**（转中文为英文），自动输入结果到当前光标位置。  
**核心流程**：热键（Option + F8：转录；Shift + Option + F8：翻译）→ 录音 → ASR处理（Whisper/SenseVoice/Local）→ 可选翻译（SiliconFlow/Ollama）→ 智能输入（终端逐字符/GUI粘贴）。  
**平台**：macOS/Linux/Windows（自适应），依赖麦克风/辅助权限。  
**UI**：PyQt5控制面板（API Key管理、启动/停止、实时日志）。  
**依赖**：openai, httpx, pynput, pyqt5, sounddevice/soundfile, numpy 等（requirements.txt）。

## 目录结构
```
legacy_python_app/
├── main.py                  # 主入口：VoiceAssistant协调器
├── control_ui.py            # PyQt5控制UI
├── test_terminal_input.py   # 终端输入测试脚本
├── src/
│   ├── audio/recorder.py    # 录音器
│   ├── keyboard/            # 键盘监听/输入
│   │   ├── listener.py      # KeyboardManager (热键+输入)
│   │   └── inputState.py    # InputState枚举
│   ├── llm/                 # LLM处理
│   │   ├── symbol.py        # SymbolProcessor (加标点/优化)
│   │   └── translate.py     # TranslateProcessor (翻译)
│   ├── transcription/       # ASR处理器
│   │   ├── senseVoiceSmall.py # SenseVoiceSmall/LocalASR
│   │   └── whisper.py       # WhisperProcessor
│   └── utils/logger.py      # 彩色日志
└── requirements.txt         # 依赖
```

## 关键模块/类功能详述

### 1. **main.py** - VoiceAssistant (主协调器)
- `__init__`: 初始化 AudioRecorder, ASR处理器, KeyboardManager。绑定回调（录音开始/结束/翻译/重置）。
- `start_transcription_recording()` / `stop_transcription_recording()`: 录音 → ASR.process_audio(mode=\"transcriptions\") → KeyboardManager.type_text(text)。
- `start_translation_recording()` / `stop_translation_recording()`: 同上，mode=\"translations\"。
- `reset_state()`: 重置键盘状态。
- `run()`: 启动键盘监听。
- `main()`: 根据SERVICE_PLATFORM选择ASR (groq/Whisper/siliconflow/LocalASR)，异常处理权限检查。

### 2. **control_ui.py** - ControlUI (PyQt5 UI)
- `__init__`: 监控.env变化，日志实时更新，初始化UI。
- `init_ui()`: API Key输入/保存，启动/停止按钮，日志视图（带阴影/动画样式）。
- `save_settings()`: 更新.env的SILICONFLOW_API_KEY。
- `start_main()` / `stop_main()`: subprocess运行/终止main.py。
- `update_log_view()`: 尾随日志文件（RotatingFileHandler）。

### 3. **test_terminal_input.py** - 测试脚本
- `test_terminal_detection()`: 检测终端环境（环境变量/进程）。
- `test_character_input()` / `test_smart_input()`: 测试KeyboardManager.type_text()（逐字符 vs 智能）。

### 4. **src/audio/recorder.py** - AudioRecorder
- `__init__`: 检查设备（sounddevice），采样率16000Hz。
- `start_recording()` / `stop_recording()`: 低延迟流录音 → BytesIO WAV缓冲。检查时长<1s返回\"TOO_SHORT\"。
- `_check_audio_devices()`: 列出/监控默认输入设备。

### 5. **src/keyboard/listener.py** - KeyboardManager
- `__init__`: 热键配置（TRANSCRIPTIONS_BUTTON/F8, TRANSLATIONS_BUTTON/F7），状态机（InputState）。
- `on_press` / `on_release`: Option(F8)按下>0.3s触发录音/翻译。Shift+Option=翻译。
- `type_text(text, error)`: 智能输入：
  | 环境 | 方法 | 逻辑 |
  |------|------|------|
  | 终端 | 逐字符 | detect_terminal_environment() (TERM/PS1等) |
  | GUI  | 剪贴板 | Ctrl/Cmd+V + \"✅\"标记后删除 |
- `state` setter: 更新UI临时文本（\"🎤录音...\" → \"🔄转录...\"），回调on_record_start/stop。
- `reset_state()`: 删除临时文本，恢复剪贴板。

### 6. **src/llm/symbol.py** - SymbolProcessor (Groq LLM)
- `add_symbol(text)`: 加标点（llama3-8b）。
- `optimize_result(text)`: 优化ASR结果（语音识别纠错+标点）。

### 7. **src/llm/translate.py** - TranslateProcessor / LocalTranslateProcessor
- `translate(text)`: SiliconFlow GLM-4 / Ollama GPT-OSS → 英译。

### 8. **src/transcription/senseVoiceSmall.py** - SenseVoiceSmallProcessor / LocalASRProcessor
- `process_audio(buffer, mode)`: SiliconFlow SenseVoiceSmall / Local API (http://192.168.8.107:5001)。
  - 超时10s，翻译调用LocalTranslateProcessor。
  - 返回SRT/文本，处理\"code:0/data\"格式。

### 9. **src/transcription/whisper.py** - WhisperProcessor (Groq)
- `process_audio(buffer, mode)`: Whisper-large-v3(-turbo)，繁转简/加标点/优化。

### 10. **src/utils/logger.py** - logger
- `setup_logger()`: 彩色控制台 + RotatingFileHandler (logs/app.log, 1MBx5)。

## 运行/配置
- **环境**：.env (SILICONFLOW_API_KEY/GROQ_API_KEY, SERVICE_PLATFORM=siliconflow/groq, SYSTEM_PLATFORM=win/mac)。
- **权限**：macOS 麦克风/辅助功能。
- **热键**：F8录音转录；F7+F8翻译。
- **日志**：logs/app.log (实时UI显示)。

**生成时间**：基于完整源码分析。TSX文件似React遗留，未分析Python核心。