#[macro_use]
extern crate hdk;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

// see https://holochain.github.io/rust-api/0.0.1/hdk/ for info on using the hdk library

define_zome! {
    entries: []

    genesis: || { Ok(()) }

    functions: {}
}
