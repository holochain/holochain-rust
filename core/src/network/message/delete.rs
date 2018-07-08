use super::MessageData;

const NAME: &str = "DEL_REQUEST";
const CODE: i8 = 3;

pub struct Delete {

    data: MessageData,
    key: String,

}

impl Delete {

    pub fn new(data: &MessageData, key: &str) -> Delete {
        Delete{
            data: data.clone(),
            key: String::from(key),
        }
    }

    pub fn key(&self) -> String {
        self.key.clone()
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
    use network::message::Message;
    use network::message::MessageData;
    use super::Delete;

    #[test]
    fn new() {
        // smoke test
        let data = MessageData::new("body", "from", "time");
        let k = "";
        let _put = Delete::new(&data, k);
    }

    #[test]
    fn type_name() {
        let data = MessageData::new("body", "from", "time");
        let k = "";
        assert_eq!("DEL_REQUEST", Delete::new(&data, k).type_name());
    }

    #[test]
    fn type_code() {
        let data = MessageData::new("body", "from", "time");
        let k = "";
        assert_eq!(3, Delete::new(&data, k).type_code());
    }

    #[test]
    fn data() {
        let data = MessageData::new("body", "from", "time");
        let k = "";
        assert_eq!(data, Delete::new(&data, k).data());
    }

    #[test]
    fn key() {
        let data = MessageData::new("body", "from", "time");
        let k = "foo";

        assert_eq!(k, Delete::new(&data, k).key());
    }

}
