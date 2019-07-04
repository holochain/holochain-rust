use holochain_core_types::{entry::Entry, signature::Provenance};

use holochain_json_api::{error::JsonError, json::*};
use holochain_persistence_api::cas::content::Address;

/// Structure used to specify additional options to a commit_entry_result call.
#[derive(Deserialize, Debug, Serialize, DefaultJson, PartialEq, Clone)]
pub struct CommitEntryOptions {
    pub provenance: Vec<Provenance>,
}

impl Default for CommitEntryOptions {
    /// The default CommitEntryOptions has no additional provenance.
    fn default() -> Self {
        CommitEntryOptions { provenance: vec![] }
    }
}

impl CommitEntryOptions {
    pub fn new(provenance: Vec<Provenance>) -> Self {
        Self { provenance }
    }

    pub fn provenance(&self) -> Vec<Provenance> {
        self.provenance.clone()
    }
}

/// The arguments required to execute a commit_entry_result() call.
#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct CommitEntryArgs {
    pub entry: Entry,
    pub options: CommitEntryOptions,
}

impl CommitEntryArgs {
    pub fn new(entry: Entry, options: CommitEntryOptions) -> Self {
        Self { entry, options }
    }

    pub fn entry(&self) -> Entry {
        self.entry.clone()
    }

    pub fn options(&self) -> CommitEntryOptions {
        self.options.clone()
    }
}

/// Represents any useful information to return after
/// entries are committed
#[derive(Deserialize, Debug, Clone, Serialize, DefaultJson)]
pub struct CommitEntryResult {
    pub address: Address,
}

impl CommitEntryResult {
    pub fn new(address: Address) -> Self {
        Self { address }
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }
}
