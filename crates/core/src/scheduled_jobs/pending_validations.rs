use crate::{
    context::Context,
    network::chain_pair::ChainPair,
    nucleus::actions::remove_pending_validation::remove_pending_validation,
    workflows::{hold_entry::hold_entry_workflow, hold_link::hold_link_workflow},
};
use holochain_core_types::error::HolochainError;

use crate::workflows::{
    hold_entry_remove::hold_remove_workflow, hold_entry_update::hold_update_workflow,
    remove_link::remove_link_workflow,
};
use holochain_core_types::{
    entry::{deletion_entry::DeletionEntry, Entry},
    network::entry_aspect::EntryAspect,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::cas::content::{Address, AddressableContent};
use std::{convert::TryFrom, fmt, sync::Arc};

pub type PendingValidation = Arc<PendingValidationStruct>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, DefaultJson)]
pub enum ValidatingWorkflow {
    HoldEntry,
    HoldLink,
    RemoveLink,
    UpdateEntry,
    RemoveEntry,
}

impl Into<String> for ValidatingWorkflow {
    fn into(self) -> String {
        match self {
            ValidatingWorkflow::HoldEntry => String::from("HoldEntry"),
            ValidatingWorkflow::HoldLink => String::from("HoldLink"),
            ValidatingWorkflow::RemoveLink => String::from("RemoveLink"),
            ValidatingWorkflow::UpdateEntry => String::from("UpdateEntry"),
            ValidatingWorkflow::RemoveEntry => String::from("RemoveEntry"),
        }
    }
}

impl TryFrom<String> for ValidatingWorkflow {
    type Error = HolochainError;
    fn try_from(s: String) -> Result<ValidatingWorkflow, HolochainError> {
        match s.as_ref() {
            "HoldEntry" => Ok(ValidatingWorkflow::HoldEntry),
            "HoldLink" => Ok(ValidatingWorkflow::HoldLink),
            "RemoveLink" => Ok(ValidatingWorkflow::RemoveLink),
            "UpdateEntry" => Ok(ValidatingWorkflow::UpdateEntry),
            "RemoveEntry" => Ok(ValidatingWorkflow::RemoveEntry),
            _ => Err(HolochainError::SerializationError(String::from(
                "No ValidatingWorkflow",
            ))),
        }
    }
}

impl fmt::Display for ValidatingWorkflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidatingWorkflow::HoldEntry => write!(f, "HoldEntryWorkflow"),
            ValidatingWorkflow::HoldLink => write!(f, "HoldLinkWorkflow"),
            ValidatingWorkflow::RemoveLink => write!(f, "RemoveLinkWorkflow"),
            ValidatingWorkflow::UpdateEntry => write!(f, "UpdateEntryWorkflow"),
            ValidatingWorkflow::RemoveEntry => write!(f, "RemoveEntryWorkflow"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, DefaultJson)]
pub struct PendingValidationStruct {
    pub chain_pair: ChainPair,
    pub dependencies: Vec<Address>,
    pub workflow: ValidatingWorkflow,
}

impl PendingValidationStruct {
    pub fn new(chain_pair: ChainPair, workflow: ValidatingWorkflow) -> Self {
        Self {
            chain_pair,
            dependencies: Vec::new(),
            workflow,
        }
    }
}

impl TryFrom<EntryAspect> for PendingValidationStruct {
    type Error = HolochainError;
    fn try_from(aspect: EntryAspect) -> Result<PendingValidationStruct, HolochainError> {
        match aspect {
            EntryAspect::Content(entry, header) => {
                match try_validate_from_entry_and_header(entry, header) {
                    Ok(chain_pair) => {
                        Ok(PendingValidationStruct::new(
                            chain_pair,
                            ValidatingWorkflow::HoldEntry,
                        )),
                        Err(error) => {
                            let error_msg = format!(
                                "Tried to process EntryAspect::Content(entry, header)\n
                                where entry is:\n{#:?}\nHeader:\n{#:?}\nError:{}",
                                entry, header, error
                            );
                            Err(HolochainError::ValidationFailed(String::from(error_msg)))
                        },
                    }
                }
            },
            EntryAspect::Header(_header) => Err(HolochainError::NotImplemented(String::from(
                "EntryAspect::Header",
            ))),
            EntryAspect::LinkAdd(link_data, header) => {
                let entry = Entry::LinkAdd(link_data);LinkAdd;
                // TODO: These match statements could be refactored as there
                // is a  lot of repetition.
                match try_validate_from_entry_and_header(entry, header) {
                    Ok(chain_pair) => Ok(PendingValidationStruct::new(
                        chain_pair,
                        ValidatingWorkflow::HoldLink,
                    )),
                    Err(error) => {
                        let error_msg = format!(
                            "Tried to process EntryAspect::LinkAdd:\n{}", error
                        );
                        Err(HolochainError::ValidationFailed(String::from(error_msg)))
                    },
                }
            }
            EntryAspect::LinkRemove((link_data, links_to_remove), header) => {
                let entry = Entry::LinkRemove((link_data, links_to_remove));
                match try_validate_from_entry_and_header(entry, header) {
                    Ok(chain_pair) => Ok(PendingValidationStruct::new(
                        chain_pair,
                        ValidatingWorkflow::RemoveLink,
                    )),
                    Err(error) => {
                        let error_msg = format!(
                            "Tried to process\n
                            EntryAspect::LinkRemove((link_data, links_to_remove),
                            \nheader: {}. entry:\n{#:?}\nheader:\nlink_data:\n
                            {#:?}\nlinks_to_remove:\n{:#?}\nGot error:\n{}",
                            entry, header, link_data, links_to_remove, error
                        );
                        Err(HolochainError::ValidationFailed(String::from(error_msg)))
                    },
                }
            },
            EntryAspect::Update(entry, header) => {
                match try_validate_from_entry_and_header(entry, header) {
                    Ok(chain_pair) => Ok(PendingValidationStruct::new(
                        chain_pair,
                        ValidatingWorkflow::UpdateEntry,
                    )),
                    Err(error) => {
                        let error_msg = format!(
                            "Tried to process\n
                            EntryAspect::Update(entry, header),\n
                            entry:\n{#:?}\nheader:\n{:#?}\nGot error:\n{}",
                            entry, header, link_data, links_to_remove, error
                        );
                        Err(HolochainError::ValidationFailed(String::from(error_msg)))
                    },
                }
            },
            EntryAspect::Deletion(header) => {
                // reconstruct the deletion entry from the header.
                let deleted_entry_address = header.link_update_delete().ok_or_else(|| {
                    HolochainError::ValidationFailed(String::from(
                        "Deletion header is missing deletion link",
                    ))
                })?;
                let entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address));

                match ChainPair::try_validate_from_entry_and_header(entry, header) => {
                    Ok(chain_pair) => {Ok(PendingValidationStruct::new(
                        chain_pair,
                        ValidatingWorkflow::RemoveEntry,
                    ))},
                    Err(error) => {
                        HolochainError::ValidationFailed(
                        "Tried to process EntryAspect::Deletion(header)\n
                        where header is:\n{#:?}\nGot error:\n{}",
                        header, error
                    )},
                }
            },
        }
    }
}

pub fn run_pending_validations(context: Arc<Context>) {
    let pending_validations = context
        .state()
        .expect("Couldn't get state in run_pending_validations")
        .nucleus()
        .pending_validations
        .clone();

    pending_validations.iter().for_each(|(_, pending)| {
        log_debug!(
            context,
            "scheduled_jobs/run_pending_validations: found pending validation for {}: {}",
            pending.chain_pair.entry().entry_type(),
            pending.chain_pair.entry().address()
        );
        retry_validation(pending.clone(), context.clone());
    });
}
