use crate::{
    cas::content::Address,
    entry::{entry_type::EntryType, Entry, ToEntry},
    error::HolochainError,
    json::JsonString,
};
use std::convert::TryInto;

//-------------------------------------------------------------------------------------------------
// LinkAddEntry
//-------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct DeletionEntry {
    deleted_entry_address: Address,
}

impl DeletionEntry {
    pub fn new(deleted_entry_address: Address) -> Self {
        DeletionEntry {
            deleted_entry_address,
        }
    }

    pub fn deleted_entry_address(self) -> Address {
        self.deleted_entry_address
    }
}

impl ToEntry for DeletionEntry {
    // Convert a LinkEntry into a JSON array of Links
    fn to_entry(&self) -> Entry {
        Entry::new(EntryType::Delete, self.to_owned())
    }

    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::Delete, entry.entry_type());
        entry
            .value()
            .to_owned()
            .try_into()
            .expect("could not convert Entry to DeletionEntry")
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        cas::content::AddressableContent,
        entry::{test_entry_a, ToEntry},
    };

    pub fn test_deletion_entry() -> DeletionEntry {
        let entry = test_entry_a();
        DeletionEntry::new(entry.address())
    }

    #[test]
    fn deletion_smoke_test() {
        assert_eq!(
            test_entry_a().address(),
            test_deletion_entry().deleted_entry_address()
        );
    }

    #[test]
    fn deletion_entry_to_entry_test() {
        assert_eq!(
            test_deletion_entry(),
            DeletionEntry::from_entry(&test_deletion_entry().to_entry()),
        );
    }
}
