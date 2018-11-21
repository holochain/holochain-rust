use futures::executor::block_on;
use holochain_core_types::cas::content::Address;
use dht::actions::remove_entry::remove_entry;
use nucleus::{
    ribosome::{api::ZomeApiResult, Runtime},
};
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};


/// ZomeApiFunction::RemoveEntry function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected Address argument
/// Returns only a RibosomeReturnCode as I32
pub fn invoke_remove_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {

    println!("\n invoke_REMOVE_entry!!");

    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let try_address = Address::try_from(args_str.clone());
    // Exit on error
    if try_address.is_err() {
        println!(
            "invoke_remove_entry failed to deserialize Address: {:?}",
            args_str
        );
        return ribosome_error_code!(ArgumentDeserializationFailed);
    }
    let address = try_address.unwrap();

    let future = remove_entry(&runtime.context,  &runtime.context.action_channel, address);
    println!("\t invoke_remove_entry: future");
    let result = block_on(future);

    println!("\t invoke_remove_entry: result = {:?}", result);

    match result {
        Err(_) => ribosome_error_code!(Unspecified),
        Ok(_) => ribosome_success!(),
    }
}