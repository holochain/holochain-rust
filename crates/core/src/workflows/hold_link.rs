use crate::{
    context::Context, dht::actions::hold_aspect::hold_aspect,
    network::entry_with_header::EntryWithHeader, nucleus::validation::validate_entry,
};

use crate::{
    nucleus::validation::ValidationError,
    workflows::{hold_entry::hold_entry_workflow, validation_package},
};
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    network::entry_aspect::EntryAspect,
    validation::{EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn hold_link_workflow(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    let link_add = match &entry_with_header.entry {
        Entry::LinkAdd(link_add) => link_add,
        _ => {
            return Err(HolochainError::ErrorGeneric(
                "hold_link_workflow expects entry to be an Entry::LinkAdd".to_string(),
            ))
        }
    };
    let link = link_add.link().clone();

    log_debug!(context, "workflow/hold_link: {:?}", link);
    log_debug!(context, "workflow/hold_link: getting validation package...");
    // 1. Get hold of validation package
    let maybe_validation_package = validation_package(&entry_with_header, context.clone())
        .await
        .map_err(|err| {
            let message = "Could not get validation package from source! -> Add to pending...";
            log_debug!(context, "workflow/hold_link: {}", message);
            log_debug!(context, "workflow/hold_link: Error was: {:?}", err);
            HolochainError::ValidationPending
        })?;
    let validation_package = maybe_validation_package.ok_or_else(|| {
        let message = "Source did respond to request but did not deliver validation package! (Empty response) This is weird! Let's try this again later -> Add to pending";
        log_debug!(context, "workflow/hold_link: {}", message);
        HolochainError::ValidationPending
    })?;
    log_debug!(context, "workflow/hold_link: got validation package");

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta,
    };

    // 3. Validate the entry
    log_debug!(context, "workflow/hold_link: validate...");
    validate_entry(
        entry_with_header.entry.clone(),
        None,
        validation_data,
        &context
    ).await
    .map_err(|err| {
        if let ValidationError::UnresolvedDependencies(dependencies) = &err {
            log_debug!(context, "workflow/hold_link: Link could not be validated due to unresolved dependencies and will be tried later. List of missing dependencies: {:?}", dependencies);
            HolochainError::ValidationPending
        } else {
            log_warn!(context, "workflow/hold_link: Link {:?} is NOT valid! Validation error: {:?}",
                entry_with_header.entry,
                err,
            );
            HolochainError::from(err)
        }

    })?;
    log_debug!(context, "workflow/hold_link: is valid!");

    // 3. If valid store the entry aspect in the local DHT shard
    let aspect = EntryAspect::LinkAdd(link_add.clone(), entry_with_header.header.clone());
    hold_aspect(aspect, context.clone()).await?;

    log_debug!(context, "workflow/hold_link: added! {:?}", link);

    //4. store link_add entry so we have all we need to respond to get links queries without any other network look-up
    hold_entry_workflow(&entry_with_header, context.clone()).await?;
    log_debug!(
        context,
        "workflow/hold_entry: added! {:?}",
        entry_with_header
    );

    //5. Link has been added to EAV and LinkAdd Entry has been stored on the dht
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "broken-tests")]
// too slow!
pub mod tests {
    use super::*;
    use crate::{nucleus::actions::tests::*, workflows::author_entry::author_entry};
    use holochain_core_types::{
        agent::test_agent_id, chain_header::test_chain_header, entry::test_entry_with_value,
        link::link_data::LinkData,
    };

    #[test]
    /// Test that an invalid link will be rejected by this workflow.
    ///
    /// This test simulates an attack where a node is changing its local copy of the DNA to
    /// allow otherwise invalid entries while spoofing the unmodified dna_address.
    ///
    /// hold_link_workflow is then expected to fail in its validation step
    fn test_reject_invalid_link_on_hold_workflow() {
        // Hacked DNA that regards everything as valid
        let hacked_dna = create_test_dna_with_wat("test_zome", Some(&test_wat_always_valid()));
        // Original DNA that regards nothing as valid
        let mut dna = create_test_dna_with_wat("test_zome", Some(&test_wat_always_invalid()));
        dna.uuid = String::from("test_reject_invalid_link_on_hold_workflow");

        // Address of the original DNA
        let dna_address = dna.address();

        let (_, context1) =
            test_instance_with_spoofed_dna(hacked_dna, dna_address, "alice").unwrap();
        let netname = Some("test_reject_invalid_link_on_remove_workflow");

        // Commit entry on attackers node
        let entry = test_entry_with_value("{\"stuff\":\"test entry value\"}");

        let entry_address = context1
            .block_on(author_entry(&entry, None, &context1, &Vec::new()))
            .unwrap();

        let link_add = LinkData::new_add(
            &entry_address.address,
            &entry_address.address,
            "test-tag",
            "test-link",
            test_chain_header(),
            test_agent_id(),
        );
        let link_entry = Entry::LinkAdd(link_add);

        let _ = context1
            .block_on(author_entry(&link_entry, None, &context1, &Vec::new()))
            .unwrap();

        // Get header which we need to trigger hold_entry_workflow
        let agent1_state = context1.state().unwrap().agent();
        let header = agent1_state
            .get_most_recent_header_for_entry(&link_entry)
            .expect("There must be a header in the author's source chain after commit");
        let entry_with_header = EntryWithHeader {
            entry: link_entry,
            header,
        };

        // Call hold_entry_workflow on victim DHT node
        let result = context2.block_on(hold_link_workflow(&entry_with_header, context2.clone()));

        // ... and expect validation to fail with message defined in test WAT:
        assert!(result.is_err());

        assert_eq!(
            result.err().unwrap(),
            HolochainError::ValidationFailed(String::from("FAIL wat")),
        );
    }
}
