use super::MessageData;

const NAME: &str = "MOD_REQUEST";
const CODE: i8 = 4;

pub struct Modify {

    data: MessageData,

}

impl Modify {

    pub fn new(data: &MessageData) -> Modify {
        Modify{
            data: data.clone(),
        }
    }

}

impl super::Message for Modify {

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
    use super::Modify;

    #[test]
    fn new() {
        // smoke test
        let data = MessageData::new("body", "from", "time");
        let _put = Modify::new(&data);
    }

    #[test]
    fn type_name() {
        let data = MessageData::new("body", "from", "time");
        assert_eq!("MOD_REQUEST", Modify::new(&data).type_name());
    }

    #[test]
    fn type_code() {
        let data = MessageData::new("body", "from", "time");
        assert_eq!(4, Modify::new(&data).type_code());
    }

    #[test]
    fn data() {
        let data = MessageData::new("body", "from", "time");
        assert_eq!(data, Modify::new(&data).data());
    }

}
