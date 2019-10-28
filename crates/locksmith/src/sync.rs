use backtrace::Backtrace;
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use snowflake::ProcessUniqueId;
use std::{
    borrow::{Borrow, BorrowMut},
    collections::HashMap,
    ops::{Deref, DerefMut},
    thread,
    time::{Duration, Instant},
};

// if a lock guard lives this long, it is assumed it will never die
const IMMORTAL_TIMEOUT_SECS: u64 = 60;

// this should be a bit longer than IMMORTAL_TIMEOUT, so that locks don't timeout
// before all long-running guards are detected, in the case of a deadlock.
// (But NOT longer than try-o-rama's conductor timeout)
const LOCK_TIMEOUT_SECS: u64 = 100;

// This is how often we check the elapsed time of guards
const GUARD_WATCHER_POLL_INTERVAL_MS: u64 = 1000;

// We filter out any guards alive less than this long
const ACTIVE_GUARD_MIN_ELAPSED_MS: i64 = 500;

// How often to retry getting a lock after receiving a WouldBlock error
// during try_lock
const LOCK_POLL_INTERVAL_MS: u64 = 10;

#[derive(Debug)]
pub enum HcLockErrorKind {
    HcLockTimeout,
    HcLockPoisonError,
    HcLockWouldBlock,
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
    pub fn new(lock_type: LockType, kind: HcLockErrorKind) -> Self {
        Self {
            lock_type,
            backtraces: None,
            kind,
        }
    }
}

pub type HcLockResult<T> = Result<T, HcLockError>;

struct GuardTracker {
    puid: ProcessUniqueId,
    created: Instant,
    backtrace: Backtrace,
    lock_type: LockType,
    immortal: bool,
    annotation: Option<String>,
}

impl GuardTracker {
    pub fn new(puid: ProcessUniqueId, lock_type: LockType) -> Self {
        Self {
            puid,
            lock_type,
            created: Instant::now(),
            backtrace: Backtrace::new_unresolved(),
            immortal: false,
            annotation: None,
        }
    }

    pub fn report_and_update(&mut self) -> Option<(i64, String)> {
        let elapsed = Instant::now().duration_since(self.created);
        let elapsed_ms = elapsed.as_millis() as i64;
        if elapsed_ms > ACTIVE_GUARD_MIN_ELAPSED_MS {
            if !self.immortal && elapsed.as_secs() > IMMORTAL_TIMEOUT_SECS {
                self.immortalize();
            }
            let lock_type_str = format!("{:?}", self.lock_type);
            let report = if self.immortal {
                format!(
                    "{:<6} {:<13} {:>12} [!!!]",
                    lock_type_str, self.puid, elapsed_ms
                )
            } else {
                format!("{:<6} {:<13} {:>12}", lock_type_str, self.puid, elapsed_ms)
            };
            Some((elapsed_ms, report))
        } else {
            None
        }
    }

    pub fn report_header() -> String {
        format!("{:6} {:^13} {:>12}", "KIND", "PUID", "ELAPSED (ms)")
    }

    fn immortalize(&mut self) {
        if self.immortal {
            return;
        }
        self.immortal = true;
        self.backtrace.resolve();
        let annotation = self
            .annotation
            .as_ref()
            .map(|a| format!("\nAnnotation: {}\n", a))
            .unwrap_or_default();
        error!(
            r"

        !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
        !!! IMMORTAL LOCK GUARD FOUND !!!
        !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

{type:?} guard {puid} lived for > {time} seconds.{annotation}
Backtrace at the moment of guard creation follows:

{backtrace:?}",
            type=self.lock_type,
            puid=self.puid,
            time=IMMORTAL_TIMEOUT_SECS,
            annotation=annotation,
            backtrace=self.backtrace
        );
    }
}

lazy_static! {
    static ref GUARDS: Mutex<HashMap<ProcessUniqueId, GuardTracker>> = Mutex::new(HashMap::new());
    static ref PENDING_LOCKS: Mutex<HashMap<ProcessUniqueId, (LockType, Instant, Backtrace)>> =
        Mutex::new(HashMap::new());
}

pub fn spawn_hc_guard_watcher() {
    let _ = thread::Builder::new()
        .name(format!(
            "hc_guard_watcher/{}",
            ProcessUniqueId::new().to_string()
        ))
        .spawn(move || loop {
            let mut reports: Vec<(i64, String)> = {
                GUARDS
                    .lock()
                    .values_mut()
                    .filter_map(|gt| gt.report_and_update())
                    .collect()
            };
            if reports.len() > 0 {
                reports.sort_unstable_by_key(|(elapsed, _)| -*elapsed);
                let num_active = reports.len();
                let lines: Vec<String> = reports.into_iter().map(|(_, report)| report).collect();
                let output = lines.join("\n");
                debug!(
                    "tracking {} active guard(s) alive for > {}ms:\n{}\n{}",
                    num_active,
                    ACTIVE_GUARD_MIN_ELAPSED_MS,
                    GuardTracker::report_header(),
                    output
                );
            } else {
                debug!(
                    "no active guards alive for > {}ms",
                    ACTIVE_GUARD_MIN_ELAPSED_MS
                );
            }

            thread::sleep(Duration::from_millis(GUARD_WATCHER_POLL_INTERVAL_MS));
        });
    debug!("spawn_hc_guard_watcher: SPAWNED");
}

fn _print_pending_locks() {
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
    ($HcGuard:ident, $Guard:ident, $lock_type:ident) => {
        pub struct $HcGuard<'a, T: ?Sized> {
            puid: ProcessUniqueId,
            pub inner: $Guard<'a, T>,
        }

        impl<'a, T: ?Sized> $HcGuard<'a, T> {
            pub fn new(inner: $Guard<'a, T>) -> Self {
                let puid = ProcessUniqueId::new();
                GUARDS
                    .lock()
                    .insert(puid, GuardTracker::new(puid, LockType::$lock_type));
                Self { puid, inner }
            }

            pub fn annotate<S: Into<String>>(self, annotation: S) -> Self {
                GUARDS
                    .lock()
                    .entry(self.puid)
                    .and_modify(|g| g.annotation = Some(annotation.into()));
                self
            }
        }

        impl<'a, T: ?Sized> Drop for $HcGuard<'a, T> {
            fn drop(&mut self) {
                GUARDS.lock().remove(&self.puid);
            }
        }
    };
}

guard_struct!(HcMutexGuard, MutexGuard, Lock);
guard_struct!(HcRwLockReadGuard, RwLockReadGuard, Read);
guard_struct!(HcRwLockWriteGuard, RwLockWriteGuard, Write);

impl<'a, T: ?Sized> Borrow<T> for HcMutexGuard<'a, T> {
    fn borrow(&self) -> &T {
        &self.inner
    }
}

impl<'a, T: ?Sized> BorrowMut<T> for HcMutexGuard<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

// impl<'a, T: ?Sized> AsRef<T> for HcMutexGuard<'a, T> {
//     fn as_ref(&self) -> &T {
//         &self.inner
//     }
// }

// impl<'a, T: ?Sized> AsMut<T> for HcMutexGuard<'a, T> {
//     fn as_mut(&mut self) -> &mut T {
//         &mut self.inner
//     }
// }

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

//

impl<'a, T: ?Sized> Borrow<T> for HcRwLockReadGuard<'a, T> {
    fn borrow(&self) -> &T {
        &self.inner
    }
}

// impl<'a, T: ?Sized> AsRef<T> for HcRwLockReadGuard<'a, T> {
//     fn as_ref(&self) -> &T {
//         &self.inner
//     }
// }

impl<'a, T: ?Sized> Deref for HcRwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

//

impl<'a, T: ?Sized> Borrow<T> for HcRwLockWriteGuard<'a, T> {
    fn borrow(&self) -> &T {
        &self.inner
    }
}

impl<'a, T: ?Sized> BorrowMut<T> for HcRwLockWriteGuard<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

// impl<'a, T: ?Sized> AsRef<T> for HcRwLockWriteGuard<'a, T> {
//     fn as_ref(&self) -> &T {
//         &self.inner
//     }
// }

// impl<'a, T: ?Sized> AsMut<T> for HcRwLockWriteGuard<'a, T> {
//     fn as_mut(&mut self) -> &mut T {
//         &mut self.inner
//     }
// }

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
    inner: Mutex<T>,
}

impl<T> HcMutex<T> {
    pub fn new(v: T) -> Self {
        Self {
            inner: Mutex::new(v),
        }
    }
}

#[derive(Debug)]
pub struct HcRwLock<T: ?Sized> {
    inner: RwLock<T>,
}

impl<T> HcRwLock<T> {
    pub fn new(v: T) -> Self {
        Self {
            inner: RwLock::new(v),
        }
    }
}

macro_rules! mutex_impl {
    ($HcMutex: ident, $Mutex: ident, $Guard:ident, $lock_type:ident, $lock_fn:ident, $try_lock_fn:ident, $try_lock_until_fn:ident) => {
        impl<T: ?Sized> $HcMutex<T> {
            pub fn $lock_fn(&self) -> HcLockResult<$Guard<T>> {
                let deadline = Instant::now() + Duration::from_secs(LOCK_TIMEOUT_SECS);
                self.$try_lock_until_fn(deadline)
            }

            fn $try_lock_until_fn(&self, deadline: Instant) -> HcLockResult<$Guard<T>> {
                // Set a number twice the expected number of iterations, just to prevent an infinite loop
                let max_iters = 2 * LOCK_TIMEOUT_SECS * 1000 / LOCK_POLL_INTERVAL_MS;
                let mut pending_puid = None;
                for _i in 0..max_iters {
                    match self.$try_lock_fn() {
                        Some(v) => {
                            if let Some(puid) = pending_puid {
                                PENDING_LOCKS.lock().remove(&puid);
                            }
                            return Ok(v);
                        }
                        None => {
                            pending_puid.get_or_insert_with(|| {
                                let p = ProcessUniqueId::new();
                                PENDING_LOCKS.lock().insert(
                                    p,
                                    (
                                        LockType::$lock_type,
                                        Instant::now(),
                                        Backtrace::new_unresolved(),
                                    ),
                                );
                                p
                            });

                            // TIMEOUT
                            if let None = deadline.checked_duration_since(Instant::now()) {
                                // PENDING_LOCKS.lock().remove(&puid);
                                return Err(HcLockError::new(
                                    LockType::$lock_type,
                                    HcLockErrorKind::HcLockTimeout,
                                ));
                            }
                        }
                    }
                    std::thread::sleep(Duration::from_millis(LOCK_POLL_INTERVAL_MS));
                }
                error!(
                    "$try_lock_until_inner_fn exceeded max_iters, this should not have happened!"
                );
                return Err(HcLockError::new(
                    LockType::$lock_type,
                    HcLockErrorKind::HcLockTimeout,
                ));
            }
            pub fn $try_lock_fn(&self) -> Option<$Guard<T>> {
                (*self).inner.$try_lock_fn().map(|g| $Guard::new(g))
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
    try_lock_until
);
mutex_impl!(
    HcRwLock,
    RwLock,
    HcRwLockReadGuard,
    Read,
    read,
    try_read,
    try_read_until
);
mutex_impl!(
    HcRwLock,
    RwLock,
    HcRwLockWriteGuard,
    Write,
    write,
    try_write,
    try_write_until
);
