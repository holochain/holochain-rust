/// Error for Sodium lib to use in your code.
#[derive(Debug)]
pub enum SodiumError {
    Generic(String),
    OutputLength(String),
}

impl SodiumError {
    pub fn new(msg: &str) -> SodiumError {
        SodiumError::Generic(msg.to_string())
    }

    pub fn with_output_length(msg: &str) -> SodiumError {
        SodiumError::OutputLength(msg.to_string())
    }
}
