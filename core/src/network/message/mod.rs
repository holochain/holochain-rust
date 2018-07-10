pub mod delete;
pub mod modify;
pub mod put;
pub mod response_ok;
pub mod response_error;
use agent::keys::Keys;

#[derive(Clone, PartialEq, Debug)]
pub struct MessageData {
    message_type: String,
    payload: String,
    author: String,
    time: String,
    signature: String,
}

impl MessageData {

    pub fn new(keys: &Keys, message_type: &str, payload: &str) -> MessageData {
        MessageData{
            message_type: String::from(message_type),
            payload: String::from(payload),
            author: keys.node_id().clone(),
            time: String::new(),
            signature: String::new(),
        }
    }

    pub fn message_type(&self) -> String {
        self.message_type.clone()
    }

    pub fn payload(&self) -> String {
        self.payload.clone()
    }

    pub fn author(&self) -> String {
        self.author.clone()
    }

    pub fn time(&self) -> String {
        self.time.clone()
    }

    pub fn signature(&self) -> String {
        self.signature.clone()
    }

}

pub trait Message {

    fn name(&self) -> &str;

    // fn code(&self) -> u8;

    fn data(&self) -> MessageData;

}

#[cfg(test)]
pub mod tests {

    use super::MessageData;
    use agent::keys::tests::test_keys;
    use agent::keys::tests::test_node_id;

    pub fn test_message_type() -> String {
        String::from("test message type")
    }

    pub fn test_message_payload() -> String {
        String::from("test message payload")
    }

    pub fn test_data() -> MessageData {
        MessageData::new(&test_keys(), &test_message_type(), &test_message_payload())
    }


    #[test]
    fn data_new() {
        // smoke test
        test_data();
    }

    #[test]
    fn data_payload() {
        assert_eq!(test_message_payload(), test_data().payload());
    }

    #[test]
    fn data_author() {
        assert_eq!(test_node_id(), test_data().author());
    }

    #[test]
    fn data_time() {
        assert_eq!("", test_data().time());
    }
}
