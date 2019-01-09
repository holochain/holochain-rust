extern crate hdk;

fn genesis() {
    let post_address = hdk::commit_entry(
        "handle",
        json!(
            {
                "content": hdk::APP_AGENT_STR,
                "version": hdk::VERSION_STR,
                "dna": hdk::DNA_NAME,
            }
        ),
    );

    hdk::link_entries(hdk::APP_AGENT_HASH, post_address.clone(), "authored_posts");

    let in_reply_to = input["in_reply_to"].to_string();
    if !in_reply_to.is_empty() {
        if hdk::get_entry(in_reply_to.clone()).is_some() {
            hdk::link_entries(in_reply_to, post_address.clone(), "comments");
        }
    }

    json!({ "address": post_address })
}

#[no_mangle]
pub extern "C" fn posts_by_agent(input: serde_json::Value) -> serde_json::Value {
    let links = hdk::get_links(input["agent"].to_string(), "authored_posts");
    json!({ "post_addresses": links })
}

#[no_mangle]
pub extern "C" fn get_post(input: serde_json::Value) -> serde_json::Value {
    json!({"post": hdk::get_entry(input["post_address"].to_string()) })
}
