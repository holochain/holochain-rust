use futures::executor::block_on;
use holochain_core_types::{
    cas::content::{Address, Content}, eav::{Attribute, Entity, EntityAttributeValue, Value},
    error::HolochainError,
};
use riker::actors::*;
use riker_default::DefaultModel;
use riker_patterns::ask::ask;
use std::collections::HashSet;

#[derive(Clone, Debug)]
/// riker protocol for all our actors
/// currently this is flat but may be nested/namespaced in the future or multi-protocol riker
/// @see https://github.com/riker-rs/riker/issues/17
pub enum Protocol {
    CasAdd(Address, Content),
    CasAddResult(Result<(), HolochainError>),

    CasFetch(Address),
    CasFetchResult(Result<Option<Content>, HolochainError>),

    CasContains(Address),
    CasContainsResult(Result<bool, HolochainError>),

    EavAdd(EntityAttributeValue),
    EavAddResult(Result<(), HolochainError>),

    EavFetch(Option<Entity>, Option<Attribute>, Option<Value>),
    EavFetchResult(Result<HashSet<EntityAttributeValue>, HolochainError>),
}

/// required by riker
impl Into<ActorMsg<Protocol>> for Protocol {
    fn into(self) -> ActorMsg<Protocol> {
        ActorMsg::User(self)
    }
}

/// this is the global state that manages every actor
/// to be thread/concurrency safe there must only ever be one actor system
/// @see https://github.com/riker-rs/riker/issues/17
/// @see http://riker.rs/actors/#creating-actors
lazy_static! {
    pub static ref SYS: ActorSystem<Protocol> = {
        let model: DefaultModel<Protocol> = DefaultModel::new();
        ActorSystem::new(&model).unwrap()
    };
}

/// convenience trait to build fake synchronous facades for actors
pub trait AskSelf {
    /// adapter for synchronous code to interact with an actor
    /// uses the ask() fn from riker patterns under the hood to create a future then block on it
    /// handles passing the actor system through to ask() to hide that implementation detail
    /// @see http://riker.rs/patterns/#ask
    fn block_on_ask(&self, message: Protocol) -> Result<Protocol, HolochainError>;
}

impl AskSelf for ActorRef<Protocol> {
    fn block_on_ask(&self, message: Protocol) -> Result<Protocol, HolochainError> {
        let a = ask(&(*SYS), self, message);
        Ok(block_on(a)?)
    }
}
