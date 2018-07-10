use super::MessageData;
use agent::keys::Keys;
use serde_json;

const NAME: &str = "ERROR_RESPONSE";

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
    pub fn code(&self) -> u8 {
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

#[derive(Clone, Serialize)]
pub struct ErrorData {
    code: u8,
    description: String,
}

impl ErrorData {

    pub fn code(&self) -> u8 {
        self.code
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

}

pub struct Error {
    error: ErrorData,
    data: MessageData,
}

impl Error {

    pub fn new(keys: &Keys, code: ErrorCode, description: &str) -> Error {
        let e = ErrorData{
            code: code.code(),
            description: String::from(description),
        };
        Error{
            error: e.clone(),
            data: MessageData::new(keys, NAME, &serde_json::to_string(&e).unwrap()),
        }
    }

    pub fn data(&self) -> MessageData {
        self.data.clone()
    }

    pub fn error(&self) -> ErrorData {
        self.error.clone()
    }

}

impl super::Message for Error {

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
    use super::ErrorCode;
    use super::Error;
    use agent::keys::tests::test_keys;

    pub fn test_error() -> Error {
        Error::new(&test_keys(), ErrorCode::Unknown, "test error description")
    }

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
        test_error();
    }

    #[test]
    fn code() {
        assert_eq!(0, Error::new(&test_keys(), ErrorCode::Unknown, "").error().code());
        assert_eq!(1, Error::new(&test_keys(), ErrorCode::HashNotFound, "").error().code());
        assert_eq!(2, Error::new(&test_keys(), ErrorCode::HashDeleted, "").error().code());
        assert_eq!(3, Error::new(&test_keys(), ErrorCode::HashModified, "").error().code());
        assert_eq!(4, Error::new(&test_keys(), ErrorCode::HashRejected, "").error().code());
        assert_eq!(5, Error::new(&test_keys(), ErrorCode::LinkNotFound, "").error().code());
        assert_eq!(6, Error::new(&test_keys(), ErrorCode::EntryTypeMismatch, "").error().code());
        assert_eq!(7, Error::new(&test_keys(), ErrorCode::BlockedListed, "").error().code());
    }

    #[test]
    fn name() {
        assert_eq!("ERROR_RESPONSE", test_error().name());
    }

}
