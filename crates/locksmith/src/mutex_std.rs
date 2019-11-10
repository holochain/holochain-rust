use crate::error::{LocksmithResult, LocksmithError, LocksmithErrorKind, LockType};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

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
    ($HcMutex: ident, $Guard:ident, $lock_type:ident, $lock_fn:ident, $try_lock_fn:ident, $try_lock_for_fn:ident, $try_lock_until_fn:ident) => {
        impl<T: ?Sized> $HcMutex<T> {
            pub fn $lock_fn(&self) -> LocksmithResult<$Guard<T>> {
                (self.0).$lock_fn().map_err(|_| LocksmithError::new(
                    LockType::$lock_type,
                    LocksmithErrorKind::LocksmithPoisonError,
                ))
            }

            pub fn $try_lock_fn(&self) -> Option<$Guard<T>> {
                ((*self).0).$try_lock_fn().map_err(|e| match e {
                    TryLockError::Poisoned(_) => panic!("POISONED GUARD found in $try_lock_fn"),
                    e => e
                }).ok()
            }
        }
    };
}

mutex_impl!(
    HcMutex,
    MutexGuard,
    Lock,
    lock,
    try_lock,
    try_lock_for,
    try_lock_until
);
mutex_impl!(
    HcRwLock,
    RwLockReadGuard,
    Read,
    read,
    try_read,
    try_read_for,
    try_read_until
);
mutex_impl!(
    HcRwLock,
    RwLockWriteGuard,
    Write,
    write,
    try_write,
    try_write_for,
    try_write_until
);
