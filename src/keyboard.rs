use std::{collections::HashSet, sync::OnceLock};

use parking_lot::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Keyboard keys
pub enum Key {
    /// A text character
    Char(char),
    /// The up arrow
    Up,
    /// The down arrow
    Down,
    /// The left arrow
    Left,
    /// The right arrow
    Right,
    /// The space key
    Space,
    /// The enter key
    Enter,
    /// The escape key
    Escape,
    /// The backspace key
    Backspace,
    /// The delete key
    Delete,
    /// The shift key/modifier
    Shift,
    /// The control key/modifier
    Control,
    /// The alt key/modifier
    Alt,
    /// The meta (also sometimes the "windows", "command" or "open apple") key/modifier
    Meta,
    /// An unknown or unrecognized key
    Unknown,
}

impl From<char> for Key {
    fn from(value: char) -> Self {
        if value == ' ' {
            Self::Space
        } else {
            Self::Char(value.to_lowercase().next().unwrap_or(value))
        }
    }
}

impl From<&str> for Key {
    fn from(value: &str) -> Self {
        if value.len() == 1 {
            Self::from(value.chars().next().unwrap())
        } else {
            match value.to_lowercase().as_str() {
                "up" => Self::Up,
                "down" => Self::Down,
                "left" => Self::Left,
                "right" => Self::Right,
                "space" => Self::Space,
                "enter" => Self::Enter,
                "escape" => Self::Escape,
                "backspace" => Self::Backspace,
                "delete" => Self::Delete,
                "shift" => Self::Shift,
                "control" => Self::Control,
                "alt" => Self::Alt,
                "meta" => Self::Meta,
                _ => Self::Unknown,
            }
        }
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        Key::from(value.as_str())
    }
}

#[derive(Debug)]
struct KeyboardState {
    pressed: HashSet<Key>,
    released: HashSet<Key>,
    held: HashSet<Key>,
}

impl KeyboardState {
    fn empty() -> Self {
        Self {
            pressed: HashSet::new(),
            released: HashSet::new(),
            held: HashSet::new(),
        }
    }
}

static KEYBOARD_STATE: OnceLock<RwLock<KeyboardState>> = OnceLock::new();

fn get_state() -> &'static RwLock<KeyboardState> {
    KEYBOARD_STATE.get_or_init(|| RwLock::new(KeyboardState::empty()))
}

/// Get whether a key is currently being held down
pub fn is_down(key: impl Into<Key>) -> bool {
    get_state().read().held.contains(&key.into())
}

/// Get whether a key was just pressed
pub fn is_pressed(key: impl Into<Key>) -> bool {
    get_state().read().pressed.contains(&key.into())
}

/// Get whether a key was just released
pub fn is_released(key: impl Into<Key>) -> bool {
    get_state().read().released.contains(&key.into())
}

/// Process a key event, used internally to handle key events
pub fn process_key_event(key: Key, pressed: bool) {
    let mut state = get_state().write();
    if pressed {
        state.held.insert(key);
        state.pressed.insert(key);
    } else {
        state.held.remove(&key);
        state.released.insert(key);
    }
}

/// Reset the keyboard's state for this frame
pub fn reset() {
    let mut state = get_state().write();
    state.pressed.clear();
    state.released.clear();
}

/// Useful structs to import
pub mod prelude {
    pub use super::Key;
}
