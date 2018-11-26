use crate::actor::{Protocol, SYS};
use holochain_core_types::{
    eav::{Attribute, Entity, EntityAttributeValue, Value},
    error::{HcResult, HolochainError},
};
use riker::actors::*;
use snowflake;
use std::collections::HashSet;

const ACTOR_ID_ROOT: &'static str = "/eav_memory_actor/";

fn actor_id() -> String {
    format!(
        "{}{}",
        ACTOR_ID_ROOT,
        snowflake::ProcessUniqueId::new().to_string()
    )
}

pub struct EavMemoryStorageActor {
    storage: HashSet<EntityAttributeValue>,
}

impl EavMemoryStorageActor {
    pub fn new() -> EavMemoryStorageActor {
        EavMemoryStorageActor {
            storage: HashSet::new(),
        }
    }

    /// actor() for riker
    fn actor() -> BoxActor<Protocol> {
        Box::new(EavMemoryStorageActor::new())
    }

    /// props() for riker
    fn props() -> BoxActorProd<Protocol> {
        Props::new(Box::new(EavMemoryStorageActor::actor))
    }

    pub fn new_ref() -> HcResult<ActorRef<Protocol>> {
        SYS.actor_of(
            EavMemoryStorageActor::props(),
            // always return the same reference to the same actor for the same path
            // consistency here provides safety for CAS methods
            &actor_id(),
        )
        .map_err(|actor_create_error| {
            HolochainError::ErrorGeneric(format!(
                "Failed to create actor in system: {:?}",
                actor_create_error
            ))
        })
    }

    fn unthreadable_add_eav(&mut self, eav: &EntityAttributeValue) -> HcResult<()> {
        self.storage.insert(eav.clone());
        Ok(())
    }

    fn unthreadable_fetch_eav(
        &self,
        entity: Option<Entity>,
        attribute: Option<Attribute>,
        value: Option<Value>,
    ) -> Result<HashSet<EntityAttributeValue>, HolochainError> {
        Ok(self
            .storage
            .iter()
            .cloned()
            .filter(|e| EntityAttributeValue::filter_on_eav::<Entity>(e.entity(), &entity))
            .filter(|e| EntityAttributeValue::filter_on_eav::<Attribute>(e.attribute(), &attribute))
            .filter(|e| EntityAttributeValue::filter_on_eav::<Value>(e.value(), &value))
            .collect::<HashSet<EntityAttributeValue>>())
    }
}

impl Actor for EavMemoryStorageActor {
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
                    Protocol::EavAdd(eav) => {
                        Protocol::EavAddResult(self.unthreadable_add_eav(&eav))
                    }
                    Protocol::EavFetch(e, a, v) => {
                        Protocol::EavFetchResult(self.unthreadable_fetch_eav(e, a, v))
                    }
                    _ => unreachable!(),
                },
                Some(context.myself()),
            )
            .expect("failed to tell FilesystemStorage sender");
    }
}
