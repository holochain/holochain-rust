use std::io::{Error, ErrorKind};

/// provide some convenience functions for working with non-blocking IO
pub trait IoErrorExt {
    fn with_would_block() -> Error;
    fn would_block(&self) -> bool;
}

impl IoErrorExt for Error {
    fn with_would_block() -> Error {
        ErrorKind::WouldBlock.into()
    }

    fn would_block(&self) -> bool {
        if let ErrorKind::WouldBlock = self.kind() {
            true
        } else {
            false
        }
    }
}
