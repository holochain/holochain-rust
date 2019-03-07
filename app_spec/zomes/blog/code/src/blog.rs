use hdk::{
    self,
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::{
        cas::content::Address, entry::Entry, error::HolochainError, json::JsonString,
    },
    holochain_wasm_utils::api_serialization::{
        get_entry::{GetEntryOptions, GetEntryResultType,EntryHistory, StatusRequestKind,GetEntryResult},
        get_links::{GetLinksOptions, GetLinksResult}
    },
    AGENT_ADDRESS, AGENT_ID_STR, DNA_ADDRESS, DNA_NAME, PUBLIC_TOKEN,
};
use post::Post;
use std::convert::TryFrom;

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
    let _dna_entry = hdk::get_entry(&DNA_ADDRESS)?;
    let _agent_entry = hdk::get_entry(&AGENT_ADDRESS)?;
    Ok(Env {
        dna_name: DNA_NAME.to_string(),
        dna_address: DNA_ADDRESS.to_string(),
        agent_id: AGENT_ID_STR.to_string(),
        agent_address: AGENT_ADDRESS.to_string(),
    })
}

pub fn handle_get_sources(address: Address) -> ZomeApiResult<Vec<Address>> {
    if let GetEntryResultType::Single(result) = hdk::get_entry_result(
        &address,
        GetEntryOptions {
            headers: true,
            ..Default::default()
        },
    )?
    .result
    {
        Ok(result
            .headers
            .into_iter()
            .map(|header| header.provenances().first().unwrap().clone().0)
            .collect())
    } else {
        unimplemented!()
    }
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
        Address::from(PUBLIC_TOKEN.to_string()),
        "sum",
        check_sum_args(num1, num2).into(),
    )
}

pub fn handle_check_send(to_agent: Address, message: String) -> ZomeApiResult<String> {
    hdk::send(to_agent, message, 10000.into())
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



pub fn handle_delete_post(content:String) -> ZomeApiResult<Address>
{
    let address = hdk::entry_address(&post_entry(content))?;
    hdk::remove_link(&AGENT_ADDRESS,&address.clone(),"authored_posts")?;
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

pub fn handle_delete_entry_post(post_address: Address) -> ZomeApiResult<()> {
    hdk::get_entry(&post_address)?;

    hdk::remove_entry(&post_address)?;

    Ok(())
}

pub fn handle_get_initial_post(post_address: Address) ->ZomeApiResult<Option<Entry>>
{
    hdk::get_entry_initial(&post_address)
}

pub fn handle_get_post_with_options_latest(post_address : Address) -> ZomeApiResult<Entry>
{
    let res = hdk::get_entry_result(
        &post_address,
        GetEntryOptions::new(StatusRequestKind::All, false, false, Default::default()),
    )?;
    let latest = res.latest().ok_or(ZomeApiError::Internal("Could not write this".into()))?;
    Ok(latest)
}

pub fn handle_my_post_with_options(post_address : Address) ->ZomeApiResult<GetEntryResult>
{
    hdk::get_entry_result(
        &post_address,
        GetEntryOptions::new(StatusRequestKind::All, false, false, Default::default()),
    )
}

pub fn handle_get_history_post(post_address : Address) -> ZomeApiResult<EntryHistory>
{
    let history = hdk::get_entry_history(&post_address)?.ok_or(ZomeApiError::Internal("Could not get History".into()));
    history
}



pub fn handle_update_post(post_address: Address, new_content: String) -> ZomeApiResult<Address> {
    let old_entry = hdk::get_entry(&post_address)?;

    if let Some(Entry::App(_, json_string)) = old_entry {
        let post = Post::try_from(json_string)?;
        let updated_post_entry = Entry::App(
            "post".into(),
            Post::new(&new_content, &post.date_created).into(),
        );

        hdk::update_entry(updated_post_entry, &post_address)
    } else {
        Err(ZomeApiError::Internal("failed to update post".into()))
    }
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

    use blog::{check_sum_args, post_entry, SumInput};
    use hdk::holochain_core_types::entry::{entry_type::AppEntryType, AppEntryValue, Entry};
    use post::Post;

    #[test]
    fn check_sum_args_test() {
        assert_eq!(check_sum_args(1, 1), SumInput { num1: 1, num2: 1 },);
    }

    #[test]
    fn post_entry_test() {
        assert_eq!(
            post_entry("foos & bars".into()),
            Entry::App(
                AppEntryType::from("post"),
                AppEntryValue::from(Post::new("foos & bars".into(), "now".into(),)),
            ),
        )
    }

}
