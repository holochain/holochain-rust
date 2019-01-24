//! The Iso8601 type is defined here. It is used in particular
//! within ChainHeader to enforce that their timestamps
//! are defined in a useful and consistent way.

//use std::cmp::Ordering;
use chrono::{offset::Utc, DateTime};
use error::error::HolochainError;
use json::JsonString;
use std::{cmp::Ordering, time::Duration};

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

/// This struct represents datetime data stored as a string
/// in the ISO 8601 format.
/// More info on the relevant [wikipedia article](https://en.wikipedia.org/wiki/ISO_8601).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Iso8601(String);

impl From<&'static str> for Iso8601 {
    fn from(s: &str) -> Iso8601 {
        Iso8601(s.to_owned())
    }
}

pub fn test_iso_8601() -> Iso8601 {
    Iso8601::from("2018-10-11T03:23:38+00:00")
}

/// PartialEq and PartialCmp for ISO 8601 / RFC 3339 timestamps w/ timezone specification.  Note
/// that two timestamps that differ in time specification may be equal, because they are the same
/// time specified in two different timezones.  Therefore, a String-based Partial{Cmp,Eq} are not
/// correct.  If conversion of any Iso8601 String fails, returns false for every test; similarly to
/// how float NaN != NaN.
///
/// Note that the timezone offset *is* *required*; to default to UTC, append a "Z" to the
/// `<YYYY>-<MM>-<DD>T<hh>:<mm>:<ss>` string, if no timezone is specified.
impl PartialEq for Iso8601 {
    fn eq(&self, rhs: &Iso8601) -> bool {
        match self.0.parse::<DateTime<Utc>>() {
            Ok(ts_lhs) => match rhs.0.parse::<DateTime<Utc>>() {
                Ok(ts_rhs) => (&ts_lhs).eq(&ts_rhs),
                Err(_e) => false,
            },
            Err(_e) => false,
        }
    }
}

impl PartialOrd for Iso8601 {
    fn partial_cmp(&self, rhs: &Iso8601) -> Option<Ordering> {
        match self.0.parse::<DateTime<Utc>>() {
            Ok(ts_lhs) => match rhs.0.parse::<DateTime<Utc>>() {
                Ok(ts_rhs) => (&ts_lhs).partial_cmp(&ts_rhs),
                Err(_e) => None, // No Ordering available
            },
            Err(_e) => None, // No Ordering available
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_iso_8601_basic() {
        // Different ways of specifying UTC "Zulu"
        assert!(
            Iso8601::from("2018-10-11T03:23:38+00:00") == Iso8601::from("2018-10-11T03:23:38Z")
        );

        // Fixed-offset ISO 8601 are comparable to UTC times
        assert!(
            Iso8601::from("2018-10-11T03:23:38-08:00") == Iso8601::from("2018-10-11T11:23:38Z")
        );
        assert!(Iso8601::from("2018-10-11T03:23:39-08:00") > Iso8601::from("2018-10-11T11:23:38Z"));
        assert!(Iso8601::from("2018-10-11T03:23:37-08:00") < Iso8601::from("2018-10-11T11:23:38Z"));

        // Ensure PartialOrd respects persistent inequality of invalid ISO 8601 DateTime strings
        assert!(Iso8601::from("boo") != Iso8601::from("2018-10-11T03:23:38Z"));
        assert!(Iso8601::from("2018-10-11T03:23:38Z") != Iso8601::from("boo"));
        assert!(Iso8601::from("boo") != Iso8601::from("boo"));
        assert!(!(Iso8601::from("2018-10-11T03:23:38Z") < Iso8601::from("boo")));
        assert!(!(Iso8601::from("boo") < Iso8601::from("2018-10-11T03:23:38Z")));
        assert!(!(Iso8601::from("boo") < Iso8601::from("boo")));
    }
}
