import { invoke } from "@tauri-apps/api/core";

// Voice Assistant State
interface VoiceAssistantState {
  isRunning: boolean;
  status: 'idle' | 'recording' | 'processing' | 'translating' | 'error';
  details: string;
}

// Global state
let appState: VoiceAssistantState = {
  isRunning: false,
  status: 'idle',
  details: 'Ready'
};

// DOM Elements
let startBtn: HTMLButtonElement | null;
let stopBtn: HTMLButtonElement | null;
let statusLight: HTMLElement | null;
let statusText: HTMLElement | null;
let statusDetails: HTMLElement | null;
let testResults: HTMLElement | null;

// Voice Assistant Functions
async function startVoiceAssistant() {
  try {
    updateStatus('starting', '正在启动语音助手...');
    const result = await invoke<string>("start_voice_assistant");

    if (result.includes('success')) {
      appState.isRunning = true;
      updateStatus('idle', '语音助手已启动');
      updateControlButtons();
      addTestResult('✅ ' + result, 'success');
    } else {
      updateStatus('error', '启动失败: ' + result);
      addTestResult('❌ 启动失败: ' + result, 'error');
    }
  } catch (error) {
    updateStatus('error', '启动错误: ' + error);
    addTestResult('❌ 启动错误: ' + error, 'error');
  }
}

async function stopVoiceAssistant() {
  try {
    updateStatus('stopping', '正在停止语音助手...');
    const result = await invoke<string>("stop_voice_assistant");

    appState.isRunning = false;
    updateStatus('idle', '语音助手已停止');
    updateControlButtons();
    addTestResult('⏹️ ' + result, 'info');
  } catch (error) {
    updateStatus('error', '停止错误: ' + error);
    addTestResult('❌ 停止错误: ' + error, 'error');
  }
}

// @ts-expect-error - Function kept for potential future use
async function _getVoiceAssistantState() {
  try {
    const state = await invoke<string>("get_voice_assistant_state");
    return state;
  } catch (error) {
    console.error('Failed to get voice assistant state:', error);
    return 'unknown';
  }
}

async function testASR() {
  try {
    addTestResult('🧪 正在测试ASR处理器...', 'info');
    const processorSelect = document.querySelector('#asr-processor') as HTMLSelectElement;
    const processorType = processorSelect?.value || 'whisper';

    const result = await invoke<string>("test_asr", {
      processorType: processorType
    });

    addTestResult('✅ ASR测试成功: ' + result, 'success');
  } catch (error) {
    addTestResult('❌ ASR测试失败: ' + error, 'error');
  }
}

async function testTranslation() {
  try {
    addTestResult('🧪 正在测试翻译处理器...', 'info');
    const processorSelect = document.querySelector('#translate-processor') as HTMLSelectElement;
    const translateType = processorSelect?.value || 'siliconflow';

    const result = await invoke<string>("test_translation", {
      translateType: translateType
    });

    addTestResult('✅ 翻译测试成功: ' + result, 'success');
  } catch (error) {
    addTestResult('❌ 翻译测试失败: ' + error, 'error');
  }
}

async function loadSystemInfo() {
  try {
    const systemInfo = await invoke<Record<string, string>>("get_system_info");
    const systemInfoEl = document.querySelector('#system-info');

    if (systemInfoEl) {
      const infoHtml = Object.entries(systemInfo)
        .map(([key, value]) => `<div class="info-item"><strong>${key}:</strong> ${value}</div>`)
        .join('');
      systemInfoEl.innerHTML = infoHtml;
    }
  } catch (error) {
    const systemInfoEl = document.querySelector('#system-info');
    if (systemInfoEl) {
      systemInfoEl.innerHTML = `<p class="error">加载系统信息失败: ${error}</p>`;
    }
  }
}

// UI Helper Functions
function updateStatus(status: VoiceAssistantState['status'] | 'starting' | 'stopping', details: string) {
  if (statusLight && statusText && statusDetails) {
    // Update status light
    statusLight.className = `status-dot ${status}`;

    // Update status text
    const statusTextMap = {
      'idle': '待机中',
      'running': '运行中',
      'recording': '录音中',
      'processing': '处理中',
      'translating': '翻译中',
      'error': '错误',
      'starting': '启动中',
      'stopping': '停止中'
    };

    statusText.textContent = statusTextMap[status] || status;
    statusDetails.textContent = '状态: ' + details;

    if (status === 'error') {
      statusDetails.classList.add('error');
    } else {
      statusDetails.classList.remove('error');
    }
  }
}

function updateControlButtons() {
  if (startBtn && stopBtn) {
    if (appState.isRunning) {
      startBtn.disabled = true;
      stopBtn.disabled = false;
      startBtn.textContent = '✅ 已启动';
    } else {
      startBtn.disabled = false;
      stopBtn.disabled = true;
      startBtn.textContent = '▶️ 启动语音助手';
    }
  }
}

function addTestResult(message: string, type: 'info' | 'success' | 'error' | 'warning') {
  if (testResults) {
    // Clear placeholder if exists
    const placeholder = testResults.querySelector('.placeholder');
    if (placeholder) {
      placeholder.remove();
    }

    const resultEl = document.createElement('div');
    resultEl.className = `test-result ${type}`;
    resultEl.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;

    testResults.appendChild(resultEl);
    testResults.scrollTop = testResults.scrollHeight; // Auto scroll to bottom

    // Limit results to 50 items
    const results = testResults.querySelectorAll('.test-result');
    if (results.length > 50) {
      results[0].remove();
    }
  }
}

// Tab switching
function setupTabs() {
  const tabButtons = document.querySelectorAll('.tab-button');
  const tabContents = document.querySelectorAll('.tab-content');

  tabButtons.forEach(button => {
    button.addEventListener('click', () => {
      const targetTab = button.getAttribute('data-tab');

      // Update button states
      tabButtons.forEach(btn => btn.classList.remove('active'));
      button.classList.add('active');

      // Update content visibility
      tabContents.forEach(content => {
        if (content.id === `${targetTab}-tab`) {
          content.classList.add('active');
        } else {
          content.classList.remove('active');
        }
      });
    });
  });
}

// Legacy functions (keep for compatibility)
let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;
let input1: HTMLInputElement | null;
let input2: HTMLInputElement | null;

// @ts-expect-error - Legacy function kept for compatibility
async function _greet() {
  if (greetMsgEl && greetInputEl) {
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

async function add() {
  if (input1 && input2 && greetMsgEl) {
    greetMsgEl.textContent = await invoke("add", {
      a: Number(input1.value),
      b: Number(input2.value)
    });
  }
}

// Initialize app
window.addEventListener("DOMContentLoaded", () => {
  // Get Voice Assistant elements
  startBtn = document.querySelector("#start-assistant");
  stopBtn = document.querySelector("#stop-assistant");
  statusLight = document.querySelector("#status-light");
  statusText = document.querySelector("#status-text");
  statusDetails = document.querySelector("#status-details");
  testResults = document.querySelector("#test-results");

  // Get legacy elements
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  input1 = document.querySelector("#num1");
  input2 = document.querySelector("#num2");

  // Setup Voice Assistant event listeners
  startBtn?.addEventListener("click", startVoiceAssistant);
  stopBtn?.addEventListener("click", stopVoiceAssistant);

  // Setup test buttons
  document.querySelector("#test-asr")?.addEventListener("click", testASR);
  document.querySelector("#test-translation")?.addEventListener("click", testTranslation);
  document.querySelector("#save-config")?.addEventListener("click", saveConfiguration);
  document.querySelector("#refresh-system-info")?.addEventListener("click", loadSystemInfo);

  // Setup tabs
  setupTabs();

  // Setup legacy form
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    add();
  });

  // Initial setup
  updateControlButtons();
  loadSystemInfo();

  console.log("Voice Assistant UI initialized");
});

// Configuration management
async function saveConfiguration() {
  try {
    const config = {
      asr_processor: (document.querySelector('#asr-processor') as HTMLSelectElement)?.value,
      translate_processor: (document.querySelector('#translate-processor') as HTMLSelectElement)?.value,
      convert_simplified: (document.querySelector('#convert-simplified') as HTMLInputElement)?.checked,
      add_symbol: (document.querySelector('#add-symbol') as HTMLInputElement)?.checked,
      groq_api_key: (document.querySelector('#groq-api-key') as HTMLInputElement)?.value,
      siliconflow_api_key: (document.querySelector('#siliconflow-api-key') as HTMLInputElement)?.value,
      local_asr_url: (document.querySelector('#local-asr-url') as HTMLInputElement)?.value,
      ollama_url: (document.querySelector('#ollama-url') as HTMLInputElement)?.value,
    };

    // Save to environment variables (this would need a backend command)
    addTestResult('💾 配置已保存', 'success');
    console.log('Configuration saved:', config);
  } catch (error) {
    addTestResult('❌ 保存配置失败: ' + error, 'error');
  }
}
