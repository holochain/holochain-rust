use hdk::{
    self,
    error::ZomeApiResult,
    holochain_core_types::{
        cas::content::Address, entry::Entry, error::HolochainError, json::JsonString,
    },
    holochain_wasm_utils::api_serialization::{
        get_entry::GetEntryOptions, get_links::GetLinksResult,
    },
    AGENT_ADDRESS,
    AGENT_ID_STR,
    DNA_NAME,
    DNA_HASH,
};
use post::Post;

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct Env {
    dna_name: String,
    dna_hash: String,
    agent_id: String,
    agent_address: String,
}

/// This handler shows how you can access the globals that are always available
/// inside a zome.  In this case it just creates an object with their values
/// and returns it as the result.
pub fn handle_show_env() -> ZomeApiResult<Env> {
    Ok(Env{
        dna_name: DNA_NAME.to_string(),
        dna_hash: DNA_HASH.to_string(),
        agent_id: AGENT_ID_STR.to_string(),
        agent_address: AGENT_ADDRESS.to_string(),
    })
}

pub fn handle_check_sum(num1: u32, num2: u32) -> ZomeApiResult<JsonString> {
    #[derive(Serialize, Deserialize, Debug, DefaultJson)]
    struct SumInput {
        num1: u32,
        num2: u32,
    };

    let call_input = SumInput {
        num1: num1,
        num2: num2,
    };
    hdk::call(hdk::THIS_INSTANCE, "summer", "main", "test_token", "sum", call_input.into())
}

pub fn handle_post_address(content: String) -> ZomeApiResult<Address> {
    let post_entry = Entry::App("post".into(), Post::new(&content, "now").into());
    hdk::entry_address(&post_entry)
}

pub fn handle_create_post(content: String, in_reply_to: Option<Address>) -> ZomeApiResult<Address> {
    let post_entry = Entry::App("post".into(), Post::new(&content, "now").into());

    let address = hdk::commit_entry(&post_entry)?;

    hdk::link_entries(&AGENT_ADDRESS, &address, "authored_posts")?;

    if let Some(in_reply_to_address) = in_reply_to {
        // return with Err if in_reply_to_address points to missing entry
        hdk::get_entry_result(in_reply_to_address.clone(), GetEntryOptions::default())?;
        hdk::link_entries(&in_reply_to_address, &address, "comments")?;
    }

    Ok(address)
}

pub fn handle_posts_by_agent(agent: Address) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&agent, "authored_posts")
}

pub fn handle_my_posts() -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&AGENT_ADDRESS, "authored_posts")
}

pub fn handle_my_posts_as_commited() -> ZomeApiResult<Vec<Address>> {
    // In the current implementation of hdk::query the second parameter
    // specifies the starting index and the third parameter the maximum
    // number of items to return, with 0 meaning all.
    // This allows for pagination.
    // Future versions will also include more parameters for more complex
    // queries.
    hdk::query( "post".into(), 0, 0)
}

pub fn handle_get_post(post_address: Address) -> ZomeApiResult<Option<Entry>> {
    // get_entry returns a Result<Option<T>, ZomeApiError>
    // where T is the type that you used to commit the entry, in this case a Blog
    // It's a ZomeApiError if something went wrong (i.e. wrong type in deserialization)
    // Otherwise its a Some(T) or a None
    hdk::get_entry(post_address)
}
