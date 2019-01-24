use serde_json::json;

use holochain_core_types::{cas::content::Address, hash::HashString};

// CONSTS
pub static ALEX_AGENT_ID: &'static str = "alex";
pub static BILLY_AGENT_ID: &'static str = "billy";

pub static META_ATTRIBUTE: &'static str = "link__yay";

pub static FETCH_ENTRY_1_ID: &'static str = "fetch_entry_1_id";
pub static FETCH_ENTRY_2_ID: &'static str = "fetch_entry_2_id";
pub static FETCH_ENTRY_3_ID: &'static str = "fetch_entry_3_id";

pub static FETCH_META_1_ID: &'static str = "fetch_meta_1_id";

lazy_static! {
    pub static ref ENTRY_ADDRESS_1: Address = HashString::from("entry_addr_1");
    pub static ref ENTRY_ADDRESS_2: Address = HashString::from("entry_addr_2");
    pub static ref ENTRY_ADDRESS_3: Address = HashString::from("entry_addr_3");
    pub static ref ENTRY_CONTENT_1: serde_json::Value = json!("hello");
    pub static ref ENTRY_CONTENT_2: serde_json::Value = json!("hello-2");
    pub static ref ENTRY_CONTENT_3: serde_json::Value = json!("hello-3");
    pub static ref META_CONTENT_1: serde_json::Value = json!("hello-meta");
    pub static ref META_CONTENT_2: serde_json::Value = json!("hello-2-meta");
    pub static ref META_CONTENT_3: serde_json::Value = json!("hello-3-meta");
    pub static ref DNA_ADDRESS: Address = HashString::from("DUMMY_DNA_ADDRESS");
}

// static PUBLISH_LIST_ID: &'static str = "publish_list_id_1";
