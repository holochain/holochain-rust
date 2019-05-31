use holochain_core_types::{cas::content::Address, hash::HashString};
use multihash::Hash;

// CONSTS
pub static META_CRUD_ATTRIBUTE: &'static str = "crud";
pub static META_LINK_ATTRIBUTE: &'static str = "link__yay";

lazy_static! {
    // Agents
    pub static ref ALEX_AGENT_ID: Address = HashString::from("alex");
    pub static ref BILLY_AGENT_ID: Address = HashString::from("billy");
    pub static ref CAMILLE_AGENT_ID: Address = HashString::from("camille");
    // DNAs
    pub static ref DNA_ADDRESS_A: Address = HashString::from("DNA_A");
    pub static ref DNA_ADDRESS_B: Address = HashString::from("DNA_B");
    pub static ref DNA_ADDRESS_C: Address = HashString::from("DNA_C");
    // Entries
    pub static ref ENTRY_ADDRESS_1: Address = HashString::from("entry_addr_1");
    pub static ref ENTRY_ADDRESS_2: Address = HashString::from("entry_addr_2");
    pub static ref ENTRY_ADDRESS_3: Address = HashString::from("entry_addr_3");
    // Aspects
    pub static ref ASPECT_CONTENT_1: Vec<u8> = "hello-1".as_bytes().to_vec();
    pub static ref ASPECT_CONTENT_2: Vec<u8> = "l-2".as_bytes().to_vec();
    pub static ref ASPECT_CONTENT_3: Vec<u8> = "ChainHeader-3".as_bytes().to_vec();
    pub static ref ASPECT_ADDRESS_1: Address = generate_address(&*ASPECT_CONTENT_1);
    pub static ref ASPECT_ADDRESS_2: Address = generate_address(&*ASPECT_CONTENT_2);
    pub static ref ASPECT_ADDRESS_3: Address = generate_address(&*ASPECT_CONTENT_3);
}

//--------------------------------------------------------------------------------------------------
// Generators
//--------------------------------------------------------------------------------------------------

pub fn generate_address(content: &[u8]) -> Address {
    HashString::encode_from_bytes(content, Hash::SHA2256)
}

//
pub fn generate_agent_id(i: u32) -> String {
    format!("agent_{}", i)
}

//
pub fn generate_dna_id(i: u32) -> Address {
    HashString::from(format!("DNA_{}", i))
}

//
pub fn generate_entry(i: u32) -> (Address, Vec<u8>) {
    let address = format!("entry_addr_{}", i);
    let content = format!("hello-{}", i);
    (address.into(), content.as_bytes().to_vec())
}
