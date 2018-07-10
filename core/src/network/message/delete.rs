use super::MessageData;
use chain::pair::Pair;
use agent::keys::Keys;

const NAME: &str = "DEL_REQUEST";

pub struct Delete {

    data: MessageData,
    pair: Pair,

}

impl Delete {

    pub fn new(keys: &Keys, pair: &Pair) -> Delete {
        Delete{
            data: MessageData::new(keys, NAME, &pair.json()),
            pair: pair.clone(),
        }
    }

    pub fn pair(&self) -> Pair {
        self.pair.clone()
    }

}

impl super::Message for Delete {

    fn name(&self) -> &str {
        NAME
    }

    fn data(&self) -> super::MessageData {
        self.data.clone()
    }

}

#[cfg(test)]
pub mod tests {
    use network::message::Message;
    use super::Delete;
    use chain::pair::tests::test_pair;
    use agent::keys::tests::test_keys;

    pub fn test_delete() -> Delete {
        Delete::new(&test_keys(), &test_pair())
    }

    #[test]
    fn new() {
        // smoke test
        test_delete();
    }

    #[test]
    fn name() {
        assert_eq!("DEL_REQUEST", test_delete().name());
    }

    #[test]
    fn pair() {
        assert_eq!(test_pair(), test_delete().pair());
    }

}
