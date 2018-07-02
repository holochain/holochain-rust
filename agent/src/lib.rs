//! holochain_agent provides a library for managing holochain agent info, including identities, keys etc..

#[derive(Clone, Debug, PartialEq)]
pub struct Identity {
    content: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    identity: Identity,
}

impl Agent {
    pub fn new(id: Identity) -> Self {
        Agent { identity: id }
    }
    pub fn from_string(text: &str) -> Self {
        Agent::new(Identity {
            content: text.to_string(),
        })
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

        let agent = Agent::from_string("jane");
        assert_eq!(agent.identity.content, "jane".to_string());
    }
}
