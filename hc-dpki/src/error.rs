pub type SeedResult<T> = Result<T, SeedError>;

/// Error for Seed lib to use in your code.
#[derive(Debug)]
pub enum SeedError {
    ErrorMessage(String),
}

impl SeedError {
    pub fn new(msg: &str) -> SeedError {
        SeedError::ErrorMessage(msg.to_string())
    }
}

pub type KeypairResult<T> = Result<T, SeedError>;

/// Error for Keypair lib to use in your code.
#[derive(Debug)]
pub enum KeypairError {
    ErrorMessage(String),
}

impl KeypairError {
    pub fn new(msg: &str) -> KeypairError {
        KeypairError::ErrorMessage(msg.to_string())
    }
}
