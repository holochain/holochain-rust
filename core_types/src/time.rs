//! ?

/// ?
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
