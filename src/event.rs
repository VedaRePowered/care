use std::{future::Future, convert::Infallible, pin::Pin, task::{Context, Waker, RawWaker, RawWakerVTable}, time::Instant};

use crate::keyboard::Key;

#[derive(Debug)]
pub enum EventData {
    KeyEvent {
        key: Key,
        pressed: bool,
    },
}

#[derive(Debug)]
pub struct Event {
    pub timestamp: Instant,
    pub data: EventData,
}

pub fn main_loop(loop_fn: impl FnMut() + 'static) {
    #[cfg(feature = "window")]
    crate::window::run(loop_fn);
    #[cfg(not(feature = "window"))]
    loop {
        loop_fn();
    }
}

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
        let _ = Pin::new(&mut fut).poll(&mut Context::from_waker(&unsafe { Waker::from_raw(RAW) }));
    })
}

pub fn handle_event(ev: Event) {
    match ev.data {
        EventData::KeyEvent { key, pressed } => crate::keyboard::process_key_event(key, pressed),
    }
}

