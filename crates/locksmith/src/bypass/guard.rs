use parking_lot::{MutexGuard, RwLockReadGuard, RwLockWriteGuard};
use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

macro_rules! guard_struct {
    ($HcGuard:ident, $Guard:ident) => {
        pub struct $HcGuard<'a, T: ?Sized> {
            inner: $Guard<'a, T>,
        }

        impl<'a, T: ?Sized> $HcGuard<'a, T> {
            pub fn new(inner: $Guard<'a, T>) -> Self {
                Self {
                    inner,
                }
            }

            /// Add some context which will output in the case that this guard
            /// lives to be an immortal
            pub fn annotate<S: Into<String>>(self, _annotation: S) -> Self {
                self
            }

            /// Declare that this mutex should be unlocked fairly when it is
            /// dropped, if it hasn't already been unlocked some other way
            pub fn use_fair_unlocking(self) -> Self {
                self
            }

            /// Explicitly consume and unlock this mutex fairly, regardless
            /// of what kind of unlocking was specified at initialization
            pub fn unlock_fair(self) {

            }
        }
    };
}

guard_struct!(HcMutexGuard, MutexGuard);
guard_struct!(HcRwLockReadGuard, RwLockReadGuard);
guard_struct!(HcRwLockWriteGuard, RwLockWriteGuard);

// HcMutexGuard

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
        &self.inner
    }
}

impl<'a, T: ?Sized> DerefMut for HcMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

// HcRwLockReadGuard

impl<'a, T: ?Sized> Borrow<T> for HcRwLockReadGuard<'a, T> {
    fn borrow(&self) -> &T {
        &self.inner
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
        &self.inner
    }
}

// HcRwLockWriteGuard

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
        &self.inner
    }
}

impl<'a, T: ?Sized> DerefMut for HcRwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
