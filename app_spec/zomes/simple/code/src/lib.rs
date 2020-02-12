use hdk::prelude::*;
// see https://developer.holochain.org/api/latest/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Simple {
    content: String,
}

impl Simple
{
    pub fn new(content:String) -> Simple
    {
        Simple{content}
    }
}

fn simple_entry(content: String) -> Entry {
    Entry::App("simple".into(), Simple::new(content).into())
}

pub fn handle_create_anchor() -> ZomeApiResult<Address> {
    let address = hdk::commit_entry(&simple_entry("ANCHOR".to_string()))?;
    Ok(address)
}

pub fn handle_create_my_link(base: Address,target : String) -> ZomeApiResult<()> {
    let address = hdk::commit_entry(&simple_entry(target))?;
    hdk::link_entries(&base, &Address::from(address), "authored_simple_posts", "tag")?;
    Ok(())
}

pub fn handle_create_my_link_with_tag(base: Address,target : String, tag : String) -> ZomeApiResult<()> {
    let address = hdk::commit_entry(&simple_entry(target))?;
    hdk::link_entries(&base, &Address::from(address), "authored_simple_posts", &tag)?;
    Ok(())
}

pub fn handle_delete_my_link(base: Address,target : String) -> ZomeApiResult<()> {
    let address = hdk::entry_address(&simple_entry(target))?;
    hdk::remove_link(&base, &Address::from(address), "authored_simple_posts","tag")?;
    Ok(())
}

pub fn handle_delete_my_link_with_tag(base: Address,target : String,tag:String) -> ZomeApiResult<()> {
    let address = hdk::entry_address(&simple_entry(target))?;
    hdk::remove_link(&base, &Address::from(address), "authored_simple_posts",&tag)?;
    Ok(())
}


pub fn handle_get_my_links(agent : Address,status_request:Option<LinksStatusRequestKind>,sort_order:Option<SortOrder>) ->ZomeApiResult<GetLinksResult>
{
    let options = GetLinksOptions{
        status_request : status_request.unwrap_or(LinksStatusRequestKind::All),
        sort_order,
        ..GetLinksOptions::default()
    };
    hdk::get_links_with_options(&agent, LinkMatch::Exactly("authored_simple_posts"), LinkMatch::Any,options)
}

pub fn handle_get_my_links_with_tag(agent : Address,status_request:LinksStatusRequestKind,tag:String) ->ZomeApiResult<GetLinksResult>
{
    let options = GetLinksOptions{
        status_request,
        ..GetLinksOptions::default()
    };
    hdk::get_links_with_options(&agent, LinkMatch::Exactly("authored_simple_posts"), LinkMatch::Exactly(&*tag),options)
}

pub fn handle_get_my_links_with_pagination(agent : Address,pagesize:usize,pagenumber:usize) ->ZomeApiResult<GetLinksResult>
{
    let options = GetLinksOptions{
        pagination : Some(Pagination::Size(SizePagination{page_size : pagesize,page_number : pagenumber})),
        ..GetLinksOptions::default()
    };
    hdk::get_links_with_options(&agent, LinkMatch::Exactly("authored_simple_posts"), LinkMatch::Any,options)
}

pub fn handle_get_my_links_with_time_pagination(agent : Address,from_seconds:i64,limit:usize) ->ZomeApiResult<GetLinksResult>
{
    let options = GetLinksOptions{
        pagination : Some(Pagination::Time(TimePagination{from_time : Iso8601::new(from_seconds,0),limit})),
        ..GetLinksOptions::default()
    };
    hdk::get_links_with_options(&agent, LinkMatch::Exactly("authored_simple_posts"), LinkMatch::Any,options)
}

pub fn handle_get_my_links_count(agent : Address,status_request : LinksStatusRequestKind,tag:String) ->ZomeApiResult<GetLinksResultCount>
{
    let options = GetLinksOptions{
        status_request,
        ..GetLinksOptions::default()
    };
    hdk::get_links_count_with_options(&agent, LinkMatch::Exactly("authored_simple_posts"),LinkMatch::Exactly(&*tag),options)
}

pub fn handle_test_emit_signal(message: String) -> ZomeApiResult<()> {
    #[derive(Debug, Serialize, Deserialize, DefaultJson)]
    struct SignalPayload {
        message: String
    }

    hdk::emit_signal("test-signal", SignalPayload{message})
}

pub fn handle_encrypt(payload : String) ->ZomeApiResult<String>
{
    hdk::encrypt(payload)
}

pub fn handle_decrypt(payload : String) ->ZomeApiResult<String>
{
    hdk::decrypt(payload)
}

pub fn definition() -> ValidatingEntryType {
    entry!(
        name: "simple",
        description: "this is a simple definition for lightweight app_spec tests",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },

        validation: | _validation_data: hdk::EntryValidationData<Simple>| {
            Ok(())
        },

        links: [
            from!(
                "%agent_id",
                link_type: "authored_simple_posts",
                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },
                validation: | _validation_data: hdk::LinkValidationData | {
                    // test validation of links based on their tag
                    Ok(())
                }
            )]
    )
}

fn get_entry_handler(address: Address) -> ZomeApiResult<Option<Entry>> {
    hdk::get_entry(&address)
}

define_zome! {

    entries: [
       definition()
    ]

    init: || {
        Ok(())
    }

    validate_agent: |validation_data : EntryValidationData::<AgentId>| {{
        if let EntryValidationData::Create{entry, ..} = validation_data {
            let agent = entry as AgentId;
            if agent.nick == "reject_agent::app" {
                Err("This agent will always be rejected".into())
            } else {
                Ok(())
            }
        } else {
            Err("Cannot update or delete an agent at this time".into())
        }
    }}

    functions: [
        create_anchor: {
            inputs: | |,
            outputs: |result: ZomeApiResult<Address>|,
            handler: handle_create_anchor
        }
        get_entry: {
            inputs: |address: Address|,
            outputs: |result: ZomeApiResult<Option<Entry>>|,
            handler: get_entry_handler
        }
        create_link: {
            inputs: |base : Address,target:String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_create_my_link
        }
        create_link_with_tag: {
            inputs: |base : Address,target:String,tag:String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_create_my_link_with_tag
        }
        delete_link: {
            inputs: |base : Address,target:String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_delete_my_link
        }
        delete_link_with_tag: {
            inputs: |base : Address,target:String,tag:String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_delete_my_link_with_tag
        }
        get_my_links: {
            inputs: |base: Address,status_request:Option<LinksStatusRequestKind>,sort_order:Option<SortOrder>|,
            outputs: |result: ZomeApiResult<GetLinksResult>|,
            handler: handle_get_my_links
        }
        get_my_links_with_tag: {
            inputs: |base: Address,status_request:LinksStatusRequestKind,tag:String|,
            outputs: |result: ZomeApiResult<GetLinksResult>|,
            handler: handle_get_my_links_with_tag
        }
        get_my_links_with_pagination: {
            inputs: |base: Address,pagesize:usize,pagenumber:usize|,
            outputs: |result: ZomeApiResult<GetLinksResult>|,
            handler: handle_get_my_links_with_pagination
        }
        get_my_links_with_time_pagination: {
            inputs: |base: Address,from_seconds:i64,limit:usize|,
            outputs: |result: ZomeApiResult<GetLinksResult>|,
            handler: handle_get_my_links_with_time_pagination
        }
        get_my_links_count: {
            inputs: |base: Address,status_request:LinksStatusRequestKind,tag:String|,
            outputs: |result: ZomeApiResult<GetLinksResultCount>|,
            handler: handle_get_my_links_count
        }
        encrypt :{
            inputs : |payload: String|,
            outputs: |result: ZomeApiResult<String>|,
            handler: handle_encrypt
        }
        decrypt :{
            inputs : |payload: String|,
            outputs: |result: ZomeApiResult<String>|,
            handler: handle_decrypt
        }
        test_emit_signal: {
            inputs: |message: String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_test_emit_signal
        }
    ]

    traits: {
        hc_public [create_anchor, get_entry, create_link, delete_link, get_my_links, test_emit_signal,get_my_links_count,create_link_with_tag,get_my_links_count_by_tag,delete_link_with_tag,get_my_links_with_tag,encrypt,decrypt,get_my_links_with_pagination,get_my_links_with_time_pagination]
    }
}
