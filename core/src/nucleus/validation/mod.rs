use crate::{context::Context,
workflows::get_entry_result::get_entry_result_workflow};
use holochain_core_types::{
    cas::content::Address,
    entry::{entry_type::{EntryType,AppEntryType}, Entry},
    error::HolochainError,
    validation::{ValidationData,EntryValidationData}
};
use holochain_wasm_utils::api_serialization::get_entry::GetEntryArgs;
use std::{sync::Arc,convert::TryFrom};
use futures_util::future::FutureExt;

mod app_entry;
mod header_address;
mod link_entry;
mod provenances;
mod remove_entry;

#[derive(Clone, Debug, PartialEq)]
/// A failed validation.
pub enum ValidationError {
    /// `Fail` means the validation function did run successfully and recognized the entry
    /// as invalid. The String parameter holds the non-zero return value of the app validation
    /// function.
    Fail(String),

    /// The entry could not get validated because known dependencies (like base and target
    /// for links) were not present yet.
    UnresolvedDependencies(Vec<Address>),

    /// A validation function for the given entry could not be found.
    /// This can happen if the entry's type is not defined in the DNA (which can only happen
    /// if somebody is sending wrong entries..) or there is no native implementation for a
    /// system entry type yet.
    NotImplemented,

    /// An error occurred that is out of the scope of validation (no state?, I/O errors..)
    Error(String),
}

/// Result of validating an entry.
/// Either Ok(()) if the entry is valid,
/// or any specialization of ValidationError.
pub type ValidationResult = Result<(), ValidationError>;

impl From<ValidationError> for HolochainError {
    fn from(ve: ValidationError) -> Self {
        match ve {
            ValidationError::Fail(reason) => HolochainError::ValidationFailed(reason),
            ValidationError::UnresolvedDependencies(_) => {
                HolochainError::ValidationFailed("Missing dependencies".to_string())
            }
            ValidationError::NotImplemented => {
                HolochainError::NotImplemented("Validation not implemented".to_string())
            }
            ValidationError::Error(e) => HolochainError::ErrorGeneric(e),
        }
    }
}

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
        EntryType::Deletion => await!(remove_entry::validate_remove_entry(
            entry.clone(),
            validation_data,
            context
        )),

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




pub fn entry_to_validation_data(
    context : Arc<Context>,
    entry: &Entry,
    maybe_link_update_delete: Option<Address>,
) -> Result<EntryValidationData, HolochainError> {
    match entry {
        Entry::App(_, _) => maybe_link_update_delete
            .map(|link_update|
            {
                get_latest_entry_for_entry_validation(context.clone(),link_update)
                .map(|latest|{
                    Ok(EntryValidationData::Modify(entry.clone(),latest.clone()))
                }).unwrap_or(Err(HolochainError::ErrorGeneric("Could not find Entry".to_string())))
            } )
            .unwrap_or(Ok(EntryValidationData::Create(entry.clone()))),
        Entry::Deletion(deletion_entry) => {
            let deletion_address = deletion_entry.clone().deleted_entry_address();
            get_latest_entry_for_entry_validation(context.clone(),deletion_address)
                .map(|latest|{
                    Ok(EntryValidationData::Delete(entry.clone(),latest.clone()))
                }).unwrap_or(Err(HolochainError::ErrorGeneric("Could not find Entry".to_string())))
        }
        Entry::LinkAdd(link) => Ok(EntryValidationData::Link(entry.clone(),link.clone())),
        Entry::LinkRemove(_) => Ok(EntryValidationData::Link(entry.clone(),link.clone())),
        Entry::CapTokenGrant(_) => Ok(EntryValidationData::Create(entry.clone())),
        _ => Err(HolochainError::NotImplemented(
            "Not implemented".to_string(),
        )),
    }
}


//high order function to get latest entry for the validation purposes
fn get_latest_entry_for_entry_validation(context : Arc<Context>,address : Address) ->Result<Entry,HolochainError>
{
    let entry_args = &GetEntryArgs {
                address: address,
                options: Default::default(),
                };
    let entry_result = context.block_on(get_entry_result_workflow(&context.clone(),entry_args))?;
    let latest = entry_result.latest().ok_or(HolochainError::ErrorGeneric("Could not Get Latest".to_string()))?;
    Ok(latest)

}