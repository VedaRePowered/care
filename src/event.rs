use std::{future::Future, time::Instant};

use crate::{
    graphics,
    keyboard::{self, Key},
    math::Vec2,
    mouse,
};

#[cfg(feature = "async-custom")]
mod custom_async;
#[cfg(not(any(feature = "async-custom", feature = "_async-tokio-internal")))]
mod polling;
#[cfg(feature = "_async-tokio-internal")]
mod tokio_event;

#[derive(Debug, Clone)]
/// Data for an event
pub enum EventData {
    /// A key pressed/released event
    KeyEvent {
        /// The key
        key: Key,
        /// Whether it was pressed (true) or released (false)
        pressed: bool,
    },
    /// A key repeat event, used for GUIs
    KeyRepeat {
        /// The key
        key: Key,
    },
    /// The user entered some text, might coincide with the key event
    TextEvent {
        /// The text that has been typed
        text: String,
    },
    /// A mouse moved event
    MouseMoved {
        /// The absolute screen position for the event
        position: Vec2,
    },
    /// A mouse click event
    MouseClick {
        /// The mouse button
        button: i32,
        /// Whether it's currently pressed
        pressed: bool,
    },
    /// The window went in or out of focus
    FocusChange {
        /// Is the window currently focused
        focused: bool,
    },
}

#[derive(Debug, Clone)]
/// An event that has occurred, usually from user input
pub struct Event {
    /// The time the event was created
    pub timestamp: Instant,
    /// The data associated with the event
    pub data: EventData,
}

/// Initialize the care game engine, including all loaded modules
///
/// This is normally called automatically
pub fn init() {
    graphics::init();
}

/// End the frame, resetting everything for the next frame
///
/// This is normally called automatically
pub fn end_frame() {
    #[cfg(feature = "graphics")]
    graphics::present();
    keyboard::reset();
    mouse::reset();
}

/// Run the game main loop, using a specific function that gets called once per frame
pub fn main_loop<T>(
    init_fn: impl FnOnce() -> T + 'static,
    mut loop_fn: impl FnMut(&mut T) + 'static,
) {
    main_loop_manual(
        move || {
            init();
            init_fn()
        },
        move |data| {
            loop_fn(data);
            end_frame();
        },
    );
}

/// Like [main_loop], but you have to call [end_frame] stuff yourself
pub fn main_loop_manual<T>(
    init_fn: impl FnOnce() -> T + 'static,
    loop_fn: impl FnMut(&mut T) + 'static,
) {
    #[cfg(feature = "window")]
    crate::window::run(init_fn, loop_fn);
    #[cfg(not(feature = "window"))]
    {
        let mut data = init_fn();
        loop {
            loop_fn(&mut data);
        }
    }
}

#[cfg(all(feature = "async-custom", feature = "_async-tokio-internal"))]
compile_error!("Only one async executor feature can be enabled at a time.");

/// Run the game main function, as a single async function
///
/// This supports multiple async executors as backends
pub fn main_async(fut: impl Future<Output = ()> + 'static + Send) {
    #[cfg(not(any(feature = "async-custom", feature = "_async-tokio-internal")))]
    polling::async_executor(fut, true);
    #[cfg(feature = "async-custom")]
    custom_async::async_executor(fut, true);
    #[cfg(feature = "_async-tokio-internal")]
    tokio_event::async_executor(fut, true);
}

/// Like [main_async], but you have to call [end_frame] stuff yourself
/// after every frame
pub fn main_async_manual(fut: impl Future<Output = ()> + 'static + Send) {
    #[cfg(not(any(feature = "async-custom", feature = "_async-tokio-internal")))]
    polling::async_executor(fut, false);
    #[cfg(feature = "async-custom")]
    custom_async::async_executor(fut, false);
    #[cfg(feature = "_async-tokio-internal")]
    tokio_event::async_executor(fut, false);
}

/// Await until the next frame
pub async fn next_frame() {
    #[cfg(not(any(feature = "async-custom", feature = "_async-tokio-internal")))]
    return polling::next_frame().await;
    #[cfg(feature = "async-custom")]
    return custom_async::next_frame().await;
    #[cfg(feature = "_async-tokio-internal")]
    return tokio_event::next_frame().await;
}

/// Await, immediately readying, so that other tasks can run along side this task without waiting
/// for anything in particular
pub async fn async_yield() {
    #[cfg(feature = "async-custom")]
    return custom_async::async_yield().await;
    #[cfg(feature = "_async-tokio-internal")]
    return tokio_event::async_yield().await;
}

/// Spawn an async task on the current executor
///
/// Panics on the "polling" executor
pub fn spawn(task: impl Future<Output = ()> + 'static + Send) {
    #[cfg(not(any(feature = "async-custom", feature = "_async-tokio-internal")))]
    panic!("The polling/null executor does not support spawning multiple tasks.");
    #[cfg(feature = "async-custom")]
    return custom_async::spawn(task);
    #[cfg(feature = "_async-tokio-internal")]
    return tokio_event::spawn(task);
}

/// Exit the game
pub fn exit() {
    #[cfg(feature = "window")]
    crate::window::exit();
}

/// Process an event, this can only send events within the game, not emulate actual mouse motion or
/// keyboard buttons
pub fn handle_event(ev: Event) {
    match &ev.data {
        EventData::KeyEvent { key, pressed } => crate::keyboard::process_key_event(*key, *pressed),
        EventData::MouseMoved { position } => crate::mouse::process_mouse_moved_event(*position),
        EventData::MouseClick { button, pressed } => {
            crate::mouse::process_mouse_click_event(*button, *pressed)
        }
        EventData::TextEvent { .. } => {}
        EventData::FocusChange { .. } => {}
        EventData::KeyRepeat { .. } => {}
    }
    crate::gui::process_event(ev);
}
