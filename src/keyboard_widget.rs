use crate::utils::decompose_with_spacing_accent;
use gtk::gdk;
use gtk::glib;
use gtk::pango;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{graphene, gsk};
use libadwaita as adw;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

/// Corner radius (in px) for on-screen keyboard keys, matching the small
/// rounding used across Adwaita widgets.
const KEY_CORNER_RADIUS: f32 = 5.0;

/// Border width (in px) for on-screen keyboard keys.
const KEY_BORDER_WIDTH: f32 = 1.0;

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

mod imp {
    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::KeyboardWidget)]
    pub struct KeyboardWidget {
        pub current_key: RefCell<Option<char>>,
        pub current_target: RefCell<Option<ResolvedKey>>,
        pub visible_keys: RefCell<Option<HashSet<char>>>,
        pub current_key_sequence: RefCell<Vec<char>>,
        pub sequence_index: RefCell<usize>,
        pub layout: RefCell<KeyboardLayout>,
        pub last_finger: RefCell<Option<Finger>>,
        pub cached_colors: RefCell<Option<HashMap<String, gdk::RGBA>>>,
        pub settings: RefCell<Option<gio::Settings>>,
        /// Keyboard-layout language, set by the window.
        #[property(get, set = Self::set_layout_language_prop, builder(crate::language::Language::default()))]
        pub layout_language: Cell<crate::language::Language>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for KeyboardWidget {
        const NAME: &'static str = "TeclasKeyboardWidget";
        type Type = super::KeyboardWidget;
        type ParentType = gtk::Widget;
    }

    #[glib::derived_properties]
    impl ObjectImpl for KeyboardWidget {
        fn constructed(&self) {
            self.parent_constructed();
            // Safe default; the window sets the real layout via `layout-code`.
            *self.layout.borrow_mut() = KeyboardLayout::default();

            glib::idle_add_local_once(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move || {
                    this.cache_colors();
                }
            ));

            // Re-cache colors when theme changes
            let style_manager = adw::StyleManager::default();
            style_manager.connect_dark_notify(glib::clone!(
                #[weak(rename_to = this)]
                self,
                move |_| {
                    glib::idle_add_local_once(glib::clone!(move || {
                        this.cache_colors();
                        this.obj().queue_draw();
                    }));
                }
            ));

            // Redraw when the finger-colors preference changes
            let settings = gio::Settings::new(crate::config::APPLICATION_ID);
            settings.connect_changed(
                Some("use-finger-colors"),
                glib::clone!(
                    #[weak(rename_to = this)]
                    self,
                    move |_, _| {
                        this.obj().queue_draw();
                    }
                ),
            );
            // Keep the settings object alive so the signal stays connected.
            self.settings.replace(Some(settings));
        }
    }

    impl WidgetImpl for KeyboardWidget {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;

            if width <= 0.0 || height <= 0.0 {
                return;
            }

            // Ensure colors are cached before drawing. The initial caching is
            // scheduled on an idle callback in `constructed`, but the widget can
            // be snapshotted before that callback runs (e.g. when the app opens
            // directly on a page containing this widget), so cache on demand.
            if self.cached_colors.borrow().is_none() {
                self.cache_colors();
            }

            Self::draw_keyboard(
                snapshot,
                &widget,
                &self.current_key,
                &self.current_target,
                &self.layout,
                &self.visible_keys,
                &self.cached_colors,
            );
        }

        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let (width, height) = Self::calculate_size(&self.layout.borrow());
            match orientation {
                gtk::Orientation::Horizontal => (width, width, -1, -1),
                gtk::Orientation::Vertical => (height, height, -1, -1),
                _ => (0, 0, -1, -1),
            }
        }
    }

    impl KeyboardWidget {
        /// `layout-language` setter: load the matching layout JSON directly.
        fn set_layout_language_prop(&self, language: crate::language::Language) {
            let layout = KeyboardLayout::load_from_json(language).unwrap_or_default();
            let current = *self.current_key.borrow();
            *self.current_target.borrow_mut() = current.and_then(|c| layout.resolve_char(c));
            *self.layout.borrow_mut() = layout;
            self.layout_language.set(language);
            let obj = self.obj();
            obj.queue_resize();
            obj.queue_draw();
        }

        fn cache_colors(&self) {
            let widget = self.obj();
            #[allow(deprecated)]
            let style_context = widget.style_context();

            #[allow(deprecated)]
            let lookup = |color_name: &str| -> gdk::RGBA {
                style_context
                    .lookup_color(color_name)
                    .unwrap_or_else(|| gdk::RGBA::new(0.0, 0.0, 0.0, 1.0))
            };

            let mut colors = HashMap::new();
            colors.insert(
                "keyboard-modifier".to_string(),
                lookup("keyboard-modifier-color"),
            );
            colors.insert(
                "keyboard-modifier-text".to_string(),
                lookup("keyboard-modifier-text-color"),
            );
            colors.insert(
                "keyboard-key-text".to_string(),
                lookup("keyboard-key-text-color"),
            );
            colors.insert("keyboard-key".to_string(), lookup("keyboard-key-color"));
            colors.insert(
                "keyboard-key-current-text".to_string(),
                lookup("keyboard-key-current-text-color"),
            );
            colors.insert(
                "keyboard-key-current".to_string(),
                lookup("keyboard-key-current-color"),
            );
            colors.insert(
                "keyboard-modifier-current".to_string(),
                lookup("keyboard-modifier-current-color"),
            );
            colors.insert(
                "keyboard-modifier-current-text".to_string(),
                lookup("keyboard-modifier-current-text-color"),
            );
            colors.insert(
                "keyboard-border".to_string(),
                lookup("keyboard-border-color"),
            );

            // Cache finger colors
            for finger in [
                Finger::LeftPinky,
                Finger::LeftRing,
                Finger::LeftMiddle,
                Finger::LeftIndex,
                Finger::LeftThumb,
                Finger::RightIndex,
                Finger::RightMiddle,
                Finger::RightRing,
                Finger::RightPinky,
                Finger::RightThumb,
                Finger::BothThumbs,
            ] {
                let class_name = Self::get_finger_css_class(&finger);
                let color_name = format!("{}-color", class_name);
                colors.insert(class_name, lookup(&color_name));
            }

            *self.cached_colors.borrow_mut() = Some(colors);
        }
    }

    impl KeyboardWidget {
        fn calculate_size(layout: &KeyboardLayout) -> (i32, i32) {
            let key_width = 50.0;
            let key_height = 50.0;
            let key_spacing = 5.0;
            let row_spacing = 5.0;

            let row0_keys = layout.keys.first().map(|r| r.len()).unwrap_or(12);
            let row0_width = row0_keys as f64 * key_width
                + (row0_keys - 1) as f64 * key_spacing
                + key_spacing
                + key_width * 2.0;

            let enter_on_row1 = layout
                .modifiers
                .get("enter")
                .is_some_and(|e| e.spans_row(1));

            let row1_keys = layout.keys.get(1).map(|r| r.len()).unwrap_or(12);
            let row1_width = key_width * 1.5
                + key_spacing
                + row1_keys as f64 * key_width
                + (row1_keys - 1) as f64 * key_spacing
                + if enter_on_row1 {
                    key_spacing + key_width * 1.75
                } else {
                    0.0
                };

            let row2_keys = layout.keys.get(2).map(|r| r.len()).unwrap_or(12);
            let row2_width = key_width * 1.75
                + key_spacing
                + row2_keys as f64 * key_width
                + row2_keys as f64 * key_spacing
                + key_width * 1.75
                - key_spacing * 2.0;

            let row3_keys = layout.keys.get(3).map(|r| r.len()).unwrap_or(11);
            let row3_width = key_width * 1.25
                + key_spacing
                + row3_keys as f64 * key_width
                + (row3_keys - 1) as f64 * key_spacing
                + key_spacing
                + key_width * 3.0;

            let row4_width = key_width * 1.5
                + key_spacing
                + key_width * 1.2
                + key_spacing
                + key_width * 1.3
                + key_spacing
                + key_width * 6.0
                + key_spacing
                + key_width * 1.3
                + key_spacing
                + key_width * 1.2
                + key_spacing
                + key_width * 1.2
                + key_spacing
                + key_width * 1.5;

            let keyboard_width = row0_width
                .max(row1_width)
                .max(row2_width)
                .max(row3_width)
                .max(row4_width) as i32;
            let keyboard_height = (5.0 * key_height + 4.0 * row_spacing) as i32;

            (keyboard_width, keyboard_height)
        }

        fn get_finger_css_class(finger: &Finger) -> String {
            format!("finger-{}", finger.to_string().replace('_', "-"))
        }

        /// Build a font description based on the widget's system font
        /// (Adwaita Sans), overriding only the point size. This keeps the
        /// keyboard's typography consistent with the rest of the app.
        fn key_font(pango_context: &pango::Context, size_pt: f64) -> pango::FontDescription {
            let mut font_desc = pango_context.font_description().unwrap_or_default();
            font_desc.set_size((size_pt * f64::from(pango::SCALE)).round() as i32);
            font_desc
        }

        #[allow(clippy::too_many_arguments)]
        fn draw_single_key(
            snapshot: &gtk::Snapshot,
            pango_context: &pango::Context,
            x: f32,
            y: f32,
            width: f32,
            height: f32,
            key_info: Option<&KeyInfo>,
            label: Option<&str>,
            is_current: bool,
            should_show_text: bool,
            key_current_color: &gdk::RGBA,
            key_text_color: &gdk::RGBA,
            key_current_text_color: &gdk::RGBA,
            key_border_color: &gdk::RGBA,
            finger_fill_color: &gdk::RGBA,
        ) {
            let bounds = graphene::Rect::new(x, y, width, height);
            let corner = graphene::Size::new(KEY_CORNER_RADIUS, KEY_CORNER_RADIUS);
            let rounded = gsk::RoundedRect::new(bounds, corner, corner, corner, corner);

            snapshot.push_rounded_clip(&rounded);
            snapshot.append_color(
                if is_current {
                    key_current_color
                } else {
                    finger_fill_color
                },
                &bounds,
            );
            snapshot.pop();

            let border_color = key_border_color;
            snapshot.append_border(
                &rounded,
                &[
                    KEY_BORDER_WIDTH,
                    KEY_BORDER_WIDTH,
                    KEY_BORDER_WIDTH,
                    KEY_BORDER_WIDTH,
                ],
                &[*border_color, *border_color, *border_color, *border_color],
            );

            if should_show_text {
                let text_color = if is_current {
                    key_current_text_color
                } else {
                    key_text_color
                };

                if let Some(label_text) = label {
                    let layout = pango::Layout::new(pango_context);
                    let font_desc = Self::key_font(pango_context, 9.0);
                    layout.set_font_description(Some(&font_desc));
                    layout.set_text(label_text);
                    let (_, logical_rect) = layout.pixel_extents();
                    let text_width = logical_rect.width();
                    let text_height = logical_rect.height();
                    snapshot.save();
                    snapshot.translate(&graphene::Point::new(
                        x + (width - text_width as f32) / 2.0,
                        y + (height - text_height as f32) / 2.0 - logical_rect.y() as f32,
                    ));
                    snapshot.append_layout(&layout, text_color);
                    snapshot.restore();
                } else if let Some(key) = key_info {
                    let base_text = if key.base.chars().next().unwrap().is_alphabetic() {
                        key.base.to_uppercase()
                    } else {
                        key.base.clone()
                    };
                    let is_alphabetic = key.base.chars().next().unwrap().is_alphabetic();

                    let layout = pango::Layout::new(pango_context);

                    if is_alphabetic {
                        let font_desc = Self::key_font(pango_context, 14.0);
                        layout.set_font_description(Some(&font_desc));
                        layout.set_text(&base_text);
                        let (text_width, text_height) = layout.pixel_size();
                        snapshot.save();
                        snapshot.translate(&graphene::Point::new(
                            x + (width - text_width as f32) / 2.0,
                            y + (height - text_height as f32) / 2.0,
                        ));
                        snapshot.append_layout(&layout, text_color);
                        snapshot.restore();
                    } else {
                        let font_desc = Self::key_font(pango_context, 14.0);
                        layout.set_font_description(Some(&font_desc));

                        layout.set_text(&base_text);
                        let (_, text_height) = layout.pixel_size();
                        snapshot.save();
                        snapshot.translate(&graphene::Point::new(
                            x + 5.0,
                            y + height - text_height as f32,
                        ));
                        snapshot.append_layout(&layout, text_color);
                        snapshot.restore();

                        if let Some(shift_text) = &key.shift {
                            layout.set_text(shift_text);
                            snapshot.save();
                            snapshot.translate(&graphene::Point::new(x + 5.0, y));
                            snapshot.append_layout(&layout, text_color);
                            snapshot.restore();
                        }

                        if let Some(altgr_text) = &key.altgr
                            && !altgr_text.is_empty()
                        {
                            layout.set_text(altgr_text);
                            let (text_width, text_height) = layout.pixel_size();
                            snapshot.save();
                            snapshot.translate(&graphene::Point::new(
                                x + width - text_width as f32 - 5.0,
                                y + height - text_height as f32,
                            ));
                            snapshot.append_layout(&layout, text_color);
                            snapshot.restore();
                        }
                    }
                }
            }
        }

        fn draw_keyboard(
            snapshot: &gtk::Snapshot,
            widget: &super::KeyboardWidget,
            current_key: &RefCell<Option<char>>,
            current_target: &RefCell<Option<ResolvedKey>>,
            layout: &RefCell<KeyboardLayout>,
            visible_keys: &RefCell<Option<HashSet<char>>>,
            cached_colors: &RefCell<Option<HashMap<String, gdk::RGBA>>>,
        ) {
            let layout_borrowed = layout.borrow();
            let visible_keys_borrowed = visible_keys.borrow();

            let key_width = 50.0;
            let key_height = 50.0;
            let key_spacing = 5.0;
            let row_spacing = 5.0;

            let pango_context = widget.pango_context();

            let colors = cached_colors.borrow();
            let colors = colors.as_ref().expect("Colors should be cached");

            let modifier_color = colors["keyboard-modifier"];
            let modifier_text_color = colors["keyboard-modifier-text"];
            let key_text_color = colors["keyboard-key-text"];
            let key_color = colors["keyboard-key"];
            let key_current_text_color = colors["keyboard-key-current-text"];
            let key_current_color = colors["keyboard-key-current"];
            let key_border_color = colors["keyboard-border"];
            let modifier_current_color = colors["keyboard-modifier-current"];
            let modifier_current_text_color = colors["keyboard-modifier-current-text"];

            let settings = gio::Settings::new(crate::config::APPLICATION_ID);
            let use_finger_colors = settings.boolean("use-finger-colors");

            let get_finger_color = |finger: &Finger| -> gdk::RGBA {
                if use_finger_colors {
                    let class_name = Self::get_finger_css_class(finger);
                    colors.get(&class_name).copied().unwrap_or(key_color)
                } else {
                    key_color
                }
            };

            let current = current_key.borrow();

            let target = *current_target.borrow();

            let is_key_current = |row: usize, col: usize| -> bool {
                target.is_some_and(|t| t.row == row && t.col == col)
            };

            let required_modifiers = target
                .map(|t| layout_borrowed.modifier_keys_for(&t))
                .unwrap_or_default();
            let is_modifier_current = |name: &str| required_modifiers.contains(&name);

            let should_show_key = |key_info: &KeyInfo| -> bool {
                visible_keys_borrowed.as_ref().is_none_or(|visible| {
                    // Check if any character this key can produce is in the visible set
                    let base_char = key_info.base.chars().next().unwrap_or(' ');
                    if visible.contains(&base_char) {
                        return true;
                    }
                    if let Some(shift) = &key_info.shift
                        && let Some(shift_char) = shift.chars().next()
                        && visible.contains(&shift_char)
                    {
                        return true;
                    }
                    if let Some(altgr) = &key_info.altgr
                        && let Some(altgr_char) = altgr.chars().next()
                        && visible.contains(&altgr_char)
                    {
                        return true;
                    }

                    // For characters not directly on keyboard, check if this key is needed
                    // to compose them (e.g., 'a' + '´' = 'á')
                    for &visible_char in visible.iter() {
                        if !layout_borrowed.contains_character(visible_char)
                            && let Some((spacing_accent, base)) =
                                decompose_with_spacing_accent(visible_char)
                        {
                            if base_char == spacing_accent || base_char == base {
                                return true;
                            }
                            if let Some(shift) = &key_info.shift
                                && let Some(shift_char) = shift.chars().next()
                                && (shift_char == spacing_accent || shift_char == base)
                            {
                                return true;
                            }
                            if let Some(altgr) = &key_info.altgr
                                && let Some(altgr_char) = altgr.chars().next()
                                && (altgr_char == spacing_accent || altgr_char == base)
                            {
                                return true;
                            }
                        }
                    }

                    false
                })
            };

            let should_show_char = |ch: char| -> bool {
                visible_keys_borrowed
                    .as_ref()
                    .is_none_or(|visible| visible.contains(&ch))
            };

            // Row 0: Number row + Backspace
            let mut x = 0.0;
            let y = 0.0;
            if let Some(row) = layout_borrowed.keys.first() {
                for (col, key_info) in row.iter().enumerate() {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y,
                        key_width,
                        key_height,
                        Some(key_info),
                        None,
                        is_key_current(0, col),
                        should_show_key(key_info),
                        &key_current_color,
                        &key_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &get_finger_color(&key_info.finger),
                    );
                    x += key_width + key_spacing;
                }
                if let Some(backspace) = layout_borrowed.modifiers.get("backspace") {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y,
                        key_width * 2.0,
                        key_height,
                        None,
                        Some(&backspace.label),
                        false,
                        true,
                        &key_current_color,
                        &modifier_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &modifier_color,
                    );
                }
            }

            // Row 1: Tab + QWERTY row + Enter (spans to row 2)
            x = 0.0;
            let y1 = y + key_height + row_spacing;
            if let Some(tab) = layout_borrowed.modifiers.get("tab") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y1,
                    key_width * 1.5,
                    key_height,
                    None,
                    Some(&tab.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
                x += key_width * 1.5 + key_spacing;
            }
            if let Some(row) = layout_borrowed.keys.get(1) {
                for (col, key_info) in row.iter().enumerate() {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y1,
                        key_width,
                        key_height,
                        Some(key_info),
                        None,
                        is_key_current(1, col),
                        should_show_key(key_info),
                        &key_current_color,
                        &key_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &get_finger_color(&key_info.finger),
                    );
                    x += key_width + key_spacing;
                }
            }
            if let Some(enter) = layout_borrowed.modifiers.get("enter")
                && enter.spans_row(1)
            {
                let enter_height = key_height * 2.0 + row_spacing;
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y1,
                    key_width * 1.75,
                    enter_height,
                    None,
                    Some(&enter.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
            }

            // Row 2: Caps Lock + Home row + Enter (if single-row)
            x = 0.0;
            let y2 = y1 + key_height + row_spacing;
            if let Some(caps) = layout_borrowed.modifiers.get("caps_lock") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y2,
                    key_width * 1.75,
                    key_height,
                    None,
                    Some(&caps.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
                x += key_width * 1.75 + key_spacing;
            }
            if let Some(row) = layout_borrowed.keys.get(2) {
                for (col, key_info) in row.iter().enumerate() {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y2,
                        key_width,
                        key_height,
                        Some(key_info),
                        None,
                        is_key_current(2, col),
                        should_show_key(key_info),
                        &key_current_color,
                        &key_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &get_finger_color(&key_info.finger),
                    );
                    x += key_width + key_spacing;
                }
            }
            if let Some(enter) = layout_borrowed.modifiers.get("enter")
                && enter.spans_row(2)
                && !enter.spans_row(1)
            {
                let row0_keys = layout_borrowed.keys.first().map(|r| r.len()).unwrap_or(13);
                let keyboard_right_edge = row0_keys as f32 * key_width
                    + (row0_keys - 1) as f32 * key_spacing
                    + key_spacing
                    + key_width * 2.0;
                let enter_width = keyboard_right_edge - x;
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y2,
                    enter_width,
                    key_height,
                    None,
                    Some(&enter.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
            }

            // Row 3: Left Shift + Bottom row + Right Shift
            x = 0.0;
            let y3 = y2 + key_height + row_spacing;
            if let Some(shift_l) = layout_borrowed.modifiers.get("shift_left") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y3,
                    key_width * 1.25,
                    key_height,
                    None,
                    Some(&shift_l.label),
                    is_modifier_current("shift_left"),
                    true,
                    &modifier_current_color,
                    &modifier_text_color,
                    &modifier_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
                x += key_width * 1.25 + key_spacing;
            }
            if let Some(row) = layout_borrowed.keys.get(3) {
                for (col, key_info) in row.iter().enumerate() {
                    Self::draw_single_key(
                        snapshot,
                        &pango_context,
                        x,
                        y3,
                        key_width,
                        key_height,
                        Some(key_info),
                        None,
                        is_key_current(3, col),
                        should_show_key(key_info),
                        &key_current_color,
                        &key_text_color,
                        &key_current_text_color,
                        &key_border_color,
                        &get_finger_color(&key_info.finger),
                    );
                    x += key_width + key_spacing;
                }
            }
            if let Some(shift_r) = layout_borrowed.modifiers.get("shift_right") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y3,
                    key_width * 3.0,
                    key_height,
                    None,
                    Some(&shift_r.label),
                    is_modifier_current("shift_right"),
                    true,
                    &modifier_current_color,
                    &modifier_text_color,
                    &modifier_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
            }

            // Row 4: Ctrl + Super + Alt + Space + Alt + Super + Ctrl
            x = 0.0;
            let y4 = y3 + key_height + row_spacing;
            if let Some(ctrl_l) = layout_borrowed.modifiers.get("ctrl_left") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.5,
                    key_height,
                    None,
                    Some(&ctrl_l.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
                x += key_width * 1.5 + key_spacing;
            }
            if let Some(super_l) = layout_borrowed.modifiers.get("super_left") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.2,
                    key_height,
                    None,
                    Some(&super_l.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
                x += key_width * 1.2 + key_spacing;
            }
            if let Some(alt_l) = layout_borrowed.modifiers.get("alt_left") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.3,
                    key_height,
                    None,
                    Some(&alt_l.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
                x += key_width * 1.3 + key_spacing;
            }
            let is_space_current = current.is_some_and(|c| c == ' ');
            let space_label = layout_borrowed.space.label.as_deref().unwrap_or("SPACE");
            Self::draw_single_key(
                snapshot,
                &pango_context,
                x,
                y4,
                key_width * 6.0,
                key_height,
                None,
                Some(space_label),
                is_space_current,
                should_show_char(' '),
                &key_current_color,
                &key_text_color,
                &key_current_text_color,
                &key_border_color,
                &get_finger_color(&layout_borrowed.space.finger),
            );
            x += key_width * 6.0 + key_spacing;
            if let Some(alt_r) = layout_borrowed.modifiers.get("alt_right") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.3,
                    key_height,
                    None,
                    Some(&alt_r.label),
                    is_modifier_current("alt_right"),
                    true,
                    &modifier_current_color,
                    &modifier_text_color,
                    &modifier_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
                x += key_width * 1.3 + key_spacing;
            }
            if let Some(super_r) = layout_borrowed.modifiers.get("super_right") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.2,
                    key_height,
                    None,
                    Some(&super_r.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
                x += key_width * 1.2 + key_spacing;
            }
            if let Some(menu) = layout_borrowed.modifiers.get("menu") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.2,
                    key_height,
                    None,
                    Some(&menu.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
                x += key_width * 1.2 + key_spacing;
            }
            if let Some(ctrl_r) = layout_borrowed.modifiers.get("ctrl_right") {
                Self::draw_single_key(
                    snapshot,
                    &pango_context,
                    x,
                    y4,
                    key_width * 1.5,
                    key_height,
                    None,
                    Some(&ctrl_r.label),
                    false,
                    true,
                    &key_current_color,
                    &modifier_text_color,
                    &key_current_text_color,
                    &key_border_color,
                    &modifier_color,
                );
            }
        }
    }
}

glib::wrapper! {
    pub struct KeyboardWidget(ObjectSubclass<imp::KeyboardWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl KeyboardWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_current_key(&self, key: Option<char>) {
        let imp = self.imp();
        let (sequence, current) = match key {
            Some(ch) if !imp.layout.borrow().contains_character(ch) => {
                match decompose_with_spacing_accent(ch) {
                    Some((spacing_accent, base)) => {
                        (vec![spacing_accent, base], Some(spacing_accent))
                    }
                    None => (Vec::new(), key),
                }
            }
            _ => (Vec::new(), key),
        };
        *imp.current_key_sequence.borrow_mut() = sequence;
        *imp.sequence_index.borrow_mut() = 0;
        self.set_target_char(current);

        if let Some(ch) = key
            && ch != ' '
            && let Some(finger) = imp.layout.borrow().get_finger_for_char(ch)
            && finger != Finger::BothThumbs
        {
            *imp.last_finger.borrow_mut() = Some(finger);
        }
        self.queue_draw();
    }

    pub fn advance_sequence(&self) {
        let imp = self.imp();
        let next = {
            let sequence = imp.current_key_sequence.borrow();
            let mut index = imp.sequence_index.borrow_mut();
            if sequence.is_empty() || *index >= sequence.len() - 1 {
                return;
            }
            *index += 1;
            sequence[*index]
        };
        self.set_target_char(Some(next));

        if let Some(finger) = imp.layout.borrow().get_finger_for_char(next)
            && finger != Finger::BothThumbs
        {
            *imp.last_finger.borrow_mut() = Some(finger);
        }
        self.queue_draw();
    }

    fn set_target_char(&self, ch: Option<char>) {
        let imp = self.imp();
        *imp.current_key.borrow_mut() = ch;
        *imp.current_target.borrow_mut() = ch.and_then(|c| imp.layout.borrow().resolve_char(c));
    }

    pub fn current_finger(&self) -> Option<Finger> {
        let current = *self.imp().current_key.borrow();
        current.and_then(|ch| self.get_finger_for_char(ch))
    }

    pub fn current_modifier_finger(&self) -> Option<Finger> {
        let imp = self.imp();
        let target = *imp.current_target.borrow();
        target.and_then(|t| imp.layout.borrow().modifier_finger_for(&t))
    }

    pub fn set_visible_keys(&self, keys: Option<HashSet<char>>) {
        *self.imp().visible_keys.borrow_mut() = keys;
        self.queue_draw();
    }

    pub fn get_finger_for_char(&self, ch: char) -> Option<Finger> {
        let imp = self.imp();

        // Special handling for space - use opposite thumb based on last finger
        if ch == ' ' {
            let last_finger = imp.last_finger.borrow();
            return Some(
                last_finger
                    .as_ref()
                    .map_or(Finger::RightThumb, |f| f.opposite_thumb()),
            );
        }

        imp.layout.borrow().get_finger_for_char(ch)
    }
}

impl Default for KeyboardWidget {
    fn default() -> Self {
        Self::new()
    }
}
