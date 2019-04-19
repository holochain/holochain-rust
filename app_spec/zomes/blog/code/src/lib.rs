#![feature(try_from)]
#![warn(unused_extern_crates)]
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
pub mod memo;
pub mod post;

use blog::Env;
use hdk::{
    error::ZomeApiResult,
    holochain_core_types::{
        cas::content::Address, entry::Entry, error::HolochainError, json::JsonString,
    },
    holochain_wasm_utils::api_serialization::{
        get_entry::{EntryHistory, GetEntryResult},
        get_links::GetLinksResult,
    },
};

define_zome! {

    entries: [
        post::definition(),
        memo::definition()
    ]

    genesis: || {
        Ok(())
    }

    receive: |message| {
        json!({
            "message": message
        }).to_string()
    }

    functions: [

        show_env: {
            inputs: | |,
            outputs: |env: ZomeApiResult<Env>|,
            handler: blog::handle_show_env
        }

        get_sources: {
            inputs: |address: Address|,
            outputs: |sources: ZomeApiResult<Vec<Address>>|,
            handler: blog::handle_get_sources
        }

        check_sum: {
            inputs: |num1: u32, num2: u32|,
            outputs: |sum: ZomeApiResult<JsonString>|,
            handler: blog::handle_check_sum
        }

        check_send: {
            inputs: |to_agent: Address, message: String|,
            outputs: |response: ZomeApiResult<JsonString>|,
            handler: blog::handle_check_send
        }

        post_address: {
            inputs: |content: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_post_address
        }

        memo_address: {
            inputs: |content: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_memo_address
        }

        create_post: {
            inputs: |content: String, in_reply_to: Option<Address>|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_create_post
        }

        create_post_with_agent: {
            inputs: |agent_id:Address,content: String, in_reply_to: Option<Address>|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_create_post_with_agent
        }
        
        create_memo: {
            inputs: |content: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_create_memo
        }

        delete_post: {
            inputs: |content: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_delete_post
        }

        delete_entry_post: {
            inputs: |post_address: Address|,
            outputs: |result: ZomeApiResult<()>|,
            handler: blog::handle_delete_entry_post
        }

        update_post: {
            inputs: |post_address: Address, new_content: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_update_post
        }

        posts_by_agent: {
            inputs: |agent: Address|,
            outputs: |post_hashes: ZomeApiResult<GetLinksResult>|,
            handler: blog::handle_posts_by_agent
        }
        
        authored_posts_with_sources : {
            inputs : |agent : Address|,
            outputs : | post_hashes : ZomeApiResult<GetLinksResult>|,
            handler : blog::handle_my_posts_get_my_sources
        }

        get_post: {
            inputs: |post_address: Address|,
            outputs: |post: ZomeApiResult<Option<Entry>>|,
            handler: blog::handle_get_post
        }

        get_memo: {
            inputs: |memo_address: Address|,
            outputs: |post: ZomeApiResult<Option<Entry>>|,
            handler: blog::handle_get_memo
        }

        get_initial_post: {
            inputs: |post_address: Address|,
            outputs: |post: ZomeApiResult<Option<Entry>>|,
            handler : blog::handle_get_initial_post
        }

        get_history_post : {
            inputs: |post_address: Address|,
            outputs: |post: ZomeApiResult<EntryHistory>|,
            handler : blog::handle_get_history_post
        }

        my_posts: {
            inputs: | |,
            outputs: |post_hashes: ZomeApiResult<GetLinksResult>|,
            handler: blog::handle_my_posts
        }

        my_memos: {
            inputs: | |,
            outputs: |memo_hashes: ZomeApiResult<Vec<Address>>|,
            handler: blog::handle_my_memos
        }

        get_post_with_options_latest :{
            inputs: |post_address: Address|,
            outputs: |post: ZomeApiResult<Entry>|,
            handler:  blog::handle_get_post_with_options_latest
        }

        get_post_with_options :{
            inputs: |post_address: Address|,
            outputs: |post: ZomeApiResult<GetEntryResult>|,
            handler:  blog::handle_my_post_with_options
        }

        my_posts_immediate_timeout: {
            inputs: | |,
            outputs: |post_hashes: ZomeApiResult<GetLinksResult>|,
            handler: blog::handle_my_posts_immediate_timeout
        }

        my_posts_as_committed: {
            inputs: | |,
            outputs: |post_hashes: ZomeApiResult<Vec<Address>>|,
            handler: blog::handle_my_posts_as_commited
        }


        recommend_post: {
            inputs: |post_address: Address, agent_address: Address|,
            outputs: |result: ZomeApiResult<()>|,
            handler: blog::handle_recommend_post
        }

        my_recommended_posts: {
            inputs: | |,
            outputs: |result: ZomeApiResult<GetLinksResult>|,
            handler: blog::handle_my_recommended_posts
        }
    ]

    traits: {
        hc_public [show_env, check_sum, check_send, get_sources, post_address, create_post, delete_post, delete_entry_post, update_post, posts_by_agent, get_post, my_posts,memo_address,get_memo,my_memos,create_memo,my_posts_as_committed, my_posts_immediate_timeout, recommend_post, my_recommended_posts,get_initial_post,get_history_post,get_post_with_options,get_post_with_options_latest,authored_posts_with_sources,create_post_with_agent]
    }
}
