use cas::content::{AddressableContent, Content};

// @TODO are these the correct key names?
// @see https://github.com/holochain/holochain-rust/issues/143
pub const STATUS_NAME: &str = "crud-status";
pub const LINK_NAME: &str = "crud-link";

bitflags! {
    #[derive(Default)]
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

impl ToString for CrudStatus {
    fn to_string(&self) -> String {
        // don't do self.bits().to_string() because that spits out values for default() and all()
        // only explicit statuses are safe as strings
        // the expectation is that strings will be stored, referenced and parsed
        match self.to_owned() {
            CrudStatus::LIVE => "1",
            CrudStatus::REJECTED => "2",
            CrudStatus::DELETED => "4",
            CrudStatus::MODIFIED => "8",
            CrudStatus::LOCKED => "16",
            _ => unreachable!(),
        }.to_string()
    }
}

impl<'a> From<&'a String> for CrudStatus {
    fn from(s: &String) -> CrudStatus {
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

impl AddressableContent for CrudStatus {
    fn content(&self) -> Content {
        self.to_string()
    }

    fn from_content(content: &Content) -> Self {
        CrudStatus::from(content)
    }
}

#[cfg(test)]
mod tests {
    use super::CrudStatus;
    use cas::{
        content::{tests::ExampleAddressableContent, AddressableContent, Content},
        eav::tests::eav_round_trip_test_runner,
        storage::{tests::ExampleContentAddressableStorage, ContentAddressableStorage},
    };
    use hash::HashString;

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
    fn crud_status_eav() {
        let zip_crud: Vec<(String, CrudStatus)> = vec![
            (String::from("1"), CrudStatus::LIVE),
            (String::from("2"), CrudStatus::REJECTED),
            (String::from("4"), CrudStatus::DELETED),
            (String::from("8"), CrudStatus::MODIFIED),
            (String::from("16"), CrudStatus::LOCKED),
        ];
        zip_crud
            .into_iter()
            .map(|c| {
                assert_eq!(CrudStatus::from_content(&c.0).content(), c.1.to_string());
            })
            .collect::<Vec<_>>();
    }

    #[test]
    fn crud_status_example_eav() {
        let entity_content = ExampleAddressableContent::from_content(&"example".to_string());
        let attribute = "favourite-badge".to_string();
        let value_content: Content = CrudStatus::from_content(&String::from("2")).content();
        eav_round_trip_test_runner(entity_content, attribute, value_content);
    }

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
        assert_eq!(CrudStatus::from_content(&"1".to_string()), CrudStatus::LIVE);
        assert_eq!(
            CrudStatus::from_content(&"2".to_string()),
            CrudStatus::REJECTED
        );
        assert_eq!(
            CrudStatus::from_content(&"4".to_string()),
            CrudStatus::DELETED
        );
        assert_eq!(
            CrudStatus::from_content(&"8".to_string()),
            CrudStatus::MODIFIED
        );
        assert_eq!(
            CrudStatus::from_content(&"16".to_string()),
            CrudStatus::LOCKED
        );

        // content()
        assert_eq!("1".to_string(), CrudStatus::LIVE.content());
        assert_eq!("2".to_string(), CrudStatus::REJECTED.content());
        assert_eq!("4".to_string(), CrudStatus::DELETED.content());
        assert_eq!("8".to_string(), CrudStatus::MODIFIED.content());
        assert_eq!("16".to_string(), CrudStatus::LOCKED.content());

        // address()
        assert_eq!(
            HashString::from("QmVaPTddRyjLjMoZnYufWc5M5CjyGNPmFEpp5HtPKEqZFG".to_string()),
            CrudStatus::LIVE.address()
        );
        assert_eq!(
            HashString::from("QmcdyB29uHtqMRZy47MrhaqFqHpHuPr7eUxWWPJbGpSRxg".to_string()),
            CrudStatus::REJECTED.address()
        );
        assert_eq!(
            HashString::from("QmTPwmaQtBLq9RXbvNyfj46X65YShYzMzn62FFbNYcieEm".to_string()),
            CrudStatus::DELETED.address()
        );
        assert_eq!(
            HashString::from("QmRKuYmrQu1oMLHDyiA2v66upmEB5JLRqVhVEYXYYM5agi".to_string()),
            CrudStatus::MODIFIED.address()
        );
        assert_eq!(
            HashString::from("QmaHXADi79HCmmGPYMmdqvyemChRmZPVGyEQYmo6oS2C3a".to_string()),
            CrudStatus::LOCKED.address()
        );
    }

    #[test]
    /// show CAS round trip
    fn cas_round_trip_test() {
        let mut content_addressable_storage = ExampleContentAddressableStorage::new();
        content_addressable_storage
            .add(&CrudStatus::LIVE)
            .expect("could not add LIVE");
        content_addressable_storage
            .add(&CrudStatus::REJECTED)
            .expect("could not add REJECTED");
        content_addressable_storage
            .add(&CrudStatus::DELETED)
            .expect("could not add DELETED");
        content_addressable_storage
            .add(&CrudStatus::MODIFIED)
            .expect("could not add MODIFIED");
        content_addressable_storage
            .add(&CrudStatus::LOCKED)
            .expect("could not add LOCKED");

        assert_eq!(
            Some(CrudStatus::LIVE),
            content_addressable_storage
                .fetch(&CrudStatus::LIVE.address())
                .expect("could not fetch LIVE"),
        );
        assert_eq!(
            Some(CrudStatus::REJECTED),
            content_addressable_storage
                .fetch(&CrudStatus::REJECTED.address())
                .expect("could not fetch REJECTED"),
        );
        assert_eq!(
            Some(CrudStatus::DELETED),
            content_addressable_storage
                .fetch(&CrudStatus::DELETED.address())
                .expect("could not fetch DELETED"),
        );
        assert_eq!(
            Some(CrudStatus::MODIFIED),
            content_addressable_storage
                .fetch(&CrudStatus::MODIFIED.address())
                .expect("could not fetch MODIFIED"),
        );
        assert_eq!(
            Some(CrudStatus::LOCKED),
            content_addressable_storage
                .fetch(&CrudStatus::LOCKED.address())
                .expect("could not fetch LOCKED"),
        );
    }
}
