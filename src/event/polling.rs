//! Simple async backend that uses polling

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::event::main_loop_manual;

pub fn async_executor(mut fut: impl Future<Output = ()> + 'static, call_end_frame: bool) {
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
    main_loop_manual(super::init, move |()| {
        let _ = pin!(&mut async move { fut })
            .poll(&mut Context::from_waker(&unsafe { Waker::from_raw(RAW) }));
        if call_end_frame {
            end_frame();
        }
    })
}

pub async fn next_frame() {
    let mut ready = false;
    std::future::poll_fn(move |_| {
        if ready {
            Poll::Ready(())
        } else {
            ready = true;
            Poll::Pending
        }
    })
    .await;
}
