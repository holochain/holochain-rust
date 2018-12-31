use std::{
    fmt,
};
pub type SodiumResult<T> = Result<T, SodiumError>;

/// Error for Sodium lib to use in your code.
#[derive(Debug)]
pub enum SodiumError {
    OutputLength(String),
}

impl SodiumError {
    pub fn new(msg: &str) -> SodiumError {
        SodiumError::OutputLength(msg.to_string())
    }
}

// impl fmt::Display for SodiumError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self::OutputLength)
//     }
// }
