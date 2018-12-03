use failure::Error;

#[derive(Debug, Fail)]
pub enum HolochainError {
    #[fail(display = "Error: {}", _0)]
    Default(Error),
}

pub type DefaultResult<T> = Result<T, Error>;
pub type HolochainResult<T> = Result<T, HolochainError>;
