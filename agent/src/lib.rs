//! holochain_agent provides a library for managing holochain agent info, including identities, keys etc..
extern crate holochain_core_types;
extern crate serde;
extern crate serde_json;

use holochain_core_types::{
    cas::content::{AddressableContent, Content},
    entry::{Entry, ToEntry},
    entry_type::EntryType,
    json::{JsonString, RawString},
};

/// Object holding an Agent's identity.
#[derive(Clone, Debug, PartialEq)]
pub struct Identity(Content);

impl From<Identity> for JsonString {
    fn from(identity: Identity) -> JsonString {
        identity.0
    }
}

impl From<JsonString> for Identity {
    fn from(json_string: JsonString) -> Identity {
        Identity(json_string)
    }
}

impl From<String> for Identity {
    fn from(s: String) -> Identity {
        // use RawString as the identity coming in as a string is not yet Content for historical
        // reasons
        Identity::from(JsonString::from(RawString::from(s)))
    }
}

impl From<Identity> for String {
    fn from(identity: Identity) -> String {
        String::from(RawString::from(identity.0))
    }
}

/// Object holding all Agent's data.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent(Identity);

impl From<String> for Agent {
    fn from(s: String) -> Agent {
        Agent::from(Identity::from(s))
    }
}

impl From<Agent> for String {
    fn from(agent: Agent) -> String {
        String::from(agent.0)
    }
}

impl From<Identity> for Agent {
    fn from(identity: Identity) -> Agent {
        Agent(identity)
    }
}

impl From<JsonString> for Agent {
    fn from(json_string: JsonString) -> Agent {
        Agent::from(Identity::from(json_string))
    }
}

impl From<Agent> for JsonString {
    fn from(agent: Agent) -> JsonString {
        (agent.0).0
    }
}

impl ToEntry for Agent {
    fn to_entry(&self) -> Entry {
        Entry::new(&EntryType::AgentId, &JsonString::from(self.to_owned()))
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
    use holochain_core_types::{
        cas::content::Content,
        json::{JsonString, RawString},
    };

    pub fn test_identity_value() -> Content {
        JsonString::from(RawString::from("bob"))
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
        assert_eq!(test_identity_value(), JsonString::from(test_identity()));
    }

    #[test]
    /// show ToString implementation for Agent
    fn agent_to_string_test() {
        assert_eq!(test_identity_value(), JsonString::from(test_agent()));
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
        let expected_content = JsonString::from("{\"value\":\"bob\",\"entry_type\":\"AgentId\"}");
        // content()
        assert_eq!(expected_content, test_agent().content(),);

        // from_content()
        assert_eq!(test_agent(), Agent::from_content(&expected_content),);
    }
}
