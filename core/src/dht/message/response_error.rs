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
    body: String,
    time: String,
    from: String,
}

impl Error {

    pub fn new(code: ErrorCode, body: &str) -> Error {
        Error{
            code: code.code(),
            body: String::from(body),
            time: String::new(),
            from: String::new(),
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

    fn time(&self) -> String {
        self.time.clone()
    }

    fn from(&self) -> String {
        self.from.clone()
    }

    fn body(&self) -> String {
        self.body.clone()
    }

}

#[cfg(test)]
mod tests {
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
    fn code() {
        // type code is always 0 for error messages
        assert_eq!(0, Error::new(ErrorCode::Unknown, "").code());
        assert_eq!(1, Error::new(ErrorCode::HashNotFound, "").code());
        assert_eq!(2, Error::new(ErrorCode::HashDeleted, "").code());
        assert_eq!(3, Error::new(ErrorCode::HashModified, "").code());
        assert_eq!(4, Error::new(ErrorCode::HashRejected, "").code());
        assert_eq!(5, Error::new(ErrorCode::LinkNotFound, "").code());
        assert_eq!(6, Error::new(ErrorCode::EntryTypeMismatch, "").code());
        assert_eq!(7, Error::new(ErrorCode::BlockedListed, "").code());
        assert_eq!(0, Error::new(ErrorCode::Unknown, "foo").code());
    }

    #[test]
    fn type_name() {
        assert_eq!(super::NAME, Error::new(ErrorCode::Unknown, "").type_name());
    }

    #[test]
    fn type_code() {
        // type code is always 0 for error messages
        assert_eq!(0, Error::new(ErrorCode::Unknown, "").type_code());
        assert_eq!(0, Error::new(ErrorCode::HashNotFound, "").type_code());
        assert_eq!(0, Error::new(ErrorCode::HashDeleted, "").type_code());
        assert_eq!(0, Error::new(ErrorCode::HashModified, "").type_code());
        assert_eq!(0, Error::new(ErrorCode::HashRejected, "").type_code());
        assert_eq!(0, Error::new(ErrorCode::LinkNotFound, "").type_code());
        assert_eq!(0, Error::new(ErrorCode::EntryTypeMismatch, "").type_code());
        assert_eq!(0, Error::new(ErrorCode::BlockedListed, "").type_code());
        assert_eq!(0, Error::new(ErrorCode::Unknown, "foo").type_code());
    }

}
