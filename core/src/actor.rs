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
    TopPair,
    TopPairResult(Option<Pair>),

    /// HashTable::setup()
    Setup,
    SetupResult(Result<(), HolochainError>),

    /// HashTable::teardown()
    Teardown,
    TeardownResult(Result<(), HolochainError>),

    /// HashTable::modify()
    Modify {
        keys: Keys,
        old_pair: Pair,
        new_pair: Pair,
    },
    ModifyResult(Result<(), HolochainError>),

    /// HashTable::retract()
    Retract {
        keys: Keys,
        pair: Pair,
    },
    RetractResult(Result<(), HolochainError>),

    /// HashTable::assert_meta()
    AssertMeta(PairMeta),
    AssertMetaResult(Result<(), HolochainError>),

    /// HashTable::get_meta()
    Meta(String),
    MetaResult(Result<Option<PairMeta>, HolochainError>),

    /// HashTable::get_pair_meta()
    PairMeta(Pair),
    PairMetaResult(Result<Vec<PairMeta>, HolochainError>),

    /// HashTable::pair()
    Pair(String),
    PairResult(Result<Option<Pair>, HolochainError>),

    /// HashTable::commit()
    Commit(Pair),
    CommitResult(Result<(), HolochainError>),
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
    fn block_on_ask(&self, message: Protocol) -> Protocol;
}

impl AskSelf for ActorRef<Protocol> {
    fn block_on_ask(&self, message: Protocol) -> Protocol {
        let a = ask(&(*SYS), self, message);
        block_on(a).unwrap()
    }
}
