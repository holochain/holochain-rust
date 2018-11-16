#[macro_use]
extern crate hdk;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

// see https://holochain.github.io/rust-api/0.0.1/hdk/ for info on using the hdk library

define_zome! {
    entries: []

    genesis: || { Ok(()) }

    functions: {}
}
