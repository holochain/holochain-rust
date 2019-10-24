use error::{ZomeApiError, ZomeApiResult};
use holochain_core_types::entry::entry_type::EntryType;
use holochain_json_api::json::JsonString;
use meta::ZomeDefinition;

#[allow(improper_ctypes)]
extern "C" {
    fn zome_setup(zd: &mut ZomeDefinition);
}

// Returns the properties defined with an entry type
// It is encouraged to using JSON to encode structured properties
// with an entry
pub fn entry_type_properties(name: &EntryType) -> ZomeApiResult<JsonString> {
    let mut zd = ZomeDefinition::new();
    unsafe { zome_setup(&mut zd) };

    let entry_def = zd.entry_types.iter().find(|elem| &elem.name == name);

    entry_def
        .map(|entry_def| entry_def.entry_type_definition.properties.clone())
        .ok_or_else(|| ZomeApiError::Internal("No matching entry type in this zome".into()))
}
