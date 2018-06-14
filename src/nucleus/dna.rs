#[derive(Clone, Debug)]
pub struct Code {
    format: String,
    language: String,
    code: String,
}

#[derive(Clone, Debug)]
pub struct Entry {}

#[derive(Clone, Debug)]
pub struct Zome {
    entry_definitions: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DNA {}
