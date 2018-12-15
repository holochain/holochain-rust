use crate::cas::content::Address;
use crate::{dna::capabilities::CapabilityType, error::HolochainError, json::JsonString};

pub type CapTokenValue = String;

/// a struct to hold the signature of the call
pub struct CallSignature {
}

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

    pub fn create(cap_type:CapabilityType, assignees: Option<Vec<Address>>) -> Result<Self,HolochainError> {
        let assignees = CapTokenGrantEntry::valid(cap_type,assignees)?;
        Ok(CapTokenGrantEntry::new(assignees))
    }


    // internal check that type and assignees are valid for create
    fn valid(cap_type: CapabilityType, assignees: Option<Vec<Address>>) -> Result<Option<Vec<Address>>,HolochainError> {
        if (cap_type == CapabilityType::Public || cap_type == CapabilityType::Transferable) &&
            (assignees.is_some() && !assignees.clone().unwrap().is_empty()) {
                return Err(HolochainError::new("there must be no assignees for public or transferable grants"))
            }
        match cap_type {
            CapabilityType::Assigned => {
                if assignees.is_none() || assignees.clone().unwrap().is_empty() {
                    return Err(HolochainError::new("Assigned grant must have 1 or more assignees"))
                }
                Ok(assignees)
            },
            CapabilityType::Public => Ok(None),
            CapabilityType::Transferable => Ok(Some(Vec::new())),
        }
    }

    pub fn token(&self) -> CapTokenValue {
        self.token.clone()
    }

    pub fn cap_type(&self) -> CapabilityType {
        match self.assignees() {
            None => CapabilityType::Public,
            Some(vec) => if vec.is_empty() {
                CapabilityType::Transferable
            } else {
                CapabilityType::Assigned
            }
        }
    }

    pub fn assignees(&self) -> Option<Vec<Address>> {
        self.assignees.clone()
    }

    pub fn verify(&self,token:CapTokenValue,from: Option<Address>,_message: &CallSignature) -> bool {
        let cap_type = self.cap_type();
        if cap_type == CapabilityType::Public {
            return true;
        }
        if !from.is_some() {return false}
        if self.token != token {return false}

        // TODO: CallSignature check against Address

        match self.cap_type() {
            CapabilityType::Public => true,
            CapabilityType::Transferable => true,
            CapabilityType::Assigned => {
                if !self.assignees().unwrap().contains(&from.unwrap()) {
                    return false;
                }
                true
            }
        }
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
        assert_eq!(entry.cap_type(), CapabilityType::Assigned);
        assert_eq!(entry.assignees().unwrap()[0],test_address)
    }

    #[test]
    fn test_cap_grant_valid() {
        assert!(CapTokenGrantEntry::valid(CapabilityType::Public,None).is_ok());
        assert!(CapTokenGrantEntry::valid(CapabilityType::Public,Some(Vec::new())).is_ok());
        assert!(CapTokenGrantEntry::valid(CapabilityType::Public,Some(vec![Address::new()])).is_err());
        assert!(CapTokenGrantEntry::valid(CapabilityType::Transferable,None).is_ok());
        assert!(CapTokenGrantEntry::valid(CapabilityType::Transferable,Some(Vec::new())).is_ok());
        assert!(CapTokenGrantEntry::valid(CapabilityType::Transferable,Some(vec![Address::new()])).is_err());
        assert!(CapTokenGrantEntry::valid(CapabilityType::Assigned,None).is_err());
        assert!(CapTokenGrantEntry::valid(CapabilityType::Assigned,Some(Vec::new())).is_err());
        assert!(CapTokenGrantEntry::valid(CapabilityType::Assigned,Some(vec![Address::new()])).is_ok());
    }

    #[test]
    fn test_create_cap_token_grant_entry() {
        let maybe_entry = CapTokenGrantEntry::create(CapabilityType::Public,None);
        assert!(maybe_entry.is_ok());
        let entry = maybe_entry.unwrap();
        assert_eq!(entry.cap_type(), CapabilityType::Public);

        let maybe_entry = CapTokenGrantEntry::create(CapabilityType::Transferable,Some(Vec::new()));
        assert!(maybe_entry.is_ok());
        let entry = maybe_entry.unwrap();
        assert_eq!(entry.cap_type(), CapabilityType::Transferable);

        let test_address = Address::new();

        let maybe_entry = CapTokenGrantEntry::create(CapabilityType::Public,Some(vec![test_address.clone()]));
        assert!(maybe_entry.is_err());
        let maybe_entry = CapTokenGrantEntry::create(CapabilityType::Transferable,None);
        assert!(maybe_entry.is_ok());
        let entry = maybe_entry.unwrap();
        assert_eq!(entry.cap_type(), CapabilityType::Transferable);

        let maybe_entry = CapTokenGrantEntry::create(CapabilityType::Assigned,Some(vec![test_address.clone()]));
        assert!(maybe_entry.is_ok());
        let entry = maybe_entry.unwrap();
        assert_eq!(entry.cap_type(), CapabilityType::Assigned);
        assert_eq!(entry.assignees().unwrap()[0],test_address)
    }

    #[test]
    fn test_cap_grant_verify() {
        let test_address1 = Address::from("some identity");
        let test_address2 = Address::from("some other identity");
        let test_call_signature = &CallSignature{};

        let entry = CapTokenGrantEntry::create(CapabilityType::Public,None).unwrap();
        let token = entry.token();
        assert!(entry.verify(token.clone(),None,test_call_signature));
        assert!(entry.verify(token.clone(),Some(test_address1.clone()),test_call_signature));
        assert!(entry.verify("Bad Token".to_string(),None,test_call_signature));

        let entry = CapTokenGrantEntry::create(CapabilityType::Transferable,None).unwrap();
        let token = entry.token();
        assert!(!entry.verify(token.clone(),None,test_call_signature));
        assert!(entry.verify(token.clone(),Some(test_address1.clone()),test_call_signature));
        assert!(entry.verify(token.clone(),Some(test_address2.clone()),test_call_signature));
        assert!(!entry.verify("Bad Token".to_string(),Some(test_address1.clone()),test_call_signature));


        let entry = CapTokenGrantEntry::create(CapabilityType::Assigned,Some(vec![test_address1.clone()])).unwrap();
        let token = entry.token();
        assert!(!entry.verify(token.clone(),None,test_call_signature));
        assert!(entry.verify(token.clone(),Some(test_address1.clone()),test_call_signature));
        assert!(!entry.verify(token.clone(),Some(test_address2.clone()),test_call_signature));
        assert!(!entry.verify("Bad Token".to_string(),Some(test_address1.clone()),test_call_signature));
    }
}
