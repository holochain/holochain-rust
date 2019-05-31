use holochain_core_types::{cas::content::Address, hash::HashString};

// CONSTS
pub static META_CRUD_ATTRIBUTE: &'static str = "crud";
pub static META_LINK_ATTRIBUTE: &'static str = "link__yay";

lazy_static! {
    // TODO: make an array of agent_ids instead?
    pub static ref ALEX_AGENT_ID: Address = HashString::from("alex");
    pub static ref BILLY_AGENT_ID: Address = HashString::from("billy");
    pub static ref CAMILLE_AGENT_ID: Address = HashString::from("camille");

    pub static ref DNA_ADDRESS_A: Address = HashString::from("DNA_A");
    pub static ref DNA_ADDRESS_B: Address = HashString::from("DNA_B");
    pub static ref DNA_ADDRESS_C: Address = HashString::from("DNA_C");
    pub static ref ENTRY_ADDRESS_1: Address = HashString::from("entry_addr_1");
    pub static ref ENTRY_ADDRESS_2: Address = HashString::from("entry_addr_2");
    pub static ref ENTRY_ADDRESS_3: Address = HashString::from("entry_addr_3");
    pub static ref ENTRY_CONTENT_1: Vec<u8> = "hello-1".as_bytes().to_vec();
    pub static ref ENTRY_CONTENT_2: Vec<u8> = "hello-2".as_bytes().to_vec();
    pub static ref ENTRY_CONTENT_3: Vec<u8> = "hello-3".as_bytes().to_vec();
}

//--------------------------------------------------------------------------------------------------
// Generators
//--------------------------------------------------------------------------------------------------

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
