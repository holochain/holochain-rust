use std::io::{Error, ErrorKind};

/// provide some convenience functions for working with non-blocking IO
pub trait IoErrorExt {
    /// new WouldBlock error
    fn with_would_block() -> Error;

    /// true if this error is of kind WouldBlock
    fn would_block(&self) -> bool;
}

impl IoErrorExt for Error {
    /// new WouldBlock error
    fn with_would_block() -> Error {
        ErrorKind::WouldBlock.into()
    }

    /// true if this error is of kind WouldBlock
    fn would_block(&self) -> bool {
        if let ErrorKind::WouldBlock = self.kind() {
            true
        } else {
            false
        }
    }
}
