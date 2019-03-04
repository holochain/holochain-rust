use crate::{
    context::Context,
    dht::actions::add_link::add_link,
    network::{
        actions::get_validation_package::get_validation_package, entry_with_header::EntryWithHeader,
    },
    nucleus::validation::validate_entry,
};

use crate::{
    nucleus::{
        actions::add_pending_validation::add_pending_validation, validation::ValidationError,
    },
    scheduled_jobs::pending_validations::ValidatingWorkflow,
};
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    validation::{EntryAction, EntryLifecycle, ValidationData},
};
use std::sync::Arc;

pub async fn hold_link_workflow<'a>(
    entry_with_header: &'a EntryWithHeader,
    context: &'a Arc<Context>,
) -> Result<(), HolochainError> {
    let EntryWithHeader { entry, header } = &entry_with_header;

    let link_add = match entry {
        Entry::LinkAdd(link_add) => link_add,
        _ => Err(HolochainError::ErrorGeneric(
            "hold_link_workflow expects entry to be an Entry::LinkAdd".to_string(),
        ))?,
    };
    let link = link_add.link().clone();

    context.log(format!("debug/workflow/hold_link: {:?}", link));
    // 1. Get validation package from source
    context.log(format!(
        "debug/workflow/hold_link: getting validation package..."
    ));
    let maybe_validation_package = await!(get_validation_package(header.clone(), &context))
        .map_err(|err| {
            let message = "Could not get validation package from source! -> Add to pending...";
            context.log(format!("debug/workflow/hold_link: {}", message));
            context.log(format!("debug/workflow/hold_link: Error was: {:?}", err));
            add_pending_validation(
                entry_with_header.to_owned(),
                Vec::new(),
                ValidatingWorkflow::HoldLink,
                context,
            );
            HolochainError::ValidationPending
        })?;
    let validation_package = maybe_validation_package.ok_or({
        let message = "Source did respond to request but did not deliver validation package! This is weird! Entry is not valid!";
        context.log(format!("debug/workflow/hold_link: {}", message));
        HolochainError::ValidationFailed("Entry not backed by source".to_string())
    })?;
    context.log(format!("debug/workflow/hold_link: got validation package"));

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta,
        action: EntryAction::Create,
    };

    // 3. Validate the entry
    context.log(format!("debug/workflow/hold_link: validate..."));
    await!(validate_entry(entry.clone(), validation_data, &context)).map_err(|err| {
        context.log(format!("debug/workflow/hold_link: invalid! {:?}", err));
        if let ValidationError::UnresolvedDependencies(dependencies) = &err {
            add_pending_validation(
                entry_with_header.to_owned(),
                dependencies.clone(),
                ValidatingWorkflow::HoldLink,
                &context,
            );
        }
        HolochainError::ValidationPending
    })?;
    context.log(format!("debug/workflow/hold_link: is valid!"));

    // 3. If valid store the entry in the local DHT shard
    await!(add_link(&link, &context))?;
    context.log(format!("debug/workflow/hold_link: added! {:?}", link));
    Ok(())
}

#[cfg(test)]
// too slow!
#[cfg(feature = "broken-tests")]
pub mod tests {
    use super::*;
    use crate::{
        network::test_utils::*, nucleus::actions::tests::*, workflows::author_entry::author_entry,
    };
    use futures::executor::block_on;
    use holochain_core_types::{
        cas::content::AddressableContent, entry::test_entry, link::link_data::LinkData,
    };
    use test_utils::*;

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
        let (_instance2, context2) = instance_by_name("jack", dna, netname);

        // Commit entry on attackers node
        let entry = test_entry();
        let entry_address = context1
            .block_on(author_entry(&entry, None, &context1))
            .unwrap();

        let link_add = LinkData::new_add(&entry_address, &entry_address, "test-tag");
        let link_entry = Entry::LinkAdd(link_add);

        let _ = context1
            .block_on(author_entry(&link_entry, None, &context1))
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
        let result = context2.block_on(hold_link_workflow(&entry_with_header, &context2));

        // ... and expect validation to fail with message defined in test WAT:
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            HolochainError::ValidationFailed(String::from("FAIL wat")),
        );
    }
}
