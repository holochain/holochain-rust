
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod common;
mod error;
mod guard;
mod mutex;
mod tracker;

pub use error::LocksmithError;
pub use guard::{
    HcMutexGuard as MutexGuard, HcRwLockReadGuard as RwLockReadGuard,
    HcRwLockWriteGuard as RwLockWriteGuard,
};
pub use mutex::{HcMutex as Mutex, HcRwLock as RwLock};
pub use tracker::spawn_locksmith_guard_watcher;
