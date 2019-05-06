use serde_json::json;

use holochain_core_types::{cas::content::Address, hash::HashString};

// CONSTS
// TODO: make an array of agent_ids instead?
pub static ALEX_AGENT_ID: &'static str = "alex";
pub static BILLY_AGENT_ID: &'static str = "billy";
pub static CAMILLE_AGENT_ID: &'static str = "camille";

pub static META_CRUD_ATTRIBUTE: &'static str = "crud";
pub static META_LINK_ATTRIBUTE: &'static str = "link__yay";

lazy_static! {
    pub static ref DNA_ADDRESS_A: Address = HashString::from("DNA_A");
    pub static ref DNA_ADDRESS_B: Address = HashString::from("DNA_B");
    pub static ref DNA_ADDRESS_C: Address = HashString::from("DNA_C");
    pub static ref ENTRY_ADDRESS_1: Address = HashString::from("entry_addr_1");
    pub static ref ENTRY_ADDRESS_2: Address = HashString::from("entry_addr_2");
    pub static ref ENTRY_ADDRESS_3: Address = HashString::from("entry_addr_3");
    pub static ref ENTRY_CONTENT_1: serde_json::Value = json!({"ry":"hello"});
    pub static ref ENTRY_CONTENT_2: serde_json::Value = json!({"ry":"hello-2"});
    pub static ref ENTRY_CONTENT_3: serde_json::Value = json!({"ry":"hello-3"});
    // TODO: Meta content should be an Address instead
    pub static ref META_CRUD_CONTENT: serde_json::Value = json!("LIVE");
    pub static ref META_LINK_CONTENT_1: serde_json::Value = json!({"mt":"hello-meta"});
    pub static ref META_LINK_CONTENT_2: serde_json::Value = json!({"mt":"hello-2-meta"});
    pub static ref META_LINK_CONTENT_3: serde_json::Value = json!({"mt":"hello-3-meta"});
}

//--------------------------------------------------------------------------------------------------
// Generators
//--------------------------------------------------------------------------------------------------

//
pub fn generate_agent_id(i: u32) -> String {
    format!("node_{}", i)
}

//
pub fn generate_dna_id(i: u32) -> Address {
    HashString::from(format!("DNA_{}", i))
}

//
pub fn generate_entry(i: u32) -> (Address, serde_json::Value) {
    let address = format!("entry_addr_{}", i);
    let content = format!("hello-{}", i);
    let entry: serde_json::Value = json!({ "ry": content });
    (address.into(), entry)
}

//
pub fn generate_meta(i: u32) -> serde_json::Value {
    let content = format!("hello-{}-meta", i);
    let meta: serde_json::Value = json!({ "mt": content });
    meta
}
