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

impl std::fmt::Display for LocksmithError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for LocksmithError {}

pub type LocksmithResult<T> = Result<T, LocksmithError>;
