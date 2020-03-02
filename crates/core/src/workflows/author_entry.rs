use crate::{
    agent::actions::commit::commit_entry,
    context::Context,
    entry::CanPublish,
    network::actions::{publish::publish, publish_header_entry::publish_header_entry},
    nucleus::{
        actions::build_validation_package::build_validation_package, validation::validate_entry,
    },

};
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    signature::Provenance,
    validation::{EntryLifecycle, ValidationData, ValidationResult},
};

use holochain_persistence_api::cas::content::{Address, AddressableContent};

use holochain_wasm_types::commit_entry::CommitEntryResult;

use crate::workflows::callback::links_utils::get_link_entries;
use std::{sync::Arc, vec::Vec};

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn author_entry<'a>(
    entry: &'a Entry,
    maybe_link_update_delete: Option<Address>,
    context: &'a Arc<Context>,
    provenances: &'a Vec<Provenance>,
) -> Result<CommitEntryResult, HolochainError> {
    let address = entry.address();
    log_debug!(
        context,
        "workflow/authoring_entry: {} with content: {:?}",
        address,
        entry
    );

    // 0. If we are trying to author a link or link removal, make sure the linked entries exist:
    if let Entry::LinkAdd(link_data) = entry {
        get_link_entries(Arc::clone(&context), &link_data.link)?;
    }
    if let Entry::LinkRemove((link_data, _)) = entry {
        get_link_entries(Arc::clone(&context), &link_data.link)?;
    }

    // 1. Build the context needed for validation of the entry
    let validation_package = build_validation_package(Arc::clone(&context), &entry, provenances)?;
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Chain,
    };

    // 2. Validate the entry
    log_debug!(
        context,
        "workflow/authoring_entry/{}: validating...",
        address
    );
    match validate_entry(
        Arc::clone(&context),
        entry.clone(),
        maybe_link_update_delete.clone(),
        validation_data,
    )
    .await {
        ValidationResult::Ok => (),
        err => return Err(HolochainError::ValidationFailed(err)),
    };
    log_debug!(context, "worflow/authoring_entry {}: is valid!", address);

    // 3. Commit the entry
    log_debug!(
        context,
        "workflow/authoring_entry/{}: committing...",
        address
    );
    let addr = commit_entry(entry.clone(), maybe_link_update_delete, &context).await?;
    log_debug!(context, "workflow/authoring_entry/{}: committed", address);

    // 4. Publish the valid entry to DHT. This will call Hold to itself
    if entry.entry_type().can_publish(context) {
        log_debug!(
            context,
            "workflow/authoring_entry/{}: publishing...",
            address
        );
        publish(entry.address(), &context).await?;
        log_debug!(context, "workflow/authoring_entry/{}: published!", address);
    } else {
        log_debug!(
            context,
            "workflow/authoring_entry/{}: entry is private, no publishing",
            address
        );
    }

    // 5. Publish the header for all types (including private entries)
    log_debug!(
        context,
        "debug/workflow/authoring_entry/{}: publishing header...",
        address
    );
    publish_header_entry(entry.address(), &context).await?;
    log_debug!(
        context,
        "debug/workflow/authoring_entry/{}: header published!",
        address
    );

    Ok(CommitEntryResult::new(addr))
}
// TODO: Bring the old in-memory network up to speed and turn on this test again!

#[cfg(test)]
pub mod tests {

    use crate::{
        holochain_wasm_types::holochain_persistence_api::cas::content::AddressableContent,
        nucleus::actions::{
            get_entry::get_entry_from_dht,
            tests::{instance_by_name, test_dna},
        },
        workflows::author_entry::author_entry,
    };
    use holochain_core_types::{
        chain_header::ChainHeader,
        entry::{test_entry_with_value, Entry},
    };
    use std::{thread, time};

    // TODO do this for all crate tests somehow
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

    #[test]
    /// test that a commit will publish and entry to the dht of a connected instance via the in-memory network
    fn test_commit_with_dht_publish() {
        enable_logging_for_test();
        let mut dna = test_dna();
        dna.uuid = "test_commit_with_dht_publish".to_string();
        let netname = Some("test_commit_with_dht_publish, the network");
        let (_instance1, context1) = instance_by_name("jill", dna.clone(), netname.clone());
        let (_instance2, context2) = instance_by_name("jack", dna, netname);

        let entry_address = context1
            .block_on(author_entry(
                &test_entry_with_value("{\"stuff\":\"test entry value\"}"),
                None,
                &context1,
                &vec![],
            ))
            .unwrap()
            .address();
        thread::sleep(time::Duration::from_millis(1000));

        let mut entry: Option<Entry> = None;
        let mut tries = 0;
        while entry.is_none() && tries < 10 {
            tries = tries + 1;
            {
                entry = get_entry_from_dht(Arc::clone(&context2), &entry_address)
                    .expect("Could not retrieve entry from DHT");
            }
            println!("Try {}: {:?}", tries, entry);
            if entry.is_none() {
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
        assert_eq!(
            entry,
            Some(test_entry_with_value("{\"stuff\":\"test entry value\"}"))
        );
    }

    #[test]
    /// test that the header of an entry can be retrieved directly by its hash by another agent connected
    /// via the in-memory network
    fn test_commit_with_dht_publish_header_is_published() {
        let mut dna = test_dna();
        dna.uuid = "test_commit_with_dht_publish_header_is_published".to_string();
        let netname = Some("test_commit_with_dht_publish_header_is_published, the network");
        let (_instance1, context1) = instance_by_name("jill", dna.clone(), netname);
        let (_instance2, context2) = instance_by_name("jack", dna, netname);

        let entry_address = context1
            .block_on(author_entry(
                &test_entry_with_value("{\"stuff\":\"test entry value\"}"),
                None,
                &context1,
                &vec![],
            ))
            .unwrap()
            .address();

        thread::sleep(time::Duration::from_millis(500));

        // get the header from the top of Jill's chain
        let state = &context1.state().unwrap();
        let header = state
            .get_headers(entry_address)
            .expect("Could not retrieve headers from authors chain")
            .into_iter()
            .next()
            .expect("No headers were found for this entry in the authors chain");
        let header_entry = Entry::ChainHeader(header);

        // try and load it by its address as Jack. This means it has been communicated over the mock network
        let mut entry: Option<Entry> = None;
        let mut tries = 0;
        while entry.is_none() && tries < 10 {
            tries = tries + 1;
            {
                entry = get_entry_from_dht(Arc::clone(&context2), &header_entry.address())
                    .expect("Could not retrieve entry from DHT");
            }
            println!("Try {}: {:?}", tries, entry);
            if entry.is_none() {
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
        assert_eq!(entry, Some(header_entry),);
    }

    #[test]
    /// test that all headers are published so an agents local chain can be reconstructed by another agent
    fn test_reconstruct_chain_via_published_headers() {
        let mut dna = test_dna();
        dna.uuid = "test_reconstruct_chain_via_published_headers".to_string();
        let netname = Some("test_reconstruct_chain_via_published_headers, the network");
        // the ordering of these is important. Jack will get Jills DNA and AgentId headers but not visa-versa
        let (_instance2, context2) = instance_by_name("jack", dna.clone(), netname);
        let (_instance1, context1) = instance_by_name("jill", dna.clone(), netname);

        // Jill publishes an entry
        context1
            .block_on(author_entry(
                &test_entry_with_value("{\"stuff\":\"test entry value number 1\"}"),
                None,
                &context1,
                &vec![],
            ))
            .unwrap()
            .address();
        thread::sleep(time::Duration::from_millis(500));

        // Jill publishes another entry
        context1
            .block_on(author_entry(
                &test_entry_with_value("{\"stuff\":\"test entry value number 2\"}"),
                None,
                &context1,
                &vec![],
            ))
            .unwrap()
            .address();
        thread::sleep(time::Duration::from_millis(500));

        // collect Jills local chain
        let state = &context1.state().unwrap();
        let jill_headers: Vec<ChainHeader> = state.agent().iter_chain().collect();
        let header = jill_headers
            .first()
            .expect("Must be at least one header in chain");

        // jack retrieves the top header addresss and reconstructs the Jills local chain by following the header back-links
        let mut jack_headers: Vec<ChainHeader> = Vec::new();
        let mut next_header_addr = header.address();
        loop {
            let mut entry: Option<Entry> = None;
            let mut tries = 0;
            while entry.is_none() && tries < 10 {
                tries = tries + 1;
                {
                    entry = get_entry_from_dht(Arc::clone(&context2), &next_header_addr)
                        .expect("Could not retrieve entry from DHT");
                }
                println!("Try {}: {:?}", tries, entry);
                if entry.is_none() {
                    thread::sleep(time::Duration::from_millis(1000));
                }
            }
            if let Some(Entry::ChainHeader(header)) = entry {
                jack_headers.push(header.clone());
                if let Some(next_addr) = header.link() {
                    next_header_addr = next_addr
                } else {
                    break; // chain has been followed to the genesis entry
                }
            } else {
                panic!(format!(
                    "Could not retrieve header at address: {}",
                    next_header_addr
                ))
            }
        }

        assert_eq!(jack_headers.len(), 4,);

        assert_eq!(jack_headers, jill_headers,);
    }
}
