use hdk::{
    self,
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::{
        cas::content::Address,
        dna::capabilities::CapabilityRequest,
        entry::{cap_entries::CapabilityType, entry_type::EntryType, Entry},
        error::HolochainError,
        json::JsonString,
        signature::{Provenance, Signature},
    },
    holochain_wasm_utils::api_serialization::{
        commit_entry::CommitEntryOptions,
        get_entry::{
            EntryHistory, GetEntryOptions, GetEntryResult, GetEntryResultType, StatusRequestKind,
        },
        get_links::{GetLinksOptions, GetLinksResult},
        QueryArgsOptions, QueryResult,
    },
    AGENT_ADDRESS, AGENT_ID_STR, CAPABILITY_REQ, DNA_ADDRESS, DNA_NAME, PROPERTIES, PUBLIC_TOKEN,
};

use memo::Memo;
use post::Post;
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

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

pub fn handle_ping(to_agent: Address, message: String) -> ZomeApiResult<String> {

    let response = hdk::send(to_agent, JsonString::from(Message::Ping(PingPayload(message))).to_string(), 10000.into())?;

    // A JSON-encoded Result<..., HolochainError> is expected
    //let response_message: Result<Message, HolochainError> = JsonString::from_json(&response).try_into()?;
    let response_message: Result<Message, HolochainError> = serde_json::from_str(&response)
        .map_err(|e| HolochainError::ErrorGeneric(format!(
            "handle_ping: couldn't extract Result<Message, HolochainError> {:?}: {}",
            &response, e )))?;

    // We expect only a Message::PingPayload w/ a String in response to our Message::Ping
    match response_message {
        Err(e) => Err(e.into()),
        Ok(Message::Ping(PingPayload(string))) => Ok(string),
        other => Err(HolochainError::ErrorGeneric(format!(
            "Incorrect response to hdk::send of Message::Ping: {:?}",
            other)).into()),
    }
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
    "HcScj5GbxXdTq69sfnz3jcA4u5f35zftsuu5Eb3dBxHjgd9byUUW6JmN3Bvzqqr";

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

// The hdk::send/receive Message types we know about.  These are Serialized to JSON to hdk::send,
// and a Result<Message, HolochainError> response is Serialized by the receive callback.
#[derive(Serialize, Deserialize, Debug, DefaultJson, PartialEq)]
enum Message {
    PostRequest(PostMessageBody),
    PostReply(Address),
    Ping(PingPayload),
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, PartialEq)]
struct PingPayload(String);

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

/// See if the Post w/ the given Address appears in the source-chain; if not return an Err
fn validate_post_in_source_chain(
    post_addr: Address
) -> Result<Address, HolochainError> {
    // Confirm Entry hits the local source-chain and is immediately accessible via hdk::query.
    // First, get an Vec<Address, Entry>
    match hdk::query_result(
        "post".into(),
        QueryArgsOptions{ entries: true, ..Default::default() }
    ) {
        Ok(QueryResult::Entries(addr_entry_vec)) => {
            // Convert Vec<(Address, Entry)> int Vec<(Address, Post)>, catching any HolochainErrors,
            // filtering out any that aren't the one just added above w/ Address == post_addr.  This
            // is silly, but allows us to catch any non-Post Entry...
            match addr_entry_vec
                .iter()
                .map(|(addr, entry)| {
                    match entry {
                        Entry::App(_entry_type, entry_value)
                            => Ok((addr.to_owned(), Post::try_from(entry_value)?)),
                        unknown
                            => Err(HolochainError::ErrorGeneric(format!(
                                "Unexpected hdk::query response entry type for post: {:?}", &unknown))),
                    }
                })
                .filter(|addr_post_maybe| {
                    match addr_post_maybe {
                        Ok((addr, _post)) => if *addr == post_addr {
                            hdk::debug(format!(
                                "Found just-committed Post {} in hdk::query results", &post_addr)).ok();
                            true
                        } else {
                            false
                        },
                        Err(_) => true,
                    }
                })
                .collect::<Result<Vec<(Address, Post)>, HolochainError>>()
            {
                Err(e) => Err(e),
                Ok(addr_post_vec) => {
                    // The last entry in the Vec<(Address, Post)> must be the one we just
                    // posted.  Unless we filter out all others, this *might* not be the case
                    // (eg. if a different Thread also just committed a Post).
                    if addr_post_vec.len() < 1 || addr_post_vec[addr_post_vec.len() - 1].0 != post_addr {
                        Err(HolochainError::ErrorGeneric(format!(
                            "Couldn't find the Post we just committed: {:#?}", addr_post_vec )))
                    } else {
                        Ok(post_addr)
                    }
                },
            }
        },
        other => Err(HolochainError::ErrorGeneric(format!(
            "Unexpected hdk::query response for post: {:?}", other))),
    }
}

// post calls the create_post zome function handler after checking the supplied signature
fn handle_receive_post(
    from: Address,
    post_body: PostMessageBody
) -> Result<Message, HolochainError> {
    // check that the claim matches a grant and correctly signed the content
    if !check_claim_against_grant(
        &post_body.claim,
        Provenance::new(from, post_body.signature),
        post_body.args.content.clone(),
    ) {
        Err(HolochainError::ErrorGeneric(format!("error: no matching grant for claim")))
    } else {
        let response = match hdk::commit_entry(&post_entry(post_body.args.content)) {
            Err(err) => Err(HolochainError::ErrorGeneric(format!(
                "error: couldn't create post: {}", err))),
            Ok(post_addr) => Ok(Message::PostReply(post_addr)),
        };

        if let Ok(Message::PostReply(post_addr)) = response {
            match validate_post_in_source_chain(post_addr) {
                Ok(post_addr) => {
                    // Success; re-constitute the original result.  TODO: When hdk::commit_entry /
                    // hdk::query testing is complete, remove all of the surrounding code involving
                    // validate_post_in_source_chain, and just retain the following:
                    Ok(Message::PostReply(post_addr))

                    // TODO: This BadCallError failure was due to accessing hdk::AGENT_ADDRESS.
                    // Someone who understands the semantics of "authored_posts" should re-enable
                    // this code:

                    // let _ = hdk::link_entries(&AGENT_ADDRESS, &Address::from(x.clone()), "authored_posts");
                    /*
                        When we figure out why link_entries above throws an BadCall wasm error
                        Then we can reinstate calling the creating using the handler as below
                        match handle_create_post(post_body.args.content, post_body.args.in_reply_to) {
                        Err(err) => format!("error: couldn't create post: {}", err),
                        Ok(address) => address.to_string(),
                     */
                },
                Err(e) => Err(e),
            }
        } else {
            response // Err(...) from check_claim_against_grant
        }
    }
}

// ping simply returns the payload, with the sender's Address and our Address
fn handle_receive_ping(
    from: Address,
    payload: PingPayload
) -> Result<Message, HolochainError> {
    Ok(Message::Ping(PingPayload(format!(
        "got {} from {} at {}", payload.0, &from, AGENT_ADDRESS.to_string()))))
}

// this is an example of a receive function that can handle a typed messaged
pub fn handle_receive(
    from: Address,
    json_msg: JsonString
) -> String {
    let maybe_message: Result<Message, HolochainError> = json_msg.try_into();
    let response: Result<Message, HolochainError> = match maybe_message {
        Err(err) => Err(err),
        Ok(message) => match message {
            Message::Ping(payload) => handle_receive_ping(from, payload),
            Message::PostRequest(post_body) => handle_receive_post(from, post_body),
            typ => Err(HolochainError::ErrorGeneric(format!("unknown message type: {:?}", typ))),
        },
    };

    JsonString::from(response).to_string()
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
    // retrieve a previously stored claim
    let claim = find_claim("can_blog", &grantor)?;

    let post_body = PostMessageBody {
        claim,
        signature: hdk::sign(content.clone()).map(Signature::from)?,
        args: CreatePostArgs {
            content,
            in_reply_to,
        },
    };

    let message = Message::PostRequest(post_body);

    let response = hdk::send(grantor, JsonString::from(message).into(), 10000.into())?;
    // TODO: avoid serde_json::from_str() when JsonString...try_into() works for Result<...>
    //let response_message: Result<Message, HolochainError> = JsonString::from_json(&response).try_into()?;
    let response_message: Result<Message, HolochainError> = serde_json::from_str(&response)
        .map_err(|e| HolochainError::ErrorGeneric(format!(
            "handle_create_post_with_claim: couldn't extract Result<Message, HolochainError> {:?}: {}",
            &response, e )))?;

    // We expect only a Message::PostReply w/ an Address in response to our Message::PostRequest
    match response_message {
        Err(e) => Err(e.into()),
        Ok(Message::PostReply(address)) => Ok(address),
        other => Err(HolochainError::ErrorGeneric(format!(
            "Incorrect response to hdk::send of Message::PostRequest: {:?}",
            other)).into()),
    }
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
    hdk::get_links(&agent, Some("authored_posts".into()), None)
}

pub fn handle_my_posts(tag: Option<String>) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&AGENT_ADDRESS, Some("authored_posts".into()), tag)
}

pub fn handle_my_posts_with_load(tag: Option<String>) -> ZomeApiResult<Vec<Post>> {
    hdk::utils::get_links_and_load_type(&AGENT_ADDRESS, Some("authored_posts".into()), tag)
}

pub fn handle_my_memos() -> ZomeApiResult<Vec<Address>> {
    hdk::query("memo".into(), 0, 0)
}

// As memos are private we expect this will never return anything but None.
pub fn handle_get_memo(address: Address) -> ZomeApiResult<Option<Entry>> {
    hdk::get_entry(&address)
}

pub fn handle_my_posts_immediate_timeout() -> ZomeApiResult<GetLinksResult> {
    hdk::get_links_with_options(
        &AGENT_ADDRESS,
        Some("authored_posts".into()),
        None,
        GetLinksOptions {
            timeout: 0.into(),
            ..Default::default()
        },
    )
}

pub fn handle_my_posts_get_my_sources(agent: Address) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links_with_options(
        &agent,
        Some("authored_posts".into()),
        None,
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
    hdk::debug(format!("my address:\n{:?}", AGENT_ADDRESS.to_string()))?;
    hdk::debug(format!("other address:\n{:?}", agent_address.to_string()))?;
    hdk::link_entries(&agent_address, &post_address, "recommended_posts", "")
}

pub fn handle_my_recommended_posts() -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&AGENT_ADDRESS, Some("recommended_posts".into()), None)
}

pub fn handle_get_post_bridged(post_address: Address) -> ZomeApiResult<Option<Entry>> {
    // Obtains the post via bridge to another instance
    let raw_json = hdk::call(
        "test-bridge",
        "blog",
        Address::from(PUBLIC_TOKEN.to_string()),
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

    let entry: Option<Entry> = raw_json.try_into()?;

    hdk::debug(format!(
        "********DEBUG******** BRIDGING ACTUAL response from hosting-bridge {:?}",
        entry
    ))?;

    Ok(entry)
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
