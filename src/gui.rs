use std::sync::LazyLock;

use parking_lot::Mutex;

pub use egui::*;

use crate::event::{Event as CareEvent, EventData as CareEventData};
use crate::keyboard::{self, Key as CareKey};

pub(crate) struct EguiGraphics {
    pub egui_ctx: egui::Context,
    pub egui_renderer: Mutex<egui_wgpu::Renderer>,
}

impl std::fmt::Debug for EguiGraphics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiGraphics")
            .field("egui_ctx", &self.egui_ctx)
            .finish_non_exhaustive()
    }
}

type EguiCall = Box<dyn FnOnce(&egui::Context) + Send + Sync>;

#[derive(Default)]
pub(crate) struct EguiState {
    pub egui_calls: Vec<EguiCall>,
    pub egui_events: Vec<Event>,
    pub egui_mods: Modifiers,
}

impl std::fmt::Debug for EguiState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiState")
            .field("egui_events", &self.egui_events)
            .field("egui_mods", &self.egui_mods)
            .finish_non_exhaustive()
    }
}

pub(crate) static EGUI_STATE: LazyLock<Mutex<EguiState>> =
    LazyLock::new(|| Mutex::new(EguiState::default()));

pub(crate) fn process_event(event: CareEvent) {
    let event = match event.data {
        CareEventData::KeyEvent { key, pressed } => Event::Key {
            key: translate_key(key),
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: get_modifiers(),
        },
        CareEventData::MouseMoved { position } => {
            Event::PointerMoved(Pos2::new(position.x, position.y))
        }
        CareEventData::MouseClick { button, pressed } => {
            let pos = crate::mouse::get_position();
            Event::PointerButton {
                pos: Pos2::new(pos.x, pos.y),
                button: match button {
                    0 => PointerButton::Primary,
                    1 => PointerButton::Secondary,
                    2 => PointerButton::Middle,
                    3 => PointerButton::Extra1,
                    _ => PointerButton::Extra2,
                },
                pressed,
                modifiers: get_modifiers(),
            }
        }
    };
    EGUI_STATE.lock().egui_events.push(event);
}

pub(crate) fn get_calls() -> Vec<EguiCall> {
    std::mem::take(&mut EGUI_STATE.lock().egui_calls)
}

pub(crate) fn get_modifiers() -> Modifiers {
    Modifiers {
        alt: keyboard::is_down(keyboard::Key::Alt),
        ctrl: keyboard::is_down(keyboard::Key::Control),
        shift: keyboard::is_down(keyboard::Key::Shift),
        mac_cmd: keyboard::is_down(keyboard::Key::Meta),
        command: keyboard::is_down(keyboard::Key::Meta),
    }
}

pub(crate) fn get_events() -> Vec<Event> {
    std::mem::take(&mut EGUI_STATE.lock().egui_events)
}

/// Render some gui elements this frame
///
/// Actually, this gives you an Egui [Context] that you can use to render widgets
pub fn gui(call: Box<dyn FnOnce(&egui::Context) + Send + Sync>) {
    EGUI_STATE.lock().egui_calls.push(call);
}

/// A panel that covers the entire left side of the window.
///
/// The order in which you add panels matter! The first panel you add will always be the outermost, and the last you add will always be the innermost.
///
/// ⚠ Always add any CentralPanel last.
///
/// See the [egui docs](https://docs.rs/egui/latest/egui/containers/panel) for more details.
///
/// See also: [SidePanel]
pub fn left_panel(
    id: impl Into<Id>,
    call: impl FnOnce(&egui::Context, &egui::Ui) + Send + Sync + 'static,
) {
    let id = id.into();
    gui(Box::new(move |ctx| {
        egui::SidePanel::left(id).show(ctx, |ui| {
            call(ctx, ui);
        });
    }))
}

/// A panel that covers the entire right side of the window.
///
/// The order in which you add panels matter! The first panel you add will always be the outermost, and the last you add will always be the innermost.
///
/// ⚠ Always add any CentralPanel last.
///
/// See the [egui docs](https://docs.rs/egui/latest/egui/containers/panel) for more details.
///
/// See also: [SidePanel]
pub fn right_panel(
    id: impl Into<Id>,
    call: impl FnOnce(&egui::Context, &mut egui::Ui) + Send + Sync + 'static,
) {
    let id = id.into();
    gui(Box::new(move |ctx| {
        egui::SidePanel::right(id).show(ctx, |ui| {
            call(ctx, ui);
        });
    }))
}

/// A panel that covers the entire top of the window.
///
/// The order in which you add panels matter! The first panel you add will always be the outermost, and the last you add will always be the innermost.
///
/// ⚠ Always add any CentralPanel last.
///
/// See the [egui docs](https://docs.rs/egui/latest/egui/containers/panel) for more details.
///
/// See also: [TopBottomPanel]
pub fn top_panel(
    id: impl Into<Id>,
    call: impl FnOnce(&egui::Context, &mut egui::Ui) + Send + Sync + 'static,
) {
    let id = id.into();
    gui(Box::new(move |ctx| {
        egui::TopBottomPanel::top(id).show(ctx, |ui| {
            call(ctx, ui);
        });
    }))
}

/// A panel that covers the entire bottom of the window.
///
/// The order in which you add panels matter! The first panel you add will always be the outermost, and the last you add will always be the innermost.
///
/// ⚠ Always add any CentralPanel last.
///
/// See the [egui docs](https://docs.rs/egui/latest/egui/containers/panel) for more details.
///
/// See also: [TopBottomPanel]
pub fn bottom_panel(
    id: impl Into<Id>,
    call: impl FnOnce(&egui::Context, &mut egui::Ui) + Send + Sync + 'static,
) {
    let id = id.into();
    gui(Box::new(move |ctx| {
        egui::TopBottomPanel::bottom(id).show(ctx, |ui| {
            call(ctx, ui);
        });
    }))
}

/// A panel that covers the remainder of the screen, i.e. whatever area is left after adding other panels.
///
/// ⚠ CentralPanel must be added after all other panels!
///
/// ⚠ Any crate::Windows and crate::Areas will cover the top-level CentralPanel.
///
/// See the [egui docs](https://docs.rs/egui/latest/egui/containers/panel) for more details.
///
/// See also: [CentralPanel]
pub fn central_panel(call: impl FnOnce(&egui::Context, &mut egui::Ui) + Send + Sync + 'static) {
    gui(Box::new(move |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            call(ctx, ui);
        });
    }))
}

/// A panel that covers the entire bottom of the window.
///
/// The order in which you add panels matter! The first panel you add will always be the outermost, and the last you add will always be the innermost.
///
/// ⚠ Always add any CentralPanel last.
///
/// See the [egui docs](https://docs.rs/egui/latest/egui/containers/panel) for more details.
///
/// See also: [Window]
pub fn window(
    title: impl Into<WidgetText>,
    call: impl FnOnce(&egui::Context, &mut egui::Ui) + Send + Sync + 'static,
) {
    let title = title.into();
    gui(Box::new(move |ctx| {
        egui::Window::new(title).show(ctx, |ui| {
            call(ctx, ui);
        });
    }))
}

fn translate_key(key: CareKey) -> egui::Key {
    match key {
        CareKey::Down => Key::ArrowDown,
        CareKey::Left => Key::ArrowLeft,
        CareKey::Right => Key::ArrowRight,
        CareKey::Up => Key::ArrowUp,

        CareKey::Escape => Key::Escape,
        CareKey::Char('\t') => Key::Tab,
        CareKey::Backspace => Key::Backspace,
        CareKey::Enter | CareKey::Char('\n') | CareKey::Char('\r') => Key::Enter,
        CareKey::Delete => Key::Delete,

        // Punctuation
        CareKey::Space | CareKey::Char(' ') => Key::Space,
        CareKey::Char(',') | CareKey::Char('<') => Key::Comma,
        CareKey::Char('.') | CareKey::Char('>') => Key::Period,
        CareKey::Char(';') => Key::Semicolon,
        CareKey::Char(':') => Key::Colon,
        CareKey::Char('\\') => Key::Backslash,
        CareKey::Char('|') => Key::Pipe,
        CareKey::Char('/') => Key::Slash,
        CareKey::Char('?') => Key::Questionmark,
        CareKey::Char('[') => Key::OpenBracket,
        CareKey::Char('{') => Key::OpenCurlyBracket,
        CareKey::Char(']') => Key::CloseBracket,
        CareKey::Char('}') => Key::CloseCurlyBracket,
        CareKey::Char('`') | CareKey::Char('~') => Key::Backtick,
        CareKey::Char('\'') | CareKey::Char('"') => Key::Quote,

        CareKey::Char('-') | CareKey::Char('_') => Key::Minus,
        CareKey::Char('+') => Key::Plus,
        CareKey::Char('=') => Key::Equals,
        CareKey::Char('!') => Key::Exclamationmark,

        CareKey::Char('0') | CareKey::Char(')') => Key::Num0,
        CareKey::Char('1') => Key::Num1,
        CareKey::Char('2') | CareKey::Char('@') => Key::Num2,
        CareKey::Char('3') | CareKey::Char('#') => Key::Num3,
        CareKey::Char('4') | CareKey::Char('$') => Key::Num4,
        CareKey::Char('5') | CareKey::Char('%') => Key::Num5,
        CareKey::Char('6') | CareKey::Char('^') => Key::Num6,
        CareKey::Char('7') | CareKey::Char('&') => Key::Num7,
        CareKey::Char('8') | CareKey::Char('*') => Key::Num8,
        CareKey::Char('9') | CareKey::Char('(') => Key::Num9,

        CareKey::Char('A') | CareKey::Char('a') => Key::A,
        CareKey::Char('B') | CareKey::Char('b') => Key::B,
        CareKey::Char('C') | CareKey::Char('c') => Key::C,
        CareKey::Char('D') | CareKey::Char('d') => Key::D,
        CareKey::Char('E') | CareKey::Char('e') => Key::E,
        CareKey::Char('F') | CareKey::Char('f') => Key::F,
        CareKey::Char('G') | CareKey::Char('g') => Key::G,
        CareKey::Char('H') | CareKey::Char('h') => Key::H,
        CareKey::Char('I') | CareKey::Char('i') => Key::I,
        CareKey::Char('J') | CareKey::Char('j') => Key::J,
        CareKey::Char('K') | CareKey::Char('k') => Key::K,
        CareKey::Char('L') | CareKey::Char('l') => Key::L,
        CareKey::Char('M') | CareKey::Char('m') => Key::M,
        CareKey::Char('N') | CareKey::Char('n') => Key::N,
        CareKey::Char('O') | CareKey::Char('o') => Key::O,
        CareKey::Char('P') | CareKey::Char('p') => Key::P,
        CareKey::Char('Q') | CareKey::Char('q') => Key::Q,
        CareKey::Char('R') | CareKey::Char('r') => Key::R,
        CareKey::Char('S') | CareKey::Char('s') => Key::S,
        CareKey::Char('T') | CareKey::Char('t') => Key::T,
        CareKey::Char('U') | CareKey::Char('u') => Key::U,
        CareKey::Char('V') | CareKey::Char('v') => Key::V,
        CareKey::Char('W') | CareKey::Char('w') => Key::W,
        CareKey::Char('X') | CareKey::Char('x') => Key::X,
        CareKey::Char('Y') | CareKey::Char('y') => Key::Y,
        CareKey::Char('Z') | CareKey::Char('z') => Key::Z,

        _ => Key::F34,
    }
}
