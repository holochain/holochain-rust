extern crate serde_json;

// Enum for listing all System Entry Types
// Variant `Data` is for user defined entry types
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum EntryType {
    AgentId,
    Deletion,
    App(String),
    Dna,
    ChainHeader,
    Key,
    Link,
    Migration,
    /// TODO #339 - This is different kind of SystemEntry for the DHT only.
    /// Should be moved into a different enum for DHT entry types.
    LinkList,
}

pub type Address = String;
pub type HashString = String;
/// ChainHeader of a source chain "Item"
/// The hash of the ChainHeader is used as the Item's key in the source chain hash table
/// ChainHeaders are linked to next header in chain and next header of same type in chain
// @TODO - serialize properties as defined in ChainHeadersEntrySchema from golang alpha 1
// @see https://github.com/holochain/holochain-proto/blob/4d1b8c8a926e79dfe8deaa7d759f930b66a5314f/entry_headers.go#L7
// @see https://github.com/holochain/holochain-rust/issues/75
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainHeader {
    /// the type of this entry
    /// system types may have associated "subconscious" behavior
    entry_type: EntryType,
    /// ISO8601 time stamp
    timestamp: String,
    /// Key to the immediately preceding header. Only the genesis Pair can have None as valid
    link: Option<Address>,
    /// Key to the entry of this header
    entry_address: Address,
    /// agent's cryptographic signature of the entry
    entry_signature: String,
    /// Key to the most recent header of the same type, None is valid only for the first of that type
    link_same_type: Option<Address>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ValidationData {
    pub chain_header: Option<ChainHeader>,
    pub sources : Vec<HashString>,
    pub source_chain_entries : Option<Vec<serde_json::Value>>,
    pub source_chain_headers : Option<Vec<ChainHeader>>,
    pub custom : Option<serde_json::Value>,
    pub lifecycle : HcEntryLifecycle,
    pub action : HcEntryAction,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum HcEntryLifecycle {
    Chain,
    Dht,
    Meta,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum HcEntryAction {
    Commit,
    Modify,
    Delete,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum HcLinkAction {
    Commit,
    Delete,
}

//#[derive(Serialize, Deserialize)]
//#[serde(remote = "Either")]
//pub enum EitherDef<L, R> {
//    Left(L),
//    Right(R),
//}