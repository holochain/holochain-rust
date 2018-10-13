use holochain_core_types::HolochainError;
use walkdir::Error as WalkdirError;

impl From<WalkdirError> for HolochainError {
    fn from(error: WalkdirError) -> Self {
        // adapted from https://docs.rs/walkdir/2.2.5/walkdir/struct.Error.html#example
        let path = error.path().unwrap_or(Path::new("")).display();
        let reason = match error.io_error() {
            Some(inner) => reason_for_io_error(inner),
            None => String::new(),
        };
        HolochainError::IoError(format!("error at path: {}, reason: {}", path, reason))
    }
}