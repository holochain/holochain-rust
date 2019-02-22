/// Error for hc-dpki lib to use in your code.
#[derive(Debug)]
pub enum DpkiError {
    ErrorMessage(String),
}

impl DpkiError {
    pub fn new(msg: &str) -> DpkiError {
        DpkiError::ErrorMessage(msg.to_string())
    }
}
