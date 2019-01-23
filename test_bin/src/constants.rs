
use holochain_core_types::cas::content::Address;

// CONSTS
pub static ALEX_AGENT_ID: &'static str   = "alex";
pub static BILLY_AGENT_ID: &'static str  = "billy";
pub static ENTRY_ADDRESS_1: &'static str = "entry_addr_1";
pub static ENTRY_ADDRESS_2: &'static str = "entry_addr_2";
pub static ENTRY_ADDRESS_3: &'static str = "entry_addr_3";
pub static DNA_ADDRESS: &'static str     = "DUMMY_DNA_ADDRESS";
pub static META_ATTRIBUTE: &'static str  = "link__yay";

pub static FETCH_ENTRY_1_ID: &'static str = "fetch_entry_1_id";
pub static FETCH_ENTRY_2_ID: &'static str = "fetch_entry_2_id";
pub static FETCH_ENTRY_3_ID: &'static str = "fetch_entry_3_id";

//lazy_static! {
//    pub static ENTRY_1 = json!("hello");
//}

#[cfg_attr(tarpaulin, skip)]
pub fn example_dna_address() -> Address {
    DNA_ADDRESS.into()
}