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
    entry::Entry,
    link::LinkMatch,
    hash::HashString
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


pub fn handle_create_my_link(base: Address,target : String) -> ZomeApiResult<()> {
    let address = hdk::commit_entry(&simple_entry(target))?;
    hdk::link_entries(&base, &HashString::from(address), "authored_posts", "")?;
    Ok(())
}

pub fn handle_delete_my_link(base: Address,target : String) -> ZomeApiResult<()> {
    let address = hdk::entry_address(&simple_entry(target))?;
    hdk::remove_link(&base, &HashString::from(address), "authored_posts", "")?;
    Ok(())
}

pub fn handle_get_my_links(base: Address) -> ZomeApiResult<GetLinksResult> {
    hdk::get_links(&base, LinkMatch::Exactly("authored_posts"), LinkMatch::Any)
}

pub fn handle_test_emit_signal(message: String) -> ZomeApiResult<()> {
    hdk::emit_signal("test-signal", JsonString::from_json(&format!("{{\"message\": {}}}", message)))
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
                validation: | _validation_data: hdk::LinkValidationData | {
                    // test validation of links based on their tag
                    Ok(())
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
            inputs: |base : Address,target:String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_create_my_link
        }
        delete_link: {
            inputs: |base : Address,target:String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_delete_my_link
        }
        get_my_links: {
            inputs: |base: Address|,
            outputs: |result: ZomeApiResult<GetLinksResult>|,
            handler: handle_get_my_links
        }
        test_emit_signal: {
            inputs: |message: String|,
            outputs: |result: ZomeApiResult<()>|,
            handler: handle_test_emit_signal
        }
    ]

    traits: {
        hc_public [create_link,delete_link,get_my_links]
    }
}

