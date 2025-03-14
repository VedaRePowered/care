use std::{
    future::Future,
    sync::atomic::{self, AtomicI8},
    task::{Poll, Waker},
    time::Duration,
};

use parking_lot::Mutex;
use tokio::task;

use super::{end_frame, main_loop_manual};

enum FrameState {
    Running = 0,
    ReadyForNext = 1,
    NextFrame = 2,
}

static NEXT_FRAME_WAKERS: Mutex<Vec<Waker>> = Mutex::new(Vec::new());
static FRAME_STATE: AtomicI8 = AtomicI8::new(FrameState::Running as i8);

pub fn async_executor(fut: impl Future<Output = ()> + 'static + Send, call_end_frame: bool) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.spawn(async {
        fut.await;
        crate::event::exit();
    });
    main_loop_manual(super::init, move |()| {
        let mut wakers = Vec::new();
        std::mem::swap(&mut wakers, &mut NEXT_FRAME_WAKERS.lock());
        for waker in wakers {
            waker.wake();
        }
        // Blocking the main thread here certainly isn't ideal, but it'll have to do because it is
        // the only method to get optimal input latency with the current event system. Effectively,
        // we simply need the code to 'run' in between these calls.
        while FRAME_STATE
            .compare_exchange(
                FrameState::ReadyForNext as i8,
                FrameState::NextFrame as i8,
                atomic::Ordering::Relaxed,
                atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            std::thread::sleep(Duration::from_micros(500));
        }
        if call_end_frame {
            end_frame();
        }
        // TODO: swap graphics queues?
    });
}

#[derive(Debug)]
pub struct NextFrame;

impl Future for NextFrame {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut nfw = NEXT_FRAME_WAKERS.lock();
        if FRAME_STATE
            .compare_exchange(
                FrameState::Running as i8,
                FrameState::ReadyForNext as i8,
                atomic::Ordering::Relaxed,
                atomic::Ordering::Relaxed,
            )
            .is_ok()
        {
            nfw.push(cx.waker().clone());
            Poll::Pending
        } else if FRAME_STATE
            .compare_exchange(
                FrameState::NextFrame as i8,
                FrameState::Running as i8,
                atomic::Ordering::Relaxed,
                atomic::Ordering::Relaxed,
            )
            .is_ok()
        {
            Poll::Ready(())
        } else {
            nfw.push(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub fn next_frame() -> NextFrame {
    NextFrame
}

pub async fn async_yield() {
    task::yield_now().await;
}

pub fn spawn(task: impl Future<Output = ()> + 'static + Send) {
    task::spawn(task);
}
