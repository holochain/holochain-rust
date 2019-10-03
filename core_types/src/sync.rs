use std::{
    sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError},
    time::Duration,
};

const LOCK_TIMEOUT_MS: u64 = 90000;

#[derive(Debug)]
pub enum HcLockError {
    Timeout,
    PoisonError,
}

pub type HcLockResult<T> = Result<T, HcLockError>;

#[derive(Debug)]
pub struct HcMutex<T: ?Sized>(Mutex<T>);

impl<T> HcMutex<T> {
    pub fn new(v: T) -> Self {
        HcMutex(Mutex::new(v))
    }
}

impl<T: ?Sized> HcMutex<T> {
    pub fn lock(&self) -> HcLockResult<MutexGuard<T>> {
        self.0.try_lock().or_else(|err| match err {
            TryLockError::Poisoned(_poison_error) => Err(HcLockError::PoisonError),
            TryLockError::WouldBlock => {
                std::thread::sleep(Duration::from_millis(LOCK_TIMEOUT_MS));
                self.0.try_lock().map_err(|_| (HcLockError::Timeout))
            }
        })
    }

    pub fn try_lock(&self) -> HcLockResult<MutexGuard<T>> {
        self.0.try_lock().map_err(|err| match err {
            TryLockError::Poisoned(_poison_error) => HcLockError::PoisonError,
            TryLockError::WouldBlock => HcLockError::Timeout,
        })
    }
}

// impl<T> From<Mutex<T>> for HcMutex<T> {

// }

#[derive(Debug)]
pub struct HcRwLock<T: ?Sized>(RwLock<T>);

impl<T> HcRwLock<T> {
    pub fn new(v: T) -> Self {
        HcRwLock(RwLock::new(v))
    }
}

impl<T: ?Sized> HcRwLock<T> {
    pub fn read(&self) -> HcLockResult<RwLockReadGuard<T>> {
        self.0.try_read().or_else(|err| match err {
            TryLockError::Poisoned(_poison_error) => Err(HcLockError::PoisonError),
            TryLockError::WouldBlock => {
                std::thread::sleep(Duration::from_millis(LOCK_TIMEOUT_MS));
                self.0.try_read().map_err(|_| (HcLockError::Timeout))
            }
        })
    }

    pub fn write(&self) -> HcLockResult<RwLockWriteGuard<T>> {
        self.0.try_write().or_else(|err| match err {
            TryLockError::Poisoned(_poison_error) => Err(HcLockError::PoisonError),
            TryLockError::WouldBlock => {
                std::thread::sleep(Duration::from_millis(LOCK_TIMEOUT_MS));
                self.0.try_write().map_err(|_| (HcLockError::Timeout))
            }
        })
    }

    pub fn try_read(&self) -> HcLockResult<RwLockReadGuard<T>> {
        self.0.try_read().map_err(|err| match err {
            TryLockError::Poisoned(_poison_error) => (HcLockError::PoisonError),
            TryLockError::WouldBlock => (HcLockError::Timeout),
        })
    }

    pub fn try_write(&self) -> HcLockResult<RwLockWriteGuard<T>> {
        self.0.try_write().map_err(|err| match err {
            TryLockError::Poisoned(_poison_error) => (HcLockError::PoisonError),
            TryLockError::WouldBlock => (HcLockError::Timeout),
        })
    }
}
