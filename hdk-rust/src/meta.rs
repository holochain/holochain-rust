use entry_definition::ValidatingEntryType;
use globals::G_MEM_STACK;
use holochain_dna::zome::capabilities::Capability;
use holochain_wasm_utils::{
    api_serialization::validation::EntryValidationArgs,
    holochain_core_types::error::RibosomeErrorCode,
    memory_serialization::{load_json, load_string, store_string_into_encoded_allocation},
};
use serde_json;
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
    if maybe_name.is_err() {
        return ::holochain_wasm_utils::holochain_core_types::error::RibosomeErrorCode::ArgumentDeserializationFailed
            as u32;
    }
    let name: String = maybe_name.unwrap();

    match zd
        .entry_types
        .into_iter()
        .find(|ref entry_type| entry_type.name == name)
    {
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
    if maybe_name.is_err() {
        return ::holochain_wasm_utils::holochain_core_types::error::RibosomeErrorCode::ArgumentDeserializationFailed
            as u32;
    }
    let entry_validation_args: EntryValidationArgs = maybe_name.unwrap();

    match zd
        .entry_types
        .into_iter()
        .find(|ref entry_type| entry_type.name == entry_validation_args.entry_type)
    {
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

#[no_mangle]
pub extern "C" fn __hdk_get_json_definition(encoded_allocation_of_input: u32) -> u32 {
    ::global_fns::init_global_memory(encoded_allocation_of_input);

    let mut zd = ZomeDefinition::new();
    unsafe {
        zome_setup(&mut zd);
    }

    let mut entry_types = HashMap::new();
    for entry_type in zd.entry_types {
        entry_types.insert(entry_type.name, entry_type.entry_type_definition);
    }

    let capabilities = unsafe { __list_capabilities() };

    let json_string = serde_json::to_string(&json!({
        "entry_types": entry_types,
        "capabilities": capabilities,
    })).expect("Can't serialize DNA");

    unsafe { store_string_into_encoded_allocation(&mut G_MEM_STACK.unwrap(), &json_string) as u32 }
}

#[cfg(test)]
pub mod tests {
    use holochain_dna::zome::capabilities::Capability;
    use std::collections::HashMap;

    // Adding empty zome_setup() so that the cfg(test) build can link.
    #[no_mangle]
    pub fn zome_setup(_: &mut super::ZomeDefinition) {}
    #[no_mangle]
    fn __list_capabilities() -> HashMap<String, Capability> {
        HashMap::new()
    }
}
