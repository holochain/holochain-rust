pub mod put;
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
