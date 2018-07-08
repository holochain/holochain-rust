use super::MessageData;

const NAME: &str = "MOD_REQUEST";
const CODE: i8 = 4;

pub struct Modify {

    data: MessageData,
    old_key: String,
    new_key: String,

}

impl Modify {

    pub fn new(data: &MessageData, old_key: &str, new_key: &str) -> Modify {
        Modify{
            data: data.clone(),
            old_key: String::from(old_key),
            new_key: String::from(new_key),
        }
    }

    pub fn old_key(&self) -> String {
        self.old_key.clone()
    }

    pub fn new_key(&self) -> String {
        self.new_key.clone()
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
    use network::message::Message;
    use network::message::MessageData;
    use super::Modify;

    #[test]
    fn new() {
        // smoke test
        let data = MessageData::new("body", "from", "time");
        let ko = "";
        let kn = "";
        let _put = Modify::new(&data, ko, kn);
    }

    #[test]
    fn type_name() {
        let data = MessageData::new("body", "from", "time");
        let ko = "";
        let kn = "";
        assert_eq!("MOD_REQUEST", Modify::new(&data, ko, kn).type_name());
    }

    #[test]
    fn type_code() {
        let data = MessageData::new("body", "from", "time");
        let ko = "";
        let kn = "";
        assert_eq!(4, Modify::new(&data, ko, kn).type_code());
    }

    #[test]
    fn data() {
        let data = MessageData::new("body", "from", "time");
        let ko = "";
        let kn = "";
        assert_eq!(data, Modify::new(&data, ko, kn).data());
    }

    #[test]
    fn old_key() {
        let data = MessageData::new("body", "from", "time");
        let ko = "foo";
        let kn = "bar";
        assert_eq!(ko, Modify::new(&data, ko, kn).old_key());
    }

    #[test]
    fn new_key() {
        let data = MessageData::new("body", "from", "time");
        let ko = "foo";
        let kn = "bar";
        assert_eq!(kn, Modify::new(&data, ko, kn).new_key());
    }

}
