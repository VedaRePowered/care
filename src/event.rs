use std::{future::Future, convert::Infallible, pin::Pin, task::{Context, Waker, RawWaker, RawWakerVTable}, time::Instant};

use crate::{keyboard::Key, math::Vec2};

#[derive(Debug)]
/// Data for an event
pub enum EventData {
    /// A key pressed/released event
    KeyEvent {
        /// The key
        key: Key,
        /// Whether it was pressed (true) or released (false)
        pressed: bool,
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
}

#[derive(Debug)]
/// An event that has occurred, usually from user input
pub struct Event {
    /// The time the event was created
    pub timestamp: Instant,
    /// The data associated with the event
    pub data: EventData,
}

/// Run the game main loop, using a specific function
pub fn main_loop(loop_fn: impl FnMut() + 'static) {
    #[cfg(feature = "window")]
    crate::window::run(loop_fn);
    #[cfg(not(feature = "window"))]
    loop {
        loop_fn();
    }
}

/// Run the game main function, as an async function that must yield exactly once per frame
///
/// Eventually this will be reimplemented better, maybe with tokio
pub fn main_async(mut fut: impl Future<Output = Infallible> + Unpin + 'static) {
    // From the rust source code, a "no-op" waker, because there's only ever one future.
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        // Cloning just returns a new no-op raw waker
        |_| RAW,
        // `wake` does nothing
        |_| {},
        // `wake_by_ref` does nothing
        |_| {},
        // Dropping does nothing as we don't allocate anything
        |_| {},
        );
    const RAW: RawWaker = RawWaker::new(std::ptr::null(), &VTABLE);
    main_loop(move || {
        let val = Pin::new(&mut fut).poll(&mut Context::from_waker(&unsafe { Waker::from_raw(RAW) }));
    })
}

/// Process an event, this can only send events within the game, not emulate actual mouse motion or
/// keyboard buttons
pub fn handle_event(ev: Event) {
    match ev.data {
        EventData::KeyEvent { key, pressed } => crate::keyboard::process_key_event(key, pressed),
        EventData::MouseMoved { position } => crate::mouse::process_mouse_moved_event(position),
        EventData::MouseClick { button, pressed } => crate::mouse::process_mouse_click_event(button, pressed),
    }
}
