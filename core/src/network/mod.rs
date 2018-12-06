pub mod actions;
pub mod entry_with_header;
pub mod handler;
pub mod reducers;
pub mod state;

#[cfg(test)]
pub mod tests {
    use crate::{
        instance::tests::test_instance_and_context_by_name, network::actions::get_entry::get_entry,
    };
    use futures::executor::block_on;
    use holochain_core_types::{cas::content::AddressableContent, entry::test_entry};
    use test_utils::*;

    #[test]
    fn get_entry_roundtrip() {
        let mut dna = create_test_dna_with_wat("test_zome", "test_cap", None);
        dna.uuid = String::from("get_entry_roundtrip");
        let (_, context1) = test_instance_and_context_by_name(dna.clone(), "alice1").unwrap();
        let (_, context2) = test_instance_and_context_by_name(dna.clone(), "bob1").unwrap();

        let entry = test_entry();
        assert!(context1.file_storage.write().unwrap().add(&entry).is_ok());

        let result = block_on(get_entry(&context2, &entry.address()));
        assert!(result.is_ok());
        let maybe_entry_with_meta = result.unwrap();
        assert!(maybe_entry_with_meta.is_some());
        let entry_with_meta = maybe_entry_with_meta.unwrap();
        assert_eq!(entry_with_meta.entry, entry);
    }

    #[test]
    fn get_non_existant_entry() {
        let mut dna = create_test_dna_with_wat("test_zome", "test_cap", None);
        dna.uuid = String::from("get_non_existant_entry");
        let (_, _) = test_instance_and_context_by_name(dna.clone(), "alice2").unwrap();
        let (_, context2) = test_instance_and_context_by_name(dna.clone(), "bob2").unwrap();

        let entry = test_entry();

        let result = block_on(get_entry(&context2, &entry.address()));
        assert!(result.is_ok());
        let maybe_entry_with_meta = result.unwrap();
        assert!(maybe_entry_with_meta.is_none());
    }

    #[test]
    fn get_when_alone() {
        let mut dna = create_test_dna_with_wat("test_zome", "test_cap", None);
        dna.uuid = String::from("get_when_alone");
        let (_, context1) = test_instance_and_context_by_name(dna.clone(), "bob3").unwrap();

        let entry = test_entry();

        let result = block_on(get_entry(&context1, &entry.address()));
        assert!(result.is_ok());
        let maybe_entry_with_meta = result.unwrap();
        assert!(maybe_entry_with_meta.is_none());
    }
}
