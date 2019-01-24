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
    AGENT_ID_STR,
    DNA_NAME,
    DNA_ADDRESS,
};
use post::Post;

#[derive(Serialize, Deserialize, Debug, DefaultJson, PartialEq)]
struct SumInput {
    num1: u32,
    num2: u32,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct Env {
    dna_name: String,
    dna_address: String,
    agent_id: String,
    agent_address: String,
}

/// This handler shows how you can access the globals that are always available
/// inside a zome.  In this case it just creates an object with their values
/// and returns it as the result.
pub fn handle_show_env() -> ZomeApiResult<Env> {
    Ok(Env{
        dna_name: DNA_NAME.to_string(),
        dna_address: DNA_ADDRESS.to_string(),
        agent_id: AGENT_ID_STR.to_string(),
        agent_address: AGENT_ADDRESS.to_string(),
    })
}

fn check_sum_args(num1: u32, num2: u32) -> SumInput {
    SumInput {
        num1: num1,
        num2: num2,
    }
}

pub fn handle_check_sum(num1: u32, num2: u32) -> ZomeApiResult<JsonString> {
    hdk::call(
        hdk::THIS_INSTANCE,
        "summer",
        "test_token",
        "sum",
        check_sum_args(num1, num2).into(),
    )
}

fn post_entry(content: String) -> Entry {
    Entry::App("post".into(), Post::new(&content, "now").into())
}

pub fn handle_post_address(content: String) -> ZomeApiResult<Address> {
    hdk::entry_address(&post_entry(content))
}

pub fn handle_create_post(content: String, in_reply_to: Option<Address>) -> ZomeApiResult<Address> {
    let address = hdk::commit_entry(&post_entry(content))?;

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

#[cfg(test)]
pub mod tests {

    use blog::check_sum_args;
    use blog::SumInput;
    use post::Post;
    use blog::post_entry;
    use hdk::holochain_core_types::entry::Entry;
    use hdk::holochain_core_types::entry::AppEntryValue;
    use hdk::holochain_core_types::entry::entry_type::AppEntryType;

    #[test]
    fn check_sum_args_test() {
        assert_eq!(
            check_sum_args(1, 1),
            SumInput{
                num1: 1,
                num2: 1,
            },
        );
    }

    #[test]
    fn post_entry_test() {
        assert_eq!(
            post_entry("foos & bars".into()),
            Entry::App(
                AppEntryType::from("post"),
                AppEntryValue::from(
                    Post::new(
                        "foos & bars".into(),
                        "now".into(),
                    )
                ),
            ),
        )
    }

}
