use super::MessageData;

const NAME: &str = "DEL_REQUEST";
const CODE: i8 = 3;

pub struct Delete {

    data: MessageData,

}

impl Delete {

    pub fn new(data: &MessageData) -> Delete {
        Delete{
            data: data.clone(),
        }
    }

}

impl super::Message for Delete {

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
    use super::Delete;

    #[test]
    fn new() {
        // smoke test
        let data = MessageData::new("body", "from", "time");
        let _put = Delete::new(&data);
    }

    #[test]
    fn type_name() {
        let data = MessageData::new("body", "from", "time");
        assert_eq!("DEL_REQUEST", Delete::new(&data).type_name());
    }

    #[test]
    fn type_code() {
        let data = MessageData::new("body", "from", "time");
        assert_eq!(3, Delete::new(&data).type_code());
    }

    #[test]
    fn data() {
        let data = MessageData::new("body", "from", "time");
        assert_eq!(data, Delete::new(&data).data());
    }

}
