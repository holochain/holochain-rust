use holochain_core_types::{entry::Entry, error::HolochainError, json::*, signature::Provenance};

/// Structure used to specify what should be returned to a call to commit_entry_result()
/// The default is to return the latest entry.
#[derive(Deserialize, Debug, Serialize, DefaultJson, PartialEq, Clone)]
pub struct CommitEntryOptions {
    pub provenance: Vec<Provenance>,
}

impl Default for CommitEntryOptions {
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

#[derive(Deserialize, Debug, Serialize, DefaultJson)]
pub struct CommitEntryArgs {
    pub entry: Entry,
    pub options: CommitEntryOptions,
}

impl CommitEntryArgs {
    pub fn new(entry: Entry, options: CommitEntryOptions) -> Self {
        return Self { entry, options };
    }

    pub fn entry(&self) -> Entry {
        self.entry.clone()
    }

    pub fn options(&self) -> CommitEntryOptions {
        self.options.clone()
    }
}

#[cfg(test)]
mod tests {
    /*    use super::*;
    use holochain_core_types::{
        chain_header::test_chain_header,
        entry::{test_entry, test_entry_a, test_entry_b},
    };*/

}
