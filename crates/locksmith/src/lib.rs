#![feature(checked_duration_since)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod sync;
pub use sync::{
    spawn_locksmith_guard_watcher, HcMutex as Mutex, HcMutexGuard as MutexGuard,
    HcRwLock as RwLock, HcRwLockReadGuard as RwLockReadGuard,
    HcRwLockWriteGuard as RwLockWriteGuard, LocksmithError,
};
