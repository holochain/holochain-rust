#![feature(proc_macro_hygiene)]
extern crate hdk_proc_macros;
use hdk_proc_macros::zome;

#[zome]
pub mod someZome {
	
	#[genesis]
	fn genisis() {
		Ok(())
	}

	#[zome_fn]
	fn test_zome_fn(input: String) -> String {

	}

}