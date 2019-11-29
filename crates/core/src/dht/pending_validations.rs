use crate::{
    entry::validation_dependencies::ValidationDependencies,
    network::entry_with_header::EntryWithHeader,
};
use holochain_core_types::{
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
    pub entry_with_header: EntryWithHeader,
    pub dependencies: Vec<Address>,
    pub workflow: ValidatingWorkflow,
    uuid: ProcessUniqueId,
}

impl PendingValidationStruct {
    pub fn new(entry_with_header: EntryWithHeader, workflow: ValidatingWorkflow) -> Self {
        let dependencies = entry_with_header.get_validation_dependencies();
        Self {
            entry_with_header,
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
}

impl TryFrom<EntryAspect> for PendingValidationStruct {
    type Error = HolochainError;
    fn try_from(aspect: EntryAspect) -> Result<PendingValidationStruct, HolochainError> {
        match aspect {
            EntryAspect::Content(entry, header) => Ok(PendingValidationStruct::new(
                EntryWithHeader::try_from_entry_and_header(entry, header)?,
                ValidatingWorkflow::HoldEntry,
            )),
            EntryAspect::Header(_header) => Err(HolochainError::NotImplemented(String::from(
                "EntryAspect::Header",
            ))),
            EntryAspect::LinkAdd(link_data, header) => {
                let entry = Entry::LinkAdd(link_data);
                Ok(PendingValidationStruct::new(
                    EntryWithHeader::try_from_entry_and_header(entry, header)?,
                    ValidatingWorkflow::HoldLink,
                ))
            }
            EntryAspect::LinkRemove((link_data, links_to_remove), header) => {
                let entry = Entry::LinkRemove((link_data, links_to_remove));
                Ok(PendingValidationStruct::new(
                    EntryWithHeader::try_from_entry_and_header(entry, header)?,
                    ValidatingWorkflow::RemoveLink,
                ))
            }
            EntryAspect::Update(entry, header) => Ok(PendingValidationStruct::new(
                EntryWithHeader::try_from_entry_and_header(entry, header)?,
                ValidatingWorkflow::UpdateEntry,
            )),
            EntryAspect::Deletion(header) => {
                // reconstruct the deletion entry from the header.
                let deleted_entry_address = header.link_update_delete().ok_or_else(|| {
                    HolochainError::ValidationFailed(String::from(
                        "Deletion header is missing deletion link",
                    ))
                })?;
                let entry = Entry::Deletion(DeletionEntry::new(deleted_entry_address));

                Ok(PendingValidationStruct::new(
                    EntryWithHeader::try_from_entry_and_header(entry, header)?,
                    ValidatingWorkflow::RemoveEntry,
                ))
            }
        }
    }
}

impl From<PendingValidationStruct> for EntryAspect {
    fn from(pending: PendingValidationStruct) -> EntryAspect {
        match pending.workflow {
            ValidatingWorkflow::HoldEntry => EntryAspect::Content(
                pending.entry_with_header.entry.clone(),
                pending.entry_with_header.header.clone(),
            ),
            ValidatingWorkflow::HoldLink => {
                let link_data = unwrap_to!(pending.entry_with_header.entry => Entry::LinkAdd);
                EntryAspect::LinkAdd(link_data.clone(), pending.entry_with_header.header.clone())
            }
            ValidatingWorkflow::RemoveLink => {
                let link_data = unwrap_to!(pending.entry_with_header.entry => Entry::LinkRemove);
                EntryAspect::LinkRemove(link_data.clone(), pending.entry_with_header.header.clone())
            }
            ValidatingWorkflow::UpdateEntry => EntryAspect::Update(
                pending.entry_with_header.entry.clone(),
                pending.entry_with_header.header.clone(),
            ),
            ValidatingWorkflow::RemoveEntry => {
                EntryAspect::Deletion(pending.entry_with_header.header.clone())
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingValidationWithTimeout {
    pub pending: PendingValidation,
    pub timeout: Option<ValidationTimeout>,
}

impl PendingValidationWithTimeout {
    pub fn new(pending: PendingValidation, timeout: Option<ValidationTimeout>) -> Self {
        Self { pending, timeout }
    }
}
