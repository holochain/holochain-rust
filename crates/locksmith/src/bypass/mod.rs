mod guard;
mod mutex;

pub use crate::bypass::{
    guard::{
        HcMutexGuard as MutexGuard, HcRwLockReadGuard as RwLockReadGuard,
        HcRwLockWriteGuard as RwLockWriteGuard,
    },
    mutex::{HcMutex as Mutex, HcRwLock as RwLock},
};

pub fn spawn_locksmith_guard_watcher() {
    warn!("Locksmith is in bypass mode -- spawn_locksmith_guard_watcher is a noop")
}

#[derive(Debug)]
pub struct LocksmithError;
