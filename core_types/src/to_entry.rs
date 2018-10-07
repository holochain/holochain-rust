use entry::Entry;
use entry_type::EntryType;

pub trait ToEntry {
    fn to_entry(&self) -> (EntryType, Entry);
    fn from_entry(&Entry) -> Self;
}
