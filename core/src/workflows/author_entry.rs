use crate::{
    agent::actions::commit::commit_entry,
    context::Context,
    entry::CanPublish,
    network::actions::publish::publish,
    nucleus::{
        actions::build_validation_package::build_validation_package, validation::validate_entry,
    },
};

use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    signature::Provenance,
    validation::{EntryLifecycle, ValidationData},
};

use holochain_persistence_api::cas::content::{Address, AddressableContent};

use holochain_wasm_utils::api_serialization::commit_entry::CommitEntryResult;

use crate::nucleus::ribosome::callback::links_utils::get_link_entries;
use std::{sync::Arc, vec::Vec};

pub async fn author_entry<'a>(
    entry: &'a Entry,
    maybe_link_update_delete: Option<Address>,
    context: &'a Arc<Context>,
    provenances: &'a Vec<Provenance>,
) -> Result<CommitEntryResult, HolochainError> {
    let address = entry.address();
    context.log(format!(
        "debug/workflow/authoring_entry: {} with content: {:?}",
        address, entry
    ));

    // 0. If we are trying to author a link or link removal, make sure the linked entries exist:
    if let Entry::LinkAdd(link_data) = entry {
        get_link_entries(&link_data.link, context)?;
    }
    if let Entry::LinkRemove((link_data, _)) = entry {
        get_link_entries(&link_data.link, context)?;
    }

    // 1. Build the context needed for validation of the entry
    let validation_package = await!(build_validation_package(
        &entry,
        context.clone(),
        provenances
    ))?;
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Chain,
    };

    // 2. Validate the entry
    context.log(format!(
        "debug/workflow/authoring_entry/{}: validating...",
        address
    ));
    await!(validate_entry(
        entry.clone(),
        maybe_link_update_delete.clone(),
        validation_data,
        &context
    ))?;
    context.log(format!("Authoring entry {}: is valid!", address));

    // 3. Commit the entry
    context.log(format!(
        "debug/workflow/authoring_entry/{}: committing...",
        address
    ));
    let addr = await!(commit_entry(
        entry.clone(),
        maybe_link_update_delete,
        &context
    ))?;
    context.log(format!(
        "debug/workflow/authoring_entry/{}: committed",
        address
    ));

    // 4. Publish the valid entry to DHT. This will call Hold to itself
    if entry.entry_type().can_publish(context) {
        context.log(format!(
            "debug/workflow/authoring_entry/{}: publishing...",
            address
        ));
        await!(publish(entry.address(), &context))?;
        context.log(format!(
            "debug/workflow/authoring_entry/{}: published!",
            address
        ));
    } else {
        context.log(format!(
            "debug/workflow/authoring_entry/{}: entry is private, no publishing",
            address
        ));
    }
    Ok(CommitEntryResult::new(addr))
}

#[cfg(test)]
pub mod tests {
    use super::author_entry;
    use crate::nucleus::actions::tests::*;
    use holochain_core_types::entry::test_entry_with_value;
    use holochain_json_api::json::JsonString;
    use std::{thread, time};

    // TODO do this for all crate tests somehow
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
        let (_instance1, context1) = instance_by_name("jill", dna.clone(), netname);
        let context1_url = context1
            .network().lock().as_ref().unwrap().p2p_endpoint();
        let (_instance2, context2) = instance_with_bootstrap_nodes(
            "jack", dna, Some("test_commit_with_dht_publish2, the network"), vec![context1_url]);

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

        let mut json: Option<JsonString> = None;
        let mut tries = 0;
        while json.is_none() && tries < 5 {
            tries = tries + 1;
            {
                let state = &context2.state().unwrap();
                json = state
                    .dht()
                    .content_storage()
                    .read()
                    .unwrap()
                    .fetch(&entry_address)
                    .expect("could not fetch from CAS");
            }
            println!("Try {}: {:?}", tries, json);
            if json.is_none() {
                thread::sleep(time::Duration::from_millis(1000));
            }
        }

        let x: String = json.unwrap().to_string();
        assert_eq!(
            x,
            "{\"App\":[\"testEntryType\",\"{\\\"stuff\\\":\\\"test entry value\\\"}\"]}"
                .to_string(),
        );
    }
}
