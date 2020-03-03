use crate::{
    wasm_engine::{api::ZomeApiResult, Runtime},
    workflows::get_links_count::get_link_result_count_workflow,
    NEW_RELIC_LICENSE_KEY,
};
use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
use std::convert::TryFrom;
use wasmi::{RuntimeArgs, RuntimeValue};

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_get_links_count(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    let context = runtime.context()?;
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);

    let input = match GetLinksArgs::try_from(args_str.clone()) {
        Ok(input) => {
            log_debug!(
                context,
                "get_links: invoke_get_links called with {:?}",
                input,
            );
            input
        }
        Err(_) => {
            log_error!(
                context,
                "zome: invoke_get_links failed to deserialize GetLinksArgs: {:?}",
                args_str
            );
            return ribosome_error_code!(ArgumentDeserializationFailed);
        }
    };

    let result = context.block_on(get_link_result_count_workflow(context.clone(), &input));

    runtime.store_result(result)
}
