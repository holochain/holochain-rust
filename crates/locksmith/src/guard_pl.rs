use parking_lot::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};

pub type HcMutexGuard<'a, T> = MutexGuard<'a, T>;
pub type HcRwLockReadGuard<'a, T> = RwLockReadGuard<'a, T>;
pub type HcRwLockWriteGuard<'a, T> = RwLockWriteGuard<'a, T>;
