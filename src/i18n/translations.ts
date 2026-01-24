// 翻译文件 / Translation file
export interface Translations {
  // App / 应用
  appName: string;
  appVersion: string;
  windowTitle: string;

  // Top Bar / 顶部栏
  start: string;
  stop: string;
  settings: string;
  dashboard: string;
  startVoiceAssistant: string;
  stopVoiceAssistant: string;

  // Status / 状态
  idle: string;
  active: string;
  ready: string;
  listening: string;
  processing: string;
  translating: string;
  voiceAssistantOffline: string;
  readyToListen: string;

  // Dashboard / 仪表板
  voiceAssistantOfflineDesc: string;
  readyToListenDesc: (transcribe: string, translate: string) => string;
  speakClearly: string;
  transcribe: string;
  translate: string;
  transcribeDesc: string;
  translateDesc: string;
  startVoiceAssistantFirst: string;
  pressHotkeyToStart: (hotkey: string) => string;
  voiceAssistantBusy: string;

  // Settings / 设置
  asrService: string;
  shortcuts: string;
  whisperModels: string;
  advanced: string;
  system: string;

  // ASR Settings
  serviceProvider: string;
  local: string;
  localDesc: string;
  cloud: string;
  cloudDesc: string;
  localEndpoint: string;
  localEndpointPlaceholder: string;
  localApiKey: string;
  localApiKeyPlaceholder: string;
  cloudEndpoint: string;
  cloudEndpointPlaceholder: string;
  cloudApiKey: string;
  cloudApiKeyPlaceholder: string;
  save: string;
  saving: string;
  saved: string;

  // Shortcut Settings
  shortcutsTitle: string;
  shortcutsDesc: string;
  startTranscription: string;
  startTranslation: string;
  pressKeys: string;
  triggerDelay: string;
  seconds: string;
  antiMistouch: string;
  antiMistouchDesc: string;
  saveWavFiles: string;
  saveWavFilesDesc: string;
  saveHotkeyConfig: string;
  hotkeyConfigSaved: string;

  // Shortcut Settings - Additional
  globalHotkeys: string;
  prevention: string;
  triggerDelaySeconds: string;
  enableAntiMistouch: string;
  antiMistouchFullDesc: string;
  saveShortcuts: string;

  // Model Download
  modelsTitle: string;
  modelsDesc: string;
  download: string;
  downloading: string;
  use: string;
  delete: string;
  activeModel: string;
  totalModels: string;
  downloaded: string;
  available: string;
  storageLocation: string;
  downloadingProgress: string;

  // Model Download - Additional
  whisperModelsWithIcon: string;
  loadingModels: string;
  none: string;
  sizeLabel: string;
  fileLabel: string;

  // Advanced Settings
  advancedTitle: string;
  typingDelays: string;
  clipboardUpdate: string;
  keyboardEventsSettle: string;
  typingComplete: string;
  characterInterval: string;
  shortOperation: string;
  milliseconds: string;
  restoreDefaults: string;
  saveAdvancedSettings: string;

  // System Info
  systemInfo: string;
  platform: string;
  arch: string;
  tauriVersion: string;
  osVersion: string;
  kernelVersion: string;
  memoryInfo: string;
  totalMemory: string;
  availableMemory: string;
  notAvailable: string;

  // System Info - Additional
  systemInformationWithIcon: string;
  monitorSystemStatus: string;
  systemStatusCard: string;
  hardwareInformation: string;
  softwareInformation: string;
  voiceAssistantStatusCard: string;
  noSystemInformation: string;
  unableToRetrieveSystemInfo: string;
  retrySystemInfo: string;

  // LiveData
  serviceName: string;
  status: string;
  online: string;
  offline: string;
  latency: string;
  todayUsage: string;
  successRate: string;
  secs: string;

  // Language
  language: string;
  english: string;
  chinese: string;

  // History
  recentHistory: string;
  clear: string;
  noHistoryYet: string;
  unknownTime: string;

  // LiveData (additional)
  activeService: string;
  lastLatency: string;
  todaysUsage: string;
  error: string;
  unknown: string;
  loading: string;
  noRecordingsYet: string;
  noData: string;
  liveDataUnavailable: string;
  success: string;

  // VoiceAssistantPanel
  voiceAssistantStatus: string;
  serviceStatusText: string;
  activeText: string;
  inactiveText: string;
  runningListening: (transcribe: string, translate: string) => string;
  useStartButton: string;

  // Additional Settings Text
  asrServiceSettings: string;
  voiceRecognitionProvider: string;
  shortcutsBehaviors: string;
  whisperModelsDesc: string;
  advancedSettingsDesc: string;
  configureAdvancedSettings: string;
  audioInputDevice: string;
  selectMicrophone: string;
  selectMicrophoneDesc: string;
  testButtonDesc: string;
  autoSaveDesc: string;
  systemAudioApiDesc: string;
  audioSettings: string;
  saveWavFilesLongDesc: string;

  // Advanced Settings - Additional
  advancedSettingsFullTitle: string;
  runningInTauriNative: string;
  refresh: string;
  selectAMicrophone: string;
  default: string;
  selected: string;
  currentSelection: string;
  noMicrophoneSelected: string;
  testMicrophone: string;
  noMicrophonesDetected: string;
  checkSystemAudioSettings: string;
  grantMicrophonePermission: string;
  refreshDevices: string;
  requestMicrophoneAccess: string;
  noteChangesSavedAutomatically: string;
  micTestSuccess: string;
  micTestFailed: string;

  // ASR Service Settings - Additional
  connectionConfig: string;
  localWhisperConfiguration: string;
  localWhisperDescription: string;
  localWhisperNoConfig: string;
  loadingConfiguration: string;
  saveConfiguration: string;
  testAsrWithWavFile: string;
  chooseWavFile: string;
  transcriptionResult: string;
  cloudAsrEndpoint: string;
  cloudAsrEndpointPlaceholder: string;
  cloudAsrApiKey: string;
  cloudAsrApiKeyPlaceholder: string;
  securityNotice: string;
  securityNoticeDesc: string;
  cloudAsrMultipleProviders: string;
  debugPanelTitle: string;
  copy: string;
  clearLogs: string;
  hideDebugPanel: string;
  showDebugPanel: string;
  debugPanelLogs: string;
  noDebugLogsYet: string;
  environment: string;
  tauriDesktop: string;
  browser: string;
  debugRefresh: string;

  // Post-process / Text Correction Settings
  postProcessTextCorrection: string;
  postProcessTextCorrectionDesc: string;
  postProcessEnabled: string;
  postProcessEnabledDesc: string;
  postProcessProvider: string;
  postProcessProviderOllama: string;
  postProcessProviderDeepSeek: string;
  postProcessEndpoint: string;
  postProcessEndpointPlaceholder: string;
  postProcessModel: string;
  postProcessModelPlaceholder: string;
  postProcessSystemPrompt: string;
  postProcessSystemPromptPlaceholder: string;
  postProcessTimeout: string;
  postProcessTimeoutSeconds: string;
  postProcessInfoBox: string;
  postProcessAutoSaved: string;
  postProcessApiKey: string;
  postProcessApiKeyPlaceholder: string;
}

export const translations: Record<string, Translations> = {
  'zh-CN': {
    appName: 'VoiceType',
    appVersion: 'Tauri 客户端 v1.0.0',
    windowTitle: 'VoiceType - 语音助手',

    start: '启动',
    stop: '停止',
    settings: '设置',
    dashboard: '仪表板',
    startVoiceAssistant: '启动语音助手',
    stopVoiceAssistant: '停止语音助手',

    idle: '空闲',
    active: '活跃',
    ready: '就绪',
    listening: '正在听...',
    processing: '处理中...',
    translating: '翻译中...',
    voiceAssistantOffline: '语音助手离线',
    readyToListen: '准备监听',

    voiceAssistantOfflineDesc: '请先启动语音助手以使用转录和翻译功能。',
    readyToListenDesc: (transcribe, translate) => `按下 ${transcribe}（转录）或 ${translate}（翻译）快捷键开始录音。`,
    speakClearly: '请清晰地对麦克风说话。',
    transcribe: '转录',
    translate: '翻译',
    transcribeDesc: '语音转文字',
    translateDesc: '语音翻译',
    startVoiceAssistantFirst: '启动语音助手以启用转录',
    pressHotkeyToStart: (hotkey) => `按下 ${hotkey} 快捷键开始转录`,
    voiceAssistantBusy: '语音助手忙碌 - 请稍候',

    asrService: 'ASR 服务',
    shortcuts: '快捷键',
    whisperModels: 'Whisper 模型',
    advanced: '高级',
    system: '系统',

    serviceProvider: '服务提供商',
    local: '本地',
    localDesc: '使用本地 Whisper 模型，离线可用',
    cloud: '云端',
    cloudDesc: '使用云端 API，需要网络连接',
    localEndpoint: '本地端点',
    localEndpointPlaceholder: 'http://localhost:8080',
    localApiKey: '本地 API 密钥（可选）',
    localApiKeyPlaceholder: '留空表示不需要密钥',
    cloudEndpoint: '云端端点',
    cloudEndpointPlaceholder: 'https://api.openai.com/v1',
    cloudApiKey: '云端 API 密钥',
    cloudApiKeyPlaceholder: 'sk-...',
    save: '保存',
    saving: '保存中...',
    saved: '已保存',

    shortcutsTitle: '快捷键与行为',
    shortcutsDesc: '配置全局快捷键和行为设置',
    startTranscription: '转录快捷键',
    startTranslation: '翻译快捷键',
    pressKeys: '按下按键...',
    triggerDelay: '触发延迟',
    seconds: '秒',
    antiMistouch: '防误触',
    antiMistouchDesc: '防止意外触发录音',
    saveWavFiles: '保存 WAV 文件',
    saveWavFilesDesc: '将录音保存到文件',
    saveHotkeyConfig: '保存快捷键配置',
    hotkeyConfigSaved: '快捷键配置已保存',

    // Shortcut Settings - Additional
    globalHotkeys: '全局快捷键',
    prevention: '防误触设置',
    triggerDelaySeconds: '触发延迟（秒）',
    enableAntiMistouch: '启用防误触',
    antiMistouchFullDesc: '防止短按按键时意外触发录音。',
    saveShortcuts: '保存快捷键',

    modelsTitle: 'Whisper 模型',
    modelsDesc: '下载和管理本地 Whisper 模型',
    download: '下载',
    downloading: '下载中...',
    use: '使用',
    delete: '删除',
    activeModel: '当前',
    totalModels: '总模型数',
    downloaded: '已下载',
    available: '可用',
    storageLocation: '存储位置',
    downloadingProgress: '下载进度',

    // Model Download - Additional
    whisperModelsWithIcon: '🎤 Whisper 模型',
    loadingModels: '加载模型中...',
    none: '无',
    sizeLabel: '大小：',
    fileLabel: '文件：',

    advancedTitle: '高级设置',
    typingDelays: '打字延迟设置',
    clipboardUpdate: '剪贴板更新等待',
    keyboardEventsSettle: '键盘事件处理等待',
    typingComplete: '打字完成后等待',
    characterInterval: '字符间延迟',
    shortOperation: '短操作延迟',
    milliseconds: '毫秒',
    restoreDefaults: '恢复默认值',
    saveAdvancedSettings: '保存高级设置',

    systemInfo: '系统信息',
    platform: '平台',
    arch: '架构',
    tauriVersion: 'Tauri 版本',
    osVersion: '操作系统版本',
    kernelVersion: '内核版本',
    memoryInfo: '内存信息',
    totalMemory: '总内存',
    availableMemory: '可用内存',
    notAvailable: '不可用',

    // System Info - Additional
    systemInformationWithIcon: '💻 系统信息',
    monitorSystemStatus: '监控系统状态和语音助手配置',
    systemStatusCard: '系统状态',
    hardwareInformation: '硬件信息',
    softwareInformation: '软件信息',
    voiceAssistantStatusCard: '语音助手状态',
    noSystemInformation: '暂无系统信息',
    unableToRetrieveSystemInfo: '无法获取系统信息。请确保语音助手正在 Tauri 模式下运行。',
    retrySystemInfo: '重试',

    serviceName: '服务名称',
    status: '状态',
    online: '在线',
    offline: '离线',
    latency: '延迟',
    todayUsage: '今日使用',
    successRate: '成功率',
    secs: '秒',

    language: '语言',
    english: 'English',
    chinese: '中文',

    // History
    recentHistory: '最近记录',
    clear: '清除',
    noHistoryYet: '暂无记录。开始录音以查看结果。',
    unknownTime: '未知时间',

    // LiveData
    activeService: '活动服务',
    lastLatency: '最近延迟',
    todaysUsage: '今日使用',
    error: '错误',
    unknown: '未知',
    loading: '加载中...',
    noRecordingsYet: '暂无录音',
    noData: '暂无数据',
    liveDataUnavailable: '实时数据不可用',
    success: '成功',

    // VoiceAssistantPanel
    voiceAssistantStatus: '语音助手状态',
    serviceStatusText: '服务状态：',
    activeText: '活动',
    inactiveText: '未激活',
    runningListening: (transcribe, translate) => `语音助手正在运行并监听快捷键（${transcribe}, ${translate}）`,
    useStartButton: '使用顶部栏的启动按钮来激活语音助手',

    // Additional Settings Text
    asrServiceSettings: 'ASR 服务设置',
    voiceRecognitionProvider: '语音识别提供商',
    shortcutsBehaviors: '快捷键与行为',
    whisperModelsDesc: '下载和管理本地 Whisper 模型用于离线语音识别',
    advancedSettingsDesc: '高级设置',
    configureAdvancedSettings: '配置高级音频和系统设置',
    audioInputDevice: '音频输入设备',
    selectMicrophone: '选择麦克风',
    selectMicrophoneDesc: '• 选择您喜欢的麦克风用于语音输入',
    testButtonDesc: '• 使用测试按钮验证麦克风功能',
    autoSaveDesc: '• 您的选择将自动保存',
    systemAudioApiDesc: '• 使用系统音频 API 进行设备检测',
    audioSettings: '音频设置',
    saveWavFilesLongDesc: '处理后保存录制的音频为 WAV 文件，用于调试和备份目的。',

    // Advanced Settings - Additional
    advancedSettingsFullTitle: '高级设置',
    runningInTauriNative: '✓ 运行在 Tauri 原生环境中',
    refresh: '刷新',
    selectAMicrophone: '选择麦克风...',
    default: '（默认）',
    selected: '已选择：',
    currentSelection: '当前选择',
    noMicrophoneSelected: '未选择麦克风',
    testMicrophone: '测试麦克风',
    noMicrophonesDetected: '未检测到麦克风',
    checkSystemAudioSettings: '请检查您的系统音频设置',
    grantMicrophonePermission: '请授予麦克风权限以检测音频设备',
    refreshDevices: '刷新设备',
    requestMicrophoneAccess: '请求麦克风访问权限',
    noteChangesSavedAutomatically: '<strong>注意：</strong>对此设置的更改将自动保存。',
    micTestSuccess: '✅ 麦克风测试成功！音频输入正常工作。',
    micTestFailed: '❌ 麦克风测试失败！请检查麦克风设置。',

    // ASR Service Settings - Additional
    connectionConfig: '连接配置',
    localWhisperConfiguration: '本地 Whisper 配置',
    localWhisperDescription: '本地 Whisper 使用 whisper-rs 进行设备端推理。系统会自动检测并使用您下载的模型，位于：',
    localWhisperNoConfig: '无需配置 - 只需确保模型文件存在即可。',
    loadingConfiguration: '加载配置中...',
    saveConfiguration: '保存配置',
    testAsrWithWavFile: '使用 WAV 文件测试 ASR',
    chooseWavFile: '选择 WAV 文件',
    transcriptionResult: '转录结果：',
    cloudAsrEndpoint: '云端 ASR API 端点',
    cloudAsrEndpointPlaceholder: 'https://api.example.com/v1/audio/transcriptions',
    cloudAsrApiKey: '云端 ASR API 密钥',
    cloudAsrApiKeyPlaceholder: 'sk-...',
    securityNotice: '安全提示：',
    securityNoticeDesc: 'API 密钥是敏感凭据。请勿公开分享或提交到版本控制。尽可能使用 HTTPS 端点。',
    cloudAsrMultipleProviders: '云端 ASR 支持多个提供商（SiliconFlow、Groq）。端点决定使用哪个提供商。',
    debugPanelTitle: '🔍 ASR 调试控制台',
    copy: '复制',
    clearLogs: '清除',
    hideDebugPanel: '隐藏',
    showDebugPanel: '显示',
    debugPanelLogs: '调试面板',
    noDebugLogsYet: '暂无调试日志。尝试执行某个操作...',
    environment: '环境：',
    tauriDesktop: 'Tauri 桌面应用',
    browser: '浏览器',
    debugRefresh: '🔄 调试刷新',

    // Post-process / Text Correction Settings
    postProcessTextCorrection: 'AI 文本校正',
    postProcessTextCorrectionDesc: 'ASR 完成后使用 AI 模型修正文本错误',
    postProcessEnabled: '启用文本校正',
    postProcessEnabledDesc: '使用 AI 模型在输出前修正 ASR 识别的文本错误',
    postProcessProvider: '提供商',
    postProcessProviderOllama: 'Ollama (本地)',
    postProcessProviderDeepSeek: 'DeepSeek (云端)',
    postProcessEndpoint: 'API 端点',
    postProcessEndpointPlaceholder: 'http://localhost:11434/api/chat',
    postProcessModel: '模型名称',
    postProcessModelPlaceholder: 'llama3.2:latest 或 deepseek-chat',
    postProcessSystemPrompt: '系统提示词',
    postProcessSystemPromptPlaceholder: 'You are a text correction assistant. Fix grammar, spelling, and punctuation errors in the user input. Return only the corrected text without explanation.',
    postProcessTimeout: '超时时间',
    postProcessTimeoutSeconds: '秒',
    postProcessInfoBox: '提示：需要确保 AI 服务正在运行。文本校正会增加 1-3 秒的延迟，建议使用轻量级模型以获得更快响应。',
    postProcessAutoSaved: '配置更改将自动保存到数据库。',
    postProcessApiKey: 'API 密钥',
    postProcessApiKeyPlaceholder: 'sk-...',
  },
  'en-US': {
    appName: 'VoiceType',
    appVersion: 'Tauri Client v1.0.0',
    windowTitle: 'VoiceType - Voice Assistant',

    start: 'Start',
    stop: 'Stop',
    settings: 'Settings',
    dashboard: 'Dashboard',
    startVoiceAssistant: 'Start Voice Assistant',
    stopVoiceAssistant: 'Stop Voice Assistant',

    idle: 'Idle',
    active: 'Active',
    ready: 'Ready',
    listening: 'Listening...',
    processing: 'Processing...',
    translating: 'Translating...',
    voiceAssistantOffline: 'Voice Assistant Offline',
    readyToListen: 'Ready to Listen',

    voiceAssistantOfflineDesc: 'Please start Voice Assistant first to use transcription and translation features.',
    readyToListenDesc: (transcribe, translate) => `Press ${transcribe} (transcribe) or ${translate} (translate) hotkeys to start capturing audio.`,
    speakClearly: 'Speak clearly into your microphone.',
    transcribe: 'Transcribe',
    translate: 'Translate',
    transcribeDesc: 'Speech to Text',
    translateDesc: 'Speech Translation',
    startVoiceAssistantFirst: 'Start Voice Assistant first to enable transcription',
    pressHotkeyToStart: (hotkey) => `Press ${hotkey} hotkey to start transcribing`,
    voiceAssistantBusy: 'Voice Assistant is busy - please wait',

    asrService: 'ASR Service',
    shortcuts: 'Shortcuts',
    whisperModels: 'Whisper Models',
    advanced: 'Advanced',
    system: 'System',

    serviceProvider: 'Service Provider',
    local: 'Local',
    localDesc: 'Use local Whisper model, works offline',
    cloud: 'Cloud',
    cloudDesc: 'Use cloud API, requires internet',
    localEndpoint: 'Local Endpoint',
    localEndpointPlaceholder: 'http://localhost:8080',
    localApiKey: 'Local API Key (Optional)',
    localApiKeyPlaceholder: 'Leave empty if no key required',
    cloudEndpoint: 'Cloud Endpoint',
    cloudEndpointPlaceholder: 'https://api.openai.com/v1',
    cloudApiKey: 'Cloud API Key',
    cloudApiKeyPlaceholder: 'sk-...',
    save: 'Save',
    saving: 'Saving...',
    saved: 'Saved',

    shortcutsTitle: 'Shortcuts & Behaviors',
    shortcutsDesc: 'Configure global hotkeys and behavior settings',
    startTranscription: 'Transcription Hotkey',
    startTranslation: 'Translation Hotkey',
    pressKeys: 'Press keys...',
    triggerDelay: 'Trigger Delay',
    seconds: 'seconds',
    antiMistouch: 'Anti-Mistouch',
    antiMistouchDesc: 'Prevent accidental recording triggers',
    saveWavFiles: 'Save WAV Files',
    saveWavFilesDesc: 'Save recordings to files',
    saveHotkeyConfig: 'Save Hotkey Config',
    hotkeyConfigSaved: 'Hotkey configuration saved',

    // Shortcut Settings - Additional
    globalHotkeys: 'Global Hotkeys',
    prevention: 'Prevention',
    triggerDelaySeconds: 'Trigger Delay (seconds)',
    enableAntiMistouch: 'Enable Anti-Mistouch',
    antiMistouchFullDesc: 'Prevents accidental recording when holding keys briefly.',
    saveShortcuts: 'Save Shortcuts',

    modelsTitle: 'Whisper Models',
    modelsDesc: 'Download and manage local Whisper models',
    download: 'Download',
    downloading: 'Downloading...',
    use: 'Use',
    delete: 'Delete',
    activeModel: 'Active',
    totalModels: 'Total Models',
    downloaded: 'Downloaded',
    available: 'Available',
    storageLocation: 'Storage Location',
    downloadingProgress: 'Downloading...',

    // Model Download - Additional
    whisperModelsWithIcon: '🎤 Whisper Models',
    loadingModels: 'Loading models...',
    none: 'None',
    sizeLabel: 'Size:',
    fileLabel: 'File:',

    advancedTitle: 'Advanced Settings',
    typingDelays: 'Typing Delay Settings',
    clipboardUpdate: 'Clipboard Update Wait',
    keyboardEventsSettle: 'Keyboard Events Settle Wait',
    typingComplete: 'Typing Complete Wait',
    characterInterval: 'Character Interval',
    shortOperation: 'Short Operation Wait',
    milliseconds: 'milliseconds',
    restoreDefaults: 'Restore Defaults',
    saveAdvancedSettings: 'Save Advanced Settings',

    systemInfo: 'System Information',
    platform: 'Platform',
    arch: 'Architecture',
    tauriVersion: 'Tauri Version',
    osVersion: 'OS Version',
    kernelVersion: 'Kernel Version',
    memoryInfo: 'Memory Information',
    totalMemory: 'Total Memory',
    availableMemory: 'Available Memory',
    notAvailable: 'Not Available',

    // System Info - Additional
    systemInformationWithIcon: '💻 System Information',
    monitorSystemStatus: 'Monitor your system status and Voice Assistant configuration',
    systemStatusCard: 'System Status',
    hardwareInformation: 'Hardware Information',
    softwareInformation: 'Software Information',
    voiceAssistantStatusCard: 'Voice Assistant Status',
    noSystemInformation: 'No System Information',
    unableToRetrieveSystemInfo: 'Unable to retrieve system information. Make sure Voice Assistant is running in Tauri mode.',
    retrySystemInfo: 'Retry',

    serviceName: 'Service Name',
    status: 'Status',
    online: 'Online',
    offline: 'Offline',
    latency: 'Latency',
    todayUsage: 'Today Usage',
    successRate: 'Success Rate',
    secs: 'secs',

    language: 'Language',
    english: 'English',
    chinese: '中文',

    // History
    recentHistory: 'Recent History',
    clear: 'Clear',
    noHistoryYet: 'No history yet. Start recording to see results here.',
    unknownTime: 'Unknown time',

    // LiveData
    activeService: 'Active Service',
    lastLatency: 'Last Latency',
    todaysUsage: "Today's Usage",
    error: 'Error',
    unknown: 'Unknown',
    loading: 'Loading...',
    noRecordingsYet: 'No recordings yet',
    noData: 'No data',
    liveDataUnavailable: 'Live data unavailable',
    success: 'Success',

    // VoiceAssistantPanel
    voiceAssistantStatus: 'Voice Assistant Status',
    serviceStatusText: 'Service Status:',
    activeText: 'Active',
    inactiveText: 'Inactive',
    runningListening: (transcribe, translate) => `Voice Assistant is running and listening for hotkeys (${transcribe}, ${translate})`,
    useStartButton: 'Use the Start button in the top bar to activate Voice Assistant',

    // Additional Settings Text
    asrServiceSettings: 'ASR Service Settings',
    voiceRecognitionProvider: 'Voice Recognition Provider',
    shortcutsBehaviors: 'Shortcuts & Behaviors',
    whisperModelsDesc: 'Download and manage local Whisper models for offline speech recognition',
    advancedSettingsDesc: 'Advanced Settings',
    configureAdvancedSettings: 'Configure advanced audio and system settings',
    audioInputDevice: 'Audio Input Device',
    selectMicrophone: 'Select Microphone',
    selectMicrophoneDesc: '• Select your preferred microphone for voice input',
    testButtonDesc: '• Use the test button to verify microphone functionality',
    autoSaveDesc: '• Your selection will be saved automatically',
    systemAudioApiDesc: '• Using system audio API for device detection',
    audioSettings: 'Audio Settings',
    saveWavFilesLongDesc: 'Save recorded audio as WAV files after processing for debugging and backup purposes.',

    // Advanced Settings - Additional
    advancedSettingsFullTitle: 'Advanced Settings',
    runningInTauriNative: '✓ Running in Tauri native environment',
    refresh: 'Refresh',
    selectAMicrophone: 'Select a microphone...',
    default: '(Default)',
    selected: 'Selected:',
    currentSelection: 'Current Selection',
    noMicrophoneSelected: 'No microphone selected',
    testMicrophone: 'Test Microphone',
    noMicrophonesDetected: 'No microphones detected',
    checkSystemAudioSettings: 'Please check your system audio settings',
    grantMicrophonePermission: 'Please grant microphone permission to detect audio devices',
    refreshDevices: 'Refresh Devices',
    requestMicrophoneAccess: 'Request Microphone Access',
    noteChangesSavedAutomatically: '<strong>Note:</strong> Changes to this setting will be saved automatically.',
    micTestSuccess: '✅ Microphone test successful! Audio input is working properly.',
    micTestFailed: '❌ Microphone test failed! Please check your microphone settings.',

    // ASR Service Settings - Additional
    connectionConfig: 'Connection Config',
    localWhisperConfiguration: 'Local Whisper Configuration',
    localWhisperDescription: 'Local Whisper uses whisper-rs for on-device inference. The system automatically detects and uses your downloaded model at:',
    localWhisperNoConfig: 'No configuration required - just ensure the model file is present.',
    loadingConfiguration: 'Loading configuration...',
    saveConfiguration: 'Save Configuration',
    testAsrWithWavFile: 'Test ASR with WAV File',
    chooseWavFile: 'Choose WAV File',
    transcriptionResult: 'Transcription Result:',
    cloudAsrEndpoint: 'Cloud ASR API Endpoint',
    cloudAsrEndpointPlaceholder: 'https://api.example.com/v1/audio/transcriptions',
    cloudAsrApiKey: 'Cloud ASR API Key',
    cloudAsrApiKeyPlaceholder: 'sk-...',
    securityNotice: 'Security Notice:',
    securityNoticeDesc: 'API keys are sensitive credentials. Never share them publicly or commit to version control. Use HTTPS endpoints when possible.',
    cloudAsrMultipleProviders: 'Cloud ASR supports multiple providers (SiliconFlow, Groq). The endpoint determines which provider to use.',
    debugPanelTitle: '🔍 ASR Debug Console',
    copy: 'Copy',
    clearLogs: 'Clear',
    hideDebugPanel: 'Hide',
    showDebugPanel: 'Show',
    debugPanelLogs: 'Debug Panel',
    noDebugLogsYet: 'No debug logs yet. Try performing an action...',
    environment: 'Environment:',
    tauriDesktop: 'Tauri Desktop',
    browser: 'Browser',
    debugRefresh: '🔄 Debug Refresh',

    // Post-process / Text Correction Settings
    postProcessTextCorrection: 'AI Text Correction',
    postProcessTextCorrectionDesc: 'Use AI model to fix ASR text errors after processing',
    postProcessEnabled: 'Enable Text Correction',
    postProcessEnabledDesc: 'Use AI model to correct ASR text before output',
    postProcessProvider: 'Provider',
    postProcessProviderOllama: 'Ollama (Local)',
    postProcessProviderDeepSeek: 'DeepSeek (Cloud)',
    postProcessEndpoint: 'API Endpoint',
    postProcessEndpointPlaceholder: 'http://localhost:11434/api/chat',
    postProcessModel: 'Model Name',
    postProcessModelPlaceholder: 'llama3.2:latest or deepseek-chat',
    postProcessSystemPrompt: 'System Prompt',
    postProcessSystemPromptPlaceholder: 'You are a text correction assistant. Fix grammar, spelling, and punctuation errors in the user input. Return only the corrected text without explanation.',
    postProcessTimeout: 'Timeout',
    postProcessTimeoutSeconds: 'seconds',
    postProcessInfoBox: 'Note: Make sure the AI service is running. Text correction adds 1-3 seconds of latency. Lightweight models are recommended for faster response.',
    postProcessAutoSaved: 'Configuration changes will be saved automatically.',
    postProcessApiKey: 'API Key',
    postProcessApiKeyPlaceholder: 'sk-...',
  },
};

export type Language = 'zh-CN' | 'en-US';

export const getTranslations = (lang: Language): Translations => {
  return translations[lang] || translations['en-US'];
};
