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

impl From<CrudStatus> for String {
    fn from(crud_status: CrudStatus) -> String {
        String::from(match crud_status {
            CrudStatus::LIVE => "1",
            CrudStatus::REJECTED => "2",
            CrudStatus::DELETED => "4",
            CrudStatus::MODIFIED => "8",
            CrudStatus::LOCKED => "16",
            _ => unreachable!(),
        })
    }
}

impl From<&'static str> for CrudStatus {
    fn from(s: &str) -> CrudStatus {
        CrudStatus::from(String::from(s))
    }
}

impl From<String> for CrudStatus {
    fn from(s: String) -> CrudStatus {
        match s.as_ref() {
            "1" => CrudStatus::LIVE,
            "2" => CrudStatus::REJECTED,
            "4" => CrudStatus::DELETED,
            "8" => CrudStatus::MODIFIED,
            "16" => CrudStatus::LOCKED,
            _ => unreachable!(),
        }
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
            Address, AddressableContent, AddressableContentTestSuite, Content,
            ExampleAddressableContent,
        },
        storage::{test_content_addressable_storage, ExampleContentAddressableStorage},
    };
    use eav::eav_round_trip_test_runner;
    use json::{JsonString, RawString};

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
        let entity_content =
            ExampleAddressableContent::from_content(&JsonString::from(RawString::from("example")));
        let attribute = String::from("favourite-badge");
        let value_content: Content =
            CrudStatus::from_content(&JsonString::from(RawString::from("2"))).content();
        eav_round_trip_test_runner(entity_content, attribute, value_content);
    }

    #[test]
    /// show From<CrudStatus> implementation for String
    fn to_string_test() {
        assert_eq!(String::from("1"), String::from(CrudStatus::LIVE));
        assert_eq!(String::from("2"), String::from(CrudStatus::REJECTED));
        assert_eq!(String::from("4"), String::from(CrudStatus::DELETED));
        assert_eq!(String::from("8"), String::from(CrudStatus::MODIFIED));
        assert_eq!(String::from("16"), String::from(CrudStatus::LOCKED));
    }

    #[test]
    /// show From<String> and From<&'static str> implementation for CrudStatus
    fn from_string_test() {
        assert_eq!(CrudStatus::from("1"), CrudStatus::LIVE);
        assert_eq!(CrudStatus::from("2"), CrudStatus::REJECTED);
        assert_eq!(CrudStatus::from("4"), CrudStatus::DELETED);
        assert_eq!(CrudStatus::from("8"), CrudStatus::MODIFIED);
        assert_eq!(CrudStatus::from("16"), CrudStatus::LOCKED);

        assert_eq!(CrudStatus::from(String::from("1")), CrudStatus::LIVE);
        assert_eq!(CrudStatus::from(String::from("2")), CrudStatus::REJECTED);
        assert_eq!(CrudStatus::from(String::from("4")), CrudStatus::DELETED);
        assert_eq!(CrudStatus::from(String::from("8")), CrudStatus::MODIFIED);
        assert_eq!(CrudStatus::from(String::from("16")), CrudStatus::LOCKED);
    }

    #[test]
    /// show AddressableContent implementation
    fn addressable_content_test() {
        // from_content()
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(RawString::from("1")),
            CrudStatus::LIVE,
            Address::from("QmVaPTddRyjLjMoZnYufWc5M5CjyGNPmFEpp5HtPKEqZFG"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(RawString::from("2")),
            CrudStatus::REJECTED,
            Address::from("QmcdyB29uHtqMRZy47MrhaqFqHpHuPr7eUxWWPJbGpSRxg"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(RawString::from("4")),
            CrudStatus::DELETED,
            Address::from("QmTPwmaQtBLq9RXbvNyfj46X65YShYzMzn62FFbNYcieEm"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(RawString::from("8")),
            CrudStatus::MODIFIED,
            Address::from("QmRKuYmrQu1oMLHDyiA2v66upmEB5JLRqVhVEYXYYM5agi"),
        );
        AddressableContentTestSuite::addressable_content_trait_test::<CrudStatus>(
            JsonString::from(RawString::from("16")),
            CrudStatus::LOCKED,
            Address::from("QmaHXADi79HCmmGPYMmdqvyemChRmZPVGyEQYmo6oS2C3a"),
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
