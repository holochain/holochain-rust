use crate::network::entry_with_header::EntryWithHeader;
use holochain_core_types::{
    entry::{deletion_entry::DeletionEntry, Entry},
    error::HolochainError,
    network::entry_aspect::EntryAspect,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::cas::content::Address;
use snowflake::ProcessUniqueId;
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
    uuid: ProcessUniqueId,
}

impl PendingValidationStruct {
    pub fn new(chain_pair: ChainPair, workflow: ValidatingWorkflow) -> Self {
        Self {
            chain_pair,
            dependencies: Vec::new(),
            workflow,
            uuid: ProcessUniqueId::new(),
        }
    }

    pub fn same(&self) -> Self {
        let mut clone = self.clone();
        clone.uuid = ProcessUniqueId::new();
        clone
    }
}

impl TryFrom<EntryAspect> for PendingValidationStruct {
    type Error = HolochainError;
    fn try_from(aspect: EntryAspect) -> Result<PendingValidationStruct, HolochainError> {
        match aspect {
            ea @ EntryAspect::Content(entry, header) => {
                ChainPair::try_validate_from_entry_and_header(
                    entry,
                    header,
                    ea,
                    ValidatingWorkflow::HoldEntry,
                )
            },
            EntryAspect::Header(_header) => Err(HolochainError::NotImplemented(String::from(
                "EntryAspect::Header",
            ))),
            ea @ EntryAspect::LinkAdd(link_data, header) => {
                let entry = Entry::LinkAdd(link_data);
                ChainPair::try_validate_from_entry_and_header(
                    entry,
                    header,
                    ea,
                    ValidatingWorkflow::HoldLink,
                )
            }
            ea @ EntryAspect::LinkRemove((link_data, links_to_remove), header) => {
                let entry = Entry::LinkRemove((link_data, links_to_remove));
                ChainPair::try_validate_from_entry_and_header(
                    entry,
                    header,
                    ea,
                    ValidatingWorkflow::RemoveLink,
                )
            },
            ea @ EntryAspect::Update(entry, header) => {
                ChainPair::try_validate_from_entry_and_header(
                    entry,
                    header,
                    ea,
                    ValidatingWorkflow::UpdateEntry,
                )
            },
            ea @ EntryAspect::Deletion(header) => {
                // reconstruct the deletion entry from the header.
                let deleted_entry_address = header.link_update_delete().ok_or_else(|| {
                    HolochainError::ValidationFailed(String::from(
                        "Deletion header is missing deletion link",
                    ))
                })?;
                let entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address));

                ChainPair::try_validate_from_entry_and_header(
                    entry,
                    header,
                    ea,
                    ValidatingWorkflow::RemoveEntry,
                )
            },
        }
    }
}
