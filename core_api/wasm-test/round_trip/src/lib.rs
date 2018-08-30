extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;

use holochain_wasm_utils::*;


//--------------------------------------------------------------------------------------------------
// Test function
//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct InputStruct {
    input_int_val: u8,
    input_str_val: String,
}

#[derive(Serialize, Default)]
struct OutputStruct {
    input_int_val_plus2: u8,
    input_str_val_plus_dog: String,
}


/// Create output out of some modification of input
fn test_inner(input: InputStruct) -> OutputStruct {
    OutputStruct {
        input_int_val_plus2: input.input_int_val + 2,
        input_str_val_plus_dog: format!("{}.puppy", input.input_str_val),
    }
}


//--------------------------------------------------------------------------------------------------
//  Exported functions with required signature (=pointer to serialized complex parameter)
//--------------------------------------------------------------------------------------------------

/// Function called by Holochain Instance
/// encoded_allocation_of_input : encoded memory offset and length of the memory allocation
/// holding input arguments
/// returns encoded allocation used to store output
#[no_mangle]
pub extern "C" fn test(encoded_allocation_of_input: usize) -> i32 {
    let mut mem_stack = SinglePageStack::new_from_encoded(encoded_allocation_of_input as u32);
    let input = deserialize_allocation(encoded_allocation_of_input as u32);
    let output = test_inner(input);
    return serialize_into_encoded_allocation(&mut mem_stack, output);
}
