use agent::keys::Keys;
use cas::content::{Address, Content};
// use chain::header::ChainHeader;
use error::HolochainError;
use futures::executor::block_on;
use hash_table::{
    entry::Entry,
    entry_meta::EntryMeta,
    links_entry::{Link, LinkListEntry},
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
    CasAdd(Address, Content),
    CasAddResult(Result<(), HolochainError>),

    CasFetch(Address),
    CasFetchResult(Result<Option<Content>, HolochainError>),

    CasContains(Address),
    CasContainsResult(Result<bool, HolochainError>),

    /// Chain::set_top_chain_header()
    // SetTopChainHeader(Option<ChainHeader>),
    // SetTopChainHeaderResult(Result<Option<ChainHeader>, HolochainError>),

    /// Chain::top_chain_header()
    // GetTopChainHeader,
    // GetTopChainHeaderResult(Option<ChainHeader>),

    /// HashTable::setup()
    Setup,
    SetupResult(Result<(), HolochainError>),

    /// HashTable::teardown()
    Teardown,
    TeardownResult(Result<(), HolochainError>),

    /// HashTable::get()
    GetEntry(Address),
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
    GetMeta(Address),
    GetMetaResult(Result<Option<EntryMeta>, HolochainError>),

    /// HashTable::metas_from_entry()
    MetasFromEntry(Entry),
    MetasFromEntryResult(Result<Vec<EntryMeta>, HolochainError>),

    /// HashTable::meta_from_request()
    MetaFromRequest {
        entry_address: Address,
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
    fn block_on_ask(&self, message: Protocol) -> Result<Protocol, HolochainError>;
}

impl AskSelf for ActorRef<Protocol> {
    fn block_on_ask(&self, message: Protocol) -> Result<Protocol, HolochainError> {
        let a = ask(&(*SYS), self, message);
        Ok(block_on(a)?)
    }
}
