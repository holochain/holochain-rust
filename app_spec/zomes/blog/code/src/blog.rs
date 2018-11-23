use hdk::error::ZomeApiError;
use hdk::holochain_core_types::error::HolochainError;
use hdk::{
    self,
    holochain_wasm_utils::api_serialization::get_entry::{
        GetEntryOptions,
    },
    holochain_core_types::hash::HashString,
    holochain_core_types::json::JsonString,
    holochain_core_types::entry::Entry,
    holochain_core_types::entry::entry_type::AppEntryType,
    AGENT_ADDRESS,
};
use hdk::holochain_core_types::cas::content::Address;

use post::Post;

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct AddressResponse {
    address: Address
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
struct MultiAddressResponse {
    addresses: Vec<Address>
}

pub fn handle_check_sum(num1: u32, num2: u32) -> JsonString {
    #[derive(Serialize, Deserialize, Debug, DefaultJson)]
    struct SumInput {
        num1: u32,
        num2: u32,
    };

    let call_input = SumInput {
        num1: num1,
        num2: num2,
    };
    let maybe_result = hdk::call("summer", "main", "sum", call_input.into());
    match maybe_result {
        Ok(result) => result.into(),
        Err(hdk_error) => hdk_error.into(),
    }
}

pub fn handle_hash_post(content: String) -> JsonString {
    let post_entry = Entry::App(AppEntryType::from("post"),
        Post {
            content: content.to_string(),
            date_created: "now".to_string()
        }.into()
    );


    match hdk::entry_address(&post_entry) {
        Ok(address) => AddressResponse{address}.into(),
        Err(hdk_error) => hdk_error.into(),
    }
}

pub fn handle_create_post(content: String, in_reply_to: HashString) -> JsonString {

    let post_entry = Entry::App(AppEntryType::from("post"),
        Post {
            content: content.to_string(),
            date_created: "now".to_string()
        }.into()
    );

    match hdk::commit_entry(&post_entry) {
        Ok(address) => {
            let link_result = hdk::link_entries(
                &HashString::from(AGENT_ADDRESS.to_string()),
                &address,
                "authored_posts"
            );

            if link_result.is_err() {
                return link_result.into()
            }

            let in_reply_to = in_reply_to;
            if !in_reply_to.to_string().is_empty() {
                if let Ok(_) = hdk::get_entry_result(in_reply_to.clone(), GetEntryOptions{}) {
                    let _ = hdk::link_entries(&in_reply_to, &address, "comments");
                }
            }
            AddressResponse{address}.into()
        }
        Err(hdk_error) => hdk_error.into(),
    }
}

pub fn handle_posts_by_agent(agent: HashString) -> JsonString {
    match hdk::get_links(&agent, "authored_posts") {
        Ok(result) => MultiAddressResponse{addresses: result.addresses().clone()}.into(),
        Err(hdk_error) => hdk_error.into(),
    }
}

pub fn handle_my_posts() -> JsonString {
    match hdk::get_links(&HashString::from(AGENT_ADDRESS.to_string()), "authored_posts") {
        Ok(result) => MultiAddressResponse{addresses: result.addresses().clone()}.into(),
        Err(hdk_error) => hdk_error.into(),
    }
}

pub fn handle_my_posts_as_commited() -> JsonString {
    // In the current implementation of hdk::query the second parameter
    // specifies the starting index and the third parameter the maximum
    // number of items to return, with 0 meaning all.
    // This allows for pagination.
    // Future versions will also include more parameters for more complex
    // queries.
    match hdk::query("post", 0, 0) {
        Ok(posts) => MultiAddressResponse{addresses: posts}.into(),
        Err(hdk_error) => hdk_error.into(),
    }
}

pub fn handle_get_post(post_address: HashString) -> JsonString {
    // get_entry returns a Result<Option<T>, ZomeApiError>
    // where T is the type that you used to commit the entry, in this case a Blog
    // It's a ZomeApiError if something went wrong (i.e. wrong type in deserialization)
    // Otherwise its a Some(T) or a None
    let result : Result<Option<Entry>,ZomeApiError> = hdk::get_entry(post_address);
    match result {
        // In the case we don't get an error
        // it might be an entry ...
        Ok(Some(Entry::App(_, entry_value))) => {
            entry_value
        },
        Ok(None) => {}.into(),

        // This error means that the string in `entry`
        // is not a stringified JSON which should not
        // happen but might be a bug somewhere else:
        Err(err) => err.into(),
        _ => unreachable!(),
    }
}
