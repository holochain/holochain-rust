use crate::{
    context::Context,
    dht::actions::hold::hold_entry,
    network::actions::get_validation_package::get_validation_package,
    network::entry_with_header::EntryWithHeader,
    nucleus::actions::validate::validate_entry,
};

use holochain_core_types::{
    cas::content::Address,
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn hold_entry_workflow<'a>(
    entry_with_header: &'a EntryWithHeader,
    context: &'a Arc<Context>,
) -> Result<Address, HolochainError> {
    let EntryWithHeader{entry, header} = &entry_with_header;

    // 1. Get validation package from source
    let maybe_validation_package = await!(get_validation_package(header.clone(), &context))?;
    let validation_package = maybe_validation_package.ok_or("Could not get validation package from source".to_string())?;

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        sources: header.sources().clone(),
        lifecycle: EntryLifecycle::Dht,
        action: EntryAction::Create,
    };

    // 3. Validate the entry
    await!(validate_entry(entry.clone(), validation_data, &context))?;

    // 3. If valid store the entry in the local DHT shard
    await!(hold_entry(entry, &context))
}

/*
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::nucleus::actions::tests::*;
    use futures::executor::block_on;
    use holochain_core_types::entry::test_entry;
    use std::{thread, time};

    #[test]
    /// test that a commit will publish and entry to the dht of a connected instance via the mock network
    fn test_commit_with_dht_publish() {
        let dna = test_dna();
        let (_instance1, context1) = instance_by_name("jill", dna.clone());
        let (_instance2, context2) = instance_by_name("jack", dna);

        let entry_address = block_on(author_entry(&test_entry(), None, &context1));

        let entry_address = entry_address.unwrap();
        thread::sleep(time::Duration::from_millis(1000));

        let state = &context2.state().unwrap();
        let json = state
            .dht()
            .content_storage()
            .read()
            .unwrap()
            .fetch(&entry_address)
            .expect("could not fetch from CAS");

        let x: String = json.unwrap().to_string();
        assert_eq!(
            x,
            "{\"App\":[\"testEntryType\",\"\\\"test entry value\\\"\"]}".to_string(),
        );
    }
}
*/