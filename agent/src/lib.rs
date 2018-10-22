//! holochain_agent provides a library for managing holochain agent info, including identities, keys etc..
extern crate holochain_core_types;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate serde;

use holochain_core_types::{
    cas::content::{AddressableContent, Content}, entry::{Entry, ToEntry}, entry_type::EntryType,
};

/// Object holding an Agent's identity.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Identity(Content);

impl Identity {
    pub fn new(content: Content) -> Self {
        Identity(content)
    }
}

impl ToString for Identity {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<String> for Identity {
    fn from(s: String) -> Identity {
        Identity::new(s)
    }
}

/// Object holding all Agent's data.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Agent(Identity);

impl Agent {
    pub fn new(id: Identity) -> Self {
        Agent(id)
    }
}

impl ToString for Agent {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<String> for Agent {
    fn from(s: String) -> Agent {
        Agent::new(Identity::from(s))
    }
}

impl ToEntry for Agent {
    fn to_entry(&self) -> Entry {
        Entry::new(&EntryType::AgentId, &self.to_string())
    }

    fn from_entry(entry: &Entry) -> Self {
        assert_eq!(&EntryType::AgentId, entry.entry_type());
        Agent::from(entry.value().to_owned())
    }
}

impl AddressableContent for Agent {
    fn content(&self) -> Content {
        self.to_entry().content()
    }

    fn from_content(content: &Content) -> Self {
        Agent::from_entry(&Entry::from_content(content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use holochain_core_types::cas::content::Content;

    pub fn test_identity_value() -> Content {
        "bob".to_string()
    }

    pub fn test_identity() -> Identity {
        Identity(test_identity_value())
    }

    pub fn test_agent() -> Agent {
        Agent(test_identity())
    }

    #[test]
    /// smoke test new identities
    fn new_identity_test() {
        test_identity();
    }

    #[test]
    /// smoke test new agents
    fn new_agent_test() {
        test_agent();
    }

    #[test]
    /// show ToString implementation for Identity
    fn identity_to_string_test() {
        assert_eq!(test_identity_value(), test_identity().to_string(),);
    }

    #[test]
    /// show ToString implementation for Agent
    fn agent_to_string_test() {
        assert_eq!(test_identity_value(), test_agent().to_string(),)
    }

    #[test]
    /// show ToEntry implementation for Agent
    fn agent_to_entry_test() {
        // to_entry()
        assert_eq!(
            Entry::new(&EntryType::AgentId, &test_identity_value()),
            test_agent().to_entry(),
        );

        // from_entry()
        assert_eq!(
            test_agent(),
            Agent::from_entry(&Entry::new(&EntryType::AgentId, &test_identity_value())),
        );
    }

    #[test]
    /// show AddressableContent implementation for Agent
    fn agent_addressable_content_test() {
        let expected_content = String::from("{\"value\":\"bob\",\"entry_type\":\"AgentId\"}");
        // content()
        assert_eq!(expected_content, test_agent().content(),);

        // from_content()
        assert_eq!(test_agent(), Agent::from_content(&expected_content),);
    }
}
