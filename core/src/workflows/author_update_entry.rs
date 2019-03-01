use crate::{
    agent::actions::commit::commit_entry,
    context::Context,
    network::actions::publish::publish,
    nucleus::actions::{
        build_validation_package::build_validation_package, validate::validate_entry,
    },
};

use holochain_core_types::{
    cas::content::{Address, AddressableContent},
    entry::Entry,
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn author_update_entry<'a>(
    entry: &'a Entry,
    maybe_link_update_delete: Option<Address>,
    context: &'a Arc<Context>,
) -> Result<Address, HolochainError> {
    let address = entry.address();
    context.log(format!(
        "debug/workflow/authoring_entry: {} with content: {:?}",
        address, entry
    ));

    // 1. Build the context needed for validation of the entry
    let validation_package = await!(build_validation_package(&entry, &context))?;
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Chain,
        action: EntryAction::Modify,
    };

    // 2. Validate the entry
    context.log(format!(
        "debug/workflow/authoring_entry/{}: validating...",
        address
    ));
    await!(validate_entry(entry.clone(), validation_data, &context))?;
    context.log(format!("Authoring entry {}: is valid!", address));

    // 3. Commit the entry
    context.log(format!(
        "debug/workflow/authoring_entry/{}: committing...",
        address
    ));
    let addr = await!(commit_entry(entry.clone(), maybe_link_update_delete.clone(), &context))?;
    context.log(format!(
        "debug/workflow/authoring_entry/{}: committed",
        address
    ));


    // 4. Publish the valid entry to DHT. This will call Hold to itself
    //TODO: missing a general public/private sharing check here, for now just
    // using the entry_type can_publish() function which isn't enough
    
    if entry.entry_type().can_publish() {
        context.log(format!(
            "debug/workflow/authoring_entry/{}: publishing...",
            address
        ));
        await!(publish(addr.clone(), &context))?;
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
    Ok(addr)
}

#[cfg(test)]
pub mod tests {
    use super::author_entry;
    use crate::nucleus::actions::tests::*;
    use holochain_core_types::{entry::test_entry, json::JsonString};
    use std::{thread, time};

    #[test]
    #[cfg(not(windows))]
    /// test that a commit will publish and entry to the dht of a connected instance via the in-memory network
    fn test_commit_with_dht_publish() {
        let mut dna = test_dna();
        dna.uuid = "test_commit_with_dht_publish".to_string();
        let netname = Some("test_commit_with_dht_publish, the network");
        let (_instance1, context1) = instance_by_name("jill", dna.clone(), netname);
        let (_instance2, context2) = instance_by_name("jack", dna, netname);

        let entry_address = context1
            .block_on(author_entry(&test_entry(), None, &context1))
            .unwrap();
        thread::sleep(time::Duration::from_millis(500));

        let mut json: Option<JsonString> = None;
        let mut tries = 0;
        while json.is_none() && tries < 120 {
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
            "{\"App\":[\"testEntryType\",\"\\\"test entry value\\\"\"]}".to_string(),
        );
    }
}
