use super::MessageData;
use chain::pair::Pair;

const NAME: &str = "PUT_REQUEST";
const CODE: i8 = 2;

pub struct Put {

    data: MessageData,
    pair: Pair,

}

impl Put {

    pub fn new(data: &MessageData, pair: &Pair) -> Put {
        Put{
            data: data.clone(),
            pair: pair.clone(),
        }
    }

    pub fn pair(&self) -> String {
        self.pair.clone()
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
        let pair = "";
        let _put = Put::new(&data, k);
    }

    #[test]
    fn type_name() {
        let data = MessageData::new("body", "from", "time");
        let k = "";
        assert_eq!("PUT_REQUEST", Put::new(&data, k).type_name());
    }

    #[test]
    fn type_code() {
        let data = MessageData::new("body", "from", "time");
        let k = "";
        assert_eq!(2, Put::new(&data, k).type_code());
    }

    #[test]
    fn data() {
        let data = MessageData::new("body", "from", "time");
        let k = "";
        assert_eq!(data, Put::new(&data, k).data());
    }

    #[test]
    fn key() {
        let data = MessageData::new("body", "from", "time");
        let k = "some key";
        assert_eq!(k, Put::new(&data, k).key())
    }

}
