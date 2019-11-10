use crate::{
    error::{LockType, LocksmithError, LocksmithErrorKind, LocksmithResult},
    guard::{HcMutexGuard, HcRwLockReadGuard, HcRwLockWriteGuard},
};
use parking_lot::{Mutex, RwLock};
use std::time::{Instant, Duration};

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
    ($HcMutex: ident, $HcGuard:ident, $lock_type:ident, $lock_fn:ident, $try_lock_fn:ident, $try_lock_until_fn:ident, $new_guard_fn:ident) => {
        impl<T: ?Sized> $HcMutex<T> {
            pub fn $lock_fn(&self) -> LocksmithResult<$HcGuard<T>> {
                let deadline = Instant::now() + Duration::from_secs(120);
                self.$try_lock_until_fn(deadline)
            }

            fn $try_lock_until_fn(&self, deadline: Instant) -> LocksmithResult<$HcGuard<T>> {
                // Set a number twice the expected number of iterations, just to prevent an infinite loop
                let max_iters = 2 * 120 * 1000 * 10;
                for _i in 0..max_iters {
                    match self.$try_lock_fn() {
                        Some(v) => {
                            return Ok(v);
                        }
                        None => {

                            // TIMEOUT
                            if let None = deadline.checked_duration_since(Instant::now()) {
                                // PENDING_LOCKS.lock().remove(&puid);
                                return Err(LocksmithError::new(
                                    LockType::$lock_type,
                                    LocksmithErrorKind::LocksmithTimeout,
                                ));
                            }
                        }
                    }
                    std::thread::sleep(Duration::from_nanos(100));
                }
                error!(
                    "$try_lock_until_inner_fn exceeded max_iters, this should not have happened!"
                );
                return Err(LocksmithError::new(
                    LockType::$lock_type,
                    LocksmithErrorKind::LocksmithTimeout,
                ));
            }
            pub fn $try_lock_fn(&self) -> Option<$HcGuard<T>> {
                (*self).inner.$try_lock_fn().map(|g| $HcGuard::new(g))
            }
        }
    };
}

mutex_impl!(
    HcMutex,
    HcMutexGuard,
    Lock,
    lock,
    try_lock,
    try_lock_until,
    new_guard
);
mutex_impl!(
    HcRwLock,
    HcRwLockReadGuard,
    Read,
    read,
    try_read,
    try_read_until,
    new_guard_read
);
mutex_impl!(
    HcRwLock,
    HcRwLockWriteGuard,
    Write,
    write,
    try_write,
    try_write_until,
    new_guard_write
);
