use std::sync::LazyLock;
use std::time::Instant;

use parking_lot::Mutex;

pub use egui::*;

use crate::event::{Event as CareEvent, EventData as CareEventData};
use crate::keyboard::{self, Key as CareKey};
use crate::window::window_size;

pub(crate) struct EguiGraphics {
    pub egui_ctx: egui::Context,
    pub egui_renderer: Mutex<egui_wgpu::Renderer>,
    pub start_time: Instant,
}

impl std::fmt::Debug for EguiGraphics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiGraphics")
            .field("egui_ctx", &self.egui_ctx)
            .finish_non_exhaustive()
    }
}

#[derive(Default)]
pub(crate) struct EguiState {
    pub egui_events: Vec<Event>,
    pub egui_mods: Modifiers,
    pub full_output: Option<FullOutput>,
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
    let mut events = match event.data {
        CareEventData::KeyEvent { key, pressed } => translate_key(key)
            .iter()
            .map(|&key| Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: get_modifiers(),
            })
            .collect(),
        CareEventData::KeyRepeat { key } => translate_key(key)
            .iter()
            .map(|&key| Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: get_modifiers(),
            })
            .collect(),
        CareEventData::MouseMoved { position } => {
            vec![Event::PointerMoved(Pos2::new(position.x, position.y))]
        }
        CareEventData::MouseClick { button, pressed } => {
            let pos = crate::mouse::get_position();
            vec![Event::PointerButton {
                pos: Pos2::new(pos.x, pos.y),
                button: match button {
                    1 => PointerButton::Primary,
                    2 => PointerButton::Secondary,
                    3 => PointerButton::Middle,
                    4 => PointerButton::Extra1,
                    _ => PointerButton::Extra2,
                },
                pressed,
                modifiers: get_modifiers(),
            }]
        }
        CareEventData::TextEvent { text } => vec![Event::Text(text.replace(['\x7f', '\x08'], ""))],
        CareEventData::FocusChange { focused } => vec![Event::WindowFocused(focused)],
    };
    EGUI_STATE.lock().egui_events.append(&mut events);
}

pub(crate) fn get_full_output() -> Option<FullOutput> {
    std::mem::take(&mut EGUI_STATE.lock().full_output)
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

/// Render the gui with egui
///
/// **IMPORTANT**: Only call this function once per frame
///
/// This gives you an Egui [Context] that you can use to render widgets
pub fn gui<'a>(call: impl FnMut(&egui::Context) + 'a) {
    let window_size = window_size();
    let egui_state = &crate::graphics::GRAPHICS_STATE.egui;
    let full_output = egui_state.egui_ctx.run(
        egui::RawInput {
            viewport_id: egui::ViewportId::ROOT,
            viewports: [(egui::ViewportId::ROOT, egui::ViewportInfo::default())]
                .into_iter()
                .collect(),
            screen_rect: Some(egui::Rect::from_min_max(
                egui::Pos2::ZERO,
                egui::Pos2::new(window_size.x, window_size.y),
            )),
            max_texture_side: None,
            time: Some(egui_state.start_time.elapsed().as_secs_f64()),
            predicted_dt: 1.0 / 60.0,
            modifiers: crate::gui::get_modifiers(),
            events: crate::gui::get_events(),
            hovered_files: Vec::new(),
            dropped_files: Vec::new(),
            focused: true,
            system_theme: None,
        },
        call,
    );
    EGUI_STATE.lock().full_output = Some(full_output);
}

fn translate_key(key: CareKey) -> &'static [egui::Key] {
    match key {
        CareKey::Down => &[Key::ArrowDown],
        CareKey::Left => &[Key::ArrowLeft],
        CareKey::Right => &[Key::ArrowRight],
        CareKey::Up => &[Key::ArrowUp],

        CareKey::Escape => &[Key::Escape],
        CareKey::Char('\t') => &[Key::Tab],
        CareKey::Backspace => &[Key::Backspace],
        CareKey::Enter | CareKey::Char('\n') | CareKey::Char('\r') => &[Key::Enter],
        CareKey::Delete => &[Key::Delete],

        // Punctuation
        CareKey::Space | CareKey::Char(' ') => &[Key::Space],
        CareKey::Char(',') | CareKey::Char('<') => &[Key::Comma],
        CareKey::Char('.') | CareKey::Char('>') => &[Key::Period],
        CareKey::Char(';') => &[Key::Semicolon],
        CareKey::Char(':') => &[Key::Colon],
        CareKey::Char('\\') => &[Key::Backslash],
        CareKey::Char('|') => &[Key::Pipe],
        CareKey::Char('/') => &[Key::Slash],
        CareKey::Char('?') => &[Key::Questionmark],
        CareKey::Char('[') => &[Key::OpenBracket],
        CareKey::Char('{') => &[Key::OpenCurlyBracket],
        CareKey::Char(']') => &[Key::CloseBracket],
        CareKey::Char('}') => &[Key::CloseCurlyBracket],
        CareKey::Char('`') | CareKey::Char('~') => &[Key::Backtick],
        CareKey::Char('\'') | CareKey::Char('"') => &[Key::Quote],

        CareKey::Char('-') | CareKey::Char('_') => &[Key::Minus],
        CareKey::Char('+') => &[Key::Plus],
        CareKey::Char('=') => &[Key::Equals],
        CareKey::Char('!') => &[Key::Exclamationmark],

        CareKey::Char('0') | CareKey::Char(')') => &[Key::Num0],
        CareKey::Char('1') => &[Key::Num1],
        CareKey::Char('2') | CareKey::Char('@') => &[Key::Num2],
        CareKey::Char('3') | CareKey::Char('#') => &[Key::Num3],
        CareKey::Char('4') | CareKey::Char('$') => &[Key::Num4],
        CareKey::Char('5') | CareKey::Char('%') => &[Key::Num5],
        CareKey::Char('6') | CareKey::Char('^') => &[Key::Num6],
        CareKey::Char('7') | CareKey::Char('&') => &[Key::Num7],
        CareKey::Char('8') | CareKey::Char('*') => &[Key::Num8],
        CareKey::Char('9') | CareKey::Char('(') => &[Key::Num9],

        CareKey::Char('A') | CareKey::Char('a') => &[Key::A],
        CareKey::Char('B') | CareKey::Char('b') => &[Key::B],
        CareKey::Char('C') | CareKey::Char('c') => &[Key::C],
        CareKey::Char('D') | CareKey::Char('d') => &[Key::D],
        CareKey::Char('E') | CareKey::Char('e') => &[Key::E],
        CareKey::Char('F') | CareKey::Char('f') => &[Key::F],
        CareKey::Char('G') | CareKey::Char('g') => &[Key::G],
        CareKey::Char('H') | CareKey::Char('h') => &[Key::H],
        CareKey::Char('I') | CareKey::Char('i') => &[Key::I],
        CareKey::Char('J') | CareKey::Char('j') => &[Key::J],
        CareKey::Char('K') | CareKey::Char('k') => &[Key::K],
        CareKey::Char('L') | CareKey::Char('l') => &[Key::L],
        CareKey::Char('M') | CareKey::Char('m') => &[Key::M],
        CareKey::Char('N') | CareKey::Char('n') => &[Key::N],
        CareKey::Char('O') | CareKey::Char('o') => &[Key::O],
        CareKey::Char('P') | CareKey::Char('p') => &[Key::P],
        CareKey::Char('Q') | CareKey::Char('q') => &[Key::Q],
        CareKey::Char('R') | CareKey::Char('r') => &[Key::R],
        CareKey::Char('S') | CareKey::Char('s') => &[Key::S],
        CareKey::Char('T') | CareKey::Char('t') => &[Key::T],
        CareKey::Char('U') | CareKey::Char('u') => &[Key::U],
        CareKey::Char('V') | CareKey::Char('v') => &[Key::V],
        CareKey::Char('W') | CareKey::Char('w') => &[Key::W],
        CareKey::Char('X') | CareKey::Char('x') => &[Key::X],
        CareKey::Char('Y') | CareKey::Char('y') => &[Key::Y],
        CareKey::Char('Z') | CareKey::Char('z') => &[Key::Z],

        _ => &[Key::F34],
    }
}
