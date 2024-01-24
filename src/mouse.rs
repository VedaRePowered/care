use std::{collections::HashSet, sync::OnceLock};

use parking_lot::RwLock;

use crate::math::Vec2;

#[derive(Debug)]
struct MouseState {
    position: Vec2,
    pressed: HashSet<i32>,
    released: HashSet<i32>,
    held: HashSet<i32>,
}

impl MouseState {
    fn empty() -> Self {
        Self {
            position: Vec2::new(0, 0),
            pressed: HashSet::new(),
            released: HashSet::new(),
            held: HashSet::new(),
        }
    }
}

static MOUSE_STATE: OnceLock<RwLock<MouseState>> = OnceLock::new();

fn get_state() -> &'static RwLock<MouseState> {
    MOUSE_STATE.get_or_init(|| RwLock::new(MouseState::empty()))
}

/// Get the current position of the mouse
pub fn get_position() -> Vec2 {
    get_state().read().position
}

/// Get whether a mouse button is currently being held down
pub fn is_down(button: i32) -> bool {
    get_state().read().held.contains(&button)
}

/// Get whether a mouse button was just pressed
pub fn is_pressed(button: i32) -> bool {
    get_state().read().pressed.contains(&button)
}

/// Get whether a mouse button was just released
pub fn is_released(button: i32) -> bool {
    get_state().read().released.contains(&button)
}

pub fn process_mouse_moved_event(position: Vec2) {
    let mut state = get_state().write();
    state.position = position;
}

pub fn process_mouse_click_event(button: i32, pressed: bool) {
    let mut state = get_state().write();
    if pressed {
        state.held.insert(button);
        state.pressed.insert(button);
    } else {
        state.held.remove(&button);
        state.released.insert(button);
    }
}

/// Reset the mouse's state for this frame
pub fn reset() {
    let mut state = get_state().write();
    state.pressed.clear();
    state.released.clear();
}
