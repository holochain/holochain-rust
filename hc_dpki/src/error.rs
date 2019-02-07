/// Error for hc-dpki lib to use in your code.
#[derive(Debug)]
pub enum DPKIError {
    ErrorMessage(String),
}

impl DPKIError {
    pub fn new(msg: &str) -> DPKIError {
        DPKIError::ErrorMessage(msg.to_string())
    }
}
