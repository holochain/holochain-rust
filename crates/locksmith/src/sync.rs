use crate::error::{LockType, LocksmithError, LocksmithErrorKind, LocksmithResult};
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

lazy_static! {

    /// if a lock guard lives this long, it is assumed it will never die
    static ref IMMORTAL_TIMEOUT: Duration = Duration::from_secs(60);

    /// this should be a bit longer than IMMORTAL_TIMEOUT, so that locks don't timeout
    /// before all long-running guards are detected, in the case of a deadlock.
    /// (But NOT longer than try-o-rama's conductor timeout)
    static ref LOCK_TIMEOUT: Duration = Duration::from_millis(100);

    /// This is how often we check the elapsed time of guards
    static ref GUARD_WATCHER_POLL_INTERVAL: Duration = Duration::from_millis(1000);

    /// We filter out any guards alive less than this long
    static ref ACTIVE_GUARD_MIN_ELAPSED: Duration = Duration::from_millis(1000);

    /// Only report about no activity if this much time has passed
    static ref ACTIVE_GUARD_NO_ACTIVITY_INTERVAL: Duration = Duration::from_secs(10);

    static ref GUARDS: Mutex<HashMap<ProcessUniqueId, GuardTracker>> = Mutex::new(HashMap::new());
}

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
        if elapsed > *ACTIVE_GUARD_MIN_ELAPSED {
            let elapsed_ms = elapsed.as_millis() as i64;
            if !self.immortal && elapsed > *IMMORTAL_TIMEOUT {
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
            time=IMMORTAL_TIMEOUT.as_secs(),
            annotation=annotation,
            backtrace=self.backtrace
        );
    }
}

pub fn spawn_locksmith_guard_watcher() {
    debug!("spawning locksmith_guard_watcher");
    let _ = thread::Builder::new()
        .name(format!(
            "locksmith_guard_watcher/{}",
            ProcessUniqueId::new().to_string()
        ))
        .spawn(move || {
            let mut inactive_for = Duration::from_millis(0);
            loop {
                let mut reports: Vec<(i64, String)> = {
                    GUARDS
                        .lock()
                        .values_mut()
                        .filter_map(|gt| gt.report_and_update())
                        .collect()
                };
                if reports.len() > 0 {
                    inactive_for = Duration::from_millis(0);
                    reports.sort_unstable_by_key(|(elapsed, _)| -*elapsed);
                    let num_active = reports.len();
                    let lines: Vec<String> =
                        reports.into_iter().map(|(_, report)| report).collect();
                    let output = lines.join("\n");
                    debug!(
                        "tracking {} active guard(s) alive for > {}ms:\n{}\n{}",
                        num_active,
                        ACTIVE_GUARD_MIN_ELAPSED.as_millis(),
                        GuardTracker::report_header(),
                        output
                    );
                } else {
                    inactive_for += *GUARD_WATCHER_POLL_INTERVAL;
                    if inactive_for > *ACTIVE_GUARD_NO_ACTIVITY_INTERVAL {
                        debug!(
                            "no active guards alive > {:?}ms for the last {:?} seconds",
                            ACTIVE_GUARD_MIN_ELAPSED.as_millis(),
                            ACTIVE_GUARD_NO_ACTIVITY_INTERVAL.as_secs(),
                        );
                        inactive_for = Duration::from_millis(0);
                    }
                }

                thread::sleep(*GUARD_WATCHER_POLL_INTERVAL);
            }
        });
}

// /////////////////////////////////////////////////////////////
// GUARDS

macro_rules! guard_struct {
    ($HcGuard:ident, $Guard:ident, $lock_type:ident) => {
        pub struct $HcGuard<'a, T: ?Sized> {
            puid: ProcessUniqueId,
            inner: Option<$Guard<'a, T>>,
            fair_unlocking: bool,
        }

        impl<'a, T: ?Sized> $HcGuard<'a, T> {
            pub fn new(inner: $Guard<'a, T>) -> Self {
                let puid = ProcessUniqueId::new();
                GUARDS
                    .lock()
                    .insert(puid, GuardTracker::new(puid, LockType::$lock_type));
                Self {
                    puid,
                    inner: Some(inner),
                    fair_unlocking: false,
                }
            }

            /// Add some context which will output in the case that this guard
            /// lives to be an immortal
            pub fn annotate<S: Into<String>>(self, annotation: S) -> Self {
                GUARDS
                    .lock()
                    .entry(self.puid)
                    .and_modify(|g| g.annotation = Some(annotation.into()));
                self
            }

            /// Declare that this mutex should be unlocked fairly when it is
            /// dropped, if it hasn't already been unlocked some other way
            pub fn use_fair_unlocking(mut self) -> Self {
                self.fair_unlocking = true;
                self
            }

            /// Explicitly consume and unlock this mutex fairly, regardless
            /// of what kind of unlocking was specified at initialization
            pub fn unlock_fair(mut self) {
                self._unlock_fair();
            }

            fn _unlock_fair(&mut self) {
                if let Some(inner) = std::mem::replace(&mut self.inner, None) {
                    $Guard::unlock_fair(inner);
                }
            }
        }

        impl<'a, T: ?Sized> Drop for $HcGuard<'a, T> {
            fn drop(&mut self) {
                GUARDS.lock().remove(&self.puid);
                if self.fair_unlocking {
                    self._unlock_fair();
                }
            }
        }
    };
}

guard_struct!(HcMutexGuard, MutexGuard, Lock);
guard_struct!(HcRwLockReadGuard, RwLockReadGuard, Read);
guard_struct!(HcRwLockWriteGuard, RwLockWriteGuard, Write);

// HcMutexGuard

impl<'a, T: ?Sized> Borrow<T> for HcMutexGuard<'a, T> {
    fn borrow(&self) -> &T {
        self.inner.as_ref().expect("accessed mutex mid-unlock!")
    }
}

impl<'a, T: ?Sized> BorrowMut<T> for HcMutexGuard<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.inner.as_mut().expect("accessed mutex mid-unlock!")
    }
}

// impl<'a, T: ?Sized> AsRef<T> for HcMutexGuard<'a, T> {
//     fn as_ref(&self) -> &T {
//         self.deref()
//     }
// }

// impl<'a, T: ?Sized> AsMut<T> for HcMutexGuard<'a, T> {
//     fn as_mut(&mut self) -> &mut T {
//         self.deref_mut()
//     }
// }

impl<'a, T: ?Sized> Deref for HcMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.inner.as_ref().expect("accessed mutex mid-unlock!")
    }
}

impl<'a, T: ?Sized> DerefMut for HcMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.inner.as_mut().expect("accessed mutex mid-unlock!")
    }
}

// HcRwLockReadGuard

impl<'a, T: ?Sized> Borrow<T> for HcRwLockReadGuard<'a, T> {
    fn borrow(&self) -> &T {
        self.inner.as_ref().expect("accessed mutex mid-unlock!")
    }
}

// impl<'a, T: ?Sized> AsRef<T> for HcRwLockReadGuard<'a, T> {
//     fn as_ref(&self) -> &T {
//         self.deref()
//     }
// }

impl<'a, T: ?Sized> Deref for HcRwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.inner.as_ref().expect("accessed mutex mid-unlock!")
    }
}

// HcRwLockWriteGuard

impl<'a, T: ?Sized> Borrow<T> for HcRwLockWriteGuard<'a, T> {
    fn borrow(&self) -> &T {
        self.inner.as_ref().expect("accessed mutex mid-unlock!")
    }
}

impl<'a, T: ?Sized> BorrowMut<T> for HcRwLockWriteGuard<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.inner.as_mut().expect("accessed mutex mid-unlock!")
    }
}

// impl<'a, T: ?Sized> AsRef<T> for HcRwLockWriteGuard<'a, T> {
//     fn as_ref(&self) -> &T {
//         self.deref()
//     }
// }

// impl<'a, T: ?Sized> AsMut<T> for HcRwLockWriteGuard<'a, T> {
//     fn as_mut(&mut self) -> &mut T {
//         self.deref_mut()
//     }
// }

impl<'a, T: ?Sized> Deref for HcRwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.inner.as_ref().expect("accessed mutex mid-unlock!")
    }
}

impl<'a, T: ?Sized> DerefMut for HcRwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.inner.as_mut().expect("accessed mutex mid-unlock!")
    }
}

// /////////////////////////////////////////////////////////////
// MUTEXES

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
