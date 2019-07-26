#![feature(proc_macro_hygiene)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate hdk;
extern crate hdk_proc_macros;

use hdk::error::ZomeApiResult;
use hdk_proc_macros::zome;

#[zome]
pub mod summer {
    #[init]
    fn init() {
        Ok(())
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData<AgentId>) {
        Ok(())
    }

    #[zome_fn("hc_public")]
    fn sum(num1: u32, num2: u32) -> ZomeApiResult<u32> {
        Ok(num1 + num2)
    }
}
