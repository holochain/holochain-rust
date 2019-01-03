# Define Zome




## Building in Rust: define_zome!

#[macro_use]
extern crate hdk;

define_zome! {
    entries: []

    genesis: || {
        Ok(())
    }

    functions: {}
}
