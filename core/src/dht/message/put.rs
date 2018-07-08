use super::MessageData;

const NAME: &str = "PUT_REQUEST";
const CODE: i8 = 2;

pub struct Put {

    data: MessageData,

}

impl Put {

    pub fn new(data: &MessageData) -> Put {
        Put{
            data: data.clone(),
        }
    }

}

impl super::Message for Put {

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
    use super::Put;

    #[test]
    fn new() {
        // smoke test
        let data = MessageData::new("body", "from", "time");
        let _put = Put::new(&data);
    }

    #[test]
    fn type_name() {
        let data = MessageData::new("body", "from", "time");
        assert_eq!("PUT_REQUEST", Put::new(&data).type_name());
    }

    #[test]
    fn type_code() {
        let data = MessageData::new("body", "from", "time");
        assert_eq!(2, Put::new(&data).type_code());
    }

    #[test]
    fn data() {
        let data = MessageData::new("body", "from", "time");
        assert_eq!(data, Put::new(&data).data());
    }

}
