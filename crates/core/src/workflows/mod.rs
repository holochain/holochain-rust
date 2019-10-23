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
    network::{
        actions::get_validation_package::get_validation_package, entry_with_header::EntryWithHeader,
    },
    nucleus::{
        actions::build_validation_package::build_validation_package,
        ribosome::callback::{
            validation_package::get_validation_package_definition, CallbackResult,
        },
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
async fn try_make_local_validation_package(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<ValidationPackage, HolochainError> {
    let entry = &entry_with_header.entry;
    let entry_header = &entry_with_header.header;

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

/// Gets hold of the validation package for the given entry.
/// First tries to create it locally and if that fails will try to get the
/// validation package from the source.
async fn validation_package(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<Option<ValidationPackage>, HolochainError> {
    // 1. Try to construct it locally:
    if let Ok(package) =
        try_make_local_validation_package(&entry_with_header, context.clone()).await
    {
        Ok(Some(package))
    } else {
        // If that is not possible, get the validation package from source
        get_validation_package(entry_with_header.header.clone(), &context).await
    }
}
