use crate::{context::Context, nucleus::actions::get_entry::get_entry_from_dht};
use holochain_core_types::{
    cas::content::Address,
    chain_header::ChainHeader,
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    validation::{EntryValidationData, ValidationData, ValidationPackage},
};

use std::sync::Arc;

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
    link: Option<Address>,
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
            context,
            link,
            validation_data.package
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
    context: Arc<Context>,
    entry: &Entry,
    maybe_link_update_delete: Option<Address>,
    validation_package: ValidationPackage,
) -> Result<EntryValidationData<Entry>, HolochainError> {
    match entry {
        Entry::App(_, _) => maybe_link_update_delete
            .map(|link_update| {
                get_entry_with_header(context.clone(), &link_update)
                    .map(|entry_with_header| {
                        Ok(EntryValidationData::Modify {
                            old_entry: entry_with_header.0.clone(),
                            new_entry: entry.clone(),
                            old_entry_header: entry_with_header.1.clone(),
                            validation_package: validation_package.clone(),
                        })
                    })
                    .unwrap_or(Err(HolochainError::ErrorGeneric(
                        "Could not find Entry".to_string(),
                    )))
            })
            .unwrap_or(Ok(EntryValidationData::Create {
                entry: entry.clone(),
                validation_package,
            })),
        Entry::Deletion(deletion_entry) => {
            let deletion_address = deletion_entry.clone().deleted_entry_address();
            get_entry_with_header(context.clone(), &deletion_address)
                .map(|entry_with_header| {
                    Ok(EntryValidationData::Delete {
                        old_entry: entry_with_header.0.clone(),
                        old_entry_header: entry_with_header.1.clone(),
                        validation_package,
                    })
                })
                .unwrap_or(Err(HolochainError::ErrorGeneric(
                    "Could not find Entry".to_string(),
                )))
        }
        Entry::CapTokenGrant(_) => Ok(EntryValidationData::Create {
            entry: entry.clone(),
            validation_package,
        }),
        _ => Err(HolochainError::NotImplemented(
            "Not implemented".to_string(),
        )),
    }
}

fn get_entry_with_header(
    context: Arc<Context>,
    address: &Address,
) -> Result<(Entry, ChainHeader), HolochainError> {
    let state = context.state().ok_or(HolochainError::ErrorGeneric(
        "Could not obtainn state".to_string(),
    ))?;
    let entry = get_entry_from_dht(&context.clone(), address)?.ok_or(
        HolochainError::ErrorGeneric("Could not get Entry".to_string()),
    )?;
    let headers = state.get_headers(address.clone())?;
    let entry_header = headers.last().ok_or(HolochainError::ErrorGeneric(
        "Could not get header for entry".to_string(),
    ))?;
    Ok((entry, entry_header.clone()))
}
