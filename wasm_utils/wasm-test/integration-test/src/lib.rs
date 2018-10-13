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
    dna_assert!(stack, 1 == stack.top());
    let res = {serialize(&mut stack, obj.clone())};
    assert_eq!(json!(obj).to_string().len(), stack.top() as usize);
    res.unwrap().encode()
}


//#[cfg(test)]
//pub mod tests {
//
//    use super::*;
//    use holochain_wasm_utils::{memory_allocation::*, memory_serialization::*};
//
//    #[test]
//    pub extern "C" fn can_serialize() {
//        println!("COUCOU");
//        let mut stack = SinglePageStack::default();
//        println!("stack: {:?}", stack);
//        let obj = InputTestStruct { value: "fish".to_string() };
//        assert_eq!(0, stack.top());
//        let res = serialize(&mut stack, obj);
//        println!("stack: {:?}", stack);
//        println!("res: {:?}", res);
//    }
//}