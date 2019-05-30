#![feature(try_from)]
#![feature(proc_macro_hygiene)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate hdk_proc_macros;
use hdk_proc_macros::zome;

#[zome]
pub mod summer {
    #[genesis]
    fn genesis() {
        Ok(())
    }

    #[zome_fn("hc_public")]
    fn sum(num1: u32, num2: u32) -> u32 {
        num1 + num2
    }
}
