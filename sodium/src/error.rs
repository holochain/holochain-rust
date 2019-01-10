pub type SodiumResult<T> = Result<T, SodiumError>;

/// Error for Sodium lib to use in your code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SodiumError {
    OutputLength(String),
    ErrorGeneric(String),
}

impl SodiumError {
    pub fn new(msg: &str) -> SodiumError {
        SodiumError::OutputLength(msg.to_string())
    }
}
impl From<base64::DecodeError> for SodiumError {
    fn from(error: base64::DecodeError) -> Self {
        SodiumError::ErrorGeneric(format!("base64 decode error: {}", error.to_string()))
    }
}

impl From<reed_solomon::DecoderError> for SodiumError {
    fn from(error: reed_solomon::DecoderError) -> Self {
        SodiumError::ErrorGeneric(format!("reed_solomon decode error: {:?}", error))
    }
}
