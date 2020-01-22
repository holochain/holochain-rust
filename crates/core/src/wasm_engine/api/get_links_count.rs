use crate::{
    wasm_engine::{api::ZomeApiResult, Runtime},
    workflows::get_links_count::get_link_result_count_workflow,
};
use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;

pub fn invoke_get_links_count(runtime: &mut Runtime, input: GetLinksArgs) -> ZomeApiResult {
    let result = runtime
        .context()?
        .block_on(get_link_result_count_workflow(runtime.context()?, &input));

    runtime.store_result(result)
}
