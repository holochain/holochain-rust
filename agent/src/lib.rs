//! holochain_agent provides a library for managing holochain agent info, including identities, keys etc..
extern crate holochain_core_types;
extern crate serde_json;

use holochain_core_types::{
    cas::content::AddressableContent, entry::Entry, entry_type::EntryType, entry::ToEntry,
};

/// Object holding an Agent's identity.
#[derive(Clone, Debug, PartialEq)]
pub struct Identity {
    content: String,
}

impl Identity {
    pub fn new(content: String) -> Self {
        Identity { content }
    }
}

/// Object holding all Agent's data.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    identity: Identity,
}

impl Agent {
    pub fn new(id: Identity) -> Self {
        Agent { identity: id }
    }

    pub fn from_string(text: String) -> Self {
        Agent::new(Identity { content: text })
    }
}

impl ToString for Agent {
    fn to_string(&self) -> String {
        self.identity.content.clone()
    }
}

impl ToEntry for Agent {
    fn to_entry(&self) -> Entry {
        Entry::new(&EntryType::AgentId, &self.to_string())
    }

    fn from_entry(entry: &Entry) -> Self {
        let id_content: String =
            serde_json::from_str(&entry.content()).expect("entry is not a valid AgentId Entry");
        Agent::new(Identity::new(id_content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_instantiate_agent() {
        let agent = Agent::new(Identity {
            content: "bob".to_string(),
        });
        assert_eq!(agent.identity.content, "bob".to_string());

        let agent = Agent::from_string("jane".to_string());
        assert_eq!(agent.identity.content, "jane".to_string());

        assert_eq!(agent.to_string(), "jane".to_string());
    }
}
