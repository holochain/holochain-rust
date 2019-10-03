use snowflake::ProcessUniqueId;
use backtrace::Backtrace;

use std::{
    ops::{Deref, DerefMut},
    sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError},
    time::{Duration, Instant},
    thread,
};

const LOCK_TIMEOUT_SECS: u64 = 90;
const LOCK_POLL_INTERVAL_MS: u64 = 10;

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
        _backtraces: Option<Vec<Backtrace>>,
        kind: HcLockErrorKind,
    ) -> Self {
        Self {
            lock_type,
            backtraces: None,
            // backtraces: backtraces.map(|b| {
            //     b.clone()
            //         .into_iter()
            //         .map(|mut b| {
            //             b.resolve();
            //             b
            //         })
            //         .collect::<Vec<Backtrace>>()
            // }),
            kind,
        }
    }
}

pub type HcLockResult<T> = Result<T, HcLockError>;

lazy_static! {
    static ref GUARDS: Mutex<Vec<(ProcessUniqueId, Instant, Backtrace)>> =
        Mutex::new(Vec::new());
}

pub fn spawn_hc_guard_watcher() {
    let _ = thread::spawn(move || {
        loop {
            {
                let mut guards = GUARDS.lock().expect("someone poisoned the GUARDS");
                *guards = guards.iter().filter(|(puid, instant, backtrace)| {
                    let timeout = Instant::now().duration_since(*instant).as_secs() > LOCK_TIMEOUT_SECS;
                    if timeout {
                        let mut b = backtrace.clone();
                        b.resolve();
                        println!("IMMORTAL LOCK GUARD!!! puid={:?} backtrace:\n{:?}", puid, b);
                    }
                    !timeout
                }).cloned().collect();
                println!("spawn_hc_guard_watcher: num={:?}", guards.len());
                for (puid, instant, _) in guards.iter() {
                    println!("{:?} {:?}", puid, instant);
                }
            }
            thread::sleep(Duration::from_millis(3000));
        }
    });
    println!("spawn_hc_guard_watcher: SPAWNED");
}

///////////////////////////////////////////////////////////////
/// GUARDS


pub struct HcMutexGuard<'a, T: ?Sized> {
    puid: ProcessUniqueId,
    inner: MutexGuard<'a, T>,
}

impl<'a, T: ?Sized> HcMutexGuard<'a, T> {
    pub fn new(inner: MutexGuard<'a, T>) -> Self {
        let puid = ProcessUniqueId::new();
        GUARDS.lock().expect("someone poisoned the GUARDS").push((puid, Instant::now(), Backtrace::new_unresolved()));
        Self { puid, inner }
    }
}

impl<'a, T: ?Sized> Drop for HcMutexGuard<'a, T> {
    fn drop(&mut self) {
        GUARDS.lock().expect("someone poisoned the GUARDS").retain(|(puid, _, _)| *puid != self.puid)
    }
}


pub struct HcRwLockReadGuard<'a, T: ?Sized> {
    puid: ProcessUniqueId,
    inner: RwLockReadGuard<'a, T>,
}

impl<'a, T: ?Sized> HcRwLockReadGuard<'a, T> {
    pub fn new(inner: RwLockReadGuard<'a, T>) -> Self {
        let puid = ProcessUniqueId::new();
        GUARDS.lock().expect("someone poisoned the GUARDS").push((puid, Instant::now(), Backtrace::new_unresolved()));
        Self { puid, inner }
    }
}

impl<'a, T: ?Sized> Drop for HcRwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        GUARDS.lock().expect("someone poisoned the GUARDS").retain(|(puid, _, _)| *puid != self.puid)
    }
}


pub struct HcRwLockWriteGuard<'a, T: ?Sized> {
    puid: ProcessUniqueId,
    inner: RwLockWriteGuard<'a, T>,
}

impl<'a, T: ?Sized> HcRwLockWriteGuard<'a, T> {
    pub fn new(inner: RwLockWriteGuard<'a, T>) -> Self {
        let puid = ProcessUniqueId::new();
        GUARDS.lock().expect("someone poisoned the GUARDS").push((puid, Instant::now(), Backtrace::new_unresolved()));
        Self { puid, inner }
    }
}

impl<'a, T: ?Sized> Drop for HcRwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        GUARDS.lock().expect("someone poisoned the GUARDS").retain(|(puid, _, _)| *puid != self.puid)
    }
}


// TODO: impl as appropriate
// AsRef<InnerType>
// Borrow<InnerType>
// Deref<Target=InnerType>
// AsMut<InnerType>
// BorrowMut<InnerType>
// DerefMut<Target=InnerType>

impl<'a, T: ?Sized> Deref for HcMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

impl<'a, T: ?Sized> DerefMut for HcMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.deref_mut()
    }
}

impl<'a, T: ?Sized> Deref for HcRwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

impl<'a, T: ?Sized> Deref for HcRwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

impl<'a, T: ?Sized> DerefMut for HcRwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.deref_mut()
    }
}

///////////////////////////////////////////////////////////////
/// MUTEX

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
    pub fn lock(&self) -> HcLockResult<HcMutexGuard<T>> {
        // let bts = update_backtraces(&self.backtraces);
        let deadline = Instant::now() + Duration::from_secs(LOCK_TIMEOUT_SECS);
        self.try_lock_until(deadline)
    }

    fn try_lock_until(&self, deadline: Instant) -> HcLockResult<HcMutexGuard<T>> {
        match self.try_lock() {
            Ok(v) => Ok(v),
            Err(err) => match err.kind {
                HcLockErrorKind::HcLockPoisonError => Err(err),
                HcLockErrorKind::HcLockTimeout => {
                    if let None = deadline.checked_duration_since(Instant::now()) {
                        Err(err)
                    } else {
                        std::thread::sleep(Duration::from_millis(LOCK_POLL_INTERVAL_MS));
                        self.try_lock_until(deadline)
                    }    
                }
            }
        }
    }

    pub fn try_lock(&self) -> HcLockResult<HcMutexGuard<T>> {
        let bts = update_backtraces(&self.backtraces);
        (*self)
            .inner
            .try_lock()
            .map_err(|err| match err {
                TryLockError::Poisoned(_poison_error) => {
                    HcLockError::new(LockType::Lock, bts, HcLockErrorKind::HcLockPoisonError)
                }
                TryLockError::WouldBlock => {
                    HcLockError::new(LockType::Lock, bts, HcLockErrorKind::HcLockTimeout)
                }
            })
            .map(|inner| HcMutexGuard::new(inner))
    }
}

///////////////////////////////////////////////////////////////
/// RwLock

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
    pub fn read(&self) -> HcLockResult<HcRwLockReadGuard<T>> {
        // let bts = update_backtraces(&self.backtraces);
        let deadline = Instant::now() + Duration::from_secs(LOCK_TIMEOUT_SECS);
        self.try_read_until(deadline)
    }

    fn try_read_until(&self, deadline: Instant) -> HcLockResult<HcRwLockReadGuard<T>> {
        match self.try_read() {
            Ok(v) => Ok(v),
            Err(err) => match err.kind {
                HcLockErrorKind::HcLockPoisonError => Err(err),
                HcLockErrorKind::HcLockTimeout => {
                    if let None = deadline.checked_duration_since(Instant::now()) {
                        Err(err)
                    } else {
                        std::thread::sleep(Duration::from_millis(LOCK_POLL_INTERVAL_MS));
                        self.try_read_until(deadline)
                    }    
                }
            }
        }
    }

    pub fn try_read(&self) -> HcLockResult<HcRwLockReadGuard<T>> {
        let bts = update_backtraces(&self.backtraces);
        (*self)
            .inner
            .try_read()
            .map_err(|err| match err {
                TryLockError::Poisoned(_poison_error) => {
                    (HcLockError::new(LockType::Read, bts, HcLockErrorKind::HcLockPoisonError))
                }
                TryLockError::WouldBlock => {
                    (HcLockError::new(LockType::Read, bts, HcLockErrorKind::HcLockTimeout))
                }
            })
            .map(|inner| HcRwLockReadGuard::new(inner))
    }
}

impl<T: ?Sized> HcRwLock<T> {
    pub fn write(&self) -> HcLockResult<HcRwLockWriteGuard<T>> {
        // let bts = update_backtraces(&self.backtraces);
        let deadline = Instant::now() + Duration::from_secs(LOCK_TIMEOUT_SECS);
        self.try_write_until(deadline)
    }


    fn try_write_until(&self, deadline: Instant) -> HcLockResult<HcRwLockWriteGuard<T>> {
        match self.try_write() {
            Ok(v) => Ok(v),
            Err(err) => match err.kind {
                HcLockErrorKind::HcLockPoisonError => Err(err),
                HcLockErrorKind::HcLockTimeout => {
                    if let None = deadline.checked_duration_since(Instant::now()) {
                        Err(err)
                    } else {
                        std::thread::sleep(Duration::from_millis(LOCK_POLL_INTERVAL_MS));
                        self.try_write_until(deadline)
                    }    
                }
            }
        }
    }

    pub fn try_write(&self) -> HcLockResult<HcRwLockWriteGuard<T>> {
        let bts = update_backtraces(&self.backtraces);
        (*self)
            .inner
            .try_write()
            .map_err(|err| match err {
                TryLockError::Poisoned(_poison_error) => {
                    (HcLockError::new(LockType::Write, bts, HcLockErrorKind::HcLockPoisonError))
                }
                TryLockError::WouldBlock => {
                    (HcLockError::new(LockType::Write, bts, HcLockErrorKind::HcLockTimeout))
                }
            })
            .map(|inner| HcRwLockWriteGuard::new(inner))
    }
}

///////////////////////////////////////////////////////////////
/// HELPERS

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
