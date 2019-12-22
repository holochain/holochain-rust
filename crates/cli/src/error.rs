use failure::{format_err, Error, Fail};

#[derive(Debug, Fail)]
pub enum HolochainError {
    #[fail(display = "Error: {}", _0)]
    Default(Error),
}

pub type DefaultResult<T> = Result<T, Error>;

pub type HolochainResult<T> = Result<T, HolochainError>;

impl From<String> for HolochainError {
    fn from(string: String) -> Self {
        Self::Default(format_err!("{}", string))
    }
}
