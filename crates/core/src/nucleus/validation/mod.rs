use crate::{
    context::Context, workflows::get_entry_result::get_entry_with_meta_workflow,
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::{
    chain_header::ChainHeader,
    entry::{entry_type::EntryType, Entry, EntryWithMeta},
    error::HolochainError,
    time::Timeout,
    validation::{EntryValidationData, ValidationData, ValidationResult},
};
use holochain_persistence_api::cas::content::Address;

use std::sync::Arc;

mod agent_entry;
mod app_entry;
pub mod build_from_dht;
mod header_address;
mod link_entry;
mod provenances;
mod remove_entry;

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
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn validate_entry(
    entry: Entry,
    link: Option<Address>,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    log_debug!(context, "workflow/validate_entry: {:?}", entry);
    //check_entry_type(entry.entry_type(), context)?;
    match header_address::validate_header_address(&entry, &validation_data.package.chain_header) {
        ValidationResult::Ok => {
            match provenances::validate_provenances(&validation_data) {
                ValidationResult::Ok => {
                    match entry.entry_type() {
                        // DNA entries are not validated currently and always valid
                        // TODO: Specify when DNA can be commited as an update and how to implement validation of DNA entries then.
                        EntryType::Dna => ValidationResult::Ok,

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
                        EntryType::CapTokenGrant => ValidationResult::Ok,

                        EntryType::AgentId => {
                            agent_entry::validate_agent_entry(entry.clone(), validation_data, context).await
                        }

                        // chain headers always pass for now. In future this should check that the entry is valid
                        EntryType::ChainHeader => ValidationResult::Ok,
                    }
                },
                v => v,
            }
        },
        v => v,
    }

}

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
                            old_entry_header: entry_with_header.1,
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
            let deletion_address = deletion_entry.deleted_entry_address().clone();
            get_entry_with_header(context, &deletion_address)
                .map(|entry_with_header| {
                    Ok(EntryValidationData::Delete {
                        old_entry: entry_with_header.0.entry.clone(),
                        old_entry_header: entry_with_header.1,
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

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
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
