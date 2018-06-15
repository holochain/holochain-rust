use std::error::Error;
use std::fmt;

/// module for holding Holochain specific errors

#[derive(Clone, Debug, PartialEq)]
pub enum HolochainError {
    ErrorGeneric(String),
    AllreadyInitialized,
    InstanceNotActive,
    InstanceActive,
    NotImplemented,
    Dummy
   //  SerdeError(serde_json::error::Error), TODO
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
            AllreadyInitialized => "already initialized",
            NotImplemented => "not implemented",
            InstanceNotActive => "the instance is not active",
            InstanceActive => "the instance is active",
            _ => "unknown error type",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // a test function that returns our error result
    fn raises_hc_error(yes: bool) -> Result<(), HolochainError> {
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
            assert_eq!(err_msg,"borked")
        } else {
            assert!(false)
        }
    }

    #[test]
    fn can_raise_hc_error() {
        let result = raises_hc_error(true);
        match result {
            Ok(_) => assert!(false),
            Err(err) => match err {
                HolochainError::ErrorGeneric(msg) => assert_eq!(msg, "borked"),
                _=>assert!(false)
            }
        };
    }

    #[test]
    fn can_return_result() {
        let result = raises_hc_error(false);
        let result = match result {
            Ok(x) => x,
            Err(_) => panic!(),
        };
        assert_eq!(result, ())
    }
}
