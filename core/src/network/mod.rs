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
        instance::tests::test_instance_and_context_by_name,
        network::{
            actions::{
                get_entry::get_entry, get_links::get_links,
                get_validation_package::get_validation_package, publish::publish,
            },
            test_utils::test_wat_always_valid,
        },
        workflows::author_entry::author_entry,
    };
    use holochain_core_types::{
        cas::content::{Address, AddressableContent},
        crud_status::CrudStatus,
        entry::{entry_type::test_app_entry_type, test_entry, Entry, EntryWithMeta},
        link::link_data::LinkData,
        chain_header::ChainHeader
    };
    use test_utils::*;

    // TODO: Should wait for a success or saturation response from the network module after Publish
    #[test]
    #[ignore]
    fn get_entry_roundtrip() {
        let netname = Some("get_entry_roundtrip");
        let mut dna = create_test_dna_with_wat("test_zome", None);
        dna.uuid = netname.unwrap().to_string();
        let (_, context1) =
            test_instance_and_context_by_name(dna.clone(), "alice1", netname).unwrap();
        let (_, context2) =
            test_instance_and_context_by_name(dna.clone(), "bob1", netname).unwrap();

        // Create Entry & metadata
        let entry = test_entry();

        // Store it on the network
        let result = context1.block_on(commit_entry(entry.clone(), None, &context1));
        assert!(result.is_ok(), "commit_entry() result = {:?}", result);
        let result = context1.block_on(publish(entry.address(), &context1));
        assert!(result.is_ok(), "publish() result = {:?}", result);

        // TODO: Should wait for a success or saturation response from the network module instead
        // std::thread::sleep(std::time::Duration::from_millis(2000));

        // Get it from the network
        // HACK: doing a loop because publish returns before actual confirmation from the network
        let mut maybe_entry_with_meta: Option<(EntryWithMeta,Vec<ChainHeader>)> = None;
        let mut loop_count = 0;
        while maybe_entry_with_meta.is_none() && loop_count < 10 {
            loop_count += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
            let result = context2.block_on(get_entry(
                context2.clone(),
                entry.address(),
                Default::default(),
            ));
            assert!(result.is_ok(), "get_entry() result = {:?}", result);
            maybe_entry_with_meta = result.unwrap();
        }
        assert!(
            maybe_entry_with_meta.is_some(),
            "maybe_entry_with_meta = {:?}",
            maybe_entry_with_meta
        );
        let entry_with_meta = maybe_entry_with_meta.unwrap().0;
        assert_eq!(entry_with_meta.entry, entry);
        assert_eq!(entry_with_meta.crud_status, CrudStatus::Live);
    }

    #[test]
    // flaky test
    // https://circleci.com/gh/holochain/holochain-rust/12091
    // timestamps are not being created deterministically
    #[cfg(feature = "broken-tests")]
    fn get_entry_results_roundtrip() {
        let netname = Some("get_entry_results_roundtrip");
        let mut dna = create_test_dna_with_wat("test_zome", None);
        dna.uuid = netname.unwrap().to_string();
        let (_, context1) =
            test_instance_and_context_by_name(dna.clone(), "alex", netname).unwrap();
        let (_, context2) =
            test_instance_and_context_by_name(dna.clone(), "billy", netname).unwrap();

        // Create Entry & crud-status metadata, and store it.
        let entry = test_entry();
        let header1 = create_new_chain_header(&entry, context1.clone(), &None).unwrap();
        let header2 = create_new_chain_header(&entry, context2.clone(), &None).unwrap();
        context1
            .block_on(commit_entry(entry.clone(), None, &context1))
            .unwrap();
        {
            let dht1 = context1.state().unwrap().dht();
            {
                dht1.content_storage().write().unwrap().add(&entry).unwrap();
                dht1.add_header_for_entry(&entry, &header2).unwrap();
            }
        }

        // Get it.
        let args = GetEntryArgs {
            address: entry.address(),
            options: GetEntryOptions {
                headers: true,
                ..Default::default()
            },
        };
        let result = context1.block_on(get_entry_result_workflow(&context1, &args));
        if let GetEntryResultType::Single(item) = result.unwrap().result {
            let headers = item.headers;
            assert_eq!(headers, vec![header1, header2]);
        }
    }

    #[test]
    // flaky test
    // see https://circleci.com/gh/holochain/holochain-rust/10027
    #[cfg(feature = "broken-tests")]
    fn get_non_existant_entry() {
        let netname = Some("get_non_existant_entry");
        let mut dna = create_test_dna_with_wat("test_zome", None);
        dna.uuid = netname.unwrap().to_string();
        let (_, _) = test_instance_and_context_by_name(dna.clone(), "alice2", netname).unwrap();
        let (_, context2) =
            test_instance_and_context_by_name(dna.clone(), "bob2", netname).unwrap();

        let entry = test_entry();

        let result = context2.block_on(get_entry(
            context2.clone(),
            entry.address(),
            Timeout::new(100),
        ));
        assert!(result.is_ok(), "get_entry() result = {:?}", result);
        let maybe_entry_with_meta = result.unwrap();
        assert!(maybe_entry_with_meta.is_none());
    }

    #[test]
    fn get_entry_when_alone() {
        let netname = Some("get_when_alone");
        let mut dna = create_test_dna_with_wat("test_zome", None);
        dna.uuid = netname.unwrap().to_string();
        let (_, context1) =
            test_instance_and_context_by_name(dna.clone(), "bob3", netname).unwrap();

        // Create Entry
        let entry = test_entry();

        // Store it on the network
        let result = context1.block_on(commit_entry(entry.clone(), None, &context1));
        assert!(result.is_ok(), "commit_entry() result = {:?}", result);
        let result = context1.block_on(publish(entry.address(), &context1));
        assert!(result.is_ok(), "publish() result = {:?}", result);

        // Get it from the network
        let result = context1.block_on(get_entry(
            context1.clone(),
            entry.address(),
            Default::default(),
        ));
        assert!(result.is_ok(), "get_entry() result = {:?}", result);
        let maybe_entry_with_meta = result.unwrap();
        assert!(maybe_entry_with_meta.is_some());
        let entry_with_meta = maybe_entry_with_meta.unwrap();
        assert_eq!(entry_with_meta.0.entry, entry);
        assert_eq!(entry_with_meta.0.crud_status, CrudStatus::Live);
    }

    #[test]
    fn get_validation_package_roundtrip() {
        let netname = Some("get_validation_package_roundtrip");
        let wat = &test_wat_always_valid();
        let mut dna = create_test_dna_with_wat("test_zome", Some(wat));
        dna.uuid = netname.unwrap().to_string();

        let (_, context1) =
            test_instance_and_context_by_name(dna.clone(), "alice1", netname).unwrap();

        let entry = test_entry();
        context1
            .block_on(author_entry(&entry, None, &context1))
            .expect("Could not author entry");

        let agent1_state = context1.state().unwrap().agent();
        let header = agent1_state
            .get_most_recent_header_for_entry(&entry)
            .expect("There must be a header in the author's source chain after commit");

        let (_, context2) =
            test_instance_and_context_by_name(dna.clone(), "bob1", netname).unwrap();
        let result = context2.block_on(get_validation_package(header.clone(), &context2));

        assert!(result.is_ok());
        let maybe_validation_package = result.unwrap();
        assert!(maybe_validation_package.is_some());
        let validation_package = maybe_validation_package.unwrap();
        assert_eq!(validation_package.chain_header, header);
    }

    // TODO: Should wait for a success or saturation response from the network module after Publish
    #[test]
    #[ignore]
    fn get_links_roundtrip() {
        let netname = Some("get_links_roundtrip");
        let wat = &test_wat_always_valid();
        let mut dna = create_test_dna_with_wat("test_zome", Some(wat));
        dna.uuid = netname.unwrap().to_string();
        let (_, context1) =
            test_instance_and_context_by_name(dna.clone(), "alex2", netname).unwrap();
        let (_, context2) =
            test_instance_and_context_by_name(dna.clone(), "billy2", netname).unwrap();

        let mut entry_addresses: Vec<Address> = Vec::new();
        for i in 0..3 {
            let entry = Entry::App(test_app_entry_type(), format!("entry{} value", i).into());
            let address = context1
                .block_on(commit_entry(entry.clone(), None, &context1))
                .expect("Could not commit entry for testing");
            let _ = context1
                .block_on(publish(entry.address(), &context1))
                .expect("Could not publish entry for testing");
            entry_addresses.push(address);
        }

        let link1 = LinkData::new_add(&entry_addresses[0], &entry_addresses[1], "test-tag");
        let link2 = LinkData::new_add(&entry_addresses[0], &entry_addresses[2], "test-tag");

        // Store link1 on the network
        println!("\n add_link(link1) ...");
        let entry = Entry::LinkAdd(link1);
        let result = context1.block_on(commit_entry(entry.clone(), None, &context1));
        assert!(result.is_ok(), "commit_entry() result = {:?}", result);
        let result = context1.block_on(publish(entry.address(), &context1));
        assert!(result.is_ok(), "publish() result = {:?}", result);

        // Store link2 on the network
        println!("\n add_link(link2) ...");
        let entry = Entry::LinkAdd(link2);
        let result = context1.block_on(commit_entry(entry.clone(), None, &context1));
        assert!(result.is_ok(), "commit_entry() result = {:?}", result);
        let result = context1.block_on(publish(entry.address(), &context1));
        assert!(result.is_ok(), "publish() result = {:?}", result);

        // TODO: Should wait for a success or saturation response from the network module instead
        // std::thread::sleep(std::time::Duration::from_millis(1000));

        println!("\n get_links() ...");
        let maybe_links = context2.block_on(get_links(
            context2.clone(),
            entry_addresses[0].clone(),
            String::from("test-tag"),
            Default::default(),
        ));

        assert!(maybe_links.is_ok());
        let links = maybe_links.unwrap();
        assert_eq!(links.len(), 2, "links = {:?}", links);
        // can be in any order
        assert!(
            (links[0] == entry_addresses[1] || links[0] == entry_addresses[2])
                && (links[1] == entry_addresses[1] || links[1] == entry_addresses[2])
        );
    }
}
