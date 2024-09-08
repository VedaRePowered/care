use std::sync::atomic::AtomicBool;

use parking_lot::RwLock;

pub trait Transferable<C = ()> {
    fn send(&self, context: &C) -> Vec<u8>;
    fn receive(data: &[u8], context: &C) -> Self;
}

pub trait SyncManager<C = ()> {
    fn queue_sync<T>(&self, data: SyncedValue<T, C>)
        where T: 'static + Transferable<C>;
}

pub struct SyncedValue<T, C = ()>
    where T: 'static + Transferable<C> {
    id: usize,
    dirty: AtomicBool,
    inner: RwLock<T: 'static>,
    manager: Arc<dyn SyncManager<C>>,
}
