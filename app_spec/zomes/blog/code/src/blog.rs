use hdk::{
    self,
    error::{ZomeApiError, ZomeApiResult},
    holochain_persistence_api::{
        cas::content::Address,
    },
    holochain_json_api::{
        json::JsonString,
        error::JsonError
    },
    holochain_core_types::{
        dna::capabilities::CapabilityRequest,
        entry::{cap_entries::CapabilityType, entry_type::EntryType, Entry},
        error::HolochainError,
        signature::{Provenance, Signature},
        link::LinkMatch,
    },
    holochain_wasm_utils::api_serialization::{
        commit_entry::CommitEntryOptions,
        get_entry::{
            EntryHistory, GetEntryOptions, GetEntryResult, GetEntryResultType, StatusRequestKind,
        },
        get_links::{GetLinksOptions, GetLinksResult},
        QueryArgsOptions, QueryResult,
    },
    AGENT_ADDRESS, AGENT_ID_STR, CAPABILITY_REQ, DNA_ADDRESS, DNA_NAME, PROPERTIES, PUBLIC_TOKEN
};

use memo::Memo;
use post::Post;
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

#[derive(Serialize, Deserialize, Debug, DefaultJson, PartialEq)]
pub struct SumInput {
    num1: u32,
    num2: u32,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct Env {
    dna_name: String,
    dna_address: String,
    agent_id: String,
    agent_address: String,
    cap_request: Option<CapabilityRequest>,
    properties: JsonString,
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
        cap_request: CAPABILITY_REQ.clone(),
        properties: PROPERTIES.clone(),
    })
}

pub fn handle_get_test_properties() -> ZomeApiResult<JsonString> {
    hdk::property("test_property")
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
            .map(|header| header.provenances().first().unwrap().clone().source())
            .collect())
    } else {
        unimplemented!()
    }
}

pub fn check_sum_args(num1: u32, num2: u32) -> SumInput {
    SumInput {
        num1: num1,
        num2: num2,
    }
}

pub fn handle_check_sum(num1: u32, num2: u32) -> ZomeApiResult<u32> {
    let call_json = hdk::call(
        hdk::THIS_INSTANCE,
        "summer",
        Address::from(PUBLIC_TOKEN),
        "sum",
        check_sum_args(num1, num2).into(),
    )?;
    let result: ZomeApiResult<u32> = call_json.try_into()?;
    result
}

pub fn handle_ping(to_agent: Address, message: String) -> ZomeApiResult<JsonString> {
    let json_msg = json!({
        "msg_type": "ping",
        "body" : message
    })
    .to_string();
    let received_str = hdk::send(to_agent, json_msg, 10000.into())?;
    Ok(JsonString::from_json(&received_str))
}

fn post_entry(content: String) -> Entry {
    Entry::App("post".into(), Post::new(&content, "now").into())
}

fn memo_entry(content: String) -> Entry {
    Entry::App("memo".into(), Memo::new(&content, "now").into())
}

pub fn handle_post_address(content: String) -> ZomeApiResult<Address> {
    hdk::entry_address(&post_entry(content))
}

pub static BOB_AGENT_ID: &'static str =
    "HcSCJCqoIY3uwiw34acyvNmJMyzkk4y9groHdYKBekqp7y48mvwfVTQQkzcjnfz";

fn is_my_friend(addr: Address) -> bool {
    addr == Address::from(BOB_AGENT_ID)
}

pub fn handle_request_post_grant() -> ZomeApiResult<Option<Address>> {
    // we may want to extend the testing conductor to be able to make calls with
    // arbitrary provenances.  If so we could get the caller we want from the
    // CAPABILITY_REQ global like this:
    //    let addr = CAPABILITY_REQ.provenance.source();
    // but it doesn't work yet so for this test we are hard-coding the "friend"" to bob
    let addr = Address::from(BOB_AGENT_ID);

    if is_my_friend(addr.clone()) {
        let mut functions = BTreeMap::new();
        functions.insert("blog".to_string(), vec!["create_post".to_string()]);
        Ok(Some(hdk::commit_capability_grant(
            "can_post",
            CapabilityType::Assigned,
            Some(vec![addr]),
            functions,
        )?))
    } else {
        Ok(None)
    }
}

pub fn handle_get_grants() -> ZomeApiResult<Vec<Address>> {
    hdk::query(EntryType::CapTokenGrant.into(), 0, 0)
}

pub fn handle_commit_post_claim(grantor: Address, claim: Address) -> ZomeApiResult<Address> {
    hdk::commit_capability_claim("can post", grantor, claim)
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, PartialEq)]
struct CreatePostArgs {
    content: String,
    in_reply_to: Option<Address>,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, PartialEq)]
struct Message {
    msg_type: String,
    body: JsonString,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, PartialEq)]
struct PostMessageBody {
    claim: Address,
    signature: Signature,
    args: CreatePostArgs,
}

fn check_claim_against_grant(claim: &Address, provenance: Provenance, payload: String) -> bool {
    // first make sure the payload is what was signed in the provenance
    let signed = hdk::verify_signature(provenance.clone(), payload).unwrap_or(false);
    if !signed {
        return false;
    };

    // Then look up grants and find one that matches the claim, and then check to see if the
    // source in the provenance matches one of the assignees of the grant.
    let result = match hdk::query_result(
        EntryType::CapTokenGrant.into(),
        QueryArgsOptions {
            entries: true,
            ..Default::default()
        },
    ) {
        Ok(r) => r,
        Err(_) => return false,
    };
    match result {
        QueryResult::Entries(entries) => entries
            .iter()
            .filter(|(addr, _)| claim == addr)
            .find(|(_, entry)| match entry {
                Entry::CapTokenGrant(ref grant) => match grant.assignees() {
                    Some(assignees) => assignees.contains(&provenance.source()),
                    None => false,
                },
                _ => false,
            })
            .is_some(),
        _ => false,
    }
}

// this is an example of a receive function that can handle a typed messaged
pub fn handle_receive(from: Address, json_msg: JsonString) -> String {
    let maybe_message: Result<Message, _> = json_msg.try_into();
    let response = match maybe_message {
        Err(err) => format!("error: {}", err),
        Ok(message) => match message.msg_type.as_str() {
            // ping simply returns the body of the message
            "ping" => format!("got {} from {}", message.body, from),

            // post calls the create_post zome function handler after checking the supplied signature
            "post" => {
                let maybe_post_body: Result<PostMessageBody, _> =
                    message.body.try_into();
                match maybe_post_body {
                    Err(err) => format!("error: couldn't parse body: {}", err),
                    Ok(post_body) => {
                        // check that the claim matches a grant and correctly signed the content
                        if !check_claim_against_grant(
                            &post_body.claim,
                            Provenance::new(from, post_body.signature),
                            post_body.args.content.clone(),
                        ) {
                            "error: no matching grant for claim".to_string()
                        } else {
                            let x = match hdk::commit_entry(&post_entry(post_body.args.content)) {
                                Err(err) => format!("error: couldn't create post: {}", err),
                                Ok(addr) => addr.to_string(),
                            };
                            let _ =
                                hdk::debug("For some reason this link_entries statement fails!?!?");
                            //                            let _ = hdk::link_entries(&AGENT_ADDRESS, &Address::from(x.clone()), "authored_posts");

                            x

                            /*
                                When we figure out why link_entries above throws an BadCall wasm error
                                Then we can reinstate calling the creating using the handler as below
                                match handle_create_post(post_body.args.content, post_body.args.in_reply_to) {
                                Err(err) => format!("error: couldn't create post: {}", err),
                                Ok(address) => address.to_string(),
                            }*/
                        }
                    }
                }
            }
            typ => format!("unknown message type: {}", typ),
        },
    };
    json!({
        "msg_type": "response",
        "body": response
    })
    .to_string()
}

// this simply returns the first claim which works for this test, thus the arguments are ignored.
// The exercise of a "real" find_claim function, which we may add to the hdk later, is left to the reader
fn find_claim(_identifier: &str, _grantor: &Address) -> Result<Address, HolochainError> {
    //   Ok(Address::from("Qmebh1y2kYgVG1RPhDDzDFTAskPcRWvz5YNhiNEi17vW9G"))
    let claim = hdk::query_result(
        EntryType::CapTokenClaim.into(),
        QueryArgsOptions {
            entries: true,
            ..Default::default()
        },
    )
    .and_then(|result| match result {
        QueryResult::Entries(entries) => {
            let entry = &entries[0].1;
            match entry {
                Entry::CapTokenClaim(ref claim) => Ok(claim.token()),
                _ => Err(ZomeApiError::Internal("failed to get claim".into())),
            }
        }
        _ => Err(ZomeApiError::Internal("failed to get claim".into())),
    })?;
    Ok(claim)
}

pub fn handle_create_post_with_claim(
    grantor: Address,
    content: String,
    in_reply_to: Option<Address>,
) -> ZomeApiResult<Address> {
    // retrieve a previously stored claimed
    let claim = find_claim("can_blog", &grantor)?;

    let post_body = PostMessageBody {
        claim,
        signature: hdk::sign(content.clone()).map(Signature::from)?,
        args: CreatePostArgs {
            content,
            in_reply_to,
        },
    };

    let message = Message {
        msg_type: "post".to_string(),
        body: post_body.into(),
    };

    let response = hdk::send(grantor, JsonString::from(message).into(), 10000.into())?;
    let response_message: Message = JsonString::from_json(&response).try_into()?;
    Ok(Address::from(response_message.body))
}

pub fn handle_memo_address(content: String) -> ZomeApiResult<Address> {
    hdk::entry_address(&memo_entry(content))
}

pub fn handle_create_post(content: String, in_reply_to: Option<Address>) -> ZomeApiResult<Address> {
    let address = hdk::commit_entry(&post_entry(content))?;

    hdk::link_entries(&AGENT_ADDRESS, &address, "authored_posts", "")?;

    if let Some(in_reply_to_address) = in_reply_to {
        // return with Err if in_reply_to_address points to missing entry
        hdk::get_entry_result(&in_reply_to_address, GetEntryOptions::default())?;
        hdk::link_entries(&in_reply_to_address, &address, "comments", "")?;
    }

    Ok(address)
}

pub fn handle_create_tagged_post(content: String, tag: String) -> ZomeApiResult<Address> {
    let address = hdk::commit_entry(&post_entry(content))?;
    hdk::link_entries(&AGENT_ADDRESS, &address, "authored_posts", tag.as_ref())?;
    Ok(address)
}

pub fn handle_create_post_countersigned(
    content: String,
    in_reply_to: Option<Address>,
    counter_signature: Provenance,
) -> ZomeApiResult<Address> {
    let entry = post_entry(content);

    let options = CommitEntryOptions::new(vec![counter_signature]);

    let address = hdk::commit_entry_result(&entry, options).unwrap().address();

    hdk::link_entries(&AGENT_ADDRESS, &address, "authored_posts", "")?;

    if let Some(in_reply_to_address) = in_reply_to {
        // return with Err if in_reply_to_address points to missing entry
        hdk::get_entry_result(&in_reply_to_address, GetEntryOptions::default())?;
        hdk::link_entries(&in_reply_to_address, &address, "comments", "")?;
    }

    Ok(address)
}

pub fn handle_create_post_with_agent(
    agent_id: Address,
    content: String,
    in_reply_to: Option<Address>,
) -> ZomeApiResult<Address> {
    let address = hdk::commit_entry(&post_entry(content))?;

    hdk::link_entries(&agent_id, &address, "authored_posts", "")?;

    if let Some(in_reply_to_address) = in_reply_to {
        // return with Err if in_reply_to_address points to missing entry
        hdk::get_entry_result(&in_reply_to_address, GetEntryOptions::default())?;
        hdk::link_entries(&in_reply_to_address, &address, "comments", "")?;
    }

    Ok(address)
}

pub fn handle_create_memo(content: String) -> ZomeApiResult<Address> {
    let address = hdk::commit_entry(&memo_entry(content))?;

    Ok(address)
}

pub fn handle_delete_post(content: String) -> ZomeApiResult<Address> {
    let address = hdk::entry_address(&post_entry(content))?;
    hdk::remove_link(&AGENT_ADDRESS, &address.clone(), "authored_posts", "")?;
    Ok(address)
}

pub fn handle_posts_by_agent(agent: Address) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&agent, LinkMatch::Exactly("authored_posts"), LinkMatch::Any)
}


pub fn handle_my_posts(tag: Option<String>) -> ZomeApiResult<GetLinksResult> {
    let tag = match tag {Some(ref s) => LinkMatch::Regex(s.as_ref()), None => LinkMatch::Any};
    hdk::get_links(&AGENT_ADDRESS, LinkMatch::Exactly("authored_posts"), tag)
}

pub fn handle_my_posts_with_load(tag: Option<String>) -> ZomeApiResult<Vec<Post>> {
    let tag = match tag {Some(ref s) => LinkMatch::Exactly(s.as_ref()), None => LinkMatch::Any};
    hdk::utils::get_links_and_load_type(&AGENT_ADDRESS, LinkMatch::Exactly("authored_posts"), tag)
}

pub fn handle_my_memos() -> ZomeApiResult<Vec<Address>> {
    hdk::query("memo".into(), 0, 0)
}

// As memos are private we expect this will never return anything but None.
pub fn handle_get_memo(address: Address) -> ZomeApiResult<Option<Entry>> {
    hdk::get_entry(&address)
}

pub fn handle_my_posts_immediate_timeout() -> ZomeApiResult<GetLinksResult> {
    // use a link-type that doesn't exist so that it will timeout.
    // Note that a 0 timeout doesn't no guarantee that you won't get things
    // back in the in-memory-transport case because the redux loop action that runs
    // the get-link may actually return before the one that checks the timeout.
    hdk::get_links_with_options(
        &AGENT_ADDRESS,
        LinkMatch::Exactly("non-existent-link"),
        LinkMatch::Any,
        GetLinksOptions {
            timeout: 0.into(),
            ..Default::default()
        },
    )
}

pub fn handle_my_posts_get_my_sources(agent: Address) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links_with_options(
        &agent,
        LinkMatch::Exactly("authored_posts"),
        LinkMatch::Any,
        GetLinksOptions {
            headers: true,
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

pub fn handle_delete_entry_post(post_address: Address) -> ZomeApiResult<Address> {
    hdk::get_entry(&post_address)?;

    hdk::remove_entry(&post_address)
}

pub fn handle_get_initial_post(post_address: Address) -> ZomeApiResult<Option<Entry>> {
    hdk::get_entry_initial(&post_address)
}

pub fn handle_get_post_with_options_latest(post_address: Address) -> ZomeApiResult<Entry> {
    let res = hdk::get_entry_result(
        &post_address,
        GetEntryOptions::new(StatusRequestKind::All, false, false, Default::default()),
    )?;
    let latest = res
        .latest()
        .ok_or(ZomeApiError::Internal("Could not get latest".into()))?;
    Ok(latest)
}

pub fn handle_my_post_with_options(post_address: Address) -> ZomeApiResult<GetEntryResult> {
    hdk::get_entry_result(
        &post_address,
        GetEntryOptions::new(StatusRequestKind::All, false, false, Default::default()),
    )
}

pub fn handle_get_history_post(post_address: Address) -> ZomeApiResult<EntryHistory> {
    let history = hdk::get_entry_history(&post_address)?
        .ok_or(ZomeApiError::Internal("Could not get History".into()));
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

pub fn handle_recommend_post(
    post_address: Address,
    agent_address: Address,
) -> ZomeApiResult<Address> {
    hdk::debug(format!("my address:\n{:?}", AGENT_ADDRESS))?;
    hdk::debug(format!("other address:\n{:?}", agent_address))?;
    hdk::link_entries(&agent_address, &post_address, "recommended_posts", "")
}

pub fn handle_my_recommended_posts() -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&AGENT_ADDRESS, LinkMatch::Exactly("recommended_posts"), LinkMatch::Any)
}

pub fn handle_get_post_bridged(post_address: Address) -> ZomeApiResult<Option<Entry>> {
    // Obtains the post via bridge to another instance
    let raw_json = hdk::call(
        "test-bridge",
        "blog",
        Address::from(PUBLIC_TOKEN),
        "get_post",
        json!({
            "post_address": post_address,
        })
        .into(),
    )?;

    hdk::debug(format!(
        "********DEBUG******** BRIDGING RAW response from test-bridge {:?}",
        raw_json
    ))?;

    let entry: ZomeApiResult<Option<Entry>> = raw_json.try_into()?;

    hdk::debug(format!(
        "********DEBUG******** BRIDGING ACTUAL response from hosting-bridge {:?}",
        entry
    ))?;

    entry
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
