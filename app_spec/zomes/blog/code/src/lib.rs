use hdk::prelude::*;
use hdk::{
    holochain_core_types::{signature::Provenance},
    holochain_wasm_types::{
        get_entry::{EntryHistory, GetEntryResult},
        get_links::GetLinksResult,
    },
};

mod blog;
mod memo;
mod post;

use blog::Env;

define_zome! {

    entries: [
        post::definition(),
        memo::definition()
    ]

    init: || {{
        Ok(())
    }}

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {
        Ok(())
    }

    receive: |from, msg_json| {
        blog::handle_receive(from, JsonString::from_json(&msg_json))
    }

    functions: [

        show_env: {
            inputs: | |,
            outputs: |env: ZomeApiResult<Env>|,
            handler: blog::handle_show_env
        }

        get_test_properties: {
            inputs: | |,
            outputs: |property: ZomeApiResult<JsonString>|,
            handler: blog::handle_get_test_properties
        }

        get_sources: {
            inputs: |address: Address|,
            outputs: |sources: ZomeApiResult<Vec<Address>>|,
            handler: blog::handle_get_sources
        }

        check_sum: {
            inputs: |num1: u32, num2: u32|,
            outputs: |sum: ZomeApiResult<u32>|,
            handler: blog::handle_check_sum
        }

        ping: {
            inputs: |to_agent: Address, message: String|,
            outputs: |response: ZomeApiResult<JsonString>|,
            handler: blog::handle_ping
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

        create_tagged_post: {
            inputs: |content: String, tag: String|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_create_tagged_post
        }

        create_post_with_agent: {
            inputs: |agent_id:Address, content: String, in_reply_to: Option<Address>|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_create_post_with_agent
        }

        create_post_countersigned: {
            inputs: |content: String, in_reply_to: Option<Address>, counter_signature:Provenance|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_create_post_countersigned
        }

        request_post_grant: {
            inputs: | |,
            outputs: |result: ZomeApiResult<Option<Address>>|,
            handler: blog::handle_request_post_grant
        }

        get_grants: {
            inputs: | |,
            outputs: |result: ZomeApiResult<Vec<Address>>|,
            handler: blog::handle_get_grants
        }

        commit_post_claim: {
            inputs: |grantor: Address, claim: Address|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_commit_post_claim
        }

        create_post_with_claim: {
            inputs: |grantor: Address, content: String, in_reply_to: Option<Address>|,
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_create_post_with_claim
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
            outputs: |result: ZomeApiResult<Address>|,
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
            inputs: |tag: Option<String>|,
            outputs: |post_hashes: ZomeApiResult<GetLinksResult>|,
            handler: blog::handle_my_posts
        }

        my_posts_with_load: {
            inputs: |tag: Option<String>|,
            outputs: |post_hashes: ZomeApiResult<Vec<post::Post>>|,
            handler: blog::handle_my_posts_with_load
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

        get_post_bridged: {
            inputs: |post_address: Address|,
            outputs: |post: ZomeApiResult<Option<Entry>>|,
            handler: blog::handle_get_post_bridged
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
            outputs: |result: ZomeApiResult<Address>|,
            handler: blog::handle_recommend_post
        }

        my_recommended_posts: {
            inputs: | |,
            outputs: |result: ZomeApiResult<GetLinksResult>|,
            handler: blog::handle_my_recommended_posts
        }

        get_chain_header_hashes: {
            inputs: | |,
            outputs: |result: ZomeApiResult<Vec<Address>>|,
            handler: blog::handle_get_chain_header_hashes
        }
    ]

    traits: {
        hc_public [show_env, get_test_properties, check_sum, ping, get_sources, post_address, create_post, create_tagged_post, create_post_countersigned, delete_post, delete_entry_post, update_post, posts_by_agent, get_post, my_posts, memo_address, get_memo, my_memos, create_memo, my_posts_as_committed, my_posts_immediate_timeout, recommend_post, my_recommended_posts,get_initial_post, get_history_post, get_post_with_options, get_post_with_options_latest, authored_posts_with_sources, create_post_with_agent, request_post_grant, get_grants, commit_post_claim, create_post_with_claim, get_post_bridged,my_posts_with_load, get_chain_header_hashes]
    }
}
