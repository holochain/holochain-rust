use std::error::Error;
use std::fmt;

/// starter module for holding Holochain specific errors

#[derive(Debug,PartialEq)]
pub struct HolochainError {
    details: String
}

impl HolochainError {
    pub fn new(msg: &str) -> HolochainError {
        HolochainError{details: msg.to_string()}
    }
}

impl fmt::Display for HolochainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for HolochainError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // a test function that returns our error result
    fn raises_hc_error(yes: bool) -> Result<(),HolochainError> {
        if yes {
            Err(HolochainError::new("borked"))
        } else {
            Ok(())
        }
    }

    #[test]
    fn can_raise_hc_error() {
        let result = raises_hc_error(true);
        let result = match result {
            Ok(_) => panic!(),
            Err(y) => y
        };
        assert_eq!(result.details,"borked")
    }

    #[test]
    fn can_return_result() {
        let result = raises_hc_error(false);
        let result = match result {
            Ok(x) => x,
            Err(_) => panic!(),
        };
        assert_eq!(result,())
    }
}
