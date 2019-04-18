#![feature(try_from)]
#![warn(unused_extern_crates)]
#![feature(proc_macro_hygiene)]

extern crate hdk_proc_macros;
use hdk_proc_macros::zome;

#[macro_use]
extern crate hdk;
#[macro_use]
extern crate serde_derive;
extern crate boolinator;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate holochain_core_types_derive;

pub mod blog;
pub mod post;

use hdk::{
    error::ZomeApiResult,
    entry_definition::ValidatingEntryType,
    holochain_core_types::{
        cas::content::Address,
        entry::Entry,
        json::JsonString,
    },
    holochain_wasm_utils::api_serialization::{get_links::GetLinksResult,get_entry::{EntryHistory,GetEntryResult}}
};
use blog::Env;

#[zome]
pub mod blog {

    #[entry_def]
    pub fn post_entry_def() -> ValidatingEntryType {
        post::definition()
    }

    #[genesis]
    pub fn genesis() {
        Ok(())
    }

    #[receive]
    pub fn receive(message: String) {
        json!({
            "message": message
        }).to_string()
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
    pub fn check_sum(num1: u32, num2: u32) -> ZomeApiResult<JsonString> {
        blog::handle_check_sum(num1, num2)
    }

    #[zome_fn("hc_public")]
    pub fn check_send(to_agent: Address, message: String) -> ZomeApiResult<JsonString> {
        blog::handle_check_send(to_agent, message)
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
    pub fn delete_post(content: String) -> ZomeApiResult<Address> {
        blog::handle_delete_post(content)
    }

    #[zome_fn("hc_public")]
    pub fn delete_entry_post(post_address: Address) -> ZomeApiResult<()> {
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
    pub fn get_initial_post(post_address: Address) -> ZomeApiResult<Option<Entry>> {
        blog::handle_get_initial_post(post_address)
    }

    #[zome_fn("hc_public")]
    pub fn get_history_post(post_address: Address) -> ZomeApiResult<EntryHistory> {
        blog::handle_get_history_post(post_address)
    }

    #[zome_fn("hc_public")]
    pub fn my_posts() -> ZomeApiResult<GetLinksResult> {
        blog::handle_my_posts()
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
    pub fn my_posts_immediate_timeout() -> ZomeApiResult<GetLinksResult> {
        blog::handle_my_posts_immediate_timeout()
    }

    #[zome_fn("hc_public")]
    pub fn my_posts_as_committed() -> ZomeApiResult<Vec<Address>> {
        blog::handle_my_posts_as_commited()
    }

    #[zome_fn("hc_public")]
    pub fn recommend_post(post_address: Address, agent_address: Address) -> ZomeApiResult<()> {
        blog::handle_recommend_post(post_address, agent_address)
    }

    #[zome_fn("hc_public")]
    pub fn my_recommended_posts() -> ZomeApiResult<GetLinksResult> {
        blog::handle_my_recommended_posts()
    }

}
