#![feature(checked_duration_since)]

// #[macro_use]
// extern crate lazy_static;
// #[macro_use]
// extern crate log;

// mod common;
mod error;
mod guard_pl;
mod mutex_pl;
// mod tracker;

pub use error::LocksmithError;
pub use guard_pl::{
    HcMutexGuard as MutexGuard, HcRwLockReadGuard as RwLockReadGuard,
    HcRwLockWriteGuard as RwLockWriteGuard,
};
pub use mutex_pl::{HcMutex as Mutex, HcRwLock as RwLock};
// pub use tracker::spawn_locksmith_guard_watcher;
