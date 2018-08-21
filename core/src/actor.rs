use riker_default::DefaultModel;
use riker::actors::*;
use hash_table::pair::Pair;
use error::HolochainError;
use agent::keys::Keys;
use hash_table::pair_meta::PairMeta;
use riker_patterns::ask::ask;
use futures::executor::block_on;
use futures::Async;

#[derive(Clone, Debug)]
pub enum Protocol {
    SetTopPair(Option<Pair>),
    SetTopPairResult(Result<Option<Pair>, HolochainError>),

    GetTopPair,
    GetTopPairResult(Option<Pair>),

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
    GetMeta(String),
    GetMetaResult(Result<Option<PairMeta>, HolochainError>),

    /// HashTable::get_pair_meta()
    GetPairMeta(Pair),
    GetPairMetaResult(Result<Vec<PairMeta>, HolochainError>),

    /// HashTable::get()
    GetPair(String),
    GetPairResult(Result<Option<Pair>, HolochainError>),

    /// HashTable::commit()
    Commit(Pair),
    CommitResult(Result<(), HolochainError>),
}

lazy_static! {
    pub static ref SYS: ActorSystem<Protocol> = {
        let model: DefaultModel<Protocol> = DefaultModel::new();
        ActorSystem::new(&model).unwrap()
    };
}

impl Into<ActorMsg<Protocol>> for Protocol {
    fn into(self) -> ActorMsg<Protocol> {
        ActorMsg::User(self)
    }
}

pub trait AskSelf {
    fn ask(&self, message: Protocol) -> Protocol;
}

impl AskSelf for ActorRef<Protocol> {
    fn ask(&self, message: Protocol) -> Protocol {
        println!("ask: {:?}", message);
        let a = ask(&(*SYS), self, message);
        println!("block");
        loop {
            match ::futures::Future::poll(a) {
                Ok(Async::Ready(e)) => {
                    break Ok(e)
                },
                Ok(Async::NotReady(e)) => {},
                Err(e) => { break Err(e) },
            }
        }
        // block_on(a).unwrap()
    }
}
