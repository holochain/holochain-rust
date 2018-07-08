use super::MessageData;

const NAME: &str = "OK_RESPONSE";
const CODE: i8 = 1;

pub struct Ok {
    data: MessageData,
}

impl Ok {

    pub fn new (data: &MessageData) -> Ok {
        Ok{
            data: data.clone(),
        }
    }

}

impl super::Message for Ok {

    fn type_name(&self) -> &str {
        NAME
    }

    fn type_code(&self) -> i8 {
        CODE
    }

    fn data(&self) -> super::MessageData {
        self.data.clone()
    }

}

#[cfg(test)]
mod tests {
    use dht::message::Message;
    use dht::message::MessageData;
    use super::Ok;

    #[test]
    /// tests for Ok::new()
    fn new() {
        // smoke test
        let data = MessageData::new("body", "from", "time");
        let _ok = Ok::new(&data);
    }

    #[test]
    fn type_name() {
        let data = MessageData::new("body", "from", "time");

        assert_eq!("OK_RESPONSE", Ok::new(&data).type_name());
    }

    #[test]
    fn type_code() {
        let data = MessageData::new("body", "from", "time");

        assert_eq!(1, Ok::new(&data).type_code());
    }

    #[test]
    fn data() {
        let data = MessageData::new("body", "from", "time");

        assert_eq!(data, Ok::new(&data).data());
    }

}
