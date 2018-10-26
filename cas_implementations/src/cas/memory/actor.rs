use actor::{Protocol, SYS};
use holochain_core_types::{
    cas::content::{Address, Content},
    error::HolochainError,
};
use riker::actors::*;
use snowflake;
use std::collections::HashMap;

const ACTOR_ID_ROOT: &str = "/memory_storage_actor/";

fn actor_id() -> String {
    format!(
        "{}{}",
        ACTOR_ID_ROOT,
        snowflake::ProcessUniqueId::new().to_string()
    )
}

pub struct MemoryStorageActor {
    storage: HashMap<Address, Content>,
}

impl MemoryStorageActor {
    pub fn new() -> MemoryStorageActor {
        MemoryStorageActor {
            storage: HashMap::new(),
        }
    }

    fn actor() -> BoxActor<Protocol> {
        Box::new(MemoryStorageActor::new())
    }

    fn props() -> BoxActorProd<Protocol> {
        Props::new(Box::new(MemoryStorageActor::actor))
    }

    pub fn new_ref() -> Result<ActorRef<Protocol>, HolochainError> {
        SYS.actor_of(
            MemoryStorageActor::props(),
            // all actors have the same ID to allow round trip across clones
            &actor_id(),
        ).map_err(|actor_create_error| {
            HolochainError::ErrorGeneric(format!(
                "Failed to create actor in system: {:?}",
                actor_create_error
            ))
        })
    }

    fn unthreadable_add(
        &mut self,
        address: &Address,
        content: &Content,
    ) -> Result<(), HolochainError> {
        self.storage.insert(address.clone(), content.clone());
        Ok(())
    }

    fn unthreadable_contains(&self, address: &Address) -> Result<bool, HolochainError> {
        Ok(self.storage.contains_key(address))
    }

    fn unthreadable_fetch(&self, address: &Address) -> Result<Option<Content>, HolochainError> {
        Ok(self.storage.get(address).cloned())
    }
}

impl Actor for MemoryStorageActor {
    type Msg = Protocol;

    fn receive(
        &mut self,
        context: &Context<Self::Msg>,
        message: Self::Msg,
        sender: Option<ActorRef<Self::Msg>>,
    ) {
        sender
            .try_tell(
                match message {
                    Protocol::CasAdd(address, content) => {
                        Protocol::CasAddResult(self.unthreadable_add(&address, &content))
                    }
                    Protocol::CasContains(address) => {
                        Protocol::CasContainsResult(self.unthreadable_contains(&address))
                    }
                    Protocol::CasFetch(address) => {
                        Protocol::CasFetchResult(self.unthreadable_fetch(&address))
                    }
                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("failed to tell MemoryStorageActor sender");
    }
}
