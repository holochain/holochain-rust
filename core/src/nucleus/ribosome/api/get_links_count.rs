use crate::{
    nucleus::ribosome::{api::ZomeApiResult, Runtime},
    workflows::get_links_count::get_link_result_count_workflow,
};
use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};



pub fn invoke_get_links_count(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let input = match GetLinksArgs::try_from(args_str.clone()) {
        Ok(input) => {
            context.log(format!(
                "log/get_links: invoke_get_links called with {:?}",
                input,
            ));
            input
        }
        Err(_) => {
            context.log(format!(
                "err/zome: invoke_get_links failed to deserialize GetLinksArgs: {:?}",
                args_str
            ));
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    
    let result = context.block_on(get_link_result_count_workflow(context.clone(), &input));

    runtime.store_result(result)
}


