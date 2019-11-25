pub mod application;
pub mod author_entry;
pub mod get_entry_result;
pub mod get_link_result;
pub mod get_links_count;
pub mod handle_custom_direct_message;
pub mod hold_entry;
pub mod hold_entry_remove;
pub mod hold_entry_update;
pub mod hold_link;
pub mod remove_link;
pub mod respond_validation_package_request;

use crate::{
    context::Context,
    dht::pending_validations::{PendingValidation, ValidatingWorkflow},
    network::{
        actions::get_validation_package::get_validation_package, entry_with_header::EntryWithHeader,
    },
    nucleus::{
        actions::build_validation_package::build_validation_package,
        ribosome::callback::{
            validation_package::get_validation_package_definition, CallbackResult,
        },
        validation::build_from_dht::try_make_validation_package_dht,
    },
    workflows::{
        hold_entry::hold_entry_workflow, hold_entry_remove::hold_remove_workflow,
        hold_entry_update::hold_update_workflow, hold_link::hold_link_workflow,
        remove_link::remove_link_workflow,
    },
};
use holochain_core_types::{
    error::HolochainError,
    validation::{ValidationPackage, ValidationPackageDefinition},
};
use holochain_persistence_api::cas::content::AddressableContent;
use std::sync::Arc;

/// Try to create a ValidationPackage for the given entry without calling out to some other node.
/// I.e. either create it just from/with the header if `ValidationPackageDefinition` is `Entry`,
/// or build it locally if we are the source (one of the sources).
/// Checks the DNA's validation package definition for the given entry type.
/// Fails if this entry type needs more than just the header for validation.
pub(crate) async fn try_make_local_validation_package(
    entry_with_header: &EntryWithHeader,
    validation_package_definition: &ValidationPackageDefinition,
    context: Arc<Context>,
) -> Result<ValidationPackage, HolochainError> {
    let entry_header = &entry_with_header.header;

    match validation_package_definition {
        ValidationPackageDefinition::Entry => {
            Ok(ValidationPackage::only_header(entry_header.clone()))
        }
        _ => {
            let agent = context.state()?.agent().get_agent()?;

            let overlapping_provenance = entry_with_header
                .header
                .provenances()
                .iter()
                .find(|p| p.source() == agent.address());

            if overlapping_provenance.is_some() {
                // We authored this entry, so lets build the validation package here and now:
                build_validation_package(
                    &entry_with_header.entry,
                    context.clone(),
                    entry_with_header.header.provenances(),
                )
                .await
            } else {
                Err(HolochainError::ErrorGeneric(String::from(
                    "Can't create validation package locally",
                )))
            }
        }
    }
}

/// Gets hold of the validation package for the given entry by trying several different methods.
async fn validation_package(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<Option<ValidationPackage>, HolochainError> {

    //0. Call into the DNA to get the validation package definition for this entry
    // e.g. what data is needed to validate it (chain, entry, headers, etc)
    let entry = &entry_with_header.entry;
    let validation_package_definition = get_validation_package_definition(entry, context.clone())
        .and_then(|callback_result| match callback_result {
        CallbackResult::Fail(error_string) => Err(HolochainError::ErrorGeneric(error_string)),
        CallbackResult::ValidationPackageDefinition(def) => Ok(def),
        CallbackResult::NotImplemented(reason) => Err(HolochainError::ErrorGeneric(format!(
            "ValidationPackage callback not implemented for {:?} ({})",
            entry.entry_type().clone(),
            reason
        ))),
        _ => unreachable!(),
    })?;

    // 1. Try to construct it locally.
    // This will work if the entry doesn't need a chain to validate or if this agent is the author:
    log_debug!(
        context,
        "validation_package:{} - Trying to build locally",
        entry_with_header.entry.address()
    );
    if let Ok(package) =
        try_make_local_validation_package(
            &entry_with_header,
            &validation_package_definition,
            context.clone()).await
    {
        log_debug!(
            context,
            "validation_package:{} - Successfully built locally",
            entry_with_header.entry.address()
        );
        return Ok(Some(package));
    }

    // 2. Try and get it from the author
    log_debug!(
        context,
        "validation_package:{} - Trying to retrieve from author",
        entry_with_header.entry.address()
    );
    if let Ok(Some(package)) =
        get_validation_package(entry_with_header.header.clone(), &context).await
    {
        log_debug!(
            context,
            "validation_package:{} - Successfully retrieved from author",
            entry_with_header.entry.address()
        );
        return Ok(Some(package));
    }

    // 3. Build it from the DHT (this may require many network requests (or none if full sync))
    log_debug!(
        context,
        "validation_package:{} - Trying to build from published headers",
        entry_with_header.entry.address()
    );
    if let Ok(package) = try_make_validation_package_dht(
        &entry_with_header, 
        &validation_package_definition,
        context.clone()).await
    {
        log_debug!(
            context,
            "validation_package:{} - Successfully built from published headers",
            entry_with_header.entry.address()
        );
        return Ok(Some(package));
    }

    // If all the above failed then returning an error will add this validation request to pending
    // It will then try all of the above from the start again later
    Err(HolochainError::ErrorGeneric(
        "Could not get validation package".to_string(),
    ))
}

#[cfg(test)]
pub mod tests {
    use super::validation_package;
    use crate::{
        network::entry_with_header::EntryWithHeader, nucleus::actions::tests::*,
        workflows::author_entry::author_entry,
    };
    use holochain_core_types::entry::Entry;
    use holochain_json_api::json::JsonString;
    use std::{thread, time};

    #[test]
    fn test_simulate_packge_direct_from_author() {
        let mut dna = test_dna();
        dna.uuid = "test_simulate_packge_direct_from_author".to_string();
        let netname = Some("test_simulate_packge_direct_from_author, the network");
        let (_instance1, context1) = instance_by_name("jill", dna.clone(), netname);
        let (_instance2, context2) = instance_by_name("jack", dna, netname);

        let entry = Entry::App(
            "package_chain_full".into(),
            JsonString::from_json("{\"stuff\":\"test entry value\"}"),
        );

        // jack authors the entry
        context2
            .block_on(author_entry(&entry, None, &context2, &vec![]))
            .unwrap()
            .address();

        thread::sleep(time::Duration::from_millis(500));
        // collect header from jacks local chain
        let header = context2
            .state()
            .unwrap()
            .agent()
            .iter_chain()
            .next()
            .expect("Must be able to get header for just published entry");

        let entry_with_header = EntryWithHeader { entry, header }.clone();

        let validation_package = context1
            .block_on(validation_package(&entry_with_header, context1.clone()))
            .expect("Could not recover a validation package as the non-author");

        assert_eq!(
            validation_package
                .unwrap()
                .source_chain_headers
                .unwrap()
                .len(),
            2
        );
    }
}

/// Runs the given pending validation using the right holding workflow
/// as specified by PendingValidationStruct::workflow.
pub fn run_holding_workflow(
    pending: PendingValidation,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    match pending.workflow {
        ValidatingWorkflow::HoldLink => context.block_on(hold_link_workflow(
            &pending.entry_with_header,
            context.clone(),
        )),
        ValidatingWorkflow::HoldEntry => context.block_on(hold_entry_workflow(
            &pending.entry_with_header,
            context.clone(),
        )),
        ValidatingWorkflow::RemoveLink => context.block_on(remove_link_workflow(
            &pending.entry_with_header,
            context.clone(),
        )),
        ValidatingWorkflow::UpdateEntry => context.block_on(hold_update_workflow(
            &pending.entry_with_header,
            context.clone(),
        )),
        ValidatingWorkflow::RemoveEntry => context.block_on(hold_remove_workflow(
            &pending.entry_with_header,
            context.clone(),
        )),
    }
}
