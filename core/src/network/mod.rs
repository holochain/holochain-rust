pub mod actions;
pub mod direct_message;
pub mod entry_with_header;
pub mod handler;
pub mod reducers;
pub mod state;
#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
pub mod tests {
    use crate::{
        agent::actions::commit::commit_entry,
        dht::actions::add_link::add_link,
        instance::tests::test_instance_and_context_by_name,
        network::{
            actions::{
                get_entry::get_entry, get_links::get_links,
                get_validation_package::get_validation_package,
            },
            test_utils::test_wat_always_valid,
        },
        workflows::author_entry::author_entry,
    };
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::{Address, AddressableContent},
        crud_status::{create_crud_status_eav, CrudStatus},
        entry::{entry_type::test_app_entry_type, test_entry, Entry},
        link::Link,
    };
    use test_utils::*;

    #[test]
    fn get_entry_roundtrip() {
        let mut dna = create_test_dna_with_wat("test_zome", "test_cap", None);
        dna.uuid = String::from("get_entry_roundtrip");
        let (_, context1) = test_instance_and_context_by_name(dna.clone(), "alice1").unwrap();
        let (_, context2) = test_instance_and_context_by_name(dna.clone(), "bob1").unwrap();

        // Create Entry & crud-status metadata, and store it.
        let entry = test_entry();
        let result = context1.dht_storage.write().unwrap().add(&entry);
        assert!(result.is_ok());
        let status_eav = create_crud_status_eav(&entry.address(), CrudStatus::Live)
            .expect("Could not create EAV");
        let result = context1.eav_storage.write().unwrap().add_eav(&status_eav);
        assert!(result.is_ok());

        // Get it.
        let result = block_on(get_entry(&context2, &entry.address()));
        assert!(result.is_ok());
        let maybe_entry_with_meta = result.unwrap();
        assert!(maybe_entry_with_meta.is_some());
        let entry_with_meta = maybe_entry_with_meta.unwrap();
        assert_eq!(entry_with_meta.entry, entry);
        assert_eq!(entry_with_meta.crud_status, CrudStatus::Live);
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

    #[test]
    fn get_validation_package_roundtrip() {
        let wat = &test_wat_always_valid();

        let mut dna = create_test_dna_with_wat("test_zome", "test_cap", Some(wat));
        dna.uuid = String::from("get_validation_package_roundtrip");
        let (_, context1) = test_instance_and_context_by_name(dna.clone(), "alice1").unwrap();

        let entry = test_entry();
        block_on(author_entry(&entry, None, &context1)).expect("Could not author entry");

        let agent1_state = context1.state().unwrap().agent();
        let header = agent1_state
            .get_header_for_entry(&entry)
            .expect("There must be a header in the author's source chain after commit");

        let (_, context2) = test_instance_and_context_by_name(dna.clone(), "bob1").unwrap();
        let result = block_on(get_validation_package(header.clone(), &context2));

        assert!(result.is_ok());
        let maybe_validation_package = result.unwrap();
        assert!(maybe_validation_package.is_some());
        let validation_package = maybe_validation_package.unwrap();
        assert_eq!(validation_package.chain_header, Some(header));
    }

    #[test]
    fn get_links_roundtrip() {
        let wat = &test_wat_always_valid();

        let mut dna = create_test_dna_with_wat("test_zome", "test_cap", Some(wat));
        dna.uuid = String::from("get_links_roundtrip");
        let (_, context1) = test_instance_and_context_by_name(dna.clone(), "alice1").unwrap();

        let mut entry_addresses: Vec<Address> = Vec::new();
        for i in 0..3 {
            let entry = Entry::App(test_app_entry_type(), format!("entry{} value", i).into());
            let address = block_on(commit_entry(entry, None, &context1))
                .expect("Could not commit entry for testing");
            entry_addresses.push(address);
        }

        let link1 = Link::new(&entry_addresses[0], &entry_addresses[1], "test-tag");
        let link2 = Link::new(&entry_addresses[0], &entry_addresses[2], "test-tag");

        assert!(block_on(add_link(&link1, &context1)).is_ok());
        assert!(block_on(add_link(&link2, &context1)).is_ok());

        let (_, context2) = test_instance_and_context_by_name(dna.clone(), "bob1").unwrap();

        let maybe_links = block_on(get_links(
            &context2,
            &entry_addresses[0],
            String::from("test-tag"),
        ));

        assert!(maybe_links.is_ok());
        let links = maybe_links.unwrap();
        // can be in any order
        assert!(
            (links[0] == entry_addresses[1] || links[0] == entry_addresses[2]) &&
                (links[1] == entry_addresses[1] || links[1] == entry_addresses[2])
        );
    }
}
