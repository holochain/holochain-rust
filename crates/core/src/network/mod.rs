pub mod actions;
pub mod entry_header_pair;
pub mod direct_message;
pub mod handler;
pub mod reducers;
pub mod state;
#[cfg(test)]
pub mod test_utils;

pub use holochain_core_types::network::{entry_aspect, query};

#[cfg(test)]
pub mod tests {
    use crate::{
        agent::actions::commit::commit_entry,
        instance::tests::test_instance_and_context_by_name,
        network::{
            actions::{
                publish::publish,
                query::{query, QueryMethod},
            },
            query::{
                GetLinksNetworkQuery, GetLinksNetworkResult, GetLinksQueryConfiguration,
                NetworkQueryResult,
            },
            test_utils::test_wat_always_valid,
        },
    };
    use holochain_core_types::{
        agent::test_agent_id,
        chain_header::test_chain_header,
        crud_status::CrudStatus,
        entry::{entry_type::test_app_entry_type, test_entry, Entry, EntryWithMetaAndHeader},
        link::link_data::LinkData,
    };
    use holochain_json_api::json::JsonString;
    use holochain_persistence_api::cas::content::{Address, AddressableContent};
    use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
    use test_utils::*;

    // TODO: Bring the old in-memory network up to speed and turn on this test again!
    #[cfg(feature = "broken-tests")]
    // TODO do this for all crate tests somehow
    //dry this out
    #[allow(dead_code)]
    fn enable_logging_for_test() {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "trace");
        }
        let _ = env_logger::builder()
            .default_format_timestamp(false)
            .default_format_module_path(false)
            .is_test(true)
            .try_init();
    }

    // TODO: Should wait for a success or saturation response from the network module after Publish
    #[test]
    #[ignore]
    fn get_entry_roundtrip() {
        let netname = Some("get_entry_roundtrip");
        let mut dna = create_test_dna_with_wat("test_zome", None);
        dna.uuid = netname.unwrap().to_string();
        let (_instance1, context1) =
            test_instance_and_context_by_name(dna.clone(), "alice1", netname).unwrap();
        let (_instance2, context2) =
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
        let mut maybe_entry_with_meta: Option<EntryWithMetaAndHeader> = None;
        let mut loop_count = 0;
        while maybe_entry_with_meta.is_none() && loop_count < 10 {
            loop_count += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
            let result = context2.block_on(query(
                context2.clone(),
                QueryMethod::Entry(entry.address().clone()),
                Default::default(),
            ));
            assert!(result.is_ok(), "get_entry() result = {:?}", result);
            maybe_entry_with_meta = unwrap_to!(result.unwrap()=>NetworkQueryResult::Entry).clone();
        }
        assert!(
            maybe_entry_with_meta.is_some(),
            "maybe_entry_with_meta = {:?}",
            maybe_entry_with_meta
        );
        let entry_with_meta_and_header = maybe_entry_with_meta.unwrap();
        assert_eq!(entry_with_meta_and_header.entry_with_meta.entry, entry);
        assert_eq!(
            entry_with_meta_and_header.entry_with_meta.crud_status,
            CrudStatus::Live
        );
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
        let (_instance1, context1) =
            test_instance_and_context_by_name(dna.clone(), "alex", netname).unwrap();
        let (_instance2, context2) =
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
                dht1.add(&entry).unwrap();
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
        let (_instance1, _) =
            test_instance_and_context_by_name(dna.clone(), "alice2", netname).unwrap();
        let (_instance2, context2) =
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
    // flaky test
    //  this test failed on macOSx cold builds blocking on the get_entry
    //  adding a sleep after the publish would make it work, but that's flaky!
    #[cfg(feature = "broken-tests")]
    fn get_entry_when_alone() {
        let netname = Some("get_when_alone");
        let mut dna = create_test_dna_with_wat("test_zome", None);
        dna.uuid = netname.unwrap().to_string();
        let (_instance1, context1) =
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
        assert_eq!(entry_with_meta.entry_with_meta.entry, entry);
        assert_eq!(
            entry_with_meta.entry_with_meta.crud_status,
            CrudStatus::Live
        );
    }

    // TODO: Bring the old in-memory network up to speed and turn on this test again!
    #[cfg(feature = "broken-tests")]
    #[test]
    #[cfg(feature = "broken-tests")]
    fn get_validation_package_roundtrip() {
        enable_logging_for_test();

        let wat = &test_wat_always_valid();
        let mut dna = create_test_dna_with_wat("test_zome", Some(wat));
        dna.uuid = "get_validation_package_roundtrip".to_string();

        let (_instance1, context1) = test_instance_and_context_by_name(
            dna.clone(),
            "alice1",
            Some("get_validation_package_roundtrip"),
        )
        .unwrap();
        let (_instance2, context2) = test_instance_and_context_with_memory_network_nodes(
            dna.clone(),
            "bob1",
            Some("get_validation_package_roundtrip2"),
        )
        .unwrap();

        let entry = test_entry();
        context1
            .block_on(author_entry(&entry, None, &context1, &vec![]))
            .expect("Could not author entry");

        let agent1_state = context1.state().unwrap().agent();
        let header = agent1_state
            .get_most_recent_header_for_entry(&entry)
            .expect("There must be a header in the author's source chain after commit");

        let result = context2.block_on(get_validation_package(header.clone(), &context2));

        assert!(result.is_ok(), "actual result: {:?}", result);
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
        let (_instance1, context1) =
            test_instance_and_context_by_name(dna.clone(), "alex2", netname).unwrap();
        let (_instance2, context2) =
            test_instance_and_context_by_name(dna.clone(), "billy2", netname).unwrap();

        let mut entry_addresses: Vec<Address> = Vec::new();
        for i in 0..3 {
            let entry = Entry::App(
                test_app_entry_type(),
                JsonString::from_json(&format!("entry{} value", i)),
            );
            let address = context1
                .block_on(commit_entry(entry.clone(), None, &context1))
                .expect("Could not commit entry for testing");
            let _ = context1
                .block_on(publish(entry.address(), &context1))
                .expect("Could not publish entry for testing");
            entry_addresses.push(address);
        }

        let link1 = LinkData::new_add(
            &entry_addresses[0],
            &entry_addresses[1],
            "test-tag",
            "test-link",
            test_chain_header(),
            test_agent_id(),
        );
        let link2 = LinkData::new_add(
            &entry_addresses[0],
            &entry_addresses[2],
            "test-tag",
            "test-link",
            test_chain_header(),
            test_agent_id(),
        );

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
        let get_links_args = GetLinksArgs {
            entry_address: entry_addresses[0].clone(),
            link_type: "test-link".into(),
            tag: "test-tag".into(),
            options: Default::default(),
        };

        let config = GetLinksQueryConfiguration { headers: false };
        let method = QueryMethod::Link(get_links_args.clone(), GetLinksNetworkQuery::Links(config));
        let maybe_links = context2.block_on(query(context2.clone(), method, Default::default()));

        assert!(maybe_links.is_ok());
        let link_results = maybe_links.unwrap();
        let links = match link_results {
            NetworkQueryResult::Links(query, _, _) => query,
            _ => panic!("Could not get query"),
        };
        let links = unwrap_to!(links=>GetLinksNetworkResult::Links);
        assert_eq!(links.len(), 2, "links = {:?}", links);
        // can be in any order
        assert!(
            ((links[0].address.clone(), links[0].crud_status.clone())
                == (entry_addresses[1].clone(), CrudStatus::Live)
                || (links[0].address.clone(), links[0].crud_status.clone())
                    == (entry_addresses[2].clone(), CrudStatus::Live))
                && ((links[1].address.clone(), links[0].crud_status.clone())
                    == (entry_addresses[1].clone(), CrudStatus::Live)
                    || (links[1].address.clone(), links[0].crud_status.clone())
                        == (entry_addresses[2].clone(), CrudStatus::Live))
        );
    }
}
