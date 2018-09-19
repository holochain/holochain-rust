use agent::keys::Keys;
use chain::pair::Pair;
use error::HolochainError;
use futures::executor::block_on;
use hash::HashString;
use hash_table::{
    entry::Entry,
    links_entry::{Link, LinkListEntry},
    meta::EntryMeta,
};
use nucleus::ribosome::api::get_links::GetLinksArgs;
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

    /// HashTable::get()
    GetEntry(HashString),
    GetEntryResult(Result<Option<Entry>, HolochainError>),

    /// HashTable::put()
    PutEntry(Entry),
    PutEntryResult(Result<(), HolochainError>),

    /// HashTable::modify_entry()
    ModifyEntry {
        keys: Keys,
        old: Entry,
        new: Entry,
    },
    ModifyEntryResult(Result<(), HolochainError>),

    /// HashTable::retract_entry()
    RetractEntry {
        keys: Keys,
        entry: Entry,
    },
    RetractEntryResult(Result<(), HolochainError>),

    /// HashTable::add_link()
    AddLink(Link),
    AddLinkResult(Result<(), HolochainError>),
    /// HashTable::get_links()
    GetLinks(GetLinksArgs),
    GetLinksResult(Result<Option<LinkListEntry>, HolochainError>),

    /// HashTable::assert_meta()
    AssertMeta(EntryMeta),
    AssertMetaResult(Result<(), HolochainError>),

    /// HashTable::get_meta()
    GetMeta(HashString),
    GetMetaResult(Result<Option<EntryMeta>, HolochainError>),

    /// HashTable::metas_from_entry()
    MetasFromEntry(Entry),
    MetasFromEntryResult(Result<Vec<EntryMeta>, HolochainError>),

    /// HashTable::meta_from_request()
    MetaFromRequest {
        entry_hash: HashString,
        attribute_name: String,
    },
    MetaFromRequestResult(Result<Option<EntryMeta>, HolochainError>),
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
