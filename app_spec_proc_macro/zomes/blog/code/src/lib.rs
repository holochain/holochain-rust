#![feature(proc_macro_hygiene)]
use hdk::prelude::*;
use hdk::{
    holochain_core_types::{signature::Provenance},
    holochain_wasm_utils::api_serialization::{
        get_entry::{EntryHistory, GetEntryResult},
        get_links::GetLinksResult,
    },
};
use hdk_proc_macros::zome;

mod blog;
mod memo;
mod post;

use blog::Env;

#[zome]
pub mod blog {

    #[entry_def]
    pub fn post_entry_def() -> ValidatingEntryType {
        post::definition()
    }

    #[entry_def]
    pub fn memo_entry_def() -> ValidatingEntryType {
        memo::definition()
    }

    #[init]
    pub fn init() {
        Ok(())
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData<AgentId>) {
        Ok(())
    }

    #[receive]
    pub fn receive(from: Address, msg_json: String) {
        blog::handle_receive(from, JsonString::from_json(&msg_json))
    }

    #[zome_fn("hc_public")]
    pub fn show_env() -> ZomeApiResult<Env> {
        blog::handle_show_env()
    }

    #[zome_fn("hc_public")]
    pub fn get_sources(address: Address) -> ZomeApiResult<Vec<Address>> {
        blog::handle_get_sources(address)
    }

    #[zome_fn("hc_public")]
    pub fn check_sum(num1: u32, num2: u32) -> ZomeApiResult<u32> {
        blog::handle_check_sum(num1, num2)
    }

    #[zome_fn("hc_public")]
    pub fn ping(to_agent: Address, message: String) -> ZomeApiResult<JsonString> {
        blog::handle_ping(to_agent, message)
    }

    #[zome_fn("hc_public")]
    pub fn post_address(content: String) -> ZomeApiResult<Address> {
        blog::handle_post_address(content)
    }

    #[zome_fn("hc_public")]
    pub fn create_post(content: String, in_reply_to: Option<Address>) -> ZomeApiResult<Address> {
        blog::handle_create_post(content, in_reply_to)
    }

    #[zome_fn("hc_public")]
    pub fn create_tagged_post(content: String, tag: String) -> ZomeApiResult<Address> {
        blog::handle_create_tagged_post(content, tag)
    }

    #[zome_fn("hc_public")]
    pub fn create_post_with_agent(agent_id: Address,content: String, in_reply_to: Option<Address>) -> ZomeApiResult<Address> {
        blog::handle_create_post_with_agent(agent_id, content, in_reply_to)
    }

    #[zome_fn("hc_public")]
    pub fn create_memo(content: String) -> ZomeApiResult<Address> {
        blog::handle_create_memo(content)
    }

    #[zome_fn("hc_public")]
    pub fn create_post_countersigned(content: String, in_reply_to: Option<Address>, counter_signature: Provenance) -> ZomeApiResult<Address> {
        blog::handle_create_post_countersigned(content, in_reply_to, counter_signature)
    }

    #[zome_fn("hc_public")]
    pub fn commit_post_claim(grantor: Address, claim: Address) -> ZomeApiResult<Address> {
        blog::handle_commit_post_claim(grantor, claim)
    }

    #[zome_fn("hc_public")]
    pub fn create_post_with_claim(grantor: Address, content: String, in_reply_to: Option<Address>) -> ZomeApiResult<Address> {
        blog::handle_create_post_with_claim(grantor, content, in_reply_to)
    }

    #[zome_fn("hc_public")]
    pub fn delete_post(content: String) -> ZomeApiResult<Address> {
        blog::handle_delete_post(content)
    }

    #[zome_fn("hc_public")]
    pub fn delete_entry_post(post_address: Address) -> ZomeApiResult<Address> {
        blog::handle_delete_entry_post(post_address)
    }

    #[zome_fn("hc_public")]
    pub fn update_post(post_address: Address, new_content: String) -> ZomeApiResult<Address> {
        blog::handle_update_post(post_address, new_content)
    }

    #[zome_fn("hc_public")]
    pub fn posts_by_agent(agent: Address) -> ZomeApiResult<GetLinksResult> {
        blog::handle_posts_by_agent(agent)
    }

    #[zome_fn("hc_public")]
    pub fn get_post(post_address: Address) -> ZomeApiResult<Option<Entry>> {
        blog::handle_get_post(post_address)
    }

    #[zome_fn("hc_public")]
    pub fn get_memo(memo_address: Address) -> ZomeApiResult<Option<Entry>> {
        blog::handle_get_memo(memo_address)
    }

    #[zome_fn("hc_public")]
    pub fn get_initial_post(post_address: Address) -> ZomeApiResult<Option<Entry>> {
        blog::handle_get_initial_post(post_address)
    }

    #[zome_fn("hc_public")]
    pub fn get_history_post(post_address: Address) -> ZomeApiResult<EntryHistory> {
        blog::handle_get_history_post(post_address)
    }

    #[zome_fn("hc_public")]
    pub fn my_posts(tag: Option<String>) -> ZomeApiResult<GetLinksResult> {
        blog::handle_my_posts(tag)
    }

    #[zome_fn("hc_public")]
    pub fn my_posts_with_load(tag: Option<String>) -> ZomeApiResult<Vec<post::Post>> {
        blog::handle_my_posts_with_load(tag)
    }

    #[zome_fn("hc_public")]
    pub fn my_memos() -> ZomeApiResult<Vec<Address>> {
        blog::handle_my_memos()
    }

    #[zome_fn("hc_public")]
    pub fn request_post_grant() -> ZomeApiResult<Option<Address>> {
        blog::handle_request_post_grant()
    }

    #[zome_fn("hc_public")]
    pub fn get_grants() -> ZomeApiResult<Vec<Address>> {
        blog::handle_get_grants()
    }

    #[zome_fn("hc_public")]
    pub fn memo_address(content: String) -> ZomeApiResult<Address> {
        blog::handle_memo_address(content)
    }

    #[zome_fn("hc_public")]
    pub fn get_post_with_options_latest(post_address: Address) -> ZomeApiResult<Entry> {
        blog::handle_get_post_with_options_latest(post_address)
    }

    #[zome_fn("hc_public")]
    pub fn get_post_with_options(post_address: Address) ->ZomeApiResult<GetEntryResult> {
        blog::handle_my_post_with_options(post_address)
    }

    #[zome_fn("hc_public")]
    pub fn get_post_bridged(post_address: Address) -> ZomeApiResult<Option<Entry>> {
        blog::handle_get_post_bridged(post_address)
    }

    #[zome_fn("hc_public")]
    pub fn my_posts_immediate_timeout() -> ZomeApiResult<GetLinksResult> {
        blog::handle_my_posts_immediate_timeout()
    }

    #[zome_fn("hc_public")]
    pub fn my_posts_as_committed() -> ZomeApiResult<Vec<Address>> {
        blog::handle_my_posts_as_commited()
    }

    #[zome_fn("hc_public")]
    pub fn recommend_post(post_address: Address, agent_address: Address) -> ZomeApiResult<Address> {
        blog::handle_recommend_post(post_address, agent_address)
    }

    #[zome_fn("hc_public")]
    pub fn my_recommended_posts() -> ZomeApiResult<GetLinksResult> {
        blog::handle_my_recommended_posts()
    }

    #[zome_fn("hc_public")]
    pub fn authored_posts_with_sources(agent: Address) -> ZomeApiResult<GetLinksResult> {
        blog::handle_my_posts_get_my_sources(agent)
    }

    #[zome_fn("hc_public")]
    pub fn get_test_properties() -> ZomeApiResult<JsonString> {
        hdk::property("test_property")
    }
}
