import React, { useState, useRef, useEffect, useCallback } from 'react';
import { Edit, Check, X } from 'lucide-react';

interface HotkeyInputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  autoFocus?: boolean;
}

export const HotkeyInput: React.FC<HotkeyInputProps> = ({
  label,
  value,
  onChange,
  placeholder = "Press keys...",
  disabled = false,
  autoFocus = false
}) => {
  const [isRecording, setIsRecording] = useState(false);
  const [currentKeys, setCurrentKeys] = useState<string[]>([]);
  const [editingValue, setEditingValue] = useState(value);
  const inputRef = useRef<HTMLInputElement>(null);

  // 格式化按键显示
  const formatKey = (key: string): string => {
    const keyMap: { [key: string]: string } = {
      ' ': 'Space',
      'Control': 'Ctrl',
      'ArrowLeft': 'Left',
      'ArrowRight': 'Right',
      'ArrowUp': 'Up',
      'ArrowDown': 'Down',
      'Meta': 'Cmd',
      'OS': 'Win'
    };
    return keyMap[key] || key;
  };

  // 格式化按键组合显示
  const formatKeys = (keys: string[]): string => {
    return keys
      .map(key => formatKey(key))
      .sort((a, b) => {
        // 确保修饰键在前：Ctrl -> Alt -> Shift -> Meta/Cmd -> 其他
        const order = ['Ctrl', 'Alt', 'Shift', 'Cmd', 'Win'];
        const aIndex = order.findIndex(k => a.includes(k));
        const bIndex = order.findIndex(k => b.includes(k));

        if (aIndex !== -1 && bIndex !== -1) return aIndex - bIndex;
        if (aIndex !== -1) return -1;
        if (bIndex !== -1) return 1;
        return a.localeCompare(b);
      })
      .join(' + ');
  };

  // 解析已有的快捷键字符串为按键数组
  const parseHotkeyString = (hotkey: string): string[] => {
    if (!hotkey) return [];
    return hotkey.split(' + ').map(key => {
      // 反向映射
      const reverseKeyMap: { [key: string]: string } = {
        'Space': ' ',
        'Ctrl': 'Control',
        'Cmd': 'Meta',
        'Win': 'OS',
        'Left': 'ArrowLeft',
        'Right': 'ArrowRight',
        'Up': 'ArrowUp',
        'Down': 'ArrowDown'
      };
      return reverseKeyMap[key] || key;
    });
  };

  // 开始录制
  const startRecording = useCallback(() => {
    if (disabled) return;

    setIsRecording(true);
    setCurrentKeys([]);
    setEditingValue('Press keys...');

    // 让输入框获得焦点
    if (inputRef.current) {
      inputRef.current.focus();
    }
  }, [disabled]);

  // 停止录制
  const stopRecording = useCallback((accept: boolean) => {
    setIsRecording(false);

    if (accept && currentKeys.length > 0) {
      const formattedValue = formatKeys(currentKeys);
      setEditingValue(formattedValue);
      onChange(formattedValue);
    } else {
      setEditingValue(value);
      setCurrentKeys([]);
    }
  }, [currentKeys, onChange, value]);

  // 处理按键按下
  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!isRecording) return;

    e.preventDefault();
    e.stopPropagation();

    const key = e.key;

    // 忽略某些键
    if (['Tab', 'Enter', 'Escape'].includes(key)) {
      if (key === 'Enter') {
        stopRecording(true);
      } else if (key === 'Escape') {
        stopRecording(false);
      }
      return;
    }

    // 添加修饰键
    const keys = new Set<string>();

    if (e.ctrlKey) keys.add('Control');
    if (e.altKey) keys.add('Alt');
    if (e.shiftKey) keys.add('Shift');
    if (e.metaKey) keys.add('Meta');

    // 添加主键（如果不是修饰键）
    if (!['Control', 'Alt', 'Shift', 'Meta'].includes(key)) {
      keys.add(key);
    }

    const newKeys = Array.from(keys);
    setCurrentKeys(newKeys);

    // 实时更新显示
    setEditingValue(formatKeys(newKeys));
  }, [isRecording, currentKeys, stopRecording]);

  // 处理键盘释放（用于检测快捷键组合结束）
  const handleKeyUp = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!isRecording || currentKeys.length === 0) return;

    // 如果所有按键都释放了，自动确认
    const allKeysReleased = !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey;
    if (allKeysReleased && currentKeys.length > 0) {
      setTimeout(() => {
        if (currentKeys.length > 0) {
          stopRecording(true);
        }
      }, 100);
    }
  }, [isRecording, currentKeys, stopRecording]);

  // 处理失焦
  const handleBlur = useCallback(() => {
    if (isRecording) {
      stopRecording(true);
    }
  }, [isRecording, stopRecording]);

  // 同步外部value变化
  useEffect(() => {
    if (!isRecording && value !== editingValue) {
      setEditingValue(value);
      setCurrentKeys(parseHotkeyString(value));
    }
  }, [value, isRecording, editingValue]);

  return (
    <div className="relative">
      <label className="block text-sm font-medium text-gray-700 mb-1">
        {label}
      </label>
      <div className="relative">
        <input
          ref={inputRef}
          type="text"
          value={editingValue}
          onChange={() => {}} // 阻止手动输入
          onKeyDown={handleKeyDown}
          onKeyUp={handleKeyUp}
          onBlur={handleBlur}
          onFocus={startRecording}
          placeholder={placeholder}
          disabled={disabled}
          autoFocus={autoFocus}
          readOnly={!isRecording}
          className={`
            w-full px-3 py-2 pr-10 border rounded-lg text-sm transition-colors
            ${isRecording
              ? 'border-blue-500 bg-blue-50 text-blue-900 placeholder-blue-400'
              : disabled
                ? 'border-gray-200 bg-gray-50 text-gray-500 cursor-not-allowed'
                : 'border-gray-300 bg-white text-gray-900 placeholder-gray-400 hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500'
            }
            ${!isRecording && !disabled ? 'cursor-pointer' : ''}
          `}
        />

        <div className="absolute right-2 top-1/2 transform -translate-y-1/2">
          {isRecording ? (
            <div className="flex space-x-1">
              <button
                onClick={() => stopRecording(true)}
                className="p-1 text-green-600 hover:text-green-700 transition-colors"
                title="Accept"
              >
                <Check className="w-4 h-4" />
              </button>
              <button
                onClick={() => stopRecording(false)}
                className="p-1 text-red-600 hover:text-red-700 transition-colors"
                title="Cancel"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          ) : (
            <Edit className="w-4 h-4 text-gray-400" />
          )}
        </div>
      </div>

      {isRecording && (
        <div className="mt-1 text-xs text-blue-600 animate-pulse">
          Press keys... (Enter to accept, Escape to cancel)
        </div>
      )}

      {!isRecording && value && (
        <div className="mt-1 text-xs text-gray-500">
          Click to change shortcut
        </div>
      )}
    </div>
  );
};