pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Space,
    Enter,
    Escape,
    Backspace,
    Delete,
    Shift,
    Ctrl,
    Alt,
    Meta,
}

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

static KEYBOARD_STATE: KeyboardState = KeyboardState::empty();

fn is_down(key: Key) -> bool {
    todo!()
}

/// Useful structs to import
pub mod prelude {
    pub use super::Key;
}
