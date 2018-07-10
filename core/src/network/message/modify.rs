use super::MessageData;
use chain::pair::Pair;
use agent::keys::Keys;
use serde_json;

const NAME: &str = "MOD_REQUEST";

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct ModifyData {
    old_pair: Pair,
    new_pair: Pair,
}

pub struct Modify {

    modify: ModifyData,
    data: MessageData,

}

impl Modify {

    pub fn new(keys: &Keys, old_pair: &Pair, new_pair: &Pair) -> Modify {
        let m = ModifyData{
            old_pair: old_pair.clone(),
            new_pair: new_pair.clone(),
        };
        Modify{
            data: MessageData::new(keys, NAME, &serde_json::to_string(&m).unwrap()),
            modify: m.clone(),
        }
    }

    pub fn modify(&self) -> ModifyData {
        self.modify.clone()
    }

}

impl super::Message for Modify {

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
    use super::Modify;
    use super::ModifyData;
    use agent::keys::tests::test_keys;
    use chain::pair::tests::test_pair;

    pub fn test_data() -> ModifyData {
        ModifyData{
            old_pair: test_pair(),
            new_pair: test_pair(),
        }
    }

    pub fn test_modify() -> Modify {
        Modify::new(&test_keys(), &test_pair(), &test_pair())
    }

    #[test]
    fn new() {
        // smoke test
        test_modify();
    }

    #[test]
    fn name() {
        assert_eq!("MOD_REQUEST", test_modify().name());
    }

    #[test]
    fn modify() {
        assert_eq!(test_data(), test_modify().modify());
    }

}
