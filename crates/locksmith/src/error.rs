#[derive(Debug)]
pub enum LocksmithErrorKind {
    LocksmithTimeout,
    LocksmithPoisonError,
    LocksmithWouldBlock,
}

#[derive(Debug)]
pub enum LockType {
    Lock,
    Read,
    Write,
}

#[derive(Debug)]
pub struct LocksmithError {
    lock_type: LockType,
    kind: LocksmithErrorKind,
}

impl LocksmithError {
    pub fn new(lock_type: LockType, kind: LocksmithErrorKind) -> Self {
        Self { lock_type, kind }
    }
}

pub type LocksmithResult<T> = Result<T, LocksmithError>;
