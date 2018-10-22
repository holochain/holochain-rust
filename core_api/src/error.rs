
use holochain_core_types::error::HolochainError;
use std::error::Error;
use std::fmt;

pub type HolochainResult<T> = Result<T, HolochainInstanceError>;


// TODO rename to HolochainError
#[derive(Debug, PartialEq, Clone)]
pub enum HolochainInstanceError {
    InternalFailure(HolochainError),
    InstanceNotActiveYet,
    InstanceAlreadyActive,
}

impl Error for HolochainInstanceError {
    fn description(&self) -> &str {
        match self {
            HolochainInstanceError::InternalFailure(ref err)  => {
                err.description()
            },
            HolochainInstanceError::InstanceNotActiveYet => {
                "Holochain instance is not active yet."
            },
            HolochainInstanceError::InstanceAlreadyActive => {
                "Holochain instance is already active."
            },
        }
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn cause(&self) -> Option<&Error> {
        match self {
            HolochainInstanceError::InternalFailure(ref err)  => Some(err),
            HolochainInstanceError::InstanceNotActiveYet => None,
            HolochainInstanceError::InstanceAlreadyActive => None,
        }
    }
}

impl fmt::Display for HolochainInstanceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Holochain Instance Error: {}", self.description())
    }
}

impl From<HolochainError> for HolochainInstanceError {
    fn from(error: HolochainError) -> Self {
        HolochainInstanceError::InternalFailure(error)
    }
}