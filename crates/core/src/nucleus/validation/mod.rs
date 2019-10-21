use crate::{context::Context, workflows::get_entry_result::get_entry_with_meta_workflow};
use holochain_core_types::{
    chain_header::ChainHeader,
    entry::{entry_type::EntryType, Entry, EntryWithMeta},
    error::HolochainError,
    time::Timeout,
    validation::{EntryValidationData, ValidationData},
};
use holochain_persistence_api::cas::content::Address;

use std::sync::Arc;

mod agent_entry;
mod app_entry;
mod header_address;
mod link_entry;
mod provenances;
mod remove_entry;

#[derive(Clone, Debug, PartialEq, Serialize)]
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
    Error(HolochainError),
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
            ValidationError::Error(e) => e,
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
    log_debug!(context, "workflow/validate_entry: {:?}", entry);
    //check_entry_type(entry.entry_type(), context)?;
    header_address::validate_header_address(&entry, &validation_data.package.chain_header)?;
    provenances::validate_provenances(&validation_data)?;

    match entry.entry_type() {
        // DNA entries are not validated currently and always valid
        // TODO: Specify when DNA can be commited as an update and how to implement validation of DNA entries then.
        EntryType::Dna => Ok(()),

        EntryType::App(app_entry_type) => {
            app_entry::validate_app_entry(
                entry.clone(),
                app_entry_type.clone(),
                context,
                link,
                validation_data,
            )
            .await
        }

        EntryType::LinkAdd => {
            link_entry::validate_link_entry(entry.clone(), validation_data, context).await
        }

        EntryType::LinkRemove => {
            link_entry::validate_link_entry(entry.clone(), validation_data, context).await
        }

        // Deletion entries are not validated currently and always valid
        // TODO: Specify how Deletion can be commited to chain.
        EntryType::Deletion => {
            remove_entry::validate_remove_entry(entry.clone(), validation_data, context).await
        }

        // a grant should always be private, so it should always pass
        EntryType::CapTokenGrant => Ok(()),

        EntryType::AgentId => {
            agent_entry::validate_agent_entry(entry.clone(), validation_data, context).await
        }

        // chain headers always pass for now. In future this should check that the entry is valid
        EntryType::ChainHeader => Ok(()),

        _ => Err(ValidationError::NotImplemented),
    }
}

pub fn entry_to_validation_data(
    context: Arc<Context>,
    entry: &Entry,
    maybe_link_update_delete: Option<Address>,
    validation_data: ValidationData,
) -> Result<EntryValidationData<Entry>, HolochainError> {
    match entry {
        Entry::App(_, _) => maybe_link_update_delete
            .map(|link_update| {
                get_entry_with_header(context.clone(), &link_update)
                    .map(|entry_with_header| {
                        Ok(EntryValidationData::Modify {
                            old_entry: entry_with_header.0.entry.clone(),
                            new_entry: entry.clone(),
                            old_entry_header: entry_with_header.1.clone(),
                            validation_data: validation_data.clone(),
                        })
                    })
                    .unwrap_or_else(|_| {
                        Err(HolochainError::ErrorGeneric(
                            "Could not find Entry".to_string(),
                        ))
                    })
            })
            .unwrap_or_else(|| {
                Ok(EntryValidationData::Create {
                    entry: entry.clone(),
                    validation_data: validation_data.clone(),
                })
            }),
        Entry::Deletion(deletion_entry) => {
            let deletion_address = deletion_entry.clone().deleted_entry_address();
            get_entry_with_header(context.clone(), &deletion_address)
                .map(|entry_with_header| {
                    Ok(EntryValidationData::Delete {
                        old_entry: entry_with_header.0.entry.clone(),
                        old_entry_header: entry_with_header.1.clone(),
                        validation_data: validation_data.clone(),
                    })
                })
                .unwrap_or_else(|_| {
                    Err(HolochainError::ErrorGeneric(
                        "Could not find Entry".to_string(),
                    ))
                })
        }
        Entry::CapTokenGrant(_) => Ok(EntryValidationData::Create {
            entry: entry.clone(),
            validation_data,
        }),
        _ => Err(HolochainError::NotImplemented(
            "Not implemented".to_string(),
        )),
    }
}

fn get_entry_with_header(
    context: Arc<Context>,
    address: &Address,
) -> Result<(EntryWithMeta, ChainHeader), HolochainError> {
    let pair = context.block_on(get_entry_with_meta_workflow(
        &context,
        address,
        &Timeout::default(),
    ))?;
    let entry_with_meta = pair.ok_or("Could not get chain")?;
    let latest_header = entry_with_meta
        .headers
        .last()
        .ok_or("Could not get last entry from chain")?;
    Ok((entry_with_meta.entry_with_meta, latest_header.clone()))
}
