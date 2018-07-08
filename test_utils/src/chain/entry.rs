use holochain_core::chain::entry::Entry;

pub fn test_entry() -> Entry {
    Entry::new("fooType", "some content")
}
