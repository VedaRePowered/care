//! Slightly less simple async backend that actually supports some basic spawning functionality
//!
//! If it isn't _very_ obvious, I have absolutely no idea what I'm doing here.

use std::{
    future::Future,
    pin::{pin, Pin},
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc,
    },
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use parking_lot::RwLock;

use crate::event::main_loop;

use super::end_frame;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
    awake: Arc<AtomicBool>,
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    fn new(fut: impl Future<Output = ()> + 'static) -> Self {
        Self {
            future: Box::pin(fut),
            awake: Arc::new(AtomicBool::new(true)),
        }
    }
}

pub struct Executor {
    tasks: RwLock<Vec<Task>>,
    to_spawn: RwLock<Vec<Task>>,
    wake_next_frame: RwLock<Vec<Arc<AtomicBool>>>,
}

impl Executor {
    /// Execute tasks until none of them are marked as awake
    fn run_until_sleep(&self) {
        let mut tasks = self.tasks.write();
        let mut wake_next_frame = self.wake_next_frame.write();
        let mut any_awake = true;
        for waker in wake_next_frame.drain(..) {
            waker.store(true, Ordering::Relaxed);
        }
        for new_task in self.to_spawn.write().drain(..) {
            tasks.push(new_task);
        }
        while any_awake {
            any_awake = false;
            let mut i = 0;
            while i < tasks.len() {
                let mut remove = false;
                if tasks[i].awake.load(Ordering::Relaxed) {
                    any_awake = true;
                    AWAIT_REASON.store(AwaitReason::Waker as i32, Ordering::Relaxed);
                    let awake_bool = tasks[i].awake.clone();
                    let result = pin!(&mut tasks[i].future)
                        .poll(&mut Context::from_waker(&create_waker(awake_bool)));
                    if let Poll::Ready(_) = result {
                        tasks.swap_remove(i);
                        remove = true;
                    } else {
                        match AwaitReason::from(AWAIT_REASON.load(Ordering::Relaxed)) {
                            AwaitReason::Waker => {
                                tasks[i].awake.store(false, Ordering::Relaxed);
                            }
                            AwaitReason::NextFrame => {
                                tasks[i].awake.store(false, Ordering::Relaxed);
                                wake_next_frame.push(tasks[i].awake.clone());
                            }
                            AwaitReason::Yield => {
                                // Do nothing, this task will continue immediately after running
                                // other tasks
                            }
                        }
                    }
                }
                for new_task in self.to_spawn.write().drain(..) {
                    tasks.push(new_task);
                }
                if !remove {
                    i += 1;
                }
            }
        }
    }
    /// Spawn a new task
    fn spawn(&self, task: Task) {
        self.to_spawn.write().push(task);
    }
}

static ASYNC_EXECUTOR: Executor = Executor {
    tasks: RwLock::new(Vec::new()),
    to_spawn: RwLock::new(Vec::new()),
    wake_next_frame: RwLock::new(Vec::new()),
};

enum AwaitReason {
    Waker = 0,
    Yield = 1,
    NextFrame = 2,
}

impl From<i32> for AwaitReason {
    fn from(value: i32) -> Self {
        match value {
            1 => AwaitReason::Yield,
            2 => AwaitReason::NextFrame,
            _ => AwaitReason::Waker,
        }
    }
}

static AWAIT_REASON: AtomicI32 = AtomicI32::new(0);

const WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop);

fn waker_clone(waker: *const ()) -> RawWaker {
    let waker = unsafe { &*(waker as *const Arc<AtomicBool>) };
    let waker = waker.clone();
    let waker_ptr = Box::into_raw(Box::new(waker));
    RawWaker::new(waker_ptr as *const (), &WAKER_VTABLE)
}

fn waker_wake(waker: *const ()) {
    let waker = unsafe { Box::from_raw(waker as *mut Arc<AtomicBool>) };
    waker.store(true, Ordering::Relaxed);
}

fn waker_wake_by_ref(waker: *const ()) {
    let waker = unsafe { &*(waker as *const Arc<AtomicBool>) };
    waker.store(true, Ordering::Relaxed);
}

fn waker_drop(waker: *const ()) {
    let _waker = unsafe { Box::from_raw(waker as *mut Arc<AtomicBool>) };
}

fn create_waker(waker: Arc<AtomicBool>) -> Waker {
    let waker_ptr = Box::into_raw(Box::new(waker));
    let raw_waker = RawWaker::new(waker_ptr as *const (), &WAKER_VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

pub fn async_executor(fut: impl Future<Output = ()> + 'static, call_end_frame: bool) {
    ASYNC_EXECUTOR.spawn(Task::new(async move {
        fut.await;
    }));

    main_loop(move || {
        ASYNC_EXECUTOR.run_until_sleep();
        if call_end_frame {
            end_frame();
        }
    });
}

pub async fn next_frame() {
    let mut ready = false;
    std::future::poll_fn(move |_| {
        if ready {
            Poll::Ready(())
        } else {
            AWAIT_REASON.store(AwaitReason::NextFrame as i32, Ordering::Relaxed);
            ready = true;
            Poll::Pending
        }
    })
    .await;
}

pub async fn async_yield() {
    let mut ready = false;
    std::future::poll_fn(move |_| {
        if ready {
            Poll::Ready(())
        } else {
            AWAIT_REASON.store(AwaitReason::Yield as i32, Ordering::Relaxed);
            ready = true;
            Poll::Pending
        }
    })
    .await;
}

pub fn spawn(task: impl Future<Output = ()> + 'static) {
    ASYNC_EXECUTOR.spawn(Task {
        future: Box::pin(task),
        awake: Arc::new(AtomicBool::new(true)),
    })
}
