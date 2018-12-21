//! This module defines structs that are used in the interchange
//! of data that is used for validation of chain modifying
//! agent actions between Holochain and Zomes.

extern crate serde_json;
use crate::{
    chain_header::ChainHeader, entry::Entry, error::HolochainError, hash::HashString,
    json::JsonString,
};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, DefaultJson, Default)]
pub struct ValidationPackage {
    pub chain_header: Option<ChainHeader>,
    pub source_chain_entries: Option<Vec<Entry>>,
    pub source_chain_headers: Option<Vec<ChainHeader>>,
    pub custom: Option<String>,
}

impl ValidationPackage {
    pub fn only_header(header: ChainHeader) -> ValidationPackage {
        ValidationPackage {
            chain_header: Some(header),
            source_chain_entries: None,
            source_chain_headers: None,
            custom: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, DefaultJson)]
pub enum ValidationPackageDefinition {
    /// sending only the entry
    Entry,
    /// sending all (public?) source chain entries
    ChainEntries,
    /// sending all source chain headers
    ChainHeaders,
    /// sending the whole chain, entries and headers
    ChainFull,
    /// sending something custom
    Custom(String),
}

/// This structs carries information contextual for the process
/// of validating an entry of link and is passed in to the according
/// callbacks.
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ValidationData {
    /// The validation package is data from the entry's/link's
    /// source agent that is needed to determine the validity
    /// of a given entry.
    /// What specific data gets put into the validation package
    /// has to be defined throught the validation_package
    /// callbacks in the [entry!](macro.entry.html) and
    /// [link!](macro.link.html) macros.
    pub package: ValidationPackage,
    /// The list of authors that have signed this entry.
    pub sources: Vec<HashString>,
    /// In which lifecycle of the entry creation are we running
    /// this validation callback?
    pub lifecycle: EntryLifecycle,
    /// Does the entry get committed, modified or deleted?
    pub action: EntryAction,
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
