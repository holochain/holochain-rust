#![feature(try_from)]
#![warn(unused_extern_crates)]
#![feature(proc_macro_hygiene)]

use hdk_proc_macros::zome;


#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hdk;
#[macro_use]
extern crate holochain_json_derive;

use hdk::{
    error::ZomeApiResult,
    entry_definition::ValidatingEntryType,
    holochain_core_types::{
        entry::Entry,
        dna::entry_types::Sharing,
        link::LinkMatch
    },
    holochain_persistence_api::{
        cas::content::Address,
    },
    holochain_json_api::{
       json::JsonString,
       error::JsonError
    },
    holochain_wasm_utils::api_serialization::get_links::{GetLinksResult,LinksStatusRequestKind,GetLinksOptions}
};



#[zome]
pub mod simple {

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

    #[entry_def]
    pub fn simple_entry_def() -> ValidatingEntryType {
          definition()
    }


    #[genesis]
    pub fn genesis() {
        Ok(())
    }

    #[zome_fn("hc_public")]
    pub fn create_link(base: Address,target : String) -> ZomeApiResult<()>
    {
        let address = hdk::commit_entry(&simple_entry(target))?;
        hdk::link_entries(&base, &address, "authored_posts", "")?;
        Ok(())
    }
    #[zome_fn("hc_public")]
    pub fn delete_link(base: Address,target : String) -> ZomeApiResult<()> {
        let address = hdk::entry_address(&simple_entry(target))?;
        hdk::remove_link(&base, &address, "authored_posts", "")?;
        Ok(())
    }

    #[zome_fn("hc_public")]
    pub fn get_my_links(base: Address,status_request : Option<LinksStatusRequestKind>) -> ZomeApiResult<GetLinksResult>
    {
        let options = GetLinksOptions{
            status_request : status_request.unwrap_or(LinksStatusRequestKind::All),
            ..GetLinksOptions::default()
        };
        hdk::get_links_with_options(&base, LinkMatch::Exactly("authored_posts"), LinkMatch::Any,options)
    }

    #[zome_fn("hc_public")]
    pub fn test_emit_signal(message: String) -> ZomeApiResult<()> {
        #[derive(Debug, Serialize, Deserialize, DefaultJson)]
        struct SignalPayload {
            message: String
        }

        hdk::emit_signal("test-signal", SignalPayload{message})
    }

    #[zome_fn("hc_public")]
    pub fn encrypt(payload : String) -> ZomeApiResult<String> 
    {
       hdk::encrypt(payload)
    }

    #[zome_fn("hc_public")]
    pub fn decrypt(payload : String) -> ZomeApiResult<String> 
    {
       hdk::decrypt(payload)
    }

}

