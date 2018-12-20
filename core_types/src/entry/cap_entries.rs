use crate::{
    cas::content::{Address, AddressableContent},
    dna::capabilities::{CallSignature, CapabilityType},
    entry::Entry,
    error::HolochainError,
    json::JsonString,
};

pub type CapTokenValue = Address;

/// System entry to hold a capability token for use as a caller
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct CapToken {
    token: CapTokenValue,
}

impl CapToken {
    pub fn new(token: CapTokenValue) -> Self {
        CapToken { token }
    }
    pub fn token(self) -> CapTokenValue {
        self.token
    }
}

/// System entry to hold a capabilities granted by the callee
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, DefaultJson)]
pub struct CapTokenGrant {
    assignees: Option<Vec<Address>>,
}

impl CapTokenGrant {
    fn new(assignees: Option<Vec<Address>>) -> Self {
        CapTokenGrant {
            assignees: assignees,
        }
    }

    pub fn create(
        cap_type: CapabilityType,
        assignees: Option<Vec<Address>>,
    ) -> Result<Self, HolochainError> {
        let assignees = CapTokenGrant::valid(cap_type, assignees)?;
        Ok(CapTokenGrant::new(assignees))
    }

    // internal check that type and assignees are valid for create
    fn valid(
        cap_type: CapabilityType,
        assignees: Option<Vec<Address>>,
    ) -> Result<Option<Vec<Address>>, HolochainError> {
        if (cap_type == CapabilityType::Public || cap_type == CapabilityType::Transferable)
            && (assignees.is_some() && !assignees.clone().unwrap().is_empty())
        {
            return Err(HolochainError::new(
                "there must be no assignees for public or transferable grants",
            ));
        }
        match cap_type {
            CapabilityType::Assigned => {
                if assignees.is_none() || assignees.clone().unwrap().is_empty() {
                    return Err(HolochainError::new(
                        "Assigned grant must have 1 or more assignees",
                    ));
                }
                Ok(assignees)
            }
            CapabilityType::Public => Ok(None),
            CapabilityType::Transferable => Ok(Some(Vec::new())),
        }
    }

    // the token value is address of the entry, so we can just build it
    // and take the address.
    pub fn token(&self) -> CapTokenValue {
        let addr: Address = Entry::CapTokenGrant((*self).clone()).address();
        addr
    }

    pub fn cap_type(&self) -> CapabilityType {
        match self.assignees() {
            None => CapabilityType::Public,
            Some(vec) => {
                if vec.is_empty() {
                    CapabilityType::Transferable
                } else {
                    CapabilityType::Assigned
                }
            }
        }
    }

    pub fn assignees(&self) -> Option<Vec<Address>> {
        self.assignees.clone()
    }

    /// verifies that this grant is valid for a given requester and token value
    pub fn verify(
        &self,
        token: CapTokenValue,
        from: Option<Address>,
        _message: &CallSignature,
    ) -> bool {
        let cap_type = self.cap_type();
        if cap_type == CapabilityType::Public {
            return true;
        }
        if !from.is_some() {
            return false;
        }

        if self.token() != token {
            return false;
        }

        // TODO: CallSignature check against Address

        match self.cap_type() {
            CapabilityType::Public => true,
            CapabilityType::Transferable => true,
            CapabilityType::Assigned => {
                // unwraps are safe because type comes from the shape of
                // the assignee, and the from must some by the check above.
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
    fn test_new_cap_token_grant_entry() {
        let grant = CapTokenGrant::new(None);
        assert_eq!(grant.cap_type(), CapabilityType::Public);
        let grant = CapTokenGrant::new(Some(Vec::new()));
        assert_eq!(grant.cap_type(), CapabilityType::Transferable);
        let test_address = Address::new();
        let grant = CapTokenGrant::new(Some(vec![test_address.clone()]));
        assert_eq!(grant.cap_type(), CapabilityType::Assigned);
        assert_eq!(grant.assignees().unwrap()[0], test_address)
    }

    #[test]
    fn test_cap_grant_valid() {
        assert!(CapTokenGrant::valid(CapabilityType::Public, None).is_ok());
        assert!(CapTokenGrant::valid(CapabilityType::Public, Some(Vec::new())).is_ok());
        assert!(CapTokenGrant::valid(CapabilityType::Public, Some(vec![Address::new()])).is_err());
        assert!(CapTokenGrant::valid(CapabilityType::Transferable, None).is_ok());
        assert!(CapTokenGrant::valid(CapabilityType::Transferable, Some(Vec::new())).is_ok());
        assert!(
            CapTokenGrant::valid(CapabilityType::Transferable, Some(vec![Address::new()])).is_err()
        );
        assert!(CapTokenGrant::valid(CapabilityType::Assigned, None).is_err());
        assert!(CapTokenGrant::valid(CapabilityType::Assigned, Some(Vec::new())).is_err());
        assert!(CapTokenGrant::valid(CapabilityType::Assigned, Some(vec![Address::new()])).is_ok());
    }

    #[test]
    fn test_create_cap_token_grant_entry() {
        let maybe_grant = CapTokenGrant::create(CapabilityType::Public, None);
        assert!(maybe_grant.is_ok());
        let grant = maybe_grant.unwrap();
        assert_eq!(grant.cap_type(), CapabilityType::Public);

        let maybe_grant = CapTokenGrant::create(CapabilityType::Transferable, Some(Vec::new()));
        assert!(maybe_grant.is_ok());
        let grant = maybe_grant.unwrap();
        assert_eq!(grant.cap_type(), CapabilityType::Transferable);

        let test_address = Address::new();

        let maybe_grant =
            CapTokenGrant::create(CapabilityType::Public, Some(vec![test_address.clone()]));
        assert!(maybe_grant.is_err());
        let maybe_grant = CapTokenGrant::create(CapabilityType::Transferable, None);
        assert!(maybe_grant.is_ok());
        let grant = maybe_grant.unwrap();
        assert_eq!(grant.cap_type(), CapabilityType::Transferable);

        let maybe_grant =
            CapTokenGrant::create(CapabilityType::Assigned, Some(vec![test_address.clone()]));
        assert!(maybe_grant.is_ok());
        let grant = maybe_grant.unwrap();
        assert_eq!(grant.cap_type(), CapabilityType::Assigned);
        assert_eq!(grant.assignees().unwrap()[0], test_address)
    }

    #[test]
    fn test_cap_grant_verify() {
        let test_address1 = Address::from("some identity");
        let test_address2 = Address::from("some other identity");
        let test_call_signature = &CallSignature {};

        let grant = CapTokenGrant::create(CapabilityType::Public, None).unwrap();
        let token = grant.token();
        assert!(grant.verify(token.clone(), None, test_call_signature));
        assert!(grant.verify(
            token.clone(),
            Some(test_address1.clone()),
            test_call_signature
        ));
        assert!(grant.verify(Address::from("Bad Token"), None, test_call_signature));

        let grant = CapTokenGrant::create(CapabilityType::Transferable, None).unwrap();
        let token = grant.token();
        assert!(!grant.verify(token.clone(), None, test_call_signature));
        assert!(grant.verify(
            token.clone(),
            Some(test_address1.clone()),
            test_call_signature
        ));
        assert!(grant.verify(
            token.clone(),
            Some(test_address2.clone()),
            test_call_signature
        ));
        assert!(!grant.verify(
            Address::from("Bad Token"),
            Some(test_address1.clone()),
            test_call_signature
        ));

        let grant =
            CapTokenGrant::create(CapabilityType::Assigned, Some(vec![test_address1.clone()]))
                .unwrap();
        let token = grant.token();
        assert!(!grant.verify(token.clone(), None, test_call_signature));
        assert!(grant.verify(
            token.clone(),
            Some(test_address1.clone()),
            test_call_signature
        ));
        assert!(!grant.verify(
            token.clone(),
            Some(test_address2.clone()),
            test_call_signature
        ));
        assert!(!grant.verify(
            Address::from("Bad Token"),
            Some(test_address1.clone()),
            test_call_signature
        ));
    }
}
