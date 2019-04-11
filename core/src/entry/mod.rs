//! This module extends Entry and EntryType with the CanPublish trait.

use holochain_core_types::entry::entry_type::EntryType;

use crate::context::Context;
pub trait CanPublish {
    fn can_publish(&self, context: &Context) -> bool;
}


impl CanPublish for EntryType {
    fn can_publish(&self, context: &Context) -> bool {
        match self {
            EntryType::Dna => return false,
            EntryType::CapTokenGrant => return false,
            _ => {
                if self.is_sys() {
                    return true
                }
            }
        }

        let dna = context
            .get_dna()
            .expect("DNA must be present to test if entry is publishable.");

        let entry_type_name = self.to_string();

        let maybe_def = dna.get_entry_type_def(entry_type_name.as_str());
        if maybe_def.is_none() {
            context.log(format!("core/entry: context must hold an entry \
                         type definition to publish an entry."));
            return false;
        }
        let entry_type_def = maybe_def.unwrap();

        // app entry type must be publishable
        if !entry_type_def.sharing.clone().can_publish() {
            context.log(format!("core/entry: entry {} is not publishable.",
                                entry_type_name));
            false;
        }
        true
    }
}

#[cfg(test)]
pub mod tests {

    use std::sync::{Arc, RwLock, Mutex};
    use holochain_cas_implementations::{
        cas::{memory::MemoryStorage},
        eav::{memory::EavMemoryStorage}
    };

    use crate::{
        logger::test_logger,
        persister::SimplePersister,
        state::State,
        agent::{chain_store::ChainStore, state::AgentState},
        nucleus::state::NucleusState,
    };

    use holochain_net::p2p_config::P2pConfig;

    use holochain_core_types::{
        entry::entry_type::{AppEntryType, EntryType},
        cas::{content::AddressableContent, storage::ContentAddressableStorage},
        chain_header::{ChainHeader, test_provenances},
        time::test_iso_8601,
        dna::Dna,
        json::JsonString,
        agent::AgentId,
    };

    pub fn test_chain_store(dna:&AddressableContent) -> ChainStore {
        let mut memory_storage =
            MemoryStorage::new();
        memory_storage.add(dna).expect("Failed to add dna to storage.");
        ChainStore::new(Arc::new(RwLock::new(memory_storage)))
    }

    use super::*;

    pub fn test_types() -> Vec<EntryType> { vec![
        EntryType::App(AppEntryType::from("my_private_entry")),
        EntryType::App(AppEntryType::from("my_public_entry")), EntryType::Dna, EntryType::AgentId,
        EntryType::Deletion, EntryType::LinkAdd, EntryType::LinkRemove, EntryType::LinkList,
        EntryType::ChainHeader, EntryType::ChainMigrate, EntryType::CapToken,
        EntryType::CapTokenGrant, ] }

    #[test]
    fn can_publish_test() {
        let fixture = String::from(
            r#"{
                    "name": "test",
                    "description": "test",
                    "version": "test",
                    "uuid": "00000000-0000-0000-0000-000000000000",
                    "dna_spec_version": "2.0",
                    "properties": {
                        "test": "test"
                    },
                    "zomes": {
                        "test zome": {
                            "name": "test zome",
                            "description": "test",
                            "config": {},
                            "traits": {
                                "hc_public": {
                                    "functions": []
                                }
                            },
                            "fn_declarations": [],
                            "entry_types": {
                                "my_private_entry": {
                                    "description": "my_private_entry.",
                                    "sharing": "private"
                                },
                                "my_public_entry": {
                                    "description": "my_private_entry",
                                    "sharing": "public"
                                }
                            },
                            "code": {
                                "code": ""
                            },
                            "bridges": [
                            ]
                        }
                    }
                }"#);

                let content =
                    JsonString::from_json(&fixture);
                let dna : Dna =
                    Dna::try_from_content(&content).expect("DNA parsing error.");

            assert!(dna.get_entry_type_def("my_private_entry").is_some());
            let cas = Arc::new(RwLock::new(MemoryStorage::new()));
            let mut context = Context::new(
            AgentId::generate_fake("TestAgent"), test_logger(),
            Arc::new(Mutex::new(SimplePersister::new(cas.clone()))), cas.clone(), cas.clone(),
            Arc::new(RwLock::new(EavMemoryStorage::new())),
            P2pConfig::new_with_unique_memory_backend(), None, None,);

            assert!(context.state().is_none());

            let agent_state = AgentState::new_with_top_chain_header
                (test_chain_store(&dna),
                 Some(ChainHeader::new(&EntryType::Dna, &dna.address(),
                                       &test_provenances("sig"),
                                       &None, &None, &None, &test_iso_8601())));
            let mut nucleus = NucleusState::new();
            nucleus.dna = Option::from(dna);

            let global_state =
                Arc::new(RwLock::new
                         (State::new_with_agent_and_nucleus
                          (Arc::new(context.clone()), agent_state, nucleus)));
            context.set_state(global_state.clone());
            {
                let _read_lock = global_state.read().unwrap();
                assert!(context.state().is_some());
            }
            for t in test_types() {
                match t.clone() {
                    EntryType::Dna => assert!(!t.can_publish(&context)),
                    EntryType::CapTokenGrant => assert!(!t.can_publish(&context)),
                    EntryType::App(entry_type_name) =>
                        match entry_type_name.to_string().as_str() {
                           "my_public_entry" => assert!(t.can_publish(&context)),
                           "my_private_entry" =>
                               {
                                assert!(context.get_dna().unwrap().
                                        get_entry_type_def("my_private_entry").is_some());
                                assert!(!t.can_publish(&context))
                               }
                           _ => assert!(false, "impossible entry type name")
                        },
                    _sys_entry_type =>
                        {
                            assert!(t.is_sys());
                            assert!(t.can_publish(&context));
                        }
               }
            }
    }
}
