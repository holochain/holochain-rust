pub mod actions;
pub mod handler;
pub mod reducers;
pub mod state;
mod util;

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
        let dna = create_test_dna_with_wat("test_zome", "test_cap", None);
        let (_, context1) = test_instance_and_context_by_name(dna.clone(), "alice").unwrap();
        let (_, context2) = test_instance_and_context_by_name(dna.clone(), "bob").unwrap();

        let entry = test_entry();
        assert!(context1.file_storage.write().unwrap().add(&entry).is_ok());

        let result = block_on(get_entry(entry.address(), &context2));
        assert!(result.is_ok());
        let maybe_entry = result.unwrap();
        assert!(maybe_entry.is_some());
        let received_entry = maybe_entry.unwrap();
        assert_eq!(received_entry, entry);
    }

    #[test]
    fn get_non_existant_entry() {
        let dna = create_test_dna_with_wat("test_zome", "test_cap", None);
        let (_, _) = test_instance_and_context_by_name(dna.clone(), "alice").unwrap();
        let (_, context2) = test_instance_and_context_by_name(dna.clone(), "bob").unwrap();

        let entry = test_entry();

        let result = block_on(get_entry(entry.address(), &context2));
        assert!(result.is_ok());
        let maybe_entry = result.unwrap();
        assert!(maybe_entry.is_none());
    }

    #[test]
    fn get_when_alone() {
        let dna = create_test_dna_with_wat("test_zome", "test_cap", None);
        let (_, context1) = test_instance_and_context_by_name(dna.clone(), "bob").unwrap();

        let entry = test_entry();

        let result = block_on(get_entry(entry.address(), &context1));
        assert!(result.is_ok());
        let maybe_entry = result.unwrap();
        assert!(maybe_entry.is_none());
    }
}
