type Iso8601 = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timestamp(Iso8601);

impl From<&'static str> for Timestamp {
    fn from(s: &str) -> Timestamp {
        Timestamp(s.to_owned())
    }
}

pub fn test_timestamp() -> Timestamp {
    Timestamp::from("2018-10-11T03:23:38+00:00")
}
