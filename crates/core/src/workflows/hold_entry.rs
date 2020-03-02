use crate::{
    context::Context, dht::actions::hold_aspect::hold_aspect,
    network::entry_with_header::EntryWithHeader, nucleus::validation::validate_entry,

};

use crate::{workflows::validation_package};
use holochain_core_types::{
    error::HolochainError,
    network::entry_aspect::EntryAspect,
    validation::{EntryLifecycle, ValidationData, ValidationResult},
};

use holochain_persistence_api::cas::content::AddressableContent;

use std::sync::Arc;

// #[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn hold_entry_workflow(
    context: Arc<Context>,
    entry_with_header: &EntryWithHeader,
) -> Result<(), HolochainError> {
    // 1. Get hold of validation package
    let maybe_validation_package = validation_package(Arc::clone(&context), entry_with_header)
        .await
        .map_err(|err| {
            let message = "Could not get validation package from source! -> Add to pending...";
            log_debug!(context, "workflow/hold_entry: {}", message);
            log_debug!(context, "workflow/hold_entry: Error was: {:?}", err);
            HolochainError::ValidationPending
        })?;

    let validation_package = maybe_validation_package.ok_or_else(|| {
        let message = "Source did respond to request but did not deliver validation package! (Empty response) This is weird! Let's try this again later -> Add to pending";
        log_debug!(context, "workflow/hold_entry: {}", message);
        HolochainError::ValidationPending
    })?;
    log_debug!(context, "workflow/hold_entry: got validation package");

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Dht,
    };

    // 3. Validate the entry
    match validate_entry(
        Arc::clone(&context),
        entry_with_header.entry.clone(),
        None,
        validation_data,
    ).await {
        ValidationResult::Ok => (),
        ValidationResult::UnresolvedDependencies(dependencies) => {
            log_debug!(context, "workflow/hold_entry: {} could not be validated due to unresolved dependencies and will be tried later. List of missing dependencies: {:?}",
                entry_with_header.entry.address(),
                dependencies,
            );
            return Err(HolochainError::ValidationPending);
        },
        ValidationResult::Fail(s) => {
            log_warn!(context, "workflow/hold_entry: Entry {} is NOT valid! Validation error: {:?}",
                entry_with_header.entry.address(),
                s,
            );
            return Err(HolochainError::from(s));
        },
        ValidationResult::NotImplemented => {
            log_warn!(context, "workflow/hold_entry: Entry {} could not be validated due to missing validation!",
                entry_with_header.entry.address(),
            );
            return Err(HolochainError::ValidationFailed(ValidationResult::NotImplemented));
        },
        ValidationResult::Timeout => {
            log_debug!(context, "workflow/hold_entry: {} timed out",
                entry_with_header.entry.address(),
            );
            return Err(HolochainError::Timeout);
        },
    };

    log_debug!(
        context,
        "workflow/hold_entry: is valid! {}",
        entry_with_header.entry.address()
    );

    // 4. If valid store the entry aspect in the local DHT shard
    let aspect = EntryAspect::Content(
        entry_with_header.entry.clone(),
        entry_with_header.header.clone(),
    );
    hold_aspect(aspect, context.clone()).await?;

    log_debug!(
        context,
        "workflow/hold_entry: HOLDING: {}",
        entry_with_header.entry.address()
    );

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
    use holochain_core_types::entry::test_entry;
    use test_utils::*;

    #[test]
    /// Test that an invalid entry will be rejected by this workflow.
    ///
    /// This test simulates an attack where a node is changing its local copy of the DNA to
    /// allow otherwise invalid entries while spoofing the unmodified dna_address.
    ///
    /// hold_entry_workflow is then expected to fail in its validation step
    fn test_reject_invalid_entry_on_hold_workflow() {
        // Hacked DNA that regards everything as valid
        let hacked_dna = create_test_dna_with_wat("test_zome", Some(&test_wat_always_valid()));
        // Original DNA that regards nothing as valid
        let mut dna = create_test_dna_with_wat("test_zome", Some(&test_wat_always_invalid()));
        dna.uuid = String::from("test_reject_invalid_entry_on_hold_workflow");

        // Address of the original DNA
        let dna_address = dna.address();

        let (_, context1) =
            test_instance_with_spoofed_dna(hacked_dna, dna_address, "alice").unwrap();
        let (_instance2, context2) = instance_by_name("jack", dna);

        // Commit entry on attackers node
        let entry = test_entry();
        let _entry_address = context1
            .block_on(author_entry(&entry, None, &context1))
            .unwrap();

        // Get header which we need to trigger hold_entry_workflow
        let agent1_state = context1.state().unwrap().agent();
        let header = agent1_state
            .get_most_recent_header_for_entry(&entry)
            .expect("There must be a header in the author's source chain after commit");
        let entry_with_header = EntryWithHeader { entry, header };

        // Call hold_entry_workflow on victim DHT node
        let result = context2.block_on(hold_entry_workflow(&entry_with_header, &context2));

        // ... and expect validation to fail with message defined in test WAT:
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            HolochainError::ValidationFailed(String::from("FAIL wat")),
        );
    }
}
