use lib3h::error::Lib3hError;
use lib3h_zombie_actor::prelude::*;
use std::{fmt, result};

#[derive(Debug, PartialEq)]
pub struct Sim2hError(String);
impl From<GhostError> for Sim2hError {
    fn from(err: GhostError) -> Self {
        Sim2hError(format!("{:?}", err))
    }
}
impl From<&str> for Sim2hError {
    fn from(err: &str) -> Self {
        Sim2hError(err.to_string())
    }
}
impl From<String> for Sim2hError {
    fn from(err: String) -> Self {
        Sim2hError(err)
    }
}
impl From<Lib3hError> for Sim2hError {
    fn from(err: Lib3hError) -> Self {
        Sim2hError(format!("{:?}", err))
    }
}
impl fmt::Display for Sim2hError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
pub type Sim2hResult<T> = result::Result<T, Sim2hError>;
pub const SPACE_MISMATCH_ERR_STR: &str = "space/agent id mismatch";
pub const VERIFY_FAILED_ERR_STR: &str = "message signature failed verify";
pub const SIGNER_MISMATCH_ERR_STR: &str = "message signer does not match content";
