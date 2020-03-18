//! This module extends Entry and EntryType with the CanPublish trait.

pub mod validation_dependencies;

use holochain_core_types::entry::entry_type::EntryType;

use crate::context::Context;
pub trait CanPublish {
    fn can_publish(&self, context: &Context) -> bool;
}

//#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
impl CanPublish for EntryType {
    fn can_publish(&self, context: &Context) -> bool {
        match self {
            EntryType::Dna | EntryType::CapTokenGrant | EntryType::CapTokenClaim => return false,
            _ => {
                if self.is_sys() {
                    return true;
                }
            }
        }

        let dna = context
            .get_dna()
            .expect("DNA must be present to test if entry is publishable.");

        let entry_type_name = self.to_string();
        let maybe_def = dna.get_entry_type_def(entry_type_name.as_str());
        if maybe_def.is_none() {
            log_error!("dht/context must hold an entry type definition to publish an entry.");
            return false;
        }
        let entry_type_def = maybe_def.unwrap();

        // app entry type must be publishable
        if !entry_type_def.sharing.clone().can_publish() {
            log_debug!(context, "dht/entry {} is not publishable", entry_type_name);
            return false;
        }
        true
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use holochain_core_types::entry::entry_type::{AppEntryType, EntryType};
    use holochain_persistence_api::cas::content::{Address, AddressableContent};

    use test_utils::create_arbitrary_test_dna;

    use crate::network::test_utils::test_instance_with_spoofed_dna;

    pub fn test_types() -> Vec<EntryType> {
        vec![
            EntryType::App(AppEntryType::from("testEntryType")),
            EntryType::App(AppEntryType::from("testEntryTypeC")),
            EntryType::Dna,
            EntryType::AgentId,
            EntryType::Deletion,
            EntryType::LinkAdd,
            EntryType::LinkRemove,
            EntryType::LinkList,
            EntryType::ChainHeader,
            EntryType::ChainMigrate,
            EntryType::CapTokenClaim,
            EntryType::CapTokenGrant,
        ]
    }

    #[test]
    fn can_publish_test() {
        let dna = create_arbitrary_test_dna();
        let spoofed_dna_address: Address = dna.address();
        let name = "test";
        let (_instance, context) =
            test_instance_with_spoofed_dna(dna, spoofed_dna_address, name).unwrap();
        for t in test_types() {
            match t.clone() {
                EntryType::Dna => assert!(!t.can_publish(&context)),
                EntryType::CapTokenGrant => assert!(!t.can_publish(&context)),
                EntryType::CapTokenClaim => assert!(!t.can_publish(&context)),
                EntryType::App(entry_type_name) => match entry_type_name.to_string().as_str() {
                    "testEntryType" => assert!(t.can_publish(&context)),
                    "testEntryTypeC" => {
                        assert!(context
                            .get_dna()
                            .unwrap()
                            .get_entry_type_def("testEntryTypeC")
                            .is_some());
                        assert!(!t.can_publish(&context))
                    }
                    _ => assert!(false, "impossible entry type name"),
                },
                _sys_entry_type => {
                    assert!(t.is_sys());
                    assert!(t.can_publish(&context));
                }
            }
        }
    }
}
