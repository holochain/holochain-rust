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

    pub fn pair(&self) -> Pair {
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
pub mod tests {
    use chain::pair::tests::test_pair;
    use network::message::Message;
    use super::Put;
    use network::message::tests::test_data;

    pub fn test_put() -> Put {
        Put::new(&test_data(), &test_pair())
    }

    #[test]
    fn new() {
        // smoke test
        test_put();
    }

    #[test]
    fn type_name() {
        assert_eq!("PUT_REQUEST", test_put().type_name());
    }

    #[test]
    fn type_code() {
        assert_eq!(2, test_put().type_code());
    }

    #[test]
    fn data() {
        assert_eq!(test_data(), test_put().data());
    }

    #[test]
    fn pair() {
        assert_eq!(test_pair(), test_put().pair())
    }

}
