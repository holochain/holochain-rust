use hdk::{
    self,
    error::ZomeApiResult,
    holochain_core_types::{
        cas::content::Address, entry::Entry, error::HolochainError, json::JsonString,
    },
    holochain_wasm_utils::api_serialization::{
        get_entry::GetEntryOptions,
        get_links::{GetLinksOptions, GetLinksResult},
    },
    AGENT_ADDRESS,
};
use post::Post;

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
    hdk::call(
        hdk::THIS_INSTANCE,
        "summer",
        "main",
        "test_token",
        "sum",
        call_input.into(),
    )
}

pub fn handle_check_send(to_agent: Address, message: String) -> ZomeApiResult<String> {
    hdk::send(to_agent, message)
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
        hdk::get_entry_result(&in_reply_to_address, GetEntryOptions::default())?;
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

pub fn handle_my_posts_immediate_timeout() -> ZomeApiResult<GetLinksResult> {
    hdk::get_links_with_options(
        &AGENT_ADDRESS,
        "authored_posts",
        GetLinksOptions {
            timeout: 0.into(),
            ..Default::default()
        },
    )
}

pub fn handle_my_posts_as_commited() -> ZomeApiResult<Vec<Address>> {
    // In the current implementation of hdk::query the second parameter
    // specifies the starting index and the third parameter the maximum
    // number of items to return, with 0 meaning all.
    // This allows for pagination.
    // Future versions will also include more parameters for more complex
    // queries.
    hdk::query("post".into(), 0, 0)
}

pub fn handle_get_post(post_address: Address) -> ZomeApiResult<Option<Entry>> {
    // get_entry returns a Result<Option<T>, ZomeApiError>
    // where T is the type that you used to commit the entry, in this case a Blog
    // It's a ZomeApiError if something went wrong (i.e. wrong type in deserialization)
    // Otherwise its a Some(T) or a None
    hdk::get_entry(&post_address)
}

pub fn handle_recommend_post(post_address: Address, agent_address: Address) -> ZomeApiResult<()> {
    hdk::debug(format!("my address:\n{:?}", AGENT_ADDRESS.to_string()))?;
    hdk::debug(format!("other address:\n{:?}", agent_address.to_string()))?;
    hdk::link_entries(&agent_address, &post_address, "recommended_posts")
}

pub fn handle_my_recommended_posts() -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&AGENT_ADDRESS, "recommended_posts")
}
