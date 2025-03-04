use std::{
    future::Future,
    sync::atomic::{self, AtomicBool},
    task::{Poll, Waker},
    time::Duration,
};

use parking_lot::Mutex;
use tokio::task;

use super::{end_frame, main_loop_manual};

static NEXT_FRAME_WAKERS: Mutex<Vec<Waker>> = Mutex::new(Vec::new());
static FRAME_READY: AtomicBool = AtomicBool::new(false);

pub fn async_executor(fut: impl Future<Output = ()> + 'static + Send, call_end_frame: bool) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.spawn(fut);
    main_loop_manual(super::init, move |()| {
        let mut wakers = Vec::new();
        while FRAME_READY
            .compare_exchange(
                true,
                false,
                atomic::Ordering::Acquire,
                atomic::Ordering::Acquire,
            )
            .is_err()
        {
            std::thread::sleep(Duration::from_millis(1));
        }
        if call_end_frame {
            end_frame();
        }
        // TODO: swap graphics queues
        std::mem::swap(&mut wakers, &mut NEXT_FRAME_WAKERS.lock());
        for waker in wakers {
            waker.wake();
        }
    });
}

#[derive(Debug, Default)]
pub struct NextFrame(bool);

impl Future for NextFrame {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.0 {
            NEXT_FRAME_WAKERS.lock().push(cx.waker().clone());
            FRAME_READY.store(true, atomic::Ordering::Release);
            self.0 = true;
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub fn next_frame() -> NextFrame {
    NextFrame::default()
}

pub async fn async_yield() {
    task::yield_now().await;
}

pub fn spawn(task: impl Future<Output = ()> + 'static + Send) {
    task::spawn(task);
}
