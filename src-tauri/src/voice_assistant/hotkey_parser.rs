use rdev::Key;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ParsedHotkey {
    pub modifiers: HashSet<Key>,
    pub main_key: Option<Key>,
    pub key_combination: Vec<Key>, // 顺序的按键组合用于匹配
}

impl ParsedHotkey {
    /// 解析热键字符串（如 "Ctrl + F4", "Shift + Alt + T"）为按键组合
    pub fn parse(hotkey_str: &str) -> Result<Self, String> {
        let mut modifiers = HashSet::new();
        let mut main_key: Option<Key> = None;
        let mut key_combination = Vec::new();

        if hotkey_str.trim().is_empty() {
            return Err("Hotkey string is empty".to_string());
        }

        let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();

        for part in parts {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => {
                    modifiers.insert(Key::ControlLeft);
                    key_combination.push(Key::ControlLeft);
                },
                "alt" => {
                    modifiers.insert(Key::Alt);
                    key_combination.push(Key::Alt);
                },
                "shift" => {
                    modifiers.insert(Key::ShiftLeft); // 使用Left Shift
                    key_combination.push(Key::ShiftLeft);
                },
                "meta" | "cmd" | "command" => {
                    modifiers.insert(Key::MetaLeft);
                    key_combination.push(Key::MetaLeft);
                },
                "win" | "windows" => {
                    modifiers.insert(Key::MetaLeft); // Windows键映射为Meta
                    key_combination.push(Key::MetaLeft);
                },
                "space" => {
                    main_key = Some(Key::Space);
                    key_combination.push(Key::Space);
                },
                "enter" | "return" => {
                    main_key = Some(Key::Return);
                    key_combination.push(Key::Return);
                },
                "escape" | "esc" => {
                    main_key = Some(Key::Escape);
                    key_combination.push(Key::Escape);
                },
                "tab" => {
                    main_key = Some(Key::Tab);
                    key_combination.push(Key::Tab);
                },
                "backspace" => {
                    main_key = Some(Key::Backspace);
                    key_combination.push(Key::Backspace);
                },
                "delete" => {
                    main_key = Some(Key::Delete);
                    key_combination.push(Key::Delete);
                },
                "up" => {
                    main_key = Some(Key::UpArrow);
                    key_combination.push(Key::UpArrow);
                },
                "down" => {
                    main_key = Some(Key::DownArrow);
                    key_combination.push(Key::DownArrow);
                },
                "left" => {
                    main_key = Some(Key::LeftArrow);
                    key_combination.push(Key::LeftArrow);
                },
                "right" => {
                    main_key = Some(Key::RightArrow);
                    key_combination.push(Key::RightArrow);
                },
                "home" => {
                    main_key = Some(Key::Home);
                    key_combination.push(Key::Home);
                },
                "end" => {
                    main_key = Some(Key::End);
                    key_combination.push(Key::End);
                },
                "pageup" | "page up" => {
                    main_key = Some(Key::PageUp);
                    key_combination.push(Key::PageUp);
                },
                "pagedown" | "page down" => {
                    main_key = Some(Key::PageDown);
                    key_combination.push(Key::PageDown);
                },
                // 字母键
                s if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() => {
                    let key_str = s.to_uppercase();
                    let key = match key_str.as_str() {
                        "A" => Key::KeyA,
                        "B" => Key::KeyB,
                        "C" => Key::KeyC,
                        "D" => Key::KeyD,
                        "E" => Key::KeyE,
                        "F" => Key::KeyF,
                        "G" => Key::KeyG,
                        "H" => Key::KeyH,
                        "I" => Key::KeyI,
                        "J" => Key::KeyJ,
                        "K" => Key::KeyK,
                        "L" => Key::KeyL,
                        "M" => Key::KeyM,
                        "N" => Key::KeyN,
                        "O" => Key::KeyO,
                        "P" => Key::KeyP,
                        "Q" => Key::KeyQ,
                        "R" => Key::KeyR,
                        "S" => Key::KeyS,
                        "T" => Key::KeyT,
                        "U" => Key::KeyU,
                        "V" => Key::KeyV,
                        "W" => Key::KeyW,
                        "X" => Key::KeyX,
                        "Y" => Key::KeyY,
                        "Z" => Key::KeyZ,
                        _ => return Err(format!("Unsupported alphabetic key: {}", s)),
                    };
                    main_key = Some(key);
                    key_combination.push(key);
                },
                // 数字键
                s if s.len() == 1 && s.chars().next().unwrap().is_ascii_digit() => {
                    let key = match s {
                        "0" => Key::Num0,
                        "1" => Key::Num1,
                        "2" => Key::Num2,
                        "3" => Key::Num3,
                        "4" => Key::Num4,
                        "5" => Key::Num5,
                        "6" => Key::Num6,
                        "7" => Key::Num7,
                        "8" => Key::Num8,
                        "9" => Key::Num9,
                        _ => return Err(format!("Unsupported numeric key: {}", s)),
                    };
                    main_key = Some(key);
                    key_combination.push(key);
                },
                // F键
                s if s.to_uppercase().starts_with('F') && s.len() > 1 => {
                    if let Ok(num) = s[1..].parse::<u32>() {
                        if num >= 1 && num <= 24 {
                            let key = match num {
                                1 => Key::F1,
                                2 => Key::F2,
                                3 => Key::F3,
                                4 => Key::F4,
                                5 => Key::F5,
                                6 => Key::F6,
                                7 => Key::F7,
                                8 => Key::F8,
                                9 => Key::F9,
                                10 => Key::F10,
                                11 => Key::F11,
                                12 => Key::F12,
                                13..=24 => Key::Unknown(0), // rdev不支持F13-F24，使用0作为占位符
                                _ => return Err(format!("Invalid F-key number: {}", num)),
                            };
                            main_key = Some(key);
                            key_combination.push(key);
                        } else {
                            return Err(format!("F-key number out of range: {}", num));
                        }
                    } else {
                        return Err(format!("Invalid F-key format: {}", s));
                    }
                },
                _ => {
                    return Err(format!("Unsupported key: {}", part));
                }
            }
        }

        // 确保至少有一个主键（除了修饰键外）
        if main_key.is_none() {
            return Err("Hotkey must contain at least one main key (modifier-only shortcuts not supported)".to_string());
        }

        Ok(ParsedHotkey {
            modifiers,
            main_key,
            key_combination,
        })
    }

    /// 检查当前按键状态是否匹配此热键
    pub fn matches(&self, pressed_keys: &HashSet<Key>) -> bool {
        // 检查所有必需的按键是否都被按下
        for required_key in &self.key_combination {
            if !pressed_keys.contains(required_key) {
                return false;
            }
        }

        // 所有必需的按键都找到了
        true
    }

    /// 获取热键的显示名称
    pub fn get_display_name(&self) -> String {
        let mut parts = Vec::new();

        // 添加修饰键
        if self.modifiers.contains(&Key::ControlLeft) {
            parts.push("Ctrl");
        }
        if self.modifiers.contains(&Key::Alt) {
            parts.push("Alt");
        }
        if self.modifiers.contains(&Key::ShiftLeft) || self.modifiers.contains(&Key::ShiftRight) {
            parts.push("Shift");
        }
        if self.modifiers.contains(&Key::MetaLeft) {
            parts.push("Meta");
        }

        // 添加主键
        if let Some(main_key) = &self.main_key {
            parts.push(&match main_key {
                Key::Space => "Space",
                Key::Return => "Enter",
                Key::Escape => "Esc",
                Key::Tab => "Tab",
                Key::Backspace => "Backspace",
                Key::Delete => "Delete",
                Key::UpArrow => "Up",
                Key::DownArrow => "Down",
                Key::LeftArrow => "Left",
                Key::RightArrow => "Right",
                Key::Home => "Home",
                Key::End => "End",
                Key::PageUp => "PageUp",
                Key::PageDown => "PageDown",
                Key::F1 => "F1",
                Key::F2 => "F2",
                Key::F3 => "F3",
                Key::F4 => "F4",
                Key::F5 => "F5",
                Key::F6 => "F6",
                Key::F7 => "F7",
                Key::F8 => "F8",
                Key::F9 => "F9",
                Key::F10 => "F10",
                Key::F11 => "F11",
                Key::F12 => "F12",
                Key::KeyA => "A",
                Key::KeyB => "B",
                Key::KeyC => "C",
                Key::KeyD => "D",
                Key::KeyE => "E",
                Key::KeyF => "F",
                Key::KeyG => "G",
                Key::KeyH => "H",
                Key::KeyI => "I",
                Key::KeyJ => "J",
                Key::KeyK => "K",
                Key::KeyL => "L",
                Key::KeyM => "M",
                Key::KeyN => "N",
                Key::KeyO => "O",
                Key::KeyP => "P",
                Key::KeyQ => "Q",
                Key::KeyR => "R",
                Key::KeyS => "S",
                Key::KeyT => "T",
                Key::KeyU => "U",
                Key::KeyV => "V",
                Key::KeyW => "W",
                Key::KeyX => "X",
                Key::KeyY => "Y",
                Key::KeyZ => "Z",
                Key::Num0 => "0",
                Key::Num1 => "1",
                Key::Num2 => "2",
                Key::Num3 => "3",
                Key::Num4 => "4",
                Key::Num5 => "5",
                Key::Num6 => "6",
                Key::Num7 => "7",
                Key::Num8 => "8",
                Key::Num9 => "9",
                Key::Unknown(_) => "Unknown",
                _ => "Unknown",
            });
        }

        parts.join(" + ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_hotkey() {
        let hotkey = ParsedHotkey::parse("F4").unwrap();
        assert_eq!(hotkey.get_display_name(), "F4");
    }

    #[test]
    fn test_parse_modifier_hotkey() {
        let hotkey = ParsedHotkey::parse("Ctrl + F4").unwrap();
        assert_eq!(hotkey.get_display_name(), "Ctrl + F4");
    }

    #[test]
    fn test_parse_complex_hotkey() {
        let hotkey = ParsedHotkey::parse("Ctrl + Shift + Alt + F4").unwrap();
        assert_eq!(hotkey.get_display_name(), "Ctrl + Shift + Alt + F4");
    }

    #[test]
    fn test_parse_case_insensitive() {
        let hotkey1 = ParsedHotkey::parse("ctrl + f4").unwrap();
        let hotkey2 = ParsedHotkey::parse("CTRL + F4").unwrap();
        assert_eq!(hotkey1.get_display_name(), hotkey2.get_display_name());
    }

    #[test]
    fn test_parse_invalid_hotkey() {
        assert!(ParsedHotkey::parse("").is_err());
        assert!(ParsedHotkey::parse("Ctrl").is_err()); // 只有修饰键，没有主键
    }
}