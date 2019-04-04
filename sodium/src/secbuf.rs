//! This module provides an abstraction for memory for use with libsodium

use libc::c_void;
use std::ops::{Deref, DerefMut};

use super::check_init;
use crate::error::SodiumError;

/// a trait for structures that can be used as a backing store for SecBuf
pub trait Bufferable: Send {
    fn new(s: usize) -> Box<Bufferable>
    where
        Self: Sized + Send;
    fn from_string(s: String) -> Box<Bufferable>
    where
        Self: Sized + Send;
    fn len(&self) -> usize;
    fn readable(&self);
    fn writable(&mut self);
    fn noaccess(&self);
    fn ref_(&self) -> &[u8];
    fn ref_mut(&mut self) -> &mut [u8];
}

/// this is an insecure (raw memory) buffer for use with things like public keys
#[derive(Debug)]
struct RustBuf {
    b: Box<[u8]>,
}

impl Bufferable for RustBuf {
    fn new(s: usize) -> Box<Bufferable> {
        let b = vec![0; s].into_boxed_slice();
        Box::new(RustBuf { b })
    }

    fn from_string(s: String) -> Box<Bufferable> {
        let b = s.into_bytes().into_boxed_slice();
        Box::new(RustBuf { b })
    }

    fn len(&self) -> usize {
        self.b.len()
    }

    fn readable(&self) {}

    fn writable(&mut self) {}

    fn noaccess(&self) {}

    fn ref_(&self) -> &[u8] {
        &self.b
    }

    fn ref_mut(&mut self) -> &mut [u8] {
        &mut self.b
    }
}

/// this is a secure buffer for use with things like private keys
struct SodiumBuf {
    z: *mut c_void,
    s: usize,
}

unsafe impl Send for SodiumBuf {}

impl Bufferable for SodiumBuf {
    /// warning: funky sizes may result in mis-alignment
    fn new(s: usize) -> Box<Bufferable> {
        if s % 8 != 0 {
            panic!("bad buffer size: {}, disallowing this for safety", s);
        }
        let z = unsafe {
            check_init();
            let z = rust_sodium_sys::sodium_malloc(s);
            if z.is_null() {
                panic!("cannot allocate");
            }
            rust_sodium_sys::sodium_mprotect_noaccess(z);
            z
        };
        Box::new(SodiumBuf { z, s })
    }

    fn from_string(s: String) -> Box<Bufferable> {
        let b = s.into_bytes().into_boxed_slice();
        Box::new(RustBuf { b })
    }

    fn len(&self) -> usize {
        self.s
    }

    fn readable(&self) {
        unsafe {
            rust_sodium_sys::sodium_mprotect_readonly(self.z);
        }
    }

    fn writable(&mut self) {
        unsafe {
            rust_sodium_sys::sodium_mprotect_readwrite(self.z);
        }
    }

    fn noaccess(&self) {
        unsafe {
            rust_sodium_sys::sodium_mprotect_noaccess(self.z);
        }
    }

    fn ref_(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.z as *const u8, self.s) }
    }

    fn ref_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.z as *mut u8, self.s) }
    }
}

impl Drop for SodiumBuf {
    fn drop(&mut self) {
        unsafe {
            rust_sodium_sys::sodium_free(self.z);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SecurityType {
    Insecure,
    Secure,
}

/// Represents the memory protection state of a SecBuf
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtectState {
    NoAccess,
    ReadOnly,
    ReadWrite,
}

/// A SecBuf is a memory buffer for use with libsodium functions.
/// It can be backed by insecure (raw) memory for things like public keys,
/// or secure (mlocked / mprotected) memory for things like private keys.
pub struct SecBuf {
    t: SecurityType,
    b: Box<Bufferable>,
    p: ProtectState,
}

impl Clone for SecBuf {
    fn clone(&self) -> Self {
        let mut out = match self.t {
            SecurityType::Insecure => SecBuf::with_insecure(self.len()),
            SecurityType::Secure => SecBuf::with_secure(self.len()),
        };
        self.b.readable();
        unsafe {
            let mut out_lock = out.write_lock();
            std::ptr::copy(
                self.b.ref_().as_ptr(),
                (**out_lock).as_mut_ptr(),
                self.len(),
            );
        }
        self.b.noaccess();
        out
    }
}

impl std::fmt::Debug for SecBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.b.ref_())
    }
}

impl SecBuf {
    /// create a new SecBuf backed by insecure memory (for things like public keys)
    pub fn with_insecure(s: usize) -> Self {
        SecBuf {
            t: SecurityType::Insecure,
            b: RustBuf::new(s),
            p: ProtectState::NoAccess,
        }
    }

    /// create a new SecBuf backed by secure memory (for things like private keys)
    /// warning: funky sizes may result in mis-alignment
    pub fn with_secure(s: usize) -> Self {
        SecBuf {
            t: SecurityType::Secure,
            b: SodiumBuf::new(s),
            p: ProtectState::NoAccess,
        }
    }

    pub fn with_insecure_from_string(s: String) -> Self {
        SecBuf {
            t: SecurityType::Insecure,
            b: RustBuf::from_string(s),
            p: ProtectState::NoAccess,
        }
    }

    /// what is the current memory protection state of this SecBuf?
    pub fn protect_state(&self) -> ProtectState {
        self.p.clone()
    }

    /// should be able to get size without messing with mem protection
    pub fn len(&self) -> usize {
        self.b.len()
    }

    /// make this SecBuf readable
    pub fn readable(&mut self) {
        if self.p == ProtectState::NoAccess {
            self.p = ProtectState::ReadOnly;
            self.b.readable();
        } else {
            panic!(
                "SecBuf trying to get Double Locked, Current state : {:?}",
                self.protect_state()
            );
        }
    }

    /// make this SecBuf writable
    pub fn writable(&mut self) {
        if self.p == ProtectState::NoAccess {
            self.p = ProtectState::ReadWrite;
            self.b.writable();
        } else {
            panic!(
                "SecBuf trying to get Double Locked, Current state : {:?}",
                self.protect_state()
            );
        }
    }

    /// secure this SecBuf against reading or writing
    pub fn noaccess(&mut self) {
        self.p = ProtectState::NoAccess;
        self.b.noaccess();
    }

    /// make this SecBuf readable, and return a locker object
    /// that will secure this SecBuf automatically when it goes out of scope.
    pub fn read_lock(&mut self) -> Locker {
        Locker::new(self, false)
    }

    /// make this SecBuf writable, and return a locker object
    /// that will secure this SecBuf automatically when it goes out of scope.
    pub fn write_lock(&mut self) -> Locker {
        Locker::new(self, true)
    }

    /// helper for writing data to our internal buffer
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), SodiumError> {
        if offset + data.len() > self.len() {
            return Err(SodiumError::new("SecBuf write overflow"));
        }
        unsafe {
            let mut b = self.write_lock();
            std::ptr::copy(data.as_ptr(), (**b).as_mut_ptr().add(offset), data.len());
        }
        Ok(())
    }
}

impl Deref for SecBuf {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        if self.p == ProtectState::NoAccess {
            panic!("SecBuf Deref, but state is NoAccess");
        }
        self.b.ref_()
    }
}

impl DerefMut for SecBuf {
    fn deref_mut(&mut self) -> &mut [u8] {
        if self.p != ProtectState::ReadWrite {
            panic!("SecBuf DerefMut, but state is not ReadWrite");
        }
        self.b.ref_mut()
    }
}

/// a helper object that will automatically secure a SecBuf when dropped
pub struct Locker<'a>(&'a mut SecBuf);

impl<'a> Locker<'a> {
    pub fn new(b: &'a mut SecBuf, writable: bool) -> Self {
        if writable {
            b.writable();
        } else {
            b.readable();
        }
        Locker(b)
    }
}

impl<'a> Drop for Locker<'a> {
    fn drop(&mut self) {
        self.0.noaccess();
    }
}

impl<'a> std::fmt::Debug for Locker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.b.ref_())
    }
}

impl<'a> Deref for Locker<'a> {
    type Target = SecBuf;

    fn deref(&self) -> &SecBuf {
        self.0
    }
}

impl<'a> DerefMut for Locker<'a> {
    fn deref_mut(&mut self) -> &mut SecBuf {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_secbuf_from_string() {
        let b = SecBuf::with_insecure_from_string("zooooo".to_string());
        assert_eq!(ProtectState::NoAccess, b.protect_state());
    }

    #[test]
    fn it_should_read_write_insecure() {
        let mut b = SecBuf::with_insecure(16);
        assert_eq!(ProtectState::NoAccess, b.protect_state());

        {
            let mut b = b.write_lock();
            assert_eq!(ProtectState::ReadWrite, b.protect_state());
            b[0] = 12;
        }

        {
            let b = b.read_lock();
            assert_eq!(ProtectState::ReadOnly, b.protect_state());
            assert_eq!(b[0], 12);
        }
    }

    #[test]
    fn it_should_read_write_secure() {
        let mut b = SecBuf::with_secure(16);
        assert_eq!(ProtectState::NoAccess, b.protect_state());

        {
            let mut b = b.write_lock();
            assert_eq!(ProtectState::ReadWrite, b.protect_state());
            b[0] = 12;
        }

        {
            let b = b.read_lock();
            assert_eq!(ProtectState::ReadOnly, b.protect_state());
            assert_eq!(b[0], 12);
        }
    }

    #[test]
    fn it_should_clone_insecure() {
        let mut b = SecBuf::with_insecure(16);
        b.write(0, &[1, 2, 3]).unwrap();
        let mut c = b.clone();
        {
            let c = c.read_lock();
            assert_eq!(&c[0..3], &[1, 2, 3]);
        }
    }

    #[test]
    fn it_should_clone_secure() {
        let mut b = SecBuf::with_secure(16);
        b.write(0, &[1, 2, 3]).unwrap();
        let mut c = b.clone();
        {
            let c = c.read_lock();
            assert_eq!(&c[0..3], &[1, 2, 3]);
        }
    }

    #[test]
    #[should_panic]
    fn it_should_disallow_bad_align() {
        SecBuf::with_secure(1);
    }

    #[test]
    fn it_should_debug() {
        let mut b = SecBuf::with_insecure(2);
        {
            let mut b = b.write_lock();
            b[0] = 42;
            b[1] = 222;
        }
        {
            let b = b.read_lock();
            assert_eq!("[42, 222]", format!("{:?}", *b));
        }
    }

    #[test]
    #[should_panic]
    fn it_should_panic_on_not_readable() {
        let b = SecBuf::with_insecure(1);
        assert_eq!(22, b[0]);
    }

    #[test]
    #[should_panic]
    fn it_should_panic_on_not_writeable() {
        let mut b = SecBuf::with_insecure(1);
        b[0] = 22;
    }

    #[test]
    fn it_should_write() {
        let mut b = SecBuf::with_insecure(4);
        b.write(1, &[42, 42]).unwrap();
        {
            let b = b.read_lock();
            assert_eq!("[0, 42, 42, 0]", format!("{:?}", *b));
        }
    }

    #[test]
    #[should_panic]
    fn it_should_fail_write_on_bad_offset() {
        let mut b = SecBuf::with_insecure(4);
        b.write(3, &[42, 42]).unwrap();
    }
}
