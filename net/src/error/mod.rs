#[derive(Debug, Fail)]
pub enum NetworkError {
    #[fail(display = "Network error: {}", error)]
    GenericError { error: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Error;

    pub fn fail() -> Result<(), Error> {
        Err(NetworkError::GenericError {
            error: "boink".to_string(),
        }
        .into())
    }

    pub fn test_bail() -> Result<(), Error> {
        bail!("test {}", "fish")
    }

    #[test]
    fn can_fail_with_generic_error() {
        match fail() {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err.to_string(), "Network error: boink"),
        }
    }

    #[test]
    fn can_bail() {
        match test_bail() {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err.to_string(), "test fish"),
        }
    }
}
