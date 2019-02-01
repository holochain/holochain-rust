use crate::{
    agent::state::AgentState,
    context::Context,
    network::{direct_message::DirectMessage, state::NetworkState},
    nucleus::{
        actions::initialize::Initialization,
        ribosome::fn_call::{ExecuteZomeFnResponse, ZomeFnCall},
        state::{NucleusState, ValidationResult},
    },
};
use holochain_core_types::{
    cas::content::Address,
    chain_header::ChainHeader,
    dna::Dna,
    entry::{Entry, EntryWithMeta},
    error::HolochainError,
    json::JsonString,
    link::Link,
    validation::ValidationPackage,
};
use holochain_net_connection::json_protocol::{
    FetchEntryData, FetchEntryResultData, FetchMetaData, FetchMetaResultData,
};
use snowflake;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

/// Wrapper for actions that provides a unique ID
/// The unique ID is needed for state tracking to ensure that we can differentiate between two
/// Action dispatches containing the same value when doing "time travel debug".
/// The standard approach is to drop the ActionWrapper into the key of a state history HashMap and
/// use the convenience unwrap_to! macro to extract the action data in a reducer.
/// All reducer functions must accept an ActionWrapper so all dispatchers take an ActionWrapper.
#[derive(Clone, Debug)]
pub struct ActionWrapper {
    action: Action,
    id: snowflake::ProcessUniqueId,
}

impl ActionWrapper {
    /// constructor from &Action
    /// internal snowflake ID is automatically set
    pub fn new(a: Action) -> Self {
        ActionWrapper {
            action: a,
            // auto generate id
            id: snowflake::ProcessUniqueId::new(),
        }
    }

    /// read only access to action
    pub fn action(&self) -> &Action {
        &self.action
    }

    /// read only access to id
    pub fn id(&self) -> &snowflake::ProcessUniqueId {
        &self.id
    }
}

impl PartialEq for ActionWrapper {
    fn eq(&self, other: &ActionWrapper) -> bool {
        self.id == other.id
    }
}

impl Eq for ActionWrapper {}

impl Hash for ActionWrapper {
    /// @TODO dangerous when persisted!
    /// snowflake only guarantees uniqueness per process
    /// @see https://github.com/holochain/holochain-rust/issues/203
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// All Actions for the Holochain Instance Store, according to Redux pattern.
#[derive(Clone, PartialEq, Debug)]
pub enum Action {
    // ----------------
    // Agent actions:
    // ----------------
    /// Writes an entry to the source chain.
    /// Does not validate, assumes entry is valid.
    Commit((Entry, Option<Address>)),

    // -------------
    // DHT actions:
    // -------------
    /// Adds an entry to the local DHT shard.
    /// Does not validate, assumes entry is valid.
    Hold(Entry),

    /// Adds a link to the local DHT shard's meta/EAV storage
    /// Does not validate, assumes link is valid.
    AddLink(Link),

    // ----------------
    // Network actions:
    // ----------------
    /// Create a network proxy instance from the given [NetworkSettings](struct.NetworkSettings.html)
    InitNetwork(NetworkSettings),

    /// Makes the network PUT the given entry to the DHT.
    /// Distinguishes between different entry types and does
    /// the right thing respectively.
    /// (only publish for AppEntryType, publish and publish_meta for links etc)
    Publish(Address),

    /// Fetch an Entry on the network by address
    FetchEntry(GetEntryKey),

    /// Lets the network module respond to a FETCH request.
    /// Triggered from the corresponding workflow after retrieving the
    /// requested entry from our local DHT shard.
    RespondFetch((FetchEntryData, Option<EntryWithMeta>)),

    /// We got a response for our FETCH request which needs to be added to the state.
    /// Triggered from the network handler.
    HandleFetchResult(FetchEntryResultData),

    ///
    UpdateEntry((Address, Address)),
    ///
    RemoveEntry((Address, Address)),
    ///
    GetEntryTimeout(GetEntryKey),

    /// get links from entry address and tag name
    /// Last string is the stringified process unique id of this `hdk::get_links` call.
    GetLinks(GetLinksKey),
    GetLinksTimeout(GetLinksKey),
    RespondGetLinks((FetchMetaData, Vec<Address>)),
    HandleGetLinksResult((FetchMetaResultData, String)),

    /// Makes the network module send a direct (node-to-node) message
    /// to the address given in [DirectMessageData](struct.DirectMessageData.html)
    SendDirectMessage(DirectMessageData),

    /// Makes the direct message connection with the given ID timeout by adding an
    /// Err(HolochainError::Timeout) to NetworkState::custom_direct_message_replys.
    SendDirectMessageTimeout(String),

    /// Makes the network module forget about the direct message
    /// connection with the given ID.
    /// Triggered when we got an answer to our initial DM.
    ResolveDirectConnection(String),

    /// Makes the network module DM the source of the given entry
    /// and prepare for receiveing an answer
    GetValidationPackage(ChainHeader),

    /// Updates the state to hold the response that we got for
    /// our previous request for a validation package.
    /// Triggered from the network handler when we get the response.
    HandleGetValidationPackage((Address, Option<ValidationPackage>)),

    /// Updates the state to hold the response that we got for
    /// our previous custom direct message.
    /// /// Triggered from the network handler when we get the response.
    HandleCustomSendResponse((String, Result<String, String>)),

    // ----------------
    // Nucleus actions:
    // ----------------
    /// initialize an application from a Dna
    /// not the same as genesis
    /// may call genesis internally
    InitApplication(Dna),
    /// return the result of an InitApplication action
    /// the result is Some arbitrary string
    ReturnInitializationResult(Result<Initialization, String>),

    /// execute a function in a zome WASM
    ExecuteZomeFunction(ZomeFnCall),

    /// return the result of a zome WASM function call
    ReturnZomeFunctionResult(ExecuteZomeFnResponse),

    /// Execute a zome function call called by another zome function
    Call(ZomeFnCall),

    /// A validation result is returned from a local callback execution
    /// Key is an unique id of the calling context
    /// and the hash of the entry that was validated
    ReturnValidationResult(((snowflake::ProcessUniqueId, Address), ValidationResult)),

    /// A validation package was created locally and is reported back
    /// to be added to the state
    ReturnValidationPackage(
        (
            snowflake::ProcessUniqueId,
            Result<ValidationPackage, HolochainError>,
        ),
    ),
}

/// function signature for action handler functions
// @TODO merge these into a single signature
// @see https://github.com/holochain/holochain-rust/issues/194
pub type AgentReduceFn = ReduceFn<AgentState>;
pub type NetworkReduceFn = ReduceFn<NetworkState>;
pub type NucleusReduceFn = ReduceFn<NucleusState>;
pub type ReduceFn<S> = fn(Arc<Context>, &mut S, &ActionWrapper);

/// The unique key that represents a GetLinks request, used to associate the eventual
/// response with this GetLinks request
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GetLinksKey {
    /// The address of the Link base
    pub base_address: Address,

    /// The link tag
    pub tag: String,

    /// A unique ID that is used to pair the eventual result to this request
    pub id: String,
}

/// The unique key that represents a Get request, used to associate the eventual
/// response with this Get request
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GetEntryKey {
    /// The address of the entry to get
    pub address: Address,

    /// A unique ID that is used to pair the eventual result to this request
    pub id: String,
}

/// Everything the network module needs to know in order to send a
/// direct message.
#[derive(Clone, PartialEq, Debug)]
pub struct DirectMessageData {
    /// The address of the node to send a message to
    pub address: Address,

    /// The message itself
    pub message: DirectMessage,

    /// A unique message ID that is used to identify the response and attribute
    /// it to the right context
    pub msg_id: String,

    /// Should be true if we are responding to a previous message with this message.
    /// msg_id should then be the same as the in the message that we received.
    pub is_response: bool,
}

/// Everything the network needs to initialize
#[derive(Clone, PartialEq, Debug)]
pub struct NetworkSettings {
    /// JSON config that gets passed to [P2pNetwork](struct.P2pNetwork.html)
    /// determines how to connect to the network module.
    pub config: JsonString,

    /// DNA address is needed so the network module knows which network to
    /// connect us to.
    pub dna_address: Address,

    /// The network module needs to know who we are.
    /// This is this agent's address.
    pub agent_id: String,
}

#[cfg(test)]
pub mod tests {

    use crate::{
        action::{Action, ActionWrapper, GetEntryKey},
        nucleus::ribosome::fn_call::tests::test_call_response,
    };
    use holochain_core_types::entry::{expected_entry_address, test_entry};
    use test_utils::calculate_hash;

    /// dummy action
    pub fn test_action() -> Action {
        Action::FetchEntry(GetEntryKey {
            address: expected_entry_address(),
            id: String::from("test-id"),
        })
    }

    /// dummy action wrapper with test_action()
    pub fn test_action_wrapper() -> ActionWrapper {
        ActionWrapper::new(test_action())
    }

    /// dummy action wrapper with commit of test_entry()
    pub fn test_action_wrapper_commit() -> ActionWrapper {
        ActionWrapper::new(Action::Commit((test_entry(), None)))
    }

    /// dummy action for a get of test_hash()
    pub fn test_action_wrapper_get() -> ActionWrapper {
        ActionWrapper::new(Action::FetchEntry(GetEntryKey {
            address: expected_entry_address(),
            id: snowflake::ProcessUniqueId::new().to_string(),
        }))
    }

    pub fn test_action_wrapper_rzfr() -> ActionWrapper {
        ActionWrapper::new(Action::ReturnZomeFunctionResult(test_call_response()))
    }

    #[test]
    /// smoke test actions
    fn new_action() {
        let a1 = test_action();
        let a2 = test_action();

        // unlike actions and wrappers, signals are equal to themselves
        assert_eq!(a1, a2);
    }

    #[test]
    /// tests that new action wrappers take an action and ensure uniqueness
    fn new_action_wrapper() {
        let aw1 = test_action_wrapper();
        let aw2 = test_action_wrapper();

        // snowflake enforces uniqueness
        assert_eq!(aw1, aw1);
        assert_ne!(aw1, aw2);
    }

    #[test]
    /// tests read access to actions
    fn action_wrapper_action() {
        let aw1 = test_action_wrapper();
        let aw2 = test_action_wrapper();

        assert_eq!(aw1.action(), aw2.action());
        assert_eq!(aw1.action(), &test_action());
    }

    #[test]
    /// tests read access to action wrapper ids
    fn action_wrapper_id() {
        // can't set the ID directly (by design)
        // at least test that IDs are unique, and that hitting the id() method doesn't error
        let aw1 = test_action_wrapper();
        let aw2 = test_action_wrapper();

        assert_ne!(aw1.id(), aw2.id());
    }

    #[test]
    /// tests that action wrapper hashes are unique
    fn action_wrapper_hash() {
        let aw1 = test_action_wrapper();
        let aw2 = test_action_wrapper();

        assert_ne!(calculate_hash(&aw1), calculate_hash(&aw2));
    }

}
