#[macro_use]
extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};

#[derive(Serialize, Default, Clone)]
struct InputTestStruct {
    value: String,
}


#[derive(Deserialize, Serialize, Default)]
struct OutputTestStruct {
    value: String,
}


// Can't do dna_assert!() while testing serialize() since it uses serialize() !!!
// so using normal assert! even if we get unhelpful Trap::Unreachable error message.
#[no_mangle]
pub extern "C" fn test_serialize(encoded_allocation: u32) -> u32 {
    let mut stack = SinglePageStack::default();
    let obj = InputTestStruct { value: "fish".to_string() };
    assert_eq!(0, stack.top());
    let res = {serialize(&mut stack, obj.clone())};
    assert_eq!(json!(obj).to_string().len(), stack.top() as usize);
    res.unwrap().encode()
}
