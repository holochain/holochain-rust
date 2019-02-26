use crate::{context::Context, nucleus::state::ValidationResult};
use holochain_core_types::{
    entry::{entry_type::EntryType, Entry},
    validation::ValidationData,
};
use std::sync::Arc;

mod app_entry;
mod header_address;
mod link_entry;
mod provenances;

use crate::nucleus::state::ValidationError;

/// Main validation workflow.
/// This is the high-level validate function that wraps the whole validation process and is what should
/// be called from other workflows for validating an entry.
///
/// 1. Checks if the entry's address matches the address in given header provided by
///    the validation package.
/// 2. Validates provenances given in the header by verifying the cryptographic signatures
///    against the source agent addresses.
/// 3. Finally spawns a thread to run the type specific validation callback in a Ribosome.
///
/// All of this actually happens in the functions of the sub modules. This function is the
/// main validation entry point and, like a workflow, stays high-level.
pub async fn validate_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    //check_entry_type(entry.entry_type(), context)?;
    header_address::validate_header_address(&entry, &validation_data.package.chain_header)?;
    provenances::validate_provenances(&validation_data)?;

    match entry.entry_type() {
        // DNA entries are not validated currently and always valid
        // TODO: Specify when DNA can be commited as an update and how to implement validation of DNA entries then.
        EntryType::Dna => Ok(()),

        EntryType::App(app_entry_type) => await!(app_entry::validate_app_entry(
            entry.clone(),
            app_entry_type.clone(),
            validation_data,
            context,
        )),

        EntryType::LinkAdd => await!(link_entry::validate_link_entry(
            entry.clone(),
            validation_data,
            context
        )),

        EntryType::LinkRemove => await!(link_entry::validate_link_entry(
            entry.clone(),
            validation_data,
            context
        )),

        // Deletion entries are not validated currently and always valid
        // TODO: Specify how Deletion can be commited to chain.
        EntryType::Deletion => Ok(()),

        // a grant should always be private, so it should always pass
        EntryType::CapTokenGrant => Ok(()),

        // TODO: actually check agent against app specific membrane validation rule
        // like for instance: validate_agent_id(
        //                      entry.clone(),
        //                      validation_data,
        //                      context,
        //                    )?
        EntryType::AgentId => Ok(()),

        _ => Err(ValidationError::NotImplemented),
    }
}
