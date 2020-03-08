use crate::{
    bypass::guard::{HcMutexGuard, HcRwLockReadGuard, HcRwLockWriteGuard},
    error::LocksmithResult,
};
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct HcMutex<T: ?Sized>(Mutex<T>);

#[derive(Debug)]
pub struct HcRwLock<T: ?Sized>(RwLock<T>);

impl<T> HcMutex<T> {
    pub fn new(v: T) -> Self {
        Self(Mutex::new(v))
    }
}

impl<T> HcRwLock<T> {
    pub fn new(v: T) -> Self {
        Self(RwLock::new(v))
    }
}

macro_rules! mutex_impl {
    ($HcMutex: ident, $HcGuard:ident, $Guard:ident, $lock_fn:ident, $try_lock_fn:ident, $try_lock_for_fn:ident, $try_lock_until_fn:ident, $new_guard_fn:ident) => {
        impl<T: ?Sized> $HcMutex<T> {
            pub fn $lock_fn(&self) -> LocksmithResult<$HcGuard<T>> {
                Ok((self.0).$lock_fn()).map(|g| self.$new_guard_fn(g))
            }

            pub fn $try_lock_for_fn(&self, duration: Duration) -> Option<$HcGuard<T>> {
                (self.0)
                    .$try_lock_for_fn(duration)
                    .map(|g| self.$new_guard_fn(g))
            }

            pub fn $try_lock_until_fn(&self, instant: Instant) -> Option<$HcGuard<T>> {
                (self.0)
                    .$try_lock_until_fn(instant)
                    .map(|g| self.$new_guard_fn(g))
            }

            pub fn $try_lock_fn(&self) -> Option<$HcGuard<T>> {
                (self.0).$try_lock_fn().map(|g| self.$new_guard_fn(g))
            }

            fn $new_guard_fn<'a>(&self, inner: $Guard<'a, T>) -> $HcGuard<'a, T> {
                $HcGuard::new(inner)
            }
        }
    };
}

mutex_impl!(
    HcMutex,
    HcMutexGuard,
    MutexGuard,
    lock,
    try_lock,
    try_lock_for,
    try_lock_until,
    new_guard
);
mutex_impl!(
    HcRwLock,
    HcRwLockReadGuard,
    RwLockReadGuard,
    read,
    try_read,
    try_read_for,
    try_read_until,
    new_guard_read
);
mutex_impl!(
    HcRwLock,
    HcRwLockWriteGuard,
    RwLockWriteGuard,
    write,
    try_write,
    try_write_for,
    try_write_until,
    new_guard_write
);
