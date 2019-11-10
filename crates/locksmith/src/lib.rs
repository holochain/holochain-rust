#![feature(checked_duration_since)]

// #[macro_use]
// extern crate lazy_static;
// #[macro_use]
// extern crate log;

// mod common;
mod error;
mod guard_passthru;
mod mutex_passthru;
// mod tracker;

pub use error::LocksmithError;
pub use guard_passthru::{
    HcMutexGuard as MutexGuard, HcRwLockReadGuard as RwLockReadGuard,
    HcRwLockWriteGuard as RwLockWriteGuard,
};
pub use mutex_passthru::{HcMutex as Mutex, HcRwLock as RwLock};
// pub use tracker::spawn_locksmith_guard_watcher;
