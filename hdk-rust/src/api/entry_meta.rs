use meta::ZomeDefinition;
use holochain_core_types::entry::entry_type::EntryType;
use error::{ZomeApiResult, ZomeApiError};
use holochain_json_api::json::JsonString;

#[allow(improper_ctypes)]
extern "C" {
    fn zome_setup(zd: &mut ZomeDefinition);
}

// Returns the metadata defined with an entry type
// This is most useful to expose the metadata to bridging zomes
pub fn entry_meta(name: &EntryType) -> ZomeApiResult<JsonString> {
    let mut zd = ZomeDefinition::new();
    unsafe { zome_setup(&mut zd) };

    let entry_def = zd.entry_types.iter().find(|elem| {
    	&elem.name == name
    });

    entry_def
    .map(|entry_def| {
    	entry_def.entry_type_definition.meta.clone()
    })
    .ok_or(ZomeApiError::Internal("No matching entry type in this zome".into()))
}
