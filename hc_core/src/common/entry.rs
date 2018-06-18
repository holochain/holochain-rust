#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    previous: Option<Box<Header>>,
    entry: Entry,
}

impl Header {
    pub fn new(previous: &Header, entry: &Entry) -> Self {
        Header {
            previous: match previous {
                &Header => Some(Box<previous.clone()>),
                None => None,
            },
            entry: entry.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {}

#[derive(Clone, Debug, PartialEq)]
pub struct Hash {}
