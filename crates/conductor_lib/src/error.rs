use holochain_core_types::error::HolochainError;
use std::{error::Error, fmt, option::NoneError};

pub type HolochainResult<T> = Result<T, HolochainInstanceError>;

// TODO rename to HolochainError
#[derive(Debug, PartialEq, Clone)]
pub enum HolochainInstanceError {
    InternalFailure(HolochainError),
    InstanceNotActiveYet,
    InstanceAlreadyActive,
    InstanceNotInitialized,
    NoSuchInstance,
    RequiredBridgeMissing(String),
}

impl Error for HolochainInstanceError {
    // not sure how to test this because dyn reference to the Error is not implementing PartialEq
    #[rustfmt::skip]
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            HolochainInstanceError::InternalFailure(ref err)  => Some(err),
            HolochainInstanceError::InstanceNotActiveYet => None,
            HolochainInstanceError::InstanceAlreadyActive => None,
            HolochainInstanceError::InstanceNotInitialized => None,
            HolochainInstanceError::NoSuchInstance => None,
            HolochainInstanceError::RequiredBridgeMissing(_) => None,
        }
    }
}

impl fmt::Display for HolochainInstanceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = "Holochain Instance Error";
        match self {
            HolochainInstanceError::InternalFailure(ref err) => write!(f, "{}: {}", prefix, err),
            HolochainInstanceError::InstanceNotActiveYet => {
                write!(f, "{}: Holochain instance is not active yet.", prefix)
            }
            HolochainInstanceError::InstanceAlreadyActive => {
                write!(f, "{}: Holochain instance is already active.", prefix)
            }
            HolochainInstanceError::InstanceNotInitialized => {
                write!(f, "{}: Holochain instance is not initialized.", prefix)
            }
            HolochainInstanceError::NoSuchInstance => {
                write!(f, "{}: Instance does not exist", prefix)
            }
            HolochainInstanceError::RequiredBridgeMissing(handle) => write!(
                f,
                "{}: Required bridge is not present/started: {}",
                prefix, handle
            ),
        }
    }
}

impl From<HolochainError> for HolochainInstanceError {
    fn from(error: HolochainError) -> Self {
        HolochainInstanceError::InternalFailure(error)
    }
}

impl From<HolochainInstanceError> for HolochainError {
    fn from(error: HolochainInstanceError) -> Self {
        HolochainError::new(&error.to_string())
    }
}

impl From<NoneError> for HolochainInstanceError {
    fn from(_: NoneError) -> Self {
        HolochainInstanceError::NoSuchInstance
    }
}

#[cfg(test)]
pub mod tests {

    use crate::error::HolochainInstanceError;
    use holochain_core_types::error::HolochainError;

    #[test]
    /// show ToString for HolochainInstanceError
    fn holochain_instance_error_to_string_test() {
        for (i, o) in vec![
            (
                HolochainInstanceError::InstanceNotInitialized,
                "Holochain instance is not initialized.",
            ),
            (
                HolochainInstanceError::InstanceNotActiveYet,
                "Holochain instance is not active yet.",
            ),
            (
                HolochainInstanceError::InstanceAlreadyActive,
                "Holochain instance is already active.",
            ),
            (
                HolochainInstanceError::InternalFailure(HolochainError::DnaMissing),
                "DNA is missing",
            ),
            (
                HolochainInstanceError::NoSuchInstance,
                "Instance does not exist",
            ),
            (
                HolochainInstanceError::RequiredBridgeMissing(String::from("handle")),
                &format!("Required bridge is not present/started: handle"),
            ),
        ] {
            assert_eq!(
                i.to_string(),
                format!("Holochain Instance Error: {}", o).to_string(),
            );
        }
    }

    #[test]
    /// show From<HolochainError> for HolochainInstanceError
    fn holochain_instance_error_from_holochain_error_test() {
        assert_eq!(
            HolochainInstanceError::InternalFailure(HolochainError::DnaMissing),
            HolochainInstanceError::from(HolochainError::DnaMissing),
        );
    }
}
