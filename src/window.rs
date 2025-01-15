use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

use parking_lot::RwLock;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key as WKey, NamedKey, SmolStr},
    window::Window,
};

use crate::{math::Vec2, prelude::Key};

static HAS_INITIALIZED: AtomicBool = AtomicBool::new(false);
thread_local! {
    static EVENT_LOOP: RwLock<Option<EventLoop<()>>> = const { RwLock::new(None) };
}
pub(crate) static WINDOWS: RwLock<Vec<Window>> = RwLock::new(Vec::new());

/// Must be called before creating any windows
pub fn init() {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    assert!(
        std::thread::current().name() == Some("main"),
        "Window init must be called on the main thread!"
    );
    if HAS_INITIALIZED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        EVENT_LOOP.with(|el_cell| {
            let mut el = el_cell.write();
            if el.is_none() {
                let tmp = EventLoop::new().unwrap();
                tmp.set_control_flow(ControlFlow::Poll);
                *el = Some(tmp);
            }
        })
    }
}

/// Settings specifying how to open a window, see [open_with_settings].
#[derive(Debug)]
pub struct WindowSettings<'a> {
    // Title/name of the window
    name: &'a str,
    /// Size, in pixels
    size: Option<Vec2>,
    // Whether the window is resizable or not
    resizable: bool,
    // Position, in pixels
    pos: Option<Vec2>,
}

impl Default for WindowSettings<'_> {
    fn default() -> Self {
        Self {
            name: "CÄRE game",
            size: Some((800, 600).into()),
            resizable: false,
            pos: None,
        }
    }
}

/// Open a window with the specified name
///
/// # NOTE
/// Can only be called from the main thread, calling on any other thread will panic.
pub fn open(name: &str) {
    open_with_settings(WindowSettings {
        name,
        ..WindowSettings::default()
    })
}

/// Open a window with the specified window settings
///
/// # NOTE
/// Can only be called from the main thread, calling on any other thread will panic.
pub fn open_with_settings(settings: WindowSettings) {
    // TODO: Be able to create windows in the main loop by adding windows to a queue and creating them in the main loop below
    if !HAS_INITIALIZED.load(Ordering::Acquire) {
        panic!("Attempted to open window before initializing the window library");
    }
    EVENT_LOOP.with(|el_cell| {
        let el_handle = el_cell.write();
        let el = el_handle
            .as_ref()
            .expect("You must open windows from the main thread");
        let mut attribs = Window::default_attributes()
            .with_title(settings.name)
            .with_resizable(settings.resizable);
        if let Some(size) = settings.size {
            attribs = attribs.with_inner_size(PhysicalSize::new(size.0.x, size.0.y));
        }
        if let Some(pos) = settings.pos {
            attribs = attribs.with_position(PhysicalPosition::new(pos.0.x, pos.0.y));
        }
        WINDOWS
            .write()
            .push(el.create_window(attribs).expect("Failed to open window"));
    });
}

/// WIP Function to set the main window size
pub fn set_window_size(size: impl Into<Vec2>) {
    let size = size.into();
    let mut windows = WINDOWS.write();
    let window = windows.first_mut().unwrap();
    let _ = window.request_inner_size(PhysicalSize::new(size.x(), size.y()));
}

fn convert_key(key: winit::keyboard::Key<SmolStr>) -> Key {
    match key {
        WKey::Named(NamedKey::ArrowUp) => Key::Up,
        WKey::Named(NamedKey::ArrowDown) => Key::Down,
        WKey::Named(NamedKey::ArrowLeft) => Key::Left,
        WKey::Named(NamedKey::ArrowRight) => Key::Right,
        WKey::Named(NamedKey::Space) => Key::Space,
        WKey::Named(NamedKey::Enter) => Key::Enter,
        WKey::Named(NamedKey::Escape) => Key::Escape,
        WKey::Named(NamedKey::Backspace) => Key::Backspace,
        WKey::Named(NamedKey::Delete) => Key::Delete,
        WKey::Named(NamedKey::Shift) => Key::Shift,
        WKey::Named(NamedKey::Control) => Key::Control,
        WKey::Named(NamedKey::Alt) => Key::Alt,
        WKey::Named(NamedKey::Meta) => Key::Meta,
        WKey::Character(ch) => Key::Char(ch.chars().next().unwrap()),
        _ => Key::Unknown,
    }
}

struct AppHandler<T: FnMut()> {
    loop_fn: T,
}

impl<T: FnMut()> ApplicationHandler for AppHandler<T> {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        (self.loop_fn)();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        ev: WindowEvent,
    ) {
        match ev {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if !repeat {
                    crate::event::handle_event(crate::event::Event {
                        timestamp: Instant::now(),
                        data: crate::event::EventData::KeyEvent {
                            key: convert_key(logical_key),
                            pressed: state.is_pressed(),
                        },
                    });
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                crate::event::handle_event(crate::event::Event {
                    timestamp: Instant::now(),
                    data: crate::event::EventData::MouseMoved {
                        position: Vec2::new(position.x, position.y),
                    },
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                crate::event::handle_event(crate::event::Event {
                    timestamp: Instant::now(),
                    data: crate::event::EventData::MouseClick {
                        button: match button {
                            winit::event::MouseButton::Left => 1,
                            winit::event::MouseButton::Right => 2,
                            winit::event::MouseButton::Middle => 3,
                            winit::event::MouseButton::Back => 4,
                            winit::event::MouseButton::Forward => 5,
                            winit::event::MouseButton::Other(n) => n as i32 + 6,
                        },
                        pressed: state.is_pressed(),
                    },
                });
            }
            _ => {}
        }
    }
}

// Window implementation of the event loop running function
pub(crate) fn run(loop_fn: impl FnMut()) {
    EVENT_LOOP.with(move |el_call| {
        let el = el_call
            .write()
            .take()
            .expect("Event loop must be run from the main thread");
        el.run_app(&mut AppHandler { loop_fn }).unwrap();
    });
}
