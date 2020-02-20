use crate::{
    context::Context, wasm_engine::api::ZomeApiResult,
    workflows::get_links_count::get_link_result_count_workflow, NEW_RELIC_LICENSE_KEY,
};
use holochain_wasm_utils::api_serialization::get_links::GetLinksArgs;
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn invoke_get_links_count(context: Arc<Context>, input: GetLinksArgs) -> ZomeApiResult {
    context.block_on(get_link_result_count_workflow(context.clone(), &input));
}
