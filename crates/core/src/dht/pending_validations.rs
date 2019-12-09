use crate::{
    entry::validation_dependencies::ValidationDependencies,
    network::chain_pair::ChainPair,
};
use holochain_core_types::{
    chain_header::ChainHeader,
    entry::{deletion_entry::DeletionEntry, Entry},
    error::HolochainError,
    network::entry_aspect::EntryAspect,
};
use holochain_json_api::{error::JsonError, json::JsonString};
use holochain_persistence_api::cas::content::Address;
use snowflake::ProcessUniqueId;
use std::{
    convert::TryFrom,
    fmt,
    sync::Arc,
    time::{Duration, SystemTime},
};

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
        let dependencies = chain_pair.get_validation_dependencies();
        Self {
            chain_pair,
            dependencies,
            workflow,
            uuid: ProcessUniqueId::new(),
        }
    }

    pub fn same(&self) -> Self {
        let mut clone = self.clone();
        clone.uuid = ProcessUniqueId::new();
        clone
    }

    /// Convenience function for returning a custom error in the context of validation.
    pub fn try_from_entry_and_header(
        entry: Entry,
        header: ChainHeader,
        entry_aspect: EntryAspect,
        validating_workflow: ValidatingWorkflow,
    ) -> Result<PendingValidationStruct, HolochainError> {
        match ChainPair::try_from_header_and_entry(header, entry) {
            Ok(chain_pair) => Ok(PendingValidationStruct::new(
                chain_pair,
                validating_workflow,
            )),
            Err(error) => {
                let error = format!(
                    "Tried to process {}; see the\n
                debug output for further details of its contents. {}",
                    entry_aspect, error
                );
                debug!("Tried to process {:?}", entry_aspect);
                Err(HolochainError::ValidationFailed(error))
            }
        }
    }
}

impl TryFrom<EntryAspect> for PendingValidationStruct {
    type Error = HolochainError;
    fn try_from(aspect: EntryAspect) -> Result<PendingValidationStruct, HolochainError> {
        match aspect {
            EntryAspect::Content(entry, header) => {
                PendingValidationStruct::try_from_entry_and_header(
                    entry.clone(),
                    header.clone(),
                    EntryAspect::Content(entry, header),
                    ValidatingWorkflow::HoldEntry,
                )
            }
            EntryAspect::Header(_header) => Err(HolochainError::NotImplemented(String::from(
                "EntryAspect::Header",
            ))),
            EntryAspect::LinkAdd(link_data, header) => {
                let entry = Entry::LinkAdd(link_data.clone());
                PendingValidationStruct::try_from_entry_and_header(
                    entry,
                    header.clone(),
                    EntryAspect::LinkAdd(link_data, header),
                    ValidatingWorkflow::HoldLink,
                )
            }
            EntryAspect::LinkRemove((link_data, links_to_remove), header) => {
                let entry = Entry::LinkRemove((link_data.clone(), links_to_remove.clone()));
                PendingValidationStruct::try_from_entry_and_header(
                    entry,
                    header.clone(),
                    EntryAspect::LinkRemove((link_data, links_to_remove), header),
                    ValidatingWorkflow::RemoveLink,
                )
            }
            EntryAspect::Update(entry, header) => {
                PendingValidationStruct::try_from_entry_and_header(
                    entry.clone(),
                    header.clone(),
                    EntryAspect::Update(entry, header),
                    ValidatingWorkflow::UpdateEntry,
                )
            }
            EntryAspect::Deletion(header) => {
                // reconstruct the deletion entry from the header.
                let deleted_entry_address = header.link_update_delete().ok_or_else(|| {
                    HolochainError::ValidationFailed(String::from(
                        "Deletion header is missing deletion link",
                    ))
                })?;
                let entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address));

                PendingValidationStruct::try_from_entry_and_header(
                    entry,
                    header.clone(),
                    EntryAspect::Deletion(header),
                    ValidatingWorkflow::RemoveEntry,
                )
            }
        }
    }
}

impl From<PendingValidationStruct> for EntryAspect {
    fn from(pending: PendingValidationStruct) -> EntryAspect {
        let entry = pending.chain_pair.entry();
        let header = pending.chain_pair.header();
        match pending.workflow {
            ValidatingWorkflow::HoldEntry => EntryAspect::Content(entry, header),
            ValidatingWorkflow::HoldLink => {
                let link_data = unwrap_to!(entry => Entry::LinkAdd);
                EntryAspect::LinkAdd(link_data.clone(), header)
            }
            ValidatingWorkflow::RemoveLink => {
                let link_data = unwrap_to!(entry => Entry::LinkRemove);
                EntryAspect::LinkRemove(link_data.clone(), header.clone())
            }
            ValidatingWorkflow::UpdateEntry => EntryAspect::Update(entry.clone(), header),
            ValidatingWorkflow::RemoveEntry => EntryAspect::Deletion(header.clone()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ValidationTimeout {
    pub time_of_dispatch: SystemTime,
    pub delay: Duration,
}

impl ValidationTimeout {
    pub fn new(time_of_dispatch: SystemTime, delay: Duration) -> Self {
        Self {
            time_of_dispatch,
            delay,
        }
    }
}

impl From<(SystemTime, Duration)> for ValidationTimeout {
    fn from(tuple: (SystemTime, Duration)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PendingValidationWithTimeout {
    pub pending: PendingValidation,
    pub timeout: Option<ValidationTimeout>,
}

impl PendingValidationWithTimeout {
    pub fn new(pending: PendingValidation, timeout: Option<ValidationTimeout>) -> Self {
        Self { pending, timeout }
    }
}
