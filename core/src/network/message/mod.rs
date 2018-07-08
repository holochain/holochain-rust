pub mod delete;
pub mod modify;
pub mod put;
pub mod response_ok;
pub mod response_error;

#[derive(Clone, PartialEq, Debug)]
pub struct MessageData {
    body: String,
    from: String,
    time: String,
}

impl MessageData {

    pub fn new(body: &str, from: &str, time: &str) -> MessageData {
        MessageData{
            body: String::from(body),
            from: String::from(from),
            time: String::from(time),
        }
    }

    pub fn body(&self) -> String {
        self.body.clone()
    }

    pub fn from(&self) -> String {
        self.from.clone()
    }

    pub fn time(&self) -> String {
        self.time.clone()
    }

}

pub trait Message {

    fn type_name(&self) -> &str;

    fn type_code(&self) -> i8;

    fn data(&self) -> MessageData;

}

#[cfg(test)]
pub mod tests {

    use super::MessageData;

    pub fn test_data() -> MessageData {
        MessageData::new("body", "from", "time")
    }


    #[test]
    fn data_new() {
        // smoke test
        test_data();
    }

    #[test]
    fn data_body() {
        assert_eq!("body", test_data().body());
    }

    #[test]
    fn data_from() {
        assert_eq!("from", test_data().from());
    }

    #[test]
    fn data_time() {
        assert_eq!("time", test_data().time());
    }
}
