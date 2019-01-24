//! The Iso8601 type is defined here. It is used in particular
//! within ChainHeader to enforce that their timestamps
//! are defined in a useful and consistent way.

use chrono::{
    DateTime,
    offset::{
        FixedOffset,
    },
};
use std::{
    cmp::Ordering,
    time::Duration,
    convert::TryFrom,
};
use error::HolochainError;
use json::JsonString;
use regex::Regex;

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

/// This struct represents datetime data stored as a string in the ISO 8601 and RFC 3339 (more
/// restrictive) format.
///
/// More info on the relevant [wikipedia article](https://en.wikipedia.org/wiki/ISO_8601).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Iso8601(String);

/// A static string is considered an infallible conversion; also unchecked infallible String conversion.
///
/// Since we receive Iso8601 from many remote, untrusted sources, we don't want to always force
/// checking at initial creation.  However, later on, we need to validate the timestamp, and we may
/// want to create a validated Iso8601 that we know will work in comparisons, etc.  For that, we use
/// the TryFrom method.
///
/// In the future, any invalid static-&str errors could (should?) produce a `panic!`.  This is
/// reasonable, as it would indicate an error in the code of the application, not in the logic.
impl From<&'static str> for Iso8601 {
    fn from(s: &str) -> Iso8601 {
        Iso8601(s.to_owned())
    }
}

impl From<String> for Iso8601 {
    fn from(s: String) -> Iso8601 {
        Iso8601(s)
    }
}

/// Conversion try_from on a &Iso8601 are fallible conversions, which may produce a HolochainError
/// if the timestamp is not valid ISO 8601 / RFC 3339.  We will allow some flexibilty; strip
/// surrounding whitespace, a bare timestamp missing any timezone specifier will be assumed to be
/// UTC "Zulu", make internal separators optional if unambiguous.  If you keep to straight RFC 3339
/// timestamps, then parsing will be quick, otherwise we'll employ a regular expression to parse a
/// more flexible subset of the ISO 8601 standard from your supplied timestamp, and then use the RFC
/// 3339 parser again.
impl TryFrom<&Iso8601> for DateTime<FixedOffset> {
    type Error = HolochainError;
    fn try_from(lhs: &Iso8601) -> Result<DateTime<FixedOffset>,Self::Error> {
        lazy_static! {
            static ref ISO8601_RE: Regex = Regex::new(r"(?x)
                ^
                \s*
                (?P<Y>\d{4})
                -?              # Allow unambiguous single digit months
                (?P<M>
                   0[12]
                 | 0?[3-9]
                 | 1[012]
                )
                -?              # Allow unambiguous single digit day
                (?P<D>
                   0[12]
                 | 0?[3-9]
                 | [12][0-9]
                 | 3[01]
                )
                (?:             # Optional T or space(s)
                   [Tt]
                 | \s+
                )               # Allow unambiguous single digit hour
                (?P<h>
                   0[12]
                 | 0?[3-9]
                 | 1[0-9]
                 | 2[0-3]
                )
                :?              # No need to support leap-seconds
                (?P<m>[0-5][0-9])
                :?              # The whole seconds group is optional, implies 00
                (?P<s>
                  (?:
                    \d{2}
                    (?:[.]\d+)?
                  )?
                )
                \s*              # Optional whitespace, no timezone specifier implies Z
                (?P<Z>
                   (?:[+-]\d{2}
                     (?:[:]?\d{2})?
                   )
                 | [Zz]?
                )
                \s*
                $"
            ).unwrap();
        }
        DateTime::parse_from_rfc3339(&lhs.0)
            .or_else(
                |_| ISO8601_RE.captures(&lhs.0)
                    .map_or_else(
                        || Err(HolochainError::ErrorGeneric(
                            format!("Failed to find ISO 3339 or RFC 8601 timestamp in {:?}", lhs.0))),
                        |cap| {
                            let timestamp = &format!("{:0>4}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{:0>2}{}",
                                                     &cap["Y"],
                                                     &cap["M"],
                                                     &cap["D"],
                                                     &cap["h"],
                                                     &cap["m"],
                                                     &match &cap["s"] { "" => "0", other => other },
                                                     &match &cap["Z"] { "" => "Z", other => other });
                            DateTime::parse_from_rfc3339(timestamp)
                                .map_err(|_| HolochainError::ErrorGeneric(
                                    format!("Attempting to convert RFC 3339 timestamp {:?} from ISO 8601 {:?} to a DateTime",
                                            timestamp, lhs.0)))
                        }
                    )
            )
    }
}

/// PartialEq and PartialCmp for ISO 8601 / RFC 3339 timestamps w/ timezone specification.  Note
/// that two timestamps that differ in time specification may be equal, because they are the same
/// time specified in two different timezones.  Therefore, a String-based Partial{Cmp,Eq} are not
/// correct.  If conversion of any Iso8601 String fails, returns false for every test; similarly to
/// how float NaN != NaN.
impl PartialEq for Iso8601 {
    fn eq(&self, rhs: &Iso8601) -> bool {
        match DateTime::<FixedOffset>::try_from(self) {
            Ok(dt_lhs) => match DateTime::<FixedOffset>::try_from(rhs) {
                Ok(dt_rhs) => (&dt_lhs).eq(&dt_rhs),
                Err(_e) => false,
            },
            Err(_e) => false,
        }
    }
}

impl PartialOrd for Iso8601 {
    fn partial_cmp(&self, rhs: &Iso8601) -> Option<Ordering> {
        match DateTime::<FixedOffset>::try_from(self) {
            Ok(ts_lhs) => match DateTime::<FixedOffset>::try_from(rhs) {
                Ok(ts_rhs) => (&ts_lhs).partial_cmp(&ts_rhs),
                Err(_e) => None,
            },
            Err(_e) => None,
        }
    }
}

pub fn test_iso_8601() -> Iso8601 {
    Iso8601::from("2018-10-11T03:23:38+00:00")
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_iso_8601_basic() {
        // Different ways of specifying UTC "Zulu".  A bare timestamp will be defaulted to "Zulu".
        match DateTime::<FixedOffset>::try_from( &Iso8601::from("2018-10-11T03:23:38 +00:00")) {
            Ok(ts) => assert_eq!( format!("{}", ts), "2018-10-11 03:23:38 +00:00" ),
            Err(e) => panic!("Unexpected failure of checked DateTime<FixedOffset> try_from: {:?}", e ),
        }
        match DateTime::<FixedOffset>::try_from( &Iso8601::from("2018-10-11T03:23:38Z")) {
            Ok(ts) => assert_eq!( format!("{}", ts), "2018-10-11 03:23:38 +00:00"),
            Err(e) => panic!("Unexpected failure of checked DateTime<FixedOffset> try_from: {:?}", e ),
        }
        match DateTime::<FixedOffset>::try_from( &Iso8601::from("2018-10-11T03:23:38")) {
            Ok(ts) => assert_eq!( format!("{}", ts), "2018-10-11 03:23:38 +00:00"),
            Err(e) => panic!("Unexpected failure of checked DateTime<FixedOffset> try_from: {:?}", e ),
        }
        match DateTime::<FixedOffset>::try_from( &Iso8601::from("2018-10-11 03:23:38")) {
            Ok(ts) => assert_eq!( format!("{}", ts), "2018-10-11 03:23:38 +00:00"),
            Err(e) => panic!("Unexpected failure of checked DateTime<FixedOffset> try_from: {:?}", e ),
        }
        match DateTime::<FixedOffset>::try_from( &Iso8601::from("20181011 0323")) {
            Ok(ts) => assert_eq!( format!("{}", ts), "2018-10-11 03:23:00 +00:00"),
            Err(e) => panic!("Unexpected failure of checked DateTime<FixedOffset> try_from: {:?}", e ),
        }
        // Degenerate timestamp with unambiguous single digit month, day and hour
        match DateTime::<FixedOffset>::try_from( &Iso8601::from("  201894  323  ")) {
            Ok(ts) => assert_eq!( format!("{}", ts), "2018-09-04 03:23:00 +00:00"),
            Err(e) => panic!("Unexpected failure of checked DateTime<FixedOffset> try_from: {:?}", e ),
        }
        assert!(
            Iso8601::from("2018-10-11T03:23:38+00:00") == Iso8601::from("2018-10-11T03:23:38Z")
        );
        assert!(
            Iso8601::from("2018-10-11T03:23:38") == Iso8601::from("2018-10-11T03:23:38Z")
        );
        assert!(
            Iso8601::from(" 20181011  0323  Z ") == Iso8601::from("2018-10-11T03:23:00Z")
        );

        // Fixed-offset ISO 8601 are comparable to UTC times
        assert!(
            Iso8601::from("2018-10-11T03:23:38-08:00") == Iso8601::from("2018-10-11T11:23:38Z")
        );
        assert!(Iso8601::from("2018-10-11T03:23:39-08:00") > Iso8601::from("2018-10-11T11:23:38Z"));
        assert!(Iso8601::from("2018-10-11T03:23:37-08:00") < Iso8601::from("2018-10-11T11:23:38Z"));

        // Ensure PartialOrd respects persistent inequality of invalid ISO 8601 DateTime strings
        // TODO: Since we're potentially validating all Iso8601 w/ DateTime<Utc> upon creation, we
        // can use Eq/Ord instead of ParialEq/PartialOrd.  For now, we allow invalid Iso8601 data.
        assert!(Iso8601::from("boo") != Iso8601::from("2018-10-11T03:23:38Z"));
        assert!(Iso8601::from("2018-10-11T03:23:38Z") != Iso8601::from("boo"));
        assert!(Iso8601::from("boo") != Iso8601::from("boo"));
        assert!(!(Iso8601::from("2018-10-11T03:23:38Z") < Iso8601::from("boo")));
        assert!(!(Iso8601::from("boo") < Iso8601::from("2018-10-11T03:23:38Z")));
        assert!(!(Iso8601::from("boo") < Iso8601::from("boo")));

        //let maybe_dt: Result<DateTime<FixedOffset>,HolochainError> = Iso8601::from("boo").try_into();
        match DateTime::<FixedOffset>::try_from( &Iso8601::from("boo")) {
            Ok(ts) => panic!("Unexpected success of checked DateTime<FixedOffset> try_from: {:?}", &ts ),
            Err(e) => assert_eq!(format!("{}", e), "Failed to find ISO 3339 or RFC 8601 timestamp in \"boo\""),
        }
    }
}
