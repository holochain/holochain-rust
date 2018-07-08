use super::MessageData;

const NAME: &str = "ERROR_RESPONSE";
const CODE: i8 = 0;

pub enum ErrorCode {
    Unknown,
    HashNotFound,
    HashDeleted,
    HashModified,
    HashRejected,
    LinkNotFound,
    EntryTypeMismatch,
    BlockedListed,
}

impl ErrorCode {
    pub fn code(&self) -> i8 {
        match self {
            ErrorCode::Unknown => 0,
            ErrorCode::HashNotFound => 1,
            ErrorCode::HashDeleted => 2,
            ErrorCode::HashModified => 3,
            ErrorCode::HashRejected => 4,
            ErrorCode::LinkNotFound => 5,
            ErrorCode::EntryTypeMismatch => 6,
            ErrorCode::BlockedListed => 7,
        }
    }
}

pub struct Error {
    code: i8,
    // Message trait data
    data: MessageData
}

impl Error {

    pub fn new(data: &MessageData, code: ErrorCode) -> Error {
        Error{
            code: code.code(),
            data: data.clone(),
        }
    }

    pub fn code(&self) -> i8 {
        self.code
    }

}

impl super::Message for Error {

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
    use dht::message::MessageData;
    use dht::message::Message;
    use super::ErrorCode;
    use super::Error;

    #[test]
    fn error_codes() {
        assert_eq!(ErrorCode::Unknown.code(), 0);
        assert_eq!(ErrorCode::HashNotFound.code(), 1);
        assert_eq!(ErrorCode::HashDeleted.code(), 2);
        assert_eq!(ErrorCode::HashModified.code(), 3);
        assert_eq!(ErrorCode::HashRejected.code(), 4);
        assert_eq!(ErrorCode::LinkNotFound.code(), 5);
        assert_eq!(ErrorCode::EntryTypeMismatch.code(), 6);
        assert_eq!(ErrorCode::BlockedListed.code(), 7);
    }

    #[test]
    /// tests for Error::new()
    fn new() {
        // smoke test
        let data = MessageData::new("body", "from", "time");
        let _error = Error::new(&data, ErrorCode::Unknown);
    }

    #[test]
    fn code() {
        let data = MessageData::new("body", "from", "time");

        assert_eq!(0, Error::new(&data, ErrorCode::Unknown).code());
        assert_eq!(1, Error::new(&data, ErrorCode::HashNotFound).code());
        assert_eq!(2, Error::new(&data, ErrorCode::HashDeleted).code());
        assert_eq!(3, Error::new(&data, ErrorCode::HashModified).code());
        assert_eq!(4, Error::new(&data, ErrorCode::HashRejected).code());
        assert_eq!(5, Error::new(&data, ErrorCode::LinkNotFound).code());
        assert_eq!(6, Error::new(&data, ErrorCode::EntryTypeMismatch).code());
        assert_eq!(7, Error::new(&data, ErrorCode::BlockedListed).code());
    }

    #[test]
    fn type_name() {
        let data = MessageData::new("body", "from", "time");

        assert_eq!("ERROR_RESPONSE", Error::new(&data, ErrorCode::Unknown).type_name());
    }

    #[test]
    fn type_code() {
        let data = MessageData::new("body", "from", "time");

        // type code is always 0 for error messages
        assert_eq!(0, Error::new(&data, ErrorCode::Unknown).type_code());
        assert_eq!(0, Error::new(&data, ErrorCode::HashNotFound).type_code());
        assert_eq!(0, Error::new(&data, ErrorCode::HashDeleted).type_code());
        assert_eq!(0, Error::new(&data, ErrorCode::HashModified).type_code());
        assert_eq!(0, Error::new(&data, ErrorCode::HashRejected).type_code());
        assert_eq!(0, Error::new(&data, ErrorCode::LinkNotFound).type_code());
        assert_eq!(0, Error::new(&data, ErrorCode::EntryTypeMismatch).type_code());
        assert_eq!(0, Error::new(&data, ErrorCode::BlockedListed).type_code());
        assert_eq!(0, Error::new(&data, ErrorCode::Unknown).type_code());
    }

    #[test]
    fn data() {
        let data = MessageData::new("body", "from", "time");
        let error = Error::new(&data, ErrorCode::Unknown);

        assert_eq!(data, error.data());
    }

}
