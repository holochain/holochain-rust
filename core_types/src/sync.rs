use backtrace::Backtrace;
use std::{
    sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError},
    time::Duration,
};

const LOCK_TIMEOUT_SECS: u64 = 20;

#[derive(Debug)]
pub enum HcLockErrorKind {
    HcLockTimeout,
    HcLockPoisonError,
}

#[derive(Debug)]
pub enum LockType {
    Lock,
    Read,
    Write,
}

#[derive(Debug)]
pub struct HcLockError {
    lock_type: LockType,
    backtraces: Option<Vec<Backtrace>>,
    kind: HcLockErrorKind,
}

impl HcLockError {
    pub fn new(
        lock_type: LockType,
        backtraces: Option<Vec<Backtrace>>,
        kind: HcLockErrorKind,
    ) -> Self {
        Self {
            lock_type,
            backtraces: backtraces.map(|b| {
                b.clone()
                    .into_iter()
                    .map(|mut b| {
                        b.resolve();
                        b
                    })
                    .collect::<Vec<Backtrace>>()
            }),
            kind,
        }
    }
}

pub type HcLockResult<T> = Result<T, HcLockError>;

#[derive(Debug)]
pub struct HcMutex<T: ?Sized> {
    backtraces: Mutex<Vec<Backtrace>>,
    inner: Mutex<T>,
}

impl<T> HcMutex<T> {
    pub fn new(v: T) -> Self {
        Self {
            backtraces: Mutex::new(Vec::new()),
            inner: Mutex::new(v),
        }
    }
}

impl<T: ?Sized> HcMutex<T> {
    pub fn lock(&self) -> HcLockResult<MutexGuard<T>> {
        let bts = update_backtraces(&self.backtraces);
        match (&*self).inner.try_lock() {
            Ok(v) => Ok(v),
            Err(err) => match err {
                TryLockError::Poisoned(_poison_error) => Err(HcLockError::new(
                    LockType::Lock,
                    bts,
                    HcLockErrorKind::HcLockPoisonError,
                )),
                TryLockError::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(LOCK_TIMEOUT_SECS));
                    (&*self).inner.try_lock().map_err(|_| {
                        (HcLockError::new(LockType::Lock, bts, HcLockErrorKind::HcLockTimeout))
                    })
                }
            },
        }
    }

    pub fn try_lock(&self) -> HcLockResult<MutexGuard<T>> {
        let bts = update_backtraces(&self.backtraces);
        (&*self).inner.try_lock().map_err(|err| match err {
            TryLockError::Poisoned(_poison_error) => {
                HcLockError::new(LockType::Lock, bts, HcLockErrorKind::HcLockPoisonError)
            }
            TryLockError::WouldBlock => {
                HcLockError::new(LockType::Lock, bts, HcLockErrorKind::HcLockTimeout)
            }
        })
    }
}

// impl<T> From<Mutex<T>> for HcMutex<T> {
// TODO
// }

#[derive(Debug)]
pub struct HcRwLock<T: ?Sized> {
    backtraces: Mutex<Vec<Backtrace>>,
    inner: RwLock<T>,
}

impl<T> HcRwLock<T> {
    pub fn new(v: T) -> Self {
        Self {
            backtraces: Mutex::new(Vec::new()),
            inner: RwLock::new(v),
        }
    }
}

impl<T: ?Sized> HcRwLock<T> {
    pub fn read(&self) -> HcLockResult<RwLockReadGuard<T>> {
        let bts = update_backtraces(&self.backtraces);
        match (&*self).inner.try_read() {
            Ok(v) => Ok(v),
            Err(err) => match err {
                TryLockError::Poisoned(_poison_error) => Err(HcLockError::new(
                    LockType::Read,
                    bts,
                    HcLockErrorKind::HcLockPoisonError,
                )),
                TryLockError::WouldBlock => {
                    std::thread::sleep(Duration::from_secs(LOCK_TIMEOUT_SECS));
                    (&*self).inner.try_read().map_err(|_| {
                        (HcLockError::new(LockType::Read, bts, HcLockErrorKind::HcLockTimeout))
                    })
                }
            },
        }
    }

    pub fn write(&self) -> HcLockResult<RwLockWriteGuard<T>> {
        let bts = update_backtraces(&self.backtraces);

        match (&*self).inner.try_write() {
            Ok(v) => Ok(v),
            Err(err) => match err {
                TryLockError::Poisoned(_poison_error) => Err(HcLockError::new(
                    LockType::Write,
                    bts,
                    HcLockErrorKind::HcLockPoisonError,
                )),
                TryLockError::WouldBlock => {
                    std::thread::sleep(Duration::from_secs(LOCK_TIMEOUT_SECS));
                    (&*self).inner.try_write().map_err(|_| {
                        (HcLockError::new(LockType::Write, bts, HcLockErrorKind::HcLockTimeout))
                    })
                }
            },
        }
    }

    pub fn try_read(&self) -> HcLockResult<RwLockReadGuard<T>> {
        let bts = update_backtraces(&self.backtraces);
        (&*self).inner.try_read().map_err(|err| match err {
            TryLockError::Poisoned(_poison_error) => {
                (HcLockError::new(LockType::Read, bts, HcLockErrorKind::HcLockPoisonError))
            }
            TryLockError::WouldBlock => {
                (HcLockError::new(LockType::Read, bts, HcLockErrorKind::HcLockTimeout))
            }
        })
    }

    pub fn try_write(&self) -> HcLockResult<RwLockWriteGuard<T>> {
        let bts = update_backtraces(&self.backtraces);
        (&*self).inner.try_write().map_err(|err| match err {
            TryLockError::Poisoned(_poison_error) => {
                (HcLockError::new(LockType::Write, bts, HcLockErrorKind::HcLockPoisonError))
            }
            TryLockError::WouldBlock => {
                (HcLockError::new(LockType::Write, bts, HcLockErrorKind::HcLockTimeout))
            }
        })
    }
}

fn try_lock_ok<T, P>(result: Result<T, TryLockError<P>>) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(TryLockError::WouldBlock) => None,
        Err(TryLockError::Poisoned(err)) => {
            println!("try_lock_ok found poisoned lock! {:?}", err);
            None
        }
    }
}

fn update_backtraces(mutex: &Mutex<Vec<Backtrace>>) -> Option<Vec<Backtrace>> {
    if let Some(mut bts) = try_lock_ok(mutex.try_lock()) {
        bts.push(Backtrace::new_unresolved());
        Some(bts.clone())
    } else {
        None
    }
}
