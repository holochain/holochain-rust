use crate::error::LocksmithResult;
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct HcMutex<T: ?Sized>(Mutex<T>);

#[derive(Debug)]
pub struct HcRwLock<T: ?Sized>(RwLock<T>);


impl<T> HcMutex<T> {
    pub fn new(t: T) -> Self {
        HcMutex(Mutex::new(t))
    }
}

impl<T> HcRwLock<T> {
    pub fn new(t: T) -> Self {
        HcRwLock(RwLock::new(t))
    }
}

macro_rules! mutex_impl {
    ($HcMutex: ident, $Guard:ident, $lock_fn:ident, $try_lock_fn:ident, $try_lock_for_fn:ident, $try_lock_until_fn:ident) => {
        impl<T: ?Sized> $HcMutex<T> {
            pub fn $lock_fn(&self) -> LocksmithResult<$Guard<T>> {
                Ok((self.0).$lock_fn())
            }

            pub fn $try_lock_for_fn(&self, duration: Duration) -> Option<$Guard<T>> {
                (self.0).$try_lock_for_fn(duration)
            }

            pub fn $try_lock_until_fn(&self, instant: Instant) -> Option<$Guard<T>> {
                (self.0).$try_lock_until_fn(instant)
            }

            pub fn $try_lock_fn(&self) -> Option<$Guard<T>> {
                ((*self).0).$try_lock_fn()
            }
        }

    };
}

mutex_impl!(
    HcMutex,
    MutexGuard,
    lock,
    try_lock,
    try_lock_for,
    try_lock_until
);
mutex_impl!(
    HcRwLock,
    RwLockReadGuard,
    read,
    try_read,
    try_read_for,
    try_read_until
);
mutex_impl!(
    HcRwLock,
    RwLockWriteGuard,
    write,
    try_write,
    try_write_for,
    try_write_until
);
