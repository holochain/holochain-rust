use keys::Key;

pub struct Agent {
    signing_key: Key,
    encryption_key: Key,
    dpki_root: Key,
}

impl Entry for Agent {
    fn entry_type(&self) -> &EntryType {
        &EntryType::Agent
    }
}
