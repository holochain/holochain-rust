//! The Iso8601 type is defined here. It is used in particular
//! within ChainHeader to enforce that their timestamps
//! are defined in a useful and consistent way.

use error::error::HolochainError;
use json::JsonString;
use std::time::Duration;

/// This struct represents datetime data stored as a string
/// in the ISO 8601 format.
/// More info on the relevant [wikipedia article](https://en.wikipedia.org/wiki/ISO_8601).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Iso8601(String);

impl From<&'static str> for Iso8601 {
    fn from(s: &str) -> Iso8601 {
        Iso8601(s.to_owned())
    }
}

pub fn test_iso_8601() -> Iso8601 {
    Iso8601::from("2018-10-11T03:23:38+00:00")
}

/// Represents a timeout for an HDK function
#[derive(Clone, Deserialize, Debug, Eq, PartialEq, Hash, Serialize, DefaultJson)]
pub struct Timeout(usize);

impl Timeout {
    pub fn new(timeout_ms: usize) -> Self {
        Self(timeout_ms)
    }
}

impl Default for Timeout {
    fn default() -> Timeout {
        Timeout(60000)
    }
}

impl From<Timeout> for Duration {
    fn from(Timeout(millis): Timeout) -> Duration {
        Duration::from_millis(millis as u64)
    }
}

impl From<&Timeout> for Duration {
    fn from(Timeout(millis): &Timeout) -> Duration {
        Duration::from_millis(*millis as u64)
    }
}

impl From<usize> for Timeout {
    fn from(millis: usize) -> Timeout {
        Timeout::new(millis)
    }
}
