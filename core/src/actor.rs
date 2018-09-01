use error::HolochainError;
use futures::executor::block_on;
use hash_table::{HashString, pair::Pair, meta::Meta, links_entry::Link, entry::Entry,
                 links_entry::LinkListEntry};
use riker::actors::*;
use riker_default::DefaultModel;
use riker_patterns::ask::ask;
use nucleus::ribosome::api::get_links::GetLinksArgs;
use agent::keys::Keys;

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
        old_entry: Entry,
        new_entry: Entry,
    },
    ModifyResult(Result<(), HolochainError>),

    /// HashTable::retract()
    Retract {
        keys: Keys,
        entry: Entry,
    },
    RetractResult(Result<(), HolochainError>),

    AddLink(Link),
    AddLinkResult(Result<(), HolochainError>),

    Links(GetLinksArgs),
    LinksResult(Result<Option<LinkListEntry>, HolochainError>),

    /// HashTable::assert_meta()
    AssertMeta(Meta),
    AssertMetaResult(Result<(), HolochainError>),

    /// HashTable::get_meta()
    Meta(String),
    MetaResult(Result<Option<Meta>, HolochainError>),

    MetaFor{entry_hash: HashString, attribute_name: String},
    MetaForResult(Result<Option<Meta>, HolochainError>),

    /// HashTable::get_pair_meta()
//    PairMeta(Pair),
//    PairMetaResult(Result<Vec<Meta>, HolochainError>),

    EntryMeta(Entry),
    EntryMetaResult(Result<Vec<Meta>, HolochainError>),

    /// HashTable::entry()
    Entry(String),
    EntryResult(Result<Option<Entry>, HolochainError>),


    /// HashTable::put()
    Put(Entry),
    PutResult(Result<(), HolochainError>),
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
