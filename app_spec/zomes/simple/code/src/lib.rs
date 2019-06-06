#![feature(try_from)]
#![warn(unused_extern_crates)]
#[macro_use]
extern crate hdk;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_core_types_derive;

use hdk::{
    entry_definition::ValidatingEntryType,
    error::ZomeApiResult,
};
use hdk::holochain_core_types::{
    cas::content::Address,
    dna::entry_types::Sharing,
    error::HolochainError,
    json::JsonString,
    entry::Entry
};

use hdk::holochain_wasm_utils::api_serialization::get_links::GetLinksResult;


// see https://developer.holochain.org/api/0.0.18-alpha1/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson,Clone)]
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


pub fn handle_create_my_link(base: Address,content : String) -> ZomeApiResult<()> {
    let address = hdk::commit_entry(&simple_entry(content))?;
    hdk::link_entries(&base, &address, "authored_posts", "")?;
    Ok(())
}

pub fn handle_delete_my_link(base: Address,content : String) -> ZomeApiResult<()> {
    let address = hdk::entry_address(&simple_entry(content))?;
    hdk::remove_link(&base, &address, "authored_posts", "")?;
    Ok(())
}

pub fn handle_get_my_links(base: Address) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&base, Some("authored_posts".into()), None)
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
                link_type: "authored_posts",
                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },
                validation: | validation_data: hdk::LinkValidationData | {
                    // test validation of links based on their tag
                    if let hdk::LinkValidationData::LinkAdd{link, ..} = validation_data {
                        if link.link.tag() == "muffins" {
                            Err("This is the one tag that is not allowed!".into())
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
            )]
        
    )
}

define_zome! {

    entries: [
       definition()
    ]

    genesis: || {
        Ok(())
    }

  

    functions: [

        create_link: {
            inputs: |base : Address,content:String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_create_my_link
        }
        delete_link: {
            inputs: |base : Address,content:String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_delete_my_link
        }
        get_my_links: {
            inputs: |base: Address|,
            outputs: |result: ZomeApiResult<GetLinksResult>|,
            handler: handle_get_my_links
        }
    ]

    traits: {
        hc_public [create_link,delete_link,get_my_links]
    }
}

