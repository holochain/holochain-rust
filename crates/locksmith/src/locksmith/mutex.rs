use crate::{
    common::LOCK_TIMEOUT,
    error::{LockType, LocksmithError, LocksmithErrorKind, LocksmithResult},
    guard::{HcMutexGuard, HcRwLockReadGuard, HcRwLockWriteGuard},
};
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct HcMutex<T: ?Sized> {
    fair_unlocking: bool,
    inner: Mutex<T>,
}

impl<T> HcMutex<T> {
    pub fn new(v: T) -> Self {
        Self {
            fair_unlocking: false,
            inner: Mutex::new(v),
        }
    }

    pub fn new_with_fair_unlocking(v: T) -> Self {
        Self {
            fair_unlocking: true,
            inner: Mutex::new(v),
        }
    }

    pub fn use_fair_unlocking(mut self) -> Self {
        self.fair_unlocking = true;
        self
    }
}

#[derive(Debug)]
pub struct HcRwLock<T: ?Sized> {
    fair_unlocking: bool,
    inner: RwLock<T>,
}

impl<T> HcRwLock<T> {
    pub fn new(v: T) -> Self {
        Self {
            fair_unlocking: false,
            inner: RwLock::new(v),
        }
    }

    pub fn new_with_fair_unlocking(v: T) -> Self {
        Self {
            fair_unlocking: true,
            inner: RwLock::new(v),
        }
    }

    pub fn use_fair_unlocking(mut self) -> Self {
        self.fair_unlocking = true;
        self
    }
}

macro_rules! mutex_impl {
    ($HcMutex: ident, $Mutex: ident, $HcGuard:ident, $Guard:ident, $lock_type:ident, $lock_fn:ident, $try_lock_fn:ident, $try_lock_for_fn:ident, $try_lock_until_fn:ident, $new_guard_fn:ident) => {
        impl<T: ?Sized> $HcMutex<T> {
            pub fn $lock_fn(&self) -> LocksmithResult<$HcGuard<T>> {
                self.$try_lock_for_fn(*LOCK_TIMEOUT).ok_or_else(|| {
                    LocksmithError::new(LockType::$lock_type, LocksmithErrorKind::LocksmithTimeout)
                })
            }

            pub fn $try_lock_for_fn(&self, duration: Duration) -> Option<$HcGuard<T>> {
                self.inner
                    .$try_lock_for_fn(duration)
                    .map(|g| self.$new_guard_fn(g))
            }

            pub fn $try_lock_until_fn(&self, instant: Instant) -> Option<$HcGuard<T>> {
                self.inner
                    .$try_lock_until_fn(instant)
                    .map(|g| self.$new_guard_fn(g))
            }

            pub fn $try_lock_fn(&self) -> Option<$HcGuard<T>> {
                (*self).inner.$try_lock_fn().map(|g| self.$new_guard_fn(g))
            }

            fn $new_guard_fn<'a>(&self, inner: $Guard<'a, T>) -> $HcGuard<'a, T> {
                if self.fair_unlocking {
                    $HcGuard::new(inner).use_fair_unlocking()
                } else {
                    $HcGuard::new(inner)
                }
            }
        }
    };
}

mutex_impl!(
    HcMutex,
    Mutex,
    HcMutexGuard,
    MutexGuard,
    Lock,
    lock,
    try_lock,
    try_lock_for,
    try_lock_until,
    new_guard
);
mutex_impl!(
    HcRwLock,
    RwLock,
    HcRwLockReadGuard,
    RwLockReadGuard,
    Read,
    read,
    try_read,
    try_read_for,
    try_read_until,
    new_guard_read
);
mutex_impl!(
    HcRwLock,
    RwLock,
    HcRwLockWriteGuard,
    RwLockWriteGuard,
    Write,
    write,
    try_write,
    try_write_for,
    try_write_until,
    new_guard_write
);
