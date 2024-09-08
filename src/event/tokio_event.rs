use std::{future::Future, task::{Poll, Waker}};

use parking_lot::Mutex;
use tokio::task;

use super::{end_frame, main_loop};

static NEXT_FRAME_WAKERS: Mutex<Vec<Waker>> = Mutex::new(Vec::new());

pub fn async_executor(fut: impl Future<Output = ()> + 'static + Send, call_end_frame: bool) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.spawn(fut);
    main_loop(move || {
        let mut wakers = Vec::new();
        std::mem::swap(&mut wakers, &mut NEXT_FRAME_WAKERS.lock());
        for waker in wakers {
            waker.wake();
        }
        if call_end_frame {
            end_frame();
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
