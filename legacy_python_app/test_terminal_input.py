#!/usr/bin/env python3
"""
测试终端环境下的文本输入功能
"""

import sys
import os
import time
from pynput.keyboard import Controller, Key

# 添加项目根目录到路径
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from src.keyboard.listener import KeyboardManager
from src.utils.logger import logger

def test_terminal_detection():
    """测试终端环境检测"""
    print("=== 测试终端环境检测 ===")

    # 创建键盘管理器实例
    def dummy_start(): pass
    def dummy_stop(): pass
    def dummy_translate_start(): pass
    def dummy_translate_stop(): pass
    def dummy_reset(): pass

    keyboard_manager = KeyboardManager(
        on_record_start=dummy_start,
        on_record_stop=dummy_stop,
        on_translate_start=dummy_translate_start,
        on_translate_stop=dummy_translate_stop,
        on_reset_state=dummy_reset
    )

    # 测试终端检测
    is_terminal = keyboard_manager.detect_terminal_environment()
    print(f"检测结果: {'终端环境' if is_terminal else 'GUI环境'}")

    # 显示相关环境变量
    print("\n=== 环境变量 ===")
    terminal_vars = ['TERM', 'SHELL', 'PS1', 'PROMPT', 'SSH_TTY', 'WT_SESSION', 'CONEMUANSI']
    for var in terminal_vars:
        value = os.getenv(var)
        if value:
            print(f"{var}={value}")

    return is_terminal

def test_character_input():
    """测试逐字符输入"""
    print("\n=== 测试逐字符输入 ===")
    print("请在3秒后将光标定位到文本编辑器中...")

    # 创建键盘管理器
    def dummy_start(): pass
    def dummy_stop(): pass
    def dummy_translate_start(): pass
    def dummy_translate_stop(): pass
    def dummy_reset(): pass

    keyboard_manager = KeyboardManager(
        on_record_start=dummy_start,
        on_record_stop=dummy_stop,
        on_translate_start=dummy_translate_start,
        on_translate_stop=dummy_translate_stop,
        on_reset_state=dummy_reset
    )

    # 等待3秒让用户定位光标
    time.sleep(3)

    # 测试文本
    test_text = "Hello, World! 你好世界！ 123"

    print(f"正在输入测试文本: {test_text}")
    print("使用逐字符输入方式...")

    try:
        keyboard_manager.type_text_character_by_character(test_text)
        print("✅ 逐字符输入完成")
    except Exception as e:
        print(f"❌ 逐字符输入失败: {e}")

def test_smart_input():
    """测试智能输入选择"""
    print("\n=== 测试智能输入选择 ===")
    print("请在3秒后将光标定位到文本编辑器中...")

    # 创建键盘管理器
    def dummy_start(): pass
    def dummy_stop(): pass
    def dummy_translate_start(): pass
    def dummy_translate_stop(): pass
    def dummy_reset(): pass

    keyboard_manager = KeyboardManager(
        on_record_start=dummy_start,
        on_record_stop=dummy_stop,
        on_translate_start=dummy_translate_start,
        on_translate_stop=dummy_translate_stop,
        on_reset_state=dummy_reset
    )

    # 等待3秒让用户定位光标
    time.sleep(3)

    # 测试文本
    test_text = "智能输入测试文本"

    print(f"正在输入测试文本: {test_text}")
    print("使用智能输入方式...")

    try:
        keyboard_manager.type_text(test_text)
        print("✅ 智能输入完成")
    except Exception as e:
        print(f"❌ 智能输入失败: {e}")

def main():
    """主测试函数"""
    print("Whisper-Input 终端环境文本输入测试")
    print("="*50)

    # 设置日志级别为DEBUG以查看详细信息
    logger.setLevel(1)  # DEBUG level

    try:
        # 测试1: 终端环境检测
        is_terminal = test_terminal_detection()

        # 测试2: 逐字符输入
        test_character_input()

        # 测试3: 智能输入选择
        test_smart_input()

        print("\n" + "="*50)
        print("测试完成！")
        print(f"当前环境: {'终端环境' if is_terminal else 'GUI环境'}")
        print("如果看到文本成功输入，说明功能正常工作。")

    except KeyboardInterrupt:
        print("\n测试被用户中断")
    except Exception as e:
        print(f"测试过程中出现错误: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()