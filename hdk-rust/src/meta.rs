use entry_definition::ValidatingEntryType;
use globals::G_MEM_STACK;
use holochain_core_types::{
    dna::zome::{capabilities::Capability, entry_types::EntryTypeDef},
    entry::entry_type::{AppEntryType, EntryType},
    error::HolochainError,
    json::JsonString,
};
use holochain_wasm_utils::{
    api_serialization::validation::{
        EntryValidationArgs, LinkValidationArgs, LinkValidationPackageArgs,
    },
    holochain_core_types::error::RibosomeErrorCode,
    memory_serialization::{load_json, load_string, store_string_into_encoded_allocation},
};
use std::collections::HashMap;

trait Ribosome {
    fn define_entry_type(&mut self, name: String, entry_type: ValidatingEntryType);
}

#[allow(improper_ctypes)]
pub struct ZomeDefinition {
    pub entry_types: Vec<ValidatingEntryType>,
}

impl ZomeDefinition {
    fn new() -> ZomeDefinition {
        ZomeDefinition {
            entry_types: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn define(&mut self, entry_type: ValidatingEntryType) {
        self.entry_types.push(entry_type);
    }
}

#[allow(improper_ctypes)]
extern "C" {
    fn zome_setup(zd: &mut ZomeDefinition);
    fn __list_capabilities() -> HashMap<String, Capability>;
}

#[no_mangle]
pub extern "C" fn __hdk_get_validation_package_for_entry_type(
    encoded_allocation_of_input: u32,
) -> u32 {
    ::global_fns::init_global_memory(encoded_allocation_of_input);

    let mut zd = ZomeDefinition::new();
    unsafe {
        zome_setup(&mut zd);
    }

    // Deserialize input
    let maybe_name = load_string(encoded_allocation_of_input);
    if let Err(err_code) = maybe_name {
        return err_code as u32;
    }
    let name: String = maybe_name.unwrap();

    match zd
        .entry_types
        .into_iter()
        .find(|ref validating_entry_type| {
            validating_entry_type.name == EntryType::App(AppEntryType::from(name.clone()))
        }) {
        None => RibosomeErrorCode::CallbackFailed as u32,
        Some(mut entry_type_definition) => {
            let package = (*entry_type_definition.package_creator)();
            ::global_fns::store_and_return_output(package)
        }
    }
}

#[no_mangle]
pub extern "C" fn __hdk_validate_app_entry(encoded_allocation_of_input: u32) -> u32 {
    ::global_fns::init_global_memory(encoded_allocation_of_input);

    let mut zd = ZomeDefinition::new();
    unsafe {
        zome_setup(&mut zd);
    }

    // Deserialize input
    let maybe_name = load_json(encoded_allocation_of_input);
    if let Err(hc_err) = maybe_name {
        return ::global_fns::store_and_return_output(hc_err);
    }
    let entry_validation_args: EntryValidationArgs = maybe_name.unwrap();

    match zd
        .entry_types
        .into_iter()
        .find(|ref validating_entry_type| {
            validating_entry_type.name == entry_validation_args.entry_type
        }) {
        None => RibosomeErrorCode::CallbackFailed as u32,
        Some(mut entry_type_definition) => {
            let validation_result = (*entry_type_definition.validator)(
                entry_validation_args.entry,
                entry_validation_args.validation_data,
            );

            match validation_result {
                Ok(()) => 0,
                Err(fail_string) => ::global_fns::store_and_return_output(fail_string),
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, DefaultJson)]
struct PartialZome {
    entry_types: HashMap<EntryType, EntryTypeDef>,
    capabilities: HashMap<String, Capability>,
}

#[no_mangle]
pub extern "C" fn __hdk_get_validation_package_for_link(encoded_allocation_of_input: u32) -> u32 {
    ::global_fns::init_global_memory(encoded_allocation_of_input);

    let mut zd = ZomeDefinition::new();
    unsafe {
        zome_setup(&mut zd);
    }

    // Deserialize input
    let maybe_name = load_json(encoded_allocation_of_input);
    if let Err(hc_err) = maybe_name {
        return ::global_fns::store_and_return_output(hc_err);
    }
    let link_validation_args: LinkValidationPackageArgs = maybe_name.unwrap();

    zd.entry_types
        .into_iter()
        .find(|ref validation_entry_type| {
            validation_entry_type.name == EntryType::from(link_validation_args.entry_type.clone())
        })
        .and_then(|entry_type| {
            entry_type.links.into_iter().find(|ref link_definition| {
                link_definition.tag == link_validation_args.tag
                    && link_definition.link_type == link_validation_args.direction
            })
        })
        .and_then(|mut link_definition| {
            let package = (*link_definition.package_creator)();
            Some(::global_fns::store_and_return_output(package))
        })
        .unwrap_or(RibosomeErrorCode::CallbackFailed as u32)
}

#[no_mangle]
pub extern "C" fn __hdk_validate_link(encoded_allocation_of_input: u32) -> u32 {
    ::global_fns::init_global_memory(encoded_allocation_of_input);

    let mut zd = ZomeDefinition::new();
    unsafe {
        zome_setup(&mut zd);
    }

    // Deserialize input
    let maybe_name = load_json(encoded_allocation_of_input);
    if let Err(hc_err) = maybe_name {
        return ::global_fns::store_and_return_output(hc_err);
    }
    let link_validation_args: LinkValidationArgs = maybe_name.unwrap();

    zd.entry_types
        .into_iter()
        .find(|ref validation_entry_type| {
            validation_entry_type.name == EntryType::from(link_validation_args.entry_type.clone())
        })
        .and_then(|entry_type_definition| {
            entry_type_definition
                .links
                .into_iter()
                .find(|link_definition| {
                    link_definition.tag == *link_validation_args.link.tag()
                        && link_definition.link_type == link_validation_args.direction
                })
        })
        .and_then(|mut link_definition| {
            let validation_result = (*link_definition.validator)(
                link_validation_args.link.base().clone(),
                link_validation_args.link.target().clone(),
                link_validation_args.validation_data,
            );
            Some(match validation_result {
                Ok(()) => 0,
                Err(fail_string) => ::global_fns::store_and_return_output(fail_string),
            })
        })
        .unwrap_or(RibosomeErrorCode::CallbackFailed as u32)
}

#[no_mangle]
pub extern "C" fn __hdk_get_json_definition(encoded_allocation_of_input: u32) -> u32 {
    ::global_fns::init_global_memory(encoded_allocation_of_input);

    let mut zd = ZomeDefinition::new();
    unsafe {
        zome_setup(&mut zd);
    }

    let mut entry_types = HashMap::new();
    for validating_entry_type in zd.entry_types {
        entry_types.insert(
            validating_entry_type.name,
            validating_entry_type.entry_type_definition,
        );
    }

    let capabilities = unsafe { __list_capabilities() };

    let partial_zome = PartialZome {
        entry_types,
        capabilities,
        // ..Default::default()
    };

    let json_string = JsonString::from(partial_zome);

    unsafe {
        store_string_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), &String::from(json_string))
            as u32
    }
}

#[cfg(test)]
pub mod tests {
    use holochain_core_types::dna::zome::capabilities::Capability;
    use std::collections::HashMap;

    // Adding empty zome_setup() so that the cfg(test) build can link.
    #[no_mangle]
    pub fn zome_setup(_: &mut super::ZomeDefinition) {}
    #[no_mangle]
    pub fn __list_capabilities() -> HashMap<String, Capability> {
        HashMap::new()
    }
}
