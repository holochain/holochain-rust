#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate boolinator;
use hdk::holochain_core_types::json::JsonString;


fn handle_sum(num1: u32, num2: u32) -> JsonString {
    let sum = num1 + num2;
    return json!({"sum": format!("{}",sum)}).into();
}

define_zome! {
    entries: []

    genesis: || {
        Ok(())
    }

    functions: {
        main (Public) {
            sum: {
                inputs: |num1: u32, num2: u32|,
                outputs: |sum: JsonString|,
                handler: handle_sum
            }
        }
    }
}
