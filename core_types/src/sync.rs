use std::collections::HashMap;
use backtrace::Backtrace;
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use snowflake::ProcessUniqueId;
use std::{
    ops::{Deref, DerefMut},
    sync::TryLockError,
    thread,
    time::{Duration, Instant},
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
    static ref GUARDS: Mutex<Vec<(ProcessUniqueId, Instant, Backtrace)>> = Mutex::new(Vec::new());
    static ref PENDING_LOCKS: Mutex<HashMap<ProcessUniqueId, (LockType, Instant, Backtrace)>> =
        Mutex::new(HashMap::new());
}

pub fn spawn_hc_guard_watcher() {
    let _ = thread::spawn(move || loop {
        {
            let mut guards = GUARDS.lock();
            *guards = guards
                .iter()
                .filter(|(puid, instant, backtrace)| {
                    let timeout =
                        Instant::now().duration_since(*instant).as_secs() > LOCK_TIMEOUT_SECS;
                    if timeout {
                        let mut b = backtrace.clone();
                        b.resolve();
                        debug!("IMMORTAL LOCK GUARD!!! puid={:?} backtrace:\n{:?}", puid, b);
                        print_pending_locks();
                    }
                    !timeout
                })
                .cloned()
                .collect();
            debug!("spawn_hc_guard_watcher: num={:?}", guards.len());
            for (puid, instant, _) in guards.iter() {
                debug!("{:?} {:?}", puid, instant);
            }
        }
        thread::sleep(Duration::from_millis(3000));
    });
    debug!("spawn_hc_guard_watcher: SPAWNED");
}

fn print_pending_locks() {
    for (puid, (lock_type, instant, backtrace)) in PENDING_LOCKS.lock().iter() {
        debug!(
            "PENDING LOCK {:?} locktype={:?}, pending for {:?}, backtrace:\n{:?}",
            puid,
            lock_type,
            Instant::now().duration_since(*instant),
            backtrace
        );
    }
}

// /////////////////////////////////////////////////////////////
// GUARDS

macro_rules! guard_struct {
    ($HcGuard:ident, $Guard:ident) => {
        pub struct $HcGuard<'a, T: ?Sized> {
            puid: ProcessUniqueId,
            inner: $Guard<'a, T>,
        }

        impl<'a, T: ?Sized> $HcGuard<'a, T> {
            pub fn new(inner: $Guard<'a, T>) -> Self {
                let puid = ProcessUniqueId::new();
                GUARDS
                    .lock()
                    .push((puid, Instant::now(), Backtrace::new_unresolved()));
                Self { puid, inner }
            }
        }

        impl<'a, T: ?Sized> Drop for $HcGuard<'a, T> {
            fn drop(&mut self) {
                GUARDS.lock().retain(|(puid, _, _)| *puid != self.puid)
            }
        }
    };
}

guard_struct!(HcMutexGuard, MutexGuard);
guard_struct!(HcRwLockReadGuard, RwLockReadGuard);
guard_struct!(HcRwLockWriteGuard, RwLockWriteGuard);

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

// /////////////////////////////////////////////////////////////
// MUTEXES

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

macro_rules! mutex_impl {
    ($HcMutex: ident, $Mutex: ident, $Guard:ident, $lock_type:ident, $lock_fn:ident, $try_lock_fn:ident, $try_lock_until_fn:ident, $try_lock_until_inner_fn:ident ) => {
        impl<T: ?Sized> $HcMutex<T> {
            pub fn $lock_fn(&self) -> HcLockResult<$Guard<T>> {
                // let bts = update_backtraces(&self.backtraces);
                let deadline = Instant::now() + Duration::from_secs(LOCK_TIMEOUT_SECS);
                self.$try_lock_until_fn(deadline)
            }

            fn $try_lock_until_fn(&self, deadline: Instant) -> HcLockResult<$Guard<T>> {
                self.$try_lock_until_inner_fn(deadline, None)
            }

            fn $try_lock_until_inner_fn(
                &self,
                deadline: Instant,
                puid: Option<ProcessUniqueId>,
            ) -> HcLockResult<$Guard<T>> {
                match self.$try_lock_fn() {
                    Ok(v) => {
                        if let Some(puid) = puid {
                            PENDING_LOCKS.lock().remove(&puid);
                        }
                        Ok(v)
                    }
                    Err(err) => match err.kind {
                        HcLockErrorKind::HcLockPoisonError => Err(err),
                        HcLockErrorKind::HcLockTimeout => {
                            let puid = puid.unwrap_or_else(|| {
                                let p = ProcessUniqueId::new();
                                PENDING_LOCKS.lock().insert(p, (LockType::Lock, Instant::now(), Backtrace::new_unresolved()));
                                p
                            });
                            if let None = deadline.checked_duration_since(Instant::now()) {
                                // PENDING_LOCKS.lock().remove(&puid);
                                Err(err)
                            } else {
                                std::thread::sleep(Duration::from_millis(LOCK_POLL_INTERVAL_MS));
                                self.$try_lock_until_inner_fn(deadline, Some(puid))
                            }
                        }
                    },
                }
            }

            pub fn $try_lock_fn(&self) -> HcLockResult<$Guard<T>> {
                let bts = update_backtraces(&self.backtraces);
                (*self)
                    .inner
                    .$try_lock_fn()
                    .map(|inner| $Guard::new(inner))
                    .ok_or_else(|| {
                        HcLockError::new(LockType::$lock_type, bts, HcLockErrorKind::HcLockTimeout)
                    })
            }
        }
    };
}

mutex_impl!(
    HcMutex,
    Mutex,
    HcMutexGuard,
    Lock,
    lock,
    try_lock,
    try_lock_until,
    try_lock_until_inner
);
mutex_impl!(
    HcRwLock,
    RwLock,
    HcRwLockReadGuard,
    Read,
    read,
    try_read,
    try_read_until,
    try_read_until_inner
);
mutex_impl!(
    HcRwLock,
    RwLock,
    HcRwLockWriteGuard,
    Write,
    write,
    try_write,
    try_write_until,
    try_write_until_inner
);

///////////////////////////////////////////////////////////////
/// HELPERS

fn try_lock_ok<T, P>(result: Result<T, TryLockError<P>>) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(TryLockError::WouldBlock) => None,
        Err(TryLockError::Poisoned(err)) => {
            debug!("try_lock_ok found poisoned lock! {:?}", err);
            None
        }
    }
}

fn update_backtraces(mutex: &Mutex<Vec<Backtrace>>) -> Option<Vec<Backtrace>> {
    if let Some(mut bts) = try_lock_ok::<_, ()>(mutex.try_lock().ok_or(TryLockError::WouldBlock)) {
        bts.push(Backtrace::new_unresolved());
        Some(bts.clone())
    } else {
        None
    }
}
