use std::{error::Error, fmt};

/// module for holding Holochain specific errors

#[derive(Clone, Debug, PartialEq)]
pub enum HolochainError {
    ErrorGeneric(String),
    InstanceNotActive,
    InstanceActive,
    NotImplemented,
    LoggingError,
    DnaMissing,
    ZomeNotFound(String),
    CapabilityNotFound(String),
    ZomeFunctionNotFound(String),
}

impl HolochainError {
    pub fn new(msg: &str) -> HolochainError {
        HolochainError::ErrorGeneric(msg.to_string())
    }
}

impl fmt::Display for HolochainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

use self::HolochainError::*;

impl Error for HolochainError {
    fn description(&self) -> &str {
        match self {
            ErrorGeneric(err_msg) => &err_msg,
            NotImplemented => "not implemented",
            InstanceNotActive => "the instance is not active",
            InstanceActive => "the instance is active",
            LoggingError => "logging failed",
            DnaMissing => "DNA is missing",
            ZomeNotFound(err_msg) => &err_msg,
            CapabilityNotFound(err_msg) => &err_msg,
            ZomeFunctionNotFound(err_msg) => &err_msg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // a test function that returns our error result
    fn raises_holochain_error(yes: bool) -> Result<(), HolochainError> {
        if yes {
            Err(HolochainError::new("borked"))
        } else {
            Ok(())
        }
    }

    #[test]
    fn can_instantiate() {
        let err = HolochainError::new("borked");
        if let HolochainError::ErrorGeneric(err_msg) = err {
            assert_eq!(err_msg, "borked")
        } else {
            assert!(false)
        }
    }

    #[test]
    fn can_raise_holochain_error() {
        let result = raises_holochain_error(true);
        match result {
            Ok(_) => assert!(false),
            Err(err) => match err {
                HolochainError::ErrorGeneric(msg) => assert_eq!(msg, "borked"),
                _ => assert!(false),
            },
        };
    }

    #[test]
    fn can_return_result() {
        let result = raises_holochain_error(false);
        let result = match result {
            Ok(x) => x,
            Err(_) => panic!(),
        };
        assert_eq!(result, ())
    }
}
