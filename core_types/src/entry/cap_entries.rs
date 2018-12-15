use crate::cas::content::Address;
use crate::{dna::capabilities::CapabilityType, error::HolochainError, json::JsonString};

pub type CapTokenValue = String;

/// System entry to hold a capability token for use as a caller
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

/// System entry to hold a capabilities granted by the callee
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct CapTokenGrantEntry {
    assignees: Option<Vec<Address>>,
    token: CapTokenValue,
}

fn gen_token() -> CapTokenValue {
    "fake_token".to_string()
}

impl CapTokenGrantEntry {
    pub fn new(assignees: Option<Vec<Address>>) -> Self {
        CapTokenGrantEntry {
            assignees: assignees,
            token: gen_token(),
        }
    }

    pub fn verify(cap_type: CapabilityType, assignees: Option<Vec<Address>>) -> Result<(),HolochainError> {
        if (cap_type == CapabilityType::Public || cap_type == CapabilityType::Transferable) &&
            assignees.is_some() {
                return Err(HolochainError::new("assignees must be none"))
            }
        match cap_type {
            CapabilityType::Assigned => {
                if assignees.is_none() || assignees.clone().unwrap().is_empty() {
                    return Err(HolochainError::new("Assigned grant must have 1 or more assignees"))
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }

    pub fn token(self) -> CapTokenValue {
        self.token
    }

    pub fn cap_type(self) -> CapabilityType {
        match self.assignees {
            None => CapabilityType::Public,
            Some(vec) => if vec.is_empty() {
                CapabilityType::Transferable
            } else {
                CapabilityType::Assigned
            }
        }
    }

    pub fn assignees(self) -> Option<Vec<Address>> {
        self.assignees.clone()
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
        let entry = CapTokenGrantEntry::new(None);
        assert_eq!(entry.cap_type(), CapabilityType::Public);
        let entry = CapTokenGrantEntry::new(Some(Vec::new()));
        assert_eq!(entry.cap_type(), CapabilityType::Transferable);
        let test_address = Address::new();
        let entry = CapTokenGrantEntry::new(Some(vec![test_address.clone()]));
        assert_eq!(entry.clone().cap_type(), CapabilityType::Assigned);
        assert_eq!(entry.assignees().unwrap()[0],test_address)
    }

    #[test]
    fn test_cap_grant_verify() {
        assert_eq!(CapTokenGrantEntry::verify(CapabilityType::Public,None),Ok(()));
        assert!(CapTokenGrantEntry::verify(CapabilityType::Public,Some(Vec::new())).is_err());
        assert_eq!(CapTokenGrantEntry::verify(CapabilityType::Transferable,None),Ok(()));
        assert!(CapTokenGrantEntry::verify(CapabilityType::Transferable,Some(Vec::new())).is_err());
        assert!(CapTokenGrantEntry::verify(CapabilityType::Assigned,Some(Vec::new())).is_err());
        assert!(CapTokenGrantEntry::verify(CapabilityType::Assigned,Some(vec![Address::new()])).is_ok());
    }
}
