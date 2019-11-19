#![feature(proc_macro_hygiene)]

use hdk::prelude::*;
use hdk_proc_macros::zome;

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
                link_type: "authored_simple_posts",
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

    /// doc comments are allowed on entry defs
    #[entry_def]
    pub fn simple_entry_def() -> ValidatingEntryType {
          definition()
    }


    #[init]
    pub fn init() {
        Ok(())
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData<AgentId>) {
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
    }

    /// doc comments are allowed on zome functions
    #[zome_fn("hc_public")]
    fn get_entry(address: Address) -> ZomeApiResult<Option<Entry>> {
        hdk::get_entry(&address)
    }

    pub fn create_link(base: Address,target : String) -> ZomeApiResult<()>
    {
        let address = hdk::commit_entry(&simple_entry(target))?;
        hdk::link_entries(&base, &address, "authored_simple_posts", "")?;
        Ok(())
    }

    #[zome_fn("hc_public")]
    pub fn create_link_with_tag(base: Address,target : String,tag:String) -> ZomeApiResult<()>
    {
        let address = hdk::commit_entry(&simple_entry(target))?;
        hdk::link_entries(&base, &address, "authored_simple_posts", &tag)?;
        Ok(())
    }
    #[zome_fn("hc_public")]
    pub fn delete_link(base: Address,target : String) -> ZomeApiResult<()> {
        let address = hdk::entry_address(&simple_entry(target))?;
        hdk::remove_link(&base, &address, "authored_simple_posts", "")?;
        Ok(())
    }

    #[zome_fn("hc_public")]
    pub fn get_my_links(base: Address,status_request : Option<LinksStatusRequestKind>) -> ZomeApiResult<GetLinksResult>
    {
        let options = GetLinksOptions{
            status_request : status_request.unwrap_or(LinksStatusRequestKind::All),
            ..GetLinksOptions::default()
        };
        hdk::get_links_with_options(&base, LinkMatch::Exactly("authored_simple_posts"), LinkMatch::Any,options)
    }

    #[zome_fn("hc_public")]
    pub fn get_my_links_count(base: Address,status_request : Option<LinksStatusRequestKind>) -> ZomeApiResult<GetLinksResultCount>
    {
        let options = GetLinksOptions{
            status_request : status_request.unwrap_or(LinksStatusRequestKind::All),
            ..GetLinksOptions::default()
        };
        hdk::get_links_count_with_options(&base, LinkMatch::Exactly("authored_simple_posts"), LinkMatch::Any,options)
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
