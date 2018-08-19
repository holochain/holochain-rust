use agent::keys::Keys;
use error::HolochainError;
use hash_table::{entry::Entry, pair::Pair, pair_meta::PairMeta};
use riker::actors::*;
use riker_default::DefaultModel;

#[derive(Clone, Debug)]
pub enum Protocol {
    ChainTopPair,
    ChainTopPairResult(Option<Pair>),

    ChainTopPairType(String),
    ChainTopPairTypeResult(Option<Pair>),

    ChainPushEntry(Entry),
    ChainPushEntryResult(Result<Pair, HolochainError>),

    ChainPushPair(Pair),
    ChainPushPairResult(Result<Pair, HolochainError>),

    ChainGetEntry(String),
    ChainGetEntryResult(Result<Option<Pair>, HolochainError>),

    ChainGetPair(String),
    ChainGetPairResult(Result<Option<Pair>, HolochainError>),

    /// HashTable::setup()
    HashTableSetup,
    HashTableSetupResult(Result<(), HolochainError>),

    /// HashTable::teardown()
    HashTableTeardown,
    HashTableTeardownResult(Result<(), HolochainError>),

    /// HashTable::modify()
    HashTableModify {
        keys: Keys,
        old_pair: Pair,
        new_pair: Pair,
    },
    HashTableModifyResult(Result<(), HolochainError>),

    /// HashTable::retract()
    HashTableRetract {
        keys: Keys,
        pair: Pair,
    },
    HashTableRetractResult(Result<(), HolochainError>),

    /// HashTable::assert_meta()
    HashTableAssertMeta(PairMeta),
    HashTableAssertMetaResult(Result<(), HolochainError>),

    /// HashTable::get_meta()
    HashTableGetMeta(String),
    HashTableGetMetaResult(Result<Option<PairMeta>, HolochainError>),

    /// HashTable::get_pair_meta()
    HashTableGetPairMeta(Pair),
    HashTableGetPairMetaResult(Result<Vec<PairMeta>, HolochainError>),

    /// HashTable::get()
    HashTableGetPair(String),
    HashTableGetPairResult(Result<Option<Pair>, HolochainError>),

    /// HashTable::commit()
    HashTableCommit(Pair),
    HashTableCommitResult(Result<(), HolochainError>),
}

lazy_static! {
    pub static ref SYS: ActorSystem<Protocol> = {
        let model: DefaultModel<Protocol> = DefaultModel::new();
        // let hash_table_model = HashTableModel{};
        ActorSystem::new(&model).unwrap()
    };
}

impl Into<ActorMsg<Protocol>> for Protocol {
    fn into(self) -> ActorMsg<Protocol> {
        ActorMsg::User(self)
    }
}
