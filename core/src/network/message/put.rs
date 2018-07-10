use super::MessageData;
use chain::pair::Pair;
use agent::keys::Keys;

const NAME: &str = "PUT_REQUEST";

pub struct Put {

    data: MessageData,
    pair: Pair,

}

impl Put {

    pub fn new(keys: &Keys, pair: &Pair) -> Put {
        Put{
            data: MessageData::new(keys, NAME, &pair.json()),
            pair: pair.clone(),
        }
    }

    pub fn pair(&self) -> Pair {
        self.pair.clone()
    }

}

impl super::Message for Put {

    fn name(&self) -> &str {
        NAME
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
    use agent::keys::tests::test_keys;

    pub fn test_put() -> Put {
        Put::new(&test_keys(), &test_pair())
    }

    #[test]
    fn new() {
        // smoke test
        test_put();
    }

    #[test]
    fn name() {
        assert_eq!("PUT_REQUEST", test_put().name());
    }

    #[test]
    fn pair() {
        assert_eq!(test_pair(), test_put().pair())
    }

}
