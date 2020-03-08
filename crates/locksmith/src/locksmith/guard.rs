use crate::{
    error::LockType,
    locksmith::{common::guards_guard, tracker::GuardTracker},
};
use parking_lot::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};
use snowflake::ProcessUniqueId;
use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

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
                guards_guard().insert(puid, GuardTracker::new(puid, LockType::$lock_type));
                Self {
                    puid,
                    inner: Some(inner),
                    fair_unlocking: false,
                }
            }

            /// Add some context which will output in the case that this guard
            /// lives to be an immortal
            pub fn annotate<S: Into<String>>(self, annotation: S) -> Self {
                guards_guard()
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
                if let Some(inner) = self.inner.take() {
                    $Guard::unlock_fair(inner);
                }
            }
        }

        impl<'a, T: ?Sized> Drop for $HcGuard<'a, T> {
            fn drop(&mut self) {
                guards_guard().remove(&self.puid);
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
