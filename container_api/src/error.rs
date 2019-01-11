use holochain_core_types::error::HolochainError;
use std::{error::Error, fmt, option::NoneError};

pub type HolochainResult<T> = Result<T, HolochainInstanceError>;

// TODO rename to HolochainError
#[derive(Debug, PartialEq, Clone)]
pub enum HolochainInstanceError {
    InternalFailure(HolochainError),
    InstanceNotActiveYet,
    InstanceAlreadyActive,
    NoSuchInstance,
}

impl Error for HolochainInstanceError {
    fn description(&self) -> &str {
        match self {
            HolochainInstanceError::InternalFailure(ref err) => err.description(),
            HolochainInstanceError::InstanceNotActiveYet => "Holochain instance is not active yet.",
            HolochainInstanceError::InstanceAlreadyActive => {
                "Holochain instance is already active."
            },
            HolochainInstanceError::NoSuchInstance => "Instance does not exist",
        }
    }

    // not sure how to test this because dyn reference to the Error is not implementing PartialEq
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn cause(&self) -> Option<&Error> {
        match self {
            HolochainInstanceError::InternalFailure(ref err)  => Some(err),
            HolochainInstanceError::InstanceNotActiveYet => None,
            HolochainInstanceError::InstanceAlreadyActive => None,
            HolochainInstanceError::NoSuchInstance => None,
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

impl From<NoneError> for HolochainInstanceError {
    fn from(_: NoneError) -> Self { HolochainInstanceError::NoSuchInstance }
}

#[cfg(test)]
pub mod tests {

    use crate::error::HolochainInstanceError;
    use holochain_core_types::error::HolochainError;
    use std::error::Error;

    #[test]
    /// show ToString for HolochainInstanceError
    fn holochain_instance_error_description_test() {
        for (i, o) in vec![
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
        ] {
            assert_eq!(i.description(), o,);
        }
    }

    #[test]
    /// show ToString for HolochainInstanceError
    fn holochain_instance_error_to_string_test() {
        for (i, o) in vec![
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
