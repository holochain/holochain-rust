use crate::{dna::capabilities::CapabilityType, error::HolochainError, json::JsonString};

pub type CapTokenValue = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct CapTokenEntry {
    token: CapTokenValue,
}

impl CapTokenEntry {
    pub fn new(token: CapTokenValue) -> Self {
        CapTokenEntry { token }
    }
    pub fn token(self) -> CapTokenValue {
        self.token
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct CapTokenGrantEntry {
    cap_type: CapabilityType,
    token: CapTokenValue,
}

fn gen_token() -> CapTokenValue {
    "fake_token".to_string()
}

impl CapTokenGrantEntry {
    pub fn new(cap_type: CapabilityType) -> Self {
        CapTokenGrantEntry {
            cap_type,
            token: gen_token(),
        }
    }
    pub fn token(self) -> CapTokenValue {
        self.token
    }

    pub fn cap_type(self) -> CapabilityType {
        self.cap_type
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_new_cap_token_entry() {
        let token = gen_token();
        let cap_token_entry = CapTokenEntry::new(token.clone());
        assert_eq!(token, cap_token_entry.token());
    }

    #[test]
    fn test_new_cap_token_grant_entry() {
        let cap_token_grant_entry = CapTokenGrantEntry::new(CapabilityType::Public);
        assert_eq!(cap_token_grant_entry.cap_type(), CapabilityType::Public);
    }

}
