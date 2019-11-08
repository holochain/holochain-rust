use crate::{
    error::{LockType, LocksmithResult},
    guard::{HcRwLockReadGuard, HcRwLockWriteGuard},
    mutex::HcRwLock,
};
use parking_lot::Mutex;

#[derive(Debug)]
pub struct RwLockRigged<T> {
    gate: Mutex<()>,
    rw: HcRwLock<T>,
}

impl<T> RwLockRigged<T> {
    pub fn new(v: T) -> Self {
        Self {
            gate: Mutex::new(()),
            rw: HcRwLock::new(v),
        }
    }

    pub fn read(&self) -> LocksmithResult<HcRwLockReadGuard<T>> {
        let _ = self.gate.lock();
        self.rw.read().map_err(|mut e| {
            e.lock_type = LockType::ReadRigged;
            e
        })
    }

    pub fn try_read(&self) -> Option<HcRwLockReadGuard<T>> {
        self.gate.try_lock().and_then(|_| self.rw.try_read())
    }

    pub fn write(&self) -> LocksmithResult<HcRwLockWriteGuard<T>> {
        let _gate = self.gate.lock();
        self.rw.write().map_err(|mut e| {
            e.lock_type = LockType::WriteRigged;
            e
        })
    }

    pub fn try_write(&self) -> Option<HcRwLockWriteGuard<T>> {
        self.gate.try_lock().and_then(|_gate| self.rw.try_write())
    }
}
