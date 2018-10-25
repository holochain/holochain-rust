use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum DnaError {
    ZomeNotFound(String),
    CapabilityNotFound(String),
    ZomeFunctionNotFound(String),
}

impl Error for DnaError {
    fn description(&self) -> &str {
        match self {
            DnaError::ZomeNotFound(err_msg) => &err_msg,
            DnaError::CapabilityNotFound(err_msg) => &err_msg,
            DnaError::ZomeFunctionNotFound(err_msg) => &err_msg,
        }
    }
}

impl fmt::Display for DnaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // @TODO seems weird to use debug for display
        // replacing {:?} with {} gives a stack overflow on to_string() (there's a test for this)
        // what is the right way to do this?
        // @see https://github.com/holochain/holochain-rust/issues/223
        write!(f, "{:?}", self)
    }
}
