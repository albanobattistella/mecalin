use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Finger {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    LeftThumb,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
    RightThumb,
    BothThumbs,
}

impl Finger {
    pub fn hand(&self) -> Option<&str> {
        match self {
            Finger::LeftPinky
            | Finger::LeftRing
            | Finger::LeftMiddle
            | Finger::LeftIndex
            | Finger::LeftThumb => Some("left"),
            Finger::RightIndex
            | Finger::RightMiddle
            | Finger::RightRing
            | Finger::RightPinky
            | Finger::RightThumb => Some("right"),
            Finger::BothThumbs => None,
        }
    }

    pub fn opposite_thumb(&self) -> Finger {
        match self.hand() {
            Some("left") => Finger::RightThumb,
            Some("right") => Finger::LeftThumb,
            _ => Finger::RightThumb,
        }
    }
}

impl FromStr for Finger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left_pinky" => Ok(Finger::LeftPinky),
            "left_ring" => Ok(Finger::LeftRing),
            "left_middle" => Ok(Finger::LeftMiddle),
            "left_index" => Ok(Finger::LeftIndex),
            "left_thumb" => Ok(Finger::LeftThumb),
            "right_index" => Ok(Finger::RightIndex),
            "right_middle" => Ok(Finger::RightMiddle),
            "right_ring" => Ok(Finger::RightRing),
            "right_pinky" => Ok(Finger::RightPinky),
            "right_thumb" => Ok(Finger::RightThumb),
            "both_thumbs" => Ok(Finger::BothThumbs),
            _ => Err(format!("Unknown finger: {}", s)),
        }
    }
}

impl fmt::Display for Finger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Finger::LeftPinky => "left_pinky",
            Finger::LeftRing => "left_ring",
            Finger::LeftMiddle => "left_middle",
            Finger::LeftIndex => "left_index",
            Finger::LeftThumb => "left_thumb",
            Finger::RightIndex => "right_index",
            Finger::RightMiddle => "right_middle",
            Finger::RightRing => "right_ring",
            Finger::RightPinky => "right_pinky",
            Finger::RightThumb => "right_thumb",
            Finger::BothThumbs => "both_thumbs",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub base: String,
    #[serde(default)]
    pub label: Option<String>,
    pub shift: Option<String>,
    pub altgr: Option<String>,
    pub finger: Finger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyLevel {
    Base,
    Shift,
    AltGr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedKey {
    pub row: usize,
    pub col: usize,
    pub finger: Finger,
    pub level: KeyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierKey {
    pub label: String,
    pub finger: Finger,
    #[serde(default)]
    pub rows: Option<Vec<u8>>,
}

impl ModifierKey {
    pub fn spans_row(&self, row: u8) -> bool {
        self.rows.as_ref().is_some_and(|r| r.contains(&row))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardLayout {
    pub name: String,
    pub keys: Vec<Vec<KeyInfo>>,
    pub space: KeyInfo,
    #[serde(default)]
    pub modifiers: HashMap<String, ModifierKey>,
}

impl KeyboardLayout {
    pub fn load_from_json(
        layout: crate::language::Language,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::language::Language;
        let json_data = match layout {
            Language::Us => include_str!("../data/keyboard_layouts/us.json"),
            Language::Es => include_str!("../data/keyboard_layouts/es.json"),
            Language::De => include_str!("../data/keyboard_layouts/de.json"),
            Language::Fr => include_str!("../data/keyboard_layouts/fr.json"),
            // Galician shares the Spanish keyboard layout.
            Language::Gl => include_str!("../data/keyboard_layouts/gl.json"),
            Language::It => include_str!("../data/keyboard_layouts/it.json"),
            Language::Pl => include_str!("../data/keyboard_layouts/pl.json"),
            Language::Pt => include_str!("../data/keyboard_layouts/pt.json"),
            Language::PtBr => include_str!("../data/keyboard_layouts/pt_br.json"),
        };
        Ok(serde_json::from_str(json_data)?)
    }

    pub fn get_finger_for_char(&self, ch: char) -> Option<Finger> {
        if ch == ' ' {
            return Some(self.space.finger);
        }
        self.resolve_char(ch).map(|r| r.finger)
    }

    pub fn contains_character(&self, ch: char) -> bool {
        ch == ' ' || self.resolve_char(ch).is_some()
    }

    /// Find the key and level that produce `ch`. Each level is searched
    /// across the whole layout before the next one, so a character that is
    /// on a base level never resolves to another key's Shift or AltGr level.
    pub fn resolve_char(&self, ch: char) -> Option<ResolvedKey> {
        if ch == ' ' {
            return None;
        }

        let find = |level: KeyLevel, matches: &dyn Fn(&KeyInfo) -> bool| {
            self.keys.iter().enumerate().find_map(|(row, keys)| {
                keys.iter().position(matches).map(|col| ResolvedKey {
                    row,
                    col,
                    finger: keys[col].finger,
                    level,
                })
            })
        };

        find(KeyLevel::Base, &|k| first_char(Some(&k.base)) == Some(ch))
            .or_else(|| {
                find(KeyLevel::Shift, &|k| {
                    first_char(k.shift.as_deref()) == Some(ch)
                })
            })
            .or_else(|| {
                find(KeyLevel::Shift, &|k| {
                    first_char(Some(&k.base))
                        .is_some_and(|b| b.is_alphabetic() && b.to_uppercase().eq([ch]))
                })
            })
            .or_else(|| {
                find(KeyLevel::AltGr, &|k| {
                    first_char(k.altgr.as_deref()) == Some(ch)
                })
            })
    }

    /// Modifier keys (by `modifiers` name) to hold for `target`. Shift is
    /// pressed with the hand opposite to the one typing the key.
    pub fn modifier_keys_for(&self, target: &ResolvedKey) -> Vec<&'static str> {
        let names: &[&'static str] = match target.level {
            KeyLevel::Base => &[],
            KeyLevel::AltGr => &["alt_right"],
            KeyLevel::Shift => match target.finger.hand() {
                Some("left") => &["shift_right"],
                Some("right") => &["shift_left"],
                _ => &["shift_left", "shift_right"],
            },
        };
        names
            .iter()
            .copied()
            .filter(|name| self.modifiers.contains_key(*name))
            .collect()
    }

    pub fn modifier_finger_for(&self, target: &ResolvedKey) -> Option<Finger> {
        self.modifier_keys_for(target)
            .first()
            .and_then(|name| self.modifiers.get(*name))
            .map(|m| m.finger)
    }
}

fn first_char(s: Option<&str>) -> Option<char> {
    s.and_then(|s| s.chars().next())
}

impl Default for KeyboardLayout {
    fn default() -> Self {
        Self::load_from_json(crate::language::Language::Us).unwrap_or_else(|_| Self {
            name: "US".to_string(),
            keys: vec![vec![
                KeyInfo {
                    base: "q".to_string(),
                    label: None,
                    shift: Some("Q".to_string()),
                    altgr: None,
                    finger: Finger::LeftPinky,
                },
                KeyInfo {
                    base: "w".to_string(),
                    label: None,
                    shift: Some("W".to_string()),
                    altgr: None,
                    finger: Finger::LeftRing,
                },
                KeyInfo {
                    base: "e".to_string(),
                    label: None,
                    shift: Some("E".to_string()),
                    altgr: None,
                    finger: Finger::LeftMiddle,
                },
            ]],
            space: KeyInfo {
                base: " ".to_string(),
                label: Some("SPACE".to_string()),
                shift: None,
                altgr: None,
                finger: Finger::BothThumbs,
            },
            modifiers: HashMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_json_us() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Us).unwrap();
        assert_eq!(layout.name, "US QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_es() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Es).unwrap();
        assert_eq!(layout.name, "Spanish QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_de() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::De).unwrap();
        assert_eq!(layout.name, "German QWERTZ");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_it() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::It).unwrap();
        assert_eq!(layout.name, "Italian QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_pl() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Pl).unwrap();
        assert_eq!(layout.name, "Polish QWERTY");
        assert!(!layout.keys.is_empty());
    }

    #[test]
    fn test_load_from_json_fr() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Fr).unwrap();
        assert_eq!(layout.name, "French AZERTY");
        assert!(layout.modifiers.contains_key("shift_left"));
        assert!(layout.modifiers.contains_key("alt_right"));
        let enter = layout.modifiers.get("enter").unwrap();
        assert!(enter.spans_row(1));
        assert!(enter.spans_row(2));
    }

    #[test]
    fn test_all_layouts_load() {
        use crate::language::Language;
        let expected = [
            (Language::Us, "US QWERTY"),
            (Language::Es, "Spanish QWERTY"),
            (Language::De, "German QWERTZ"),
            (Language::Fr, "French AZERTY"),
            (Language::Gl, "Galician QWERTY"),
            (Language::It, "Italian QWERTY"),
            (Language::Pl, "Polish QWERTY"),
            (Language::Pt, "Portuguese QWERTY"),
            (Language::PtBr, "Brazilian ABNT2"),
        ];
        for (language, name) in expected {
            let layout = KeyboardLayout::load_from_json(language)
                .unwrap_or_else(|e| panic!("{language:?} layout failed to load: {e}"));
            assert_eq!(layout.name, name);
        }
    }

    #[test]
    fn test_contains_character_spanish() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Es).unwrap();

        // Test base characters (lowercase comparison)
        assert!(layout.contains_character('ñ'));
        assert!(layout.contains_character('Ñ'));
        assert!(layout.contains_character('a'));
        assert!(layout.contains_character('A'));

        // Test shift characters (exact match)
        assert!(layout.contains_character('!'));
        assert!(layout.contains_character('?'));

        // Test altgr characters (exact match)
        assert!(layout.contains_character('€'));
        assert!(layout.contains_character('['));

        // Test space
        assert!(layout.contains_character(' '));

        // Test characters not in layout
        assert!(!layout.contains_character('á'));
        assert!(!layout.contains_character('é'));
    }

    #[test]
    fn test_contains_character_us() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Us).unwrap();

        // Test base characters
        assert!(layout.contains_character('a'));
        assert!(layout.contains_character('z'));

        // Test that ñ exists in US layout as altgr character
        assert!(layout.contains_character('ñ'));

        // Test space
        assert!(layout.contains_character(' '));

        // Test characters not in layout (using characters that truly don't exist)
        assert!(!layout.contains_character('Ñ')); // Uppercase ñ not in US layout
        assert!(!layout.contains_character('ą')); // Polish character
        assert!(!layout.contains_character('ę')); // Polish character
    }

    #[test]
    fn test_get_finger_for_char_us() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Us).unwrap();

        // Test base characters
        assert_eq!(layout.get_finger_for_char('a'), Some(Finger::LeftPinky));
        assert_eq!(layout.get_finger_for_char('A'), Some(Finger::LeftPinky));
        assert_eq!(layout.get_finger_for_char('f'), Some(Finger::LeftIndex));
        assert_eq!(layout.get_finger_for_char('j'), Some(Finger::RightIndex));
        assert_eq!(layout.get_finger_for_char('l'), Some(Finger::RightRing));

        // Test space
        assert_eq!(layout.get_finger_for_char(' '), Some(Finger::BothThumbs));

        // Test shift characters
        assert_eq!(layout.get_finger_for_char('!'), Some(Finger::LeftPinky));
        assert_eq!(layout.get_finger_for_char('@'), Some(Finger::LeftRing));

        // Test character not in layout
        assert_eq!(layout.get_finger_for_char('Ñ'), None);
    }

    #[test]
    fn test_get_finger_for_char_es() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Es).unwrap();

        // Test Spanish-specific characters
        assert_eq!(layout.get_finger_for_char('ñ'), Some(Finger::RightPinky));
        assert_eq!(layout.get_finger_for_char('Ñ'), Some(Finger::RightPinky));

        // Test base characters
        assert_eq!(layout.get_finger_for_char('a'), Some(Finger::LeftPinky));
        assert_eq!(layout.get_finger_for_char('s'), Some(Finger::LeftRing));

        // Test space
        assert_eq!(layout.get_finger_for_char(' '), Some(Finger::BothThumbs));
    }

    fn create_test_layout() -> KeyboardLayout {
        KeyboardLayout {
            name: "Test".to_string(),
            keys: vec![vec![
                KeyInfo {
                    base: "'".to_string(),
                    label: None,
                    shift: Some("\"".to_string()),
                    altgr: Some("´".to_string()),
                    finger: Finger::RightPinky,
                },
                KeyInfo {
                    base: "9".to_string(),
                    label: None,
                    shift: Some("(".to_string()),
                    altgr: Some("'".to_string()),
                    finger: Finger::RightRing,
                },
                KeyInfo {
                    base: "0".to_string(),
                    label: None,
                    shift: Some(")".to_string()),
                    altgr: Some("'".to_string()),
                    finger: Finger::RightPinky,
                },
            ]],
            modifiers: std::collections::HashMap::new(),
            space: KeyInfo {
                base: " ".to_string(),
                label: None,
                shift: None,
                altgr: None,
                finger: Finger::BothThumbs,
            },
        }
    }

    fn resolve(layout: &KeyboardLayout, ch: char) -> (usize, usize, KeyLevel) {
        let r = layout
            .resolve_char(ch)
            .unwrap_or_else(|| panic!("{ch:?} not resolved"));
        (r.row, r.col, r.level)
    }

    fn modifier_keys(layout: &KeyboardLayout, ch: char) -> Vec<&'static str> {
        layout.modifier_keys_for(&layout.resolve_char(ch).unwrap())
    }

    #[test]
    fn test_resolve_char_base_beats_altgr() {
        let layout = create_test_layout();
        assert_eq!(resolve(&layout, '\''), (0, 0, KeyLevel::Base));
    }

    #[test]
    fn test_resolve_char_shift() {
        let layout = create_test_layout();
        assert_eq!(resolve(&layout, '"'), (0, 0, KeyLevel::Shift));
    }

    #[test]
    fn test_resolve_char_altgr_when_no_base() {
        let layout = create_test_layout();
        assert_eq!(resolve(&layout, '´'), (0, 0, KeyLevel::AltGr));
    }

    #[test]
    fn test_resolve_char_uppercase_without_shift_entry() {
        let mut layout = create_test_layout();
        layout.keys[0].push(KeyInfo {
            base: "a".to_string(),
            label: None,
            shift: None,
            altgr: None,
            finger: Finger::LeftPinky,
        });
        assert_eq!(resolve(&layout, 'a'), (0, 3, KeyLevel::Base));
        assert_eq!(resolve(&layout, 'A'), (0, 3, KeyLevel::Shift));
    }

    #[test]
    fn test_resolve_char_empty_levels_never_match() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Fr).unwrap();
        // The "²" key has empty shift and altgr levels.
        assert_eq!(resolve(&layout, '²'), (0, 0, KeyLevel::Base));
        assert!(layout.resolve_char('\u{0}').is_none());
    }

    #[test]
    fn test_resolve_char_space_is_not_a_key() {
        let layout = create_test_layout();
        assert!(layout.resolve_char(' ').is_none());
    }

    #[test]
    fn test_resolve_char_us() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Us).unwrap();
        assert_eq!(resolve(&layout, 'a').2, KeyLevel::Base);
        assert_eq!(resolve(&layout, '1').2, KeyLevel::Base);
        assert_eq!(resolve(&layout, 'A').2, KeyLevel::Shift);
        assert_eq!(resolve(&layout, '!').2, KeyLevel::Shift);
        assert_eq!(resolve(&layout, 'a').0, resolve(&layout, 'A').0);
        assert_eq!(resolve(&layout, 'a').1, resolve(&layout, 'A').1);
    }

    #[test]
    fn test_resolve_char_es() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Es).unwrap();
        assert_eq!(resolve(&layout, '€').2, KeyLevel::AltGr);
        assert_eq!(resolve(&layout, '[').2, KeyLevel::AltGr);
        assert_eq!(resolve(&layout, 'ñ').2, KeyLevel::Base);
        assert_eq!(resolve(&layout, 'Ñ').2, KeyLevel::Shift);
        let (row, col, level) = resolve(&layout, '¨');
        assert_eq!(layout.keys[row][col].base, "´");
        assert_eq!(level, KeyLevel::Shift);
    }

    #[test]
    fn test_resolve_char_fr() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Fr).unwrap();
        assert_eq!(resolve(&layout, '&'), (0, 1, KeyLevel::Base));
        assert_eq!(resolve(&layout, '1'), (0, 1, KeyLevel::Shift));
    }

    #[test]
    fn test_modifier_keys_shift_opposite_hand() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Us).unwrap();
        assert_eq!(modifier_keys(&layout, 'A'), vec!["shift_right"]);
        assert_eq!(modifier_keys(&layout, 'P'), vec!["shift_left"]);
        assert_eq!(modifier_keys(&layout, '"'), vec!["shift_left"]);
        assert!(modifier_keys(&layout, 'a').is_empty());
    }

    #[test]
    fn test_modifier_keys_altgr() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Es).unwrap();
        let target = layout.resolve_char('€').unwrap();
        assert_eq!(layout.modifier_keys_for(&target), vec!["alt_right"]);
        assert_eq!(
            layout.modifier_finger_for(&target),
            Some(Finger::RightThumb)
        );
    }

    #[test]
    fn test_modifier_finger_shift() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Us).unwrap();
        let target = layout.resolve_char('A').unwrap();
        assert_eq!(
            layout.modifier_finger_for(&target),
            Some(Finger::RightPinky)
        );
    }

    #[test]
    fn test_modifier_keys_both_thumbs_uses_both_shifts() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Us).unwrap();
        let target = ResolvedKey {
            row: 0,
            col: 0,
            finger: Finger::BothThumbs,
            level: KeyLevel::Shift,
        };
        assert_eq!(
            layout.modifier_keys_for(&target),
            vec!["shift_left", "shift_right"]
        );
    }

    #[test]
    fn test_modifier_keys_missing_from_layout() {
        let layout = create_test_layout();
        assert!(modifier_keys(&layout, '"').is_empty());
        assert!(modifier_keys(&layout, '´').is_empty());
    }

    #[test]
    fn test_modifier_key_spans_row_two_rows() {
        let m: ModifierKey =
            serde_json::from_str(r#"{"label":"Enter","finger":"right_pinky","rows":[1,2]}"#)
                .unwrap();
        assert!(m.spans_row(1));
        assert!(m.spans_row(2));
        assert!(!m.spans_row(3));
    }

    #[test]
    fn test_modifier_key_spans_row_single_row() {
        let m: ModifierKey =
            serde_json::from_str(r#"{"label":"Enter","finger":"right_pinky","rows":[2]}"#).unwrap();
        assert!(!m.spans_row(1));
        assert!(m.spans_row(2));
    }

    #[test]
    fn test_modifier_key_spans_row_none() {
        let m: ModifierKey =
            serde_json::from_str(r#"{"label":"Shift","finger":"left_pinky"}"#).unwrap();
        assert!(!m.spans_row(1));
        assert!(!m.spans_row(2));
    }

    #[test]
    fn test_us_layout_enter_rows() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Us).unwrap();
        let enter = layout.modifiers.get("enter").unwrap();
        assert_eq!(enter.rows, Some(vec![2]));
        assert!(!enter.spans_row(1));
        assert!(enter.spans_row(2));
    }

    #[test]
    fn test_es_layout_enter_rows() {
        let layout = KeyboardLayout::load_from_json(crate::language::Language::Es).unwrap();
        let enter = layout.modifiers.get("enter").unwrap();
        assert_eq!(enter.rows, Some(vec![1, 2]));
        assert!(enter.spans_row(1));
        assert!(enter.spans_row(2));
    }
}
