use crate::cas::content::Address;

/// A bridge is the definition of a connection to another DNA that runs under the same agency,
/// i.e. in the same container.
///
/// Defining a bridge means that the code in this DNA can call zome functions of that other
/// DNA.
///
/// The other DNA can either be referenced statically by exact DNA address/hash or dynamically
/// by defining the trait that other DNA has to implement in order to be used as bridge.
///
/// Bridges can be required or optional. If a required bridge DNA is not installed this DNA
/// can't run, so required bridges are hard dependencies that have to be enforced by the container.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
#[serde(untagged)]
pub enum Bridge {
    Address(AddressBridge),
    Trait(TraitBridge),
}

/// A bridge that defines another DNA statically by its address (i.e. hash).
/// If this variant is used the other DNA gets locked in as per DNA hash
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct AddressBridge {
    /// Required or optional
    pub presence: BridgePresence,

    /// An arbitrary name of this bridge that is used as handle to reference this
    /// bridge in according zome API functions
    pub handle: String,

    /// The address (= hash) of the other DNA that we want to use.
    pub dna_address: Address,
}

/// A bridge that defines another DNA loosely by expecting a DNA that implements
/// a given trait, i.e. that has a specific set of zome functions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct TraitBridge {
    /// Required or optional
    pub presence: BridgePresence,

    /// An arbitrary name of this bridge that is used as handle to reference this
    /// bridge in according zome API functions
    pub handle: String,

    /// The unique, qualified domain name of a predefined trait.
    /// Example: org.holochain.my-trait.
    pub library_trait: String,
}

/// Required or optional
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub enum BridgePresence {
    /// A required bridge is a dependency to another DNA.
    /// This DNA won't load without it.
    Required,

    /// An optional bridge may be missing.
    /// This DNA's code can check via API functions if the other DNA is installed and connected.
    Optional,
}
