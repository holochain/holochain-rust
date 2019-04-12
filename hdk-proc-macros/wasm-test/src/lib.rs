#![feature(try_from)]
#![feature(proc_macro_hygiene)]
extern crate hdk_proc_macros;
use hdk_proc_macros::zome;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[zome]
pub mod someZome {
	
	#[genesis]
	fn genisis() {
		Ok(())
	}

	#[zome_fn]
	fn test_zome_fn(input: i32) -> JsonString {
		JsonString::from_json("hi")
	}

}