#![feature(try_from)]
#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate boolinator;
use hdk::lib3h_persistence_api::{error::PersistenceError, json::JsonString};

fn handle_sum(num1: u32, num2: u32) -> u32 {
    num1 + num2
}

define_zome! {
    entries: []

    genesis: || {
        Ok(())
    }

    functions: [
        sum: {
            inputs: |num1: u32, num2: u32|,
            outputs: |sum: u32|,
            handler: handle_sum
        }
    ]

    traits: {
        hc_public [sum]
    }

}

#[cfg(test)]
mod tests {

    use handle_sum;

    #[test]
    pub fn handle_sum_test() {
        assert_eq!(
            handle_sum(1, 1),
            2,
        );
    }

}
