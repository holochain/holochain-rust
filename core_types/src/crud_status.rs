use cas::content::{AddressableContent, Content};
use json::JsonString;
use serde_json;

// @TODO are these the correct key names?
// @see https://github.com/holochain/holochain-rust/issues/143
pub const STATUS_NAME: &str = "crud-status";
pub const LINK_NAME: &str = "crud-link";

bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    /// the CRUD status of a Pair is stored as EntryMeta in the hash table, NOT in the entry itself
    /// statuses are represented as bitflags so we can easily build masks for filtering lookups
    pub struct CrudStatus: u8 {
        const LIVE = 0x01;
        const REJECTED = 0x02;
        const DELETED = 0x04;
        const MODIFIED = 0x08;
        /// CRDT resolution in progress
        const LOCKED = 0x10;
    }
}

impl From<CrudStatus> for JsonString {
    fn from(crud_status: CrudStatus) -> JsonString {
        JsonString::from(serde_json::to_string(&crud_status).expect("failed to Jsonify CrudStatus"))
    }
}

impl From<JsonString> for CrudStatus {
    fn from(json_string: JsonString) -> CrudStatus {
        serde_json::from_str(&String::from(json_string)).expect("failed to deserialize CrudStatus")
    }
}

impl AddressableContent for CrudStatus {
    fn content(&self) -> Content {
        Content::from(self.to_owned())
    }

    fn from_content(content: &Content) -> Self {
        Self::from(content.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::CrudStatus;
    use cas::{
        content::{
            AddressableContent, AddressableContentTestSuite, Content, ExampleAddressableContent,
        },
        storage::{test_content_addressable_storage, ExampleContentAddressableStorage},
    };
    use eav::eav_round_trip_test_runner;

    #[test]
    /// test the CrudStatus bit flags as ints
    fn status_bits() {
        assert_eq!(CrudStatus::default().bits(), 0);
        assert_eq!(CrudStatus::all().bits(), 31);

        assert_eq!(CrudStatus::LIVE.bits(), 1);
        assert_eq!(CrudStatus::REJECTED.bits(), 2);
        assert_eq!(CrudStatus::DELETED.bits(), 4);
        assert_eq!(CrudStatus::MODIFIED.bits(), 8);
        assert_eq!(CrudStatus::LOCKED.bits(), 16);
    }

    #[test]
    /// test that we can build status masks from the CrudStatus bit flags
    fn bitwise() {
        let example_mask = CrudStatus::REJECTED | CrudStatus::DELETED;
        assert!(example_mask.contains(CrudStatus::REJECTED));
        assert!(example_mask.contains(CrudStatus::DELETED));
        assert!(!example_mask.contains(CrudStatus::LIVE));
        assert!(!example_mask.contains(CrudStatus::MODIFIED));
        assert!(!example_mask.contains(CrudStatus::LOCKED));
    }

    #[test]
    fn crud_status_example_eav() {
        let entity_content = ExampleAddressableContent::from_content(&"example".to_string());
        let attribute = "favourite-badge".to_string();
        let value_content: Content = CrudStatus::from_content(&String::from("2")).content();
        eav_round_trip_test_runner(entity_content, attribute, value_content);
    }

    #[test]
    /// show ToString implementation
    fn to_string_test() {
        assert_eq!("1".to_string(), CrudStatus::LIVE.to_string());
        assert_eq!("2".to_string(), CrudStatus::REJECTED.to_string());
        assert_eq!("4".to_string(), CrudStatus::DELETED.to_string());
        assert_eq!("8".to_string(), CrudStatus::MODIFIED.to_string());
        assert_eq!("16".to_string(), CrudStatus::LOCKED.to_string());
    }

    #[test]
    /// show From<String> implementation
    fn from_string_test() {
        assert_eq!(CrudStatus::from(&"1".to_string()), CrudStatus::LIVE);
        assert_eq!(CrudStatus::from(&"2".to_string()), CrudStatus::REJECTED);
        assert_eq!(CrudStatus::from(&"4".to_string()), CrudStatus::DELETED);
        assert_eq!(CrudStatus::from(&"8".to_string()), CrudStatus::MODIFIED);
        assert_eq!(CrudStatus::from(&"16".to_string()), CrudStatus::LOCKED);
    }

    #[test]
    /// show AddressableContent implementation
    fn addressable_content_test() {
        // from_content()
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            String::from("1"),
            CrudStatus::LIVE,
            String::from("QmVaPTddRyjLjMoZnYufWc5M5CjyGNPmFEpp5HtPKEqZFG"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            String::from("2"),
            CrudStatus::REJECTED,
            String::from("QmcdyB29uHtqMRZy47MrhaqFqHpHuPr7eUxWWPJbGpSRxg"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            String::from("4"),
            CrudStatus::DELETED,
            String::from("QmTPwmaQtBLq9RXbvNyfj46X65YShYzMzn62FFbNYcieEm"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            String::from("8"),
            CrudStatus::MODIFIED,
            String::from("QmRKuYmrQu1oMLHDyiA2v66upmEB5JLRqVhVEYXYYM5agi"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            String::from("16"),
            CrudStatus::LOCKED,
            String::from("QmaHXADi79HCmmGPYMmdqvyemChRmZPVGyEQYmo6oS2C3a"),
        );
    }

    #[test]
    /// show CAS round trip
    fn cas_round_trip_test() {
        let crud_statuses = vec![
            CrudStatus::LIVE,
            CrudStatus::REJECTED,
            CrudStatus::DELETED,
            CrudStatus::MODIFIED,
            CrudStatus::LOCKED,
        ];
        AddressableContentTestSuite::addressable_content_round_trip::<
            CrudStatus,
            ExampleContentAddressableStorage,
        >(crud_statuses, test_content_addressable_storage());
    }
}
