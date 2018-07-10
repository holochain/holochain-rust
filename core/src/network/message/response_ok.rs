use super::MessageData;
use agent::keys::Keys;

const NAME: &str = "OK_RESPONSE";

pub struct Ok {
    data: MessageData,
}

impl Ok {

    pub fn new (keys: &Keys) -> Ok {
        Ok{
            data: MessageData::new(keys, NAME, ""),
        }
    }

}

impl super::Message for Ok {

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
    use super::Ok;
    use agent::keys::tests::test_keys;

    pub fn test_ok() -> Ok {
        Ok::new(&test_keys())
    }

    #[test]
    /// tests for Ok::new()
    fn new() {
        // smoke test
        test_ok();
    }

    #[test]
    fn name() {
        assert_eq!("OK_RESPONSE", test_ok().name());
    }

}
