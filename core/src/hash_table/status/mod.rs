use cas::content::{AddressableContent, Content};

// @TODO are these the correct key names?
// @see https://github.com/holochain/holochain-rust/issues/143
pub const STATUS_NAME: &str = "crud-status";
pub const LINK_NAME: &str = "crud-link";

bitflags! {
    #[derive(Default)]
    /// the CRUD status of a Pair is stored as PairMeta in the hash table, NOT in the pair itself
    /// statuses are represented as bitflags so we can easily build masks for filtering lookups
    pub struct CrudStatus: u8 {
        const LIVE = 0x01;
        const REJECTED = 0x02;
        const DELETED = 0x04;
        const MODIFIED = 0x08;
    }
}

impl ToString for CrudStatus {
    fn to_string(&self) -> String {
        self.bits().to_string()
    }
}

impl<'a> From<&'a String> for CrudStatus {
    fn from(s: &String) -> CrudStatus {
        match s.as_ref() {
            "1" => CrudStatus::LIVE,
            "2" => CrudStatus::REJECTED,
            "4" => CrudStatus::DELETED,
            "8" => CrudStatus::MODIFIED,
            "255" => CrudStatus::ANY,
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
        eav::{
            tests::{eav_round_trip_test_runner, ExampleEntityAttributeValueStorage},
            EntityAttributeValue, EntityAttributeValueStorage,
        },
    };
    use std::collections::HashSet;

    #[test]
    /// test the CrudStatus bit flags as ints
    fn status_bits() {
        assert_eq!(CrudStatus::default().bits(), 0);
        assert_eq!(CrudStatus::all().bits(), 15);

        assert_eq!(CrudStatus::LIVE.bits(), 1);
        assert_eq!(CrudStatus::REJECTED.bits(), 2);
        assert_eq!(CrudStatus::DELETED.bits(), 4);
        assert_eq!(CrudStatus::MODIFIED.bits(), 8);
    }

    #[test]
    /// test that we can build status masks from the CrudStatus bit flags
    fn bitwise() {
        let example_mask = CrudStatus::REJECTED | CrudStatus::DELETED;
        assert!(example_mask.contains(CrudStatus::REJECTED));
        assert!(example_mask.contains(CrudStatus::DELETED));
        assert!(!example_mask.contains(CrudStatus::LIVE));
        assert!(!example_mask.contains(CrudStatus::MODIFIED));
    }

    #[test]
    fn crud_status_eav() {
        let zip_crud: Vec<(String, CrudStatus)> = vec![
            (String::from("1"), CrudStatus::LIVE),
            (String::from("2"), CrudStatus::REJECTED),
            (String::from("4"), CrudStatus::DELETED),
            (String::from("8"), CrudStatus::MODIFIED),
            (String::from("255"), CrudStatus::ANY),
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

}
