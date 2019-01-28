#![feature(try_from)]
#[macro_use]
extern crate holochain_wasm_utils;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_core_types_derive;
extern crate wasmi;

use holochain_wasm_utils::holochain_core_types::json::JsonString;
use holochain_wasm_utils::holochain_core_types::json::RawString;
use holochain_wasm_utils::memory::stack::WasmStack;
use holochain_wasm_utils::memory::allocation::AllocationError;

use holochain_wasm_utils::{
    holochain_core_types::error::HolochainError,
};
use holochain_wasm_utils::holochain_core_types::error::RibosomeEncodingBits;
use holochain_wasm_utils::holochain_core_types::error::RibosomeEncodedValue;
use std::convert::TryFrom;
use holochain_wasm_utils::memory::ribosome::return_code_for_allocation_result;
use holochain_wasm_utils::memory::MemoryInt;
use holochain_wasm_utils::memory::ribosome::load_ribosome_encoded_json;
use holochain_wasm_utils::memory::allocation::WasmAllocation;
use holochain_wasm_utils::holochain_core_types::bits_n_pieces::U16_MAX;
use wasmi::MemoryInstance;
use wasmi::memory_units::Pages;

#[derive(Serialize, Default, Clone, PartialEq, Deserialize, Debug, DefaultJson)]
struct TestStruct {
    value: String,
    list: Vec<String>,
}

#[derive(Serialize, Default, Clone, PartialEq, Deserialize, Debug, DefaultJson)]
struct OtherTestStruct {
    other: String,
    list: Vec<String>,
}

// ===============================================================================================
// START MEMORY
// -----------------------------------------------------------------------------------------------

#[no_mangle]
/// store string in wasm memory and return encoding
/// TODO #486
//// Can't do zome_assert!() while testing write_json() since it internally uses write_json() !
//// so using normal assert! even if we get unhelpful Trap::Unreachable error message.
pub extern "C" fn store_string(_: RibosomeEncodingBits) -> RibosomeEncodingBits {

    // start with an empty stack
    let mut stack = WasmStack::default();
    assert_eq!(
        0 as MemoryInt,
        MemoryInt::from(stack.top()),
    );

    // successfully allocate a written string
    let s = "fish";
    let allocation = match stack.write_string(s) {
        Ok(allocation) => allocation,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    assert_eq!(
        4 as MemoryInt,
        MemoryInt::from(stack.top()),
    );

    allocation.as_ribosome_encoding()

}

#[no_mangle]
pub extern "C" fn store_string_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let allmost_full_alloc = 0b11111111111111111111111111111101_00000000000000000000000000000010;

    let allocation = match WasmAllocation::try_from_ribosome_encoding(allmost_full_alloc) {
        Ok(allocation) => allocation,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    let mut stack = match WasmStack::try_from(allocation) {
        Ok(stack) => stack,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    return_code_for_allocation_result(
        stack.write_string("fish")
    ).into()
}

#[no_mangle]
// load the string from a previously stored string
pub extern "C" fn load_string(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let encoded = store_string(0);

    let allocation = match WasmAllocation::try_from_ribosome_encoding(encoded) {
        Ok(allocation) => allocation,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    let mut stack = match WasmStack::try_from(allocation) {
        Ok(allocation) => allocation,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    return_code_for_allocation_result(
        stack.write_string(
            &allocation.read_to_string()
        )
    )
    .into()

}

#[no_mangle]
pub extern "C" fn stacked_strings(_: RibosomeEncodingBits) -> RibosomeEncodingBits {

    let mut stack = WasmStack::default();

    let first = match stack.write_string("first") {
        Ok(first) => first,
        Err(first_error) => return first_error.as_ribosome_encoding(),
    };
    if let Err(second_error) = stack.write_string("second") {
        return second_error.as_ribosome_encoding();
    };

    first.as_ribosome_encoding()
}

#[no_mangle]
pub extern "C" fn big_string_output_static(_: RibosomeEncodingBits) -> RibosomeEncodingBits {

    let mut stack = WasmStack::default();

    let memory = match MemoryInstance::alloc(Pages(1), None) {
        Ok(memory) => memory,
        Err(_) => return AllocationError::ZeroLength.as_ribosome_encoding(),
    };

    // table flip emoji is 27 bytes so we need 27 pages to hold U16_MAX table flips
    if let Err(_) = memory.grow(Pages(11)) {
        return AllocationError::BadStackAlignment.as_ribosome_encoding();
    };

    // match stack.write_string(&"fooo".repeat(U16_MAX as usize)) {
    match stack.write_string(&"(ಥ⌣ಥ)".repeat(U16_MAX as usize)) {
        Ok(allocation) => allocation.as_ribosome_encoding(),
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    }

}

#[no_mangle]
/// thrash the allocation/deallocation logic a bit
pub extern "C" fn round_trip_foo(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();

    // stack should start at zero top
    assert_eq!(
        0 as MemoryInt,
        MemoryInt::from(stack.top()),
    );

    // should be able to retrieve a valid allocation from writing "foo" to the stack
    let s = "foo";
    let allocation = stack.write_string(s).unwrap();

    // the allocation should be offset 0 and length 3, i.e. starting at "foo"
    assert_eq!(
        0 as MemoryInt,
        MemoryInt::from(allocation.offset()),
    );
    assert_eq!(
        3 as MemoryInt,
        MemoryInt::from(allocation.length()),
    );
    assert_eq!(
        "foo".to_string(),
        allocation.read_to_string(),
    );

    // top of the stack should be end of "foo"
    assert_eq!(
        3 as MemoryInt,
        MemoryInt::from(stack.top()),
    );

    // should be able to retrieve a new valid allocation from writing "bar" to the stack
    let s2 = "barz";
    let allocation2 = stack.write_string(s2).unwrap();

    // allocation 1 should be unchanged by this
    assert_eq!(
        0 as MemoryInt,
        MemoryInt::from(allocation.offset()),
    );
    assert_eq!(
        3 as MemoryInt,
        MemoryInt::from(allocation.length()),
    );

    // allocation 2 starts at the end of "foo" for "barz" so is offset 3 and length 4
    assert_eq!(
        3 as MemoryInt,
        MemoryInt::from(allocation2.offset()),
    );
    assert_eq!(
        4 as MemoryInt,
        MemoryInt::from(allocation2.length()),
    );
    assert_eq!(
        "barz".to_string(),
        allocation2.read_to_string(),
    );

    // stack top should now be "foo" + "barz" = 7
    assert_eq!(
        7 as MemoryInt,
        MemoryInt::from(stack.top()),
    );

    // should NOT be able to deallocate "foo"
    assert!(stack.deallocate(allocation).is_err());

    // should be able to deallocate "barz"
    stack.deallocate(allocation2).unwrap();
    assert_eq!(
        3 as MemoryInt,
        MemoryInt::from(stack.top()),
    );

    // should be able to allocate/deallocate something else
    let allocation3 = stack.write_string("a").unwrap();
    assert_eq!(
        4 as MemoryInt,
        MemoryInt::from(stack.top()),
    );
    stack.deallocate(allocation3).unwrap();
    assert_eq!(
        3 as MemoryInt,
        MemoryInt::from(stack.top()),
    );

    // returning allocation as ribosome encoding should make "foo" visible outside wasm
    allocation.as_ribosome_encoding()
}

#[no_mangle]
pub extern "C" fn error_report(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    zome_assert!(stack, false);
    RibosomeEncodedValue::Success.into()
}

#[no_mangle]
pub extern "C" fn store_as_json(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    assert_eq!(0, MemoryInt::from(stack.top()));

    let s = "fish";
    match stack.write_json(RawString::from(s)) {
        Ok(allocation) => {
            assert_eq!(
                // "\"fish\""
                6 as MemoryInt,
                MemoryInt::from(stack.top()),
            );
            allocation.as_ribosome_encoding()
        },
        Err(allocation_error) => allocation_error.as_ribosome_encoding(),
    }
}

#[no_mangle]
pub extern "C" fn store_struct_as_json(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();
    assert_eq!(0, MemoryInt::from(stack.top()));

    let obj = TestStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };
    match stack.write_json(obj.clone()) {
        Ok(allocation) => {
            assert_eq!(
                JsonString::from(obj).to_string().len() as MemoryInt,
                MemoryInt::from(stack.top()),
            );
            allocation.as_ribosome_encoding()
        },
        Err(allocation_error) => allocation_error.as_ribosome_encoding(),
    }
}

#[no_mangle]
pub extern "C" fn load_json_struct(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let encoded = store_struct_as_json(0);

    let allocation = match WasmAllocation::try_from_ribosome_encoding(encoded) {
        Ok(allocation) => allocation,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    let mut stack = match WasmStack::try_from(allocation) {
        Ok(stack) => stack,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    let maybe_test_struct: Result<TestStruct, HolochainError> = load_ribosome_encoded_json(encoded);
    match maybe_test_struct {
        Ok(test_struct) => {
            return_code_for_allocation_result(
                stack.write_json(test_struct)
            )
            .into()
        },
        Err(holochain_err) => RibosomeEncodedValue::from(holochain_err).into(),
    }
}

#[no_mangle]
pub extern "C" fn stacked_json(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();

    let first = match stack.write_json(RawString::from("first")) {
        Ok(first) => first,
        Err(first_error) => return first_error.as_ribosome_encoding(),
    };
    if let Err(second_error) = stack.write_json(RawString::from("second")) {
        return second_error.as_ribosome_encoding();
    }

    first.as_ribosome_encoding()
}

#[no_mangle]
pub extern "C" fn stacked_json_struct(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();

    let first = match stack.write_json(TestStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    }) {
        Ok(first) => first,
        Err(first_error) => return first_error.as_ribosome_encoding(),
    };

    if let Err(second_error) = stack.write_json(TestStruct {
        value: "second".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    }) {
        return second_error.as_ribosome_encoding();
    }

    first.as_ribosome_encoding()
}

#[no_mangle]
pub extern "C" fn store_json_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let allmost_full_alloc = 0b11111111111111111111111111111101_00000000000000000000000000000010;

    let allocation = match WasmAllocation::try_from_ribosome_encoding(allmost_full_alloc) {
        Ok(allocation) => allocation,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    let mut stack = match WasmStack::try_from(allocation) {
        Ok(stack) => stack,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    let obj = TestStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    };

    match stack.write_json(obj.clone()) {
        Ok(allocation) => allocation.as_ribosome_encoding(),
        Err(allocation_error) => allocation_error.as_ribosome_encoding(),
    }
}

#[no_mangle]
pub extern "C" fn load_json_err(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();

    let encoded = 1 << 32;
    let maybe_test_struct: Result<TestStruct, HolochainError> = load_ribosome_encoded_json(encoded);
    let test_struct = match maybe_test_struct {
        Ok(test_struct) => test_struct,
        Err(holochain_error) => return RibosomeEncodedValue::from(holochain_error).into(),
    };

    match stack.write_json(test_struct) {
        Ok(allocation) => allocation.as_ribosome_encoding(),
        Err(allocation_error) => allocation_error.as_ribosome_encoding(),
    }
}

#[no_mangle]
pub extern "C" fn stacked_mix(_: RibosomeEncodingBits) -> RibosomeEncodingBits {
    let mut stack = WasmStack::default();

    if let Err(first_error) = stack.write_json(TestStruct {
        value: "first".to_string(),
        list: vec!["hello".to_string(), "world!".to_string()],
    }) {
        return first_error.as_ribosome_encoding();
    }

    if let Err(second_error) = stack.write_json(RawString::from("second")) {
        return second_error.as_ribosome_encoding();
    }

    let third = match stack.write_json(RawString::from("third")) {
        Ok(third) => third,
        Err(third_error) => return third_error.as_ribosome_encoding(),
    };

    if let Err(fourth_error) = stack.write_json(RawString::from("fourth")) {
        return fourth_error.as_ribosome_encoding();
    }

    if let Err(fifth_error) = stack.write_json(TestStruct {
        value: "fifth".to_string(),
        list: vec!["fifthlist".to_string()],
    }) {
        return fifth_error.as_ribosome_encoding();
    }

    third.as_ribosome_encoding()
}

// ===============================================================================================
// END MEMORY
// -----------------------------------------------------------------------------------------------
