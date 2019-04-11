//! This module extends Entry and EntryType with the CanPublish trait.

use holochain_core_types::entry::entry_type::{AppEntryType, EntryType};

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
                    return true;
                }
            }
        }

        let dna = context
            .get_dna()
            .expect("DNA must be present to test if entry is publishable.");

        let entry_type_name = self.to_string();
        context.log(format!("got entry type name of {}", entry_type_name));
        let maybe_def = dna.get_entry_type_def(entry_type_name.as_str());
        if maybe_def.is_none() {
            context.log("err/dht/context must hold an entry type definition to publish an entry.");
            return false;
        }
        let entry_type_def = maybe_def.unwrap();

        // app entry type must be publishable
        if !entry_type_def.sharing.clone().can_publish() {
            context.log(format!("debug/dht/entry {} is not publishable", entry_type_name));
            return false;
        }
        true
    }
}

//#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn test_types() -> Vec<EntryType> {
        vec![
            EntryType::App(AppEntryType::from("foo")),
            EntryType::Dna,
            EntryType::AgentId,
            EntryType::Deletion,
            EntryType::LinkAdd,
            EntryType::LinkRemove,
            EntryType::LinkList,
            EntryType::ChainHeader,
            EntryType::ChainMigrate,
            EntryType::CapToken,
            EntryType::CapTokenGrant,
        ]
    }

    #[test]
    fn can_publish_test() {
        /*        let dna = Dna::try_from(JsonString::from_json(
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
                                "test_type": {
                                    "description": "",
                                    "sharing": "private"
                                }
                            },
                            "code": {
                                "code": ""
                            },
                            "bridges": [
                            ]
                        }
                    }
                }"#,
            ))
            .unwrap();

            let file_storage = Arc::new(RwLock::new(
                    FilesystemStorage::new(tempdir().unwrap().path().to_str().unwrap()).unwrap(),
                    ));
            let mut context = Context::new(
                AgentId::generate_fake("TestAgent"),
                test_logger(),
                Arc::new(Mutex::new(SimplePersister::new(file_storage.clone()))),
                file_storage.clone(),
                file_storage.clone(),
                Arc::new(RwLock::new(
                        EavFileStorage::new(tempdir().unwrap().path().to_str().unwrap().to_string())
                        .unwrap(),
                        )),
                        P2pConfig::new_with_unique_memory_backend(),
                        None,
                        None,
                        );

            assert!(context.state().is_none());

            let global_state = Arc::new(RwLock::new(State::new(Arc::new(context.clone()))));
            context.set_state(global_state.clone());

            {
                let _read_lock = global_state.read().unwrap();
                assert!(context.state().is_some());
            }

            for t in test_types() {
                match t {
                    EntryType::Dna => assert!(!t.can_publish(context)),
                    EntryType::CapTokenGrant => assert!(!t.can_publish(context)),
                    _ => assert!(t.can_publish(context)),
                }
            }
        */
    }
}
