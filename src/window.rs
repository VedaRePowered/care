use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::RwLock;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::math::Vec2;

static HAS_INITIALIZED: AtomicBool = AtomicBool::new(false);
thread_local! {
    static EVENT_LOOP: RwLock<Option<EventLoop<()>>> = const { RwLock::new(None) };
}
static WINDOWS: RwLock<Vec<Window>> = RwLock::new(Vec::new());

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

impl<'a> Default for WindowSettings<'a> {
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
    if !HAS_INITIALIZED.load(Ordering::Acquire) {
        panic!("Attempted to open window before initializing the window library");
    }
    EVENT_LOOP.with(|el_cell| {
        let el_handle = el_cell.write();
        let el = el_handle
            .as_ref()
            .expect("You must open windows from the main thread");
        let mut wb = WindowBuilder::new()
            .with_title(settings.name)
            .with_resizable(settings.resizable);
        if let Some(size) = settings.size {
            wb = wb.with_inner_size(PhysicalSize::new(size.0.x, size.0.y));
        }
        if let Some(pos) = settings.pos {
            wb = wb.with_position(PhysicalPosition::new(pos.0.x, pos.0.y));
        }
        WINDOWS
            .write()
            .push(wb.build(&el).expect("Failed to open window"));
    });
}

// Window implementation of the event loop running function
pub(crate) fn run(mut loop_fn: impl FnMut()) {
    EVENT_LOOP.with(move |el_call| {
        println!("Running window loop...");
        let el = el_call
            .write()
            .take()
            .expect("Event loop must be run from the main thread");
        el.run(move |ev, _eh| {
            println!("Event: {ev:?}");
            loop_fn();
        }).unwrap();
    });
}
