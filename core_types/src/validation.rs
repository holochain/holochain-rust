//! This module defines structs that are used in the interchange
//! of data that is used for validation of chain modifying
//! agent actions between Holochain and Zomes.

use crate::{
    cas::content::Address,
    chain_header::ChainHeader,
    entry::{
        entry_type::{AppEntryType, EntryType},
        Entry,
    },
    error::HolochainError,
    json::JsonString,
    link::link_data::LinkData,
};

use chain_header::test_chain_header;

use std::convert::TryFrom;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, DefaultJson)]
pub struct ValidationPackage {
    pub chain_header: ChainHeader,
    pub source_chain_entries: Option<Vec<Entry>>,
    pub source_chain_headers: Option<Vec<ChainHeader>>,
    pub custom: Option<String>,
}

impl ValidationPackage {
    pub fn only_header(header: ChainHeader) -> ValidationPackage {
        ValidationPackage {
            chain_header: header,
            source_chain_entries: None,
            source_chain_headers: None,
            custom: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, DefaultJson)]
pub enum ValidationPackageDefinition {
    /// send the header for the entry, along with the entry
    Entry,
    /// sending all public source chain entries
    ChainEntries,
    /// sending all source chain headers
    ChainHeaders,
    /// sending the whole chain: public entries and all headers
    ChainFull,
    /// sending something custom
    Custom(String),
}

///This struct carries information needed for Validating Entry Data,
/// It is passed between callbacks and allows the user to validate
/// using each supplied variant.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum EntryValidationData<T> {
    /// The create variant contains an entry T and the validation package.
    Create {
        entry: T,
        validation_data: ValidationData,
    },
    /// The Modify variant contains the new entry T, old entry of the same type, the entry header of the old entry and a validation package
    Modify {
        new_entry: T,
        old_entry: T,
        old_entry_header: ChainHeader,
        validation_data: ValidationData,
    },
    /// The delete contains an old entry which is the entry being deleted and the old entry header of type ChainHeader and a validation package
    Delete {
        old_entry: T,
        old_entry_header: ChainHeader,
        validation_data: ValidationData,
    },
}

///This struct carries information needed for Validating Link Data,
/// It is passed between callbacks and allows the user to validate
/// using each supplied variant.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LinkValidationData {
    /// The LinkAdd variant contains a linkData and a validation package
    LinkAdd {
        link: LinkData,
        validation_data: ValidationData,
    },
    /// The LinkRemove variant contains a linkData and a validation package
    LinkRemove {
        link: LinkData,
        validation_data: ValidationData,
    },
}

impl TryFrom<EntryValidationData<Entry>> for EntryType {
    type Error = HolochainError;
    fn try_from(entry_validation: EntryValidationData<Entry>) -> Result<Self, Self::Error> {
        match entry_validation {
            EntryValidationData::Create { entry, .. } => {
                Ok(EntryType::App(AppEntryType::try_from(entry.entry_type())?))
            }
            EntryValidationData::Delete { old_entry, .. } => Ok(EntryType::App(
                AppEntryType::try_from(old_entry.entry_type())?,
            )),
            EntryValidationData::Modify { new_entry, .. } => Ok(EntryType::App(
                AppEntryType::try_from(new_entry.entry_type())?,
            )),
        }
    }
}

/// This structs carries information contextual for the process
/// of validating an entry of link and is passed in to the according
/// callbacks.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ValidationData {
    /// The validation package is data from the entry's/link's
    /// source agent that is needed to determine the validity
    /// of a given entry.
    /// What specific data gets put into the validation package
    /// has to be defined throught the validation_package
    /// callbacks in the [entry!](macro.entry.html) and
    /// [link!](macro.link.html) macros.
    pub package: ValidationPackage,
    /// In which lifecycle of the entry creation are we running
    /// this validation callback?
    pub lifecycle: EntryLifecycle,
}

impl Default for ValidationData {
    fn default() -> Self {
        Self {
            package: ValidationPackage {
                chain_header: test_chain_header(),
                source_chain_entries: None,
                source_chain_headers: None,
                custom: None,
            },
            lifecycle: EntryLifecycle::default(),
        }
    }
}

impl ValidationData {
    /// The list of authors that have signed this entry.
    pub fn sources(&self) -> Vec<Address> {
        self.package
            .chain_header
            .provenances()
            .iter()
            .map(|provenance| provenance.source())
            .collect()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum EntryLifecycle {
    Chain,
    Dht,
    Meta,
}

impl Default for EntryLifecycle {
    fn default() -> Self {
        EntryLifecycle::Chain
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum EntryAction {
    Create,
    Modify,
    Delete,
}

impl Default for EntryAction {
    fn default() -> Self {
        EntryAction::Create
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LinkAction {
    Create,
    Delete,
}
