use agent::keys::Keys;
use error::HolochainError;
use futures::executor::block_on;
use hash_table::{pair::Pair, pair_meta::PairMeta};
use riker::actors::*;
use riker_default::DefaultModel;
use riker_patterns::ask::ask;

#[derive(Clone, Debug)]
/// riker protocol for all our actors
/// currently this is flat but may be nested/namespaced in the future or multi-protocol riker
/// @see https://github.com/riker-rs/riker/issues/17
pub enum Protocol {
    /// Chain::set_top_pair()
    SetTopPair(Option<Pair>),
    SetTopPairResult(Result<Option<Pair>, HolochainError>),

    /// Chain::top_pair()
    GetTopPair,
    GetTopPairResult(Option<Pair>),

    /// HashTable::setup()
    Setup,
    SetupResult(Result<(), HolochainError>),

    /// HashTable::teardown()
    Teardown,
    TeardownResult(Result<(), HolochainError>),

    /// HashTable::modify_entry()
    ModifyPair {
        keys: Keys,
        old_pair: Pair,
        new_pair: Pair,
    },
    ModifyPairResult(Result<(), HolochainError>),

    /// HashTable::retract_pair()
    RetractPair {
        keys: Keys,
        pair: Pair,
    },
    RetractPairResult(Result<(), HolochainError>),

    /// HashTable::assert_meta()
    AssertMeta(PairMeta),
    AssertMetaResult(Result<(), HolochainError>),

    /// HashTable::pair_meta()
    GetPairMeta(String),
    GetPairMetaResult(Result<Option<PairMeta>, HolochainError>),

    /// HashTable::all_metas_for_pair()
    GetMetasForPair(Pair),
    GetMetasForPairResult(Result<Vec<PairMeta>, HolochainError>),

    /// HashTable::pair()
    GetPair(String),
    GetPairResult(Result<Option<Pair>, HolochainError>),

    /// HashTable::put_pair()
    PutPair(Pair),
    PutPairResult(Result<(), HolochainError>),
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

/// required by riker
impl Into<ActorMsg<Protocol>> for Protocol {
    fn into(self) -> ActorMsg<Protocol> {
        ActorMsg::User(self)
    }
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
        match block_on(a) {
            Ok(block_result) => Ok(block_result),
            Err(_) => Err(HolochainError::NotImplemented),
        }
    }
}
