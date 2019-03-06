//! This file contains the "secret" functions that get added to Zomes, by the HDK.
//! These functions match expectations that Holochain has... every Zome technically needs these functions,
//! but not every developer should have to write them. A notable function defined here is
//! __hdk_get_json_definition which allows Holochain to retrieve JSON defining the Zome.

use crate::{entry_definition::ValidatingEntryType, globals::G_MEM_STACK};
use holochain_core_types::{
    dna::{
        entry_types::{deserialize_entry_types, serialize_entry_types},
        zome::{ZomeEntryTypes, ZomeFnDeclarations, ZomeTraits},
    },
    entry::entry_type::{AppEntryType, EntryType},
    error::{HolochainError, RibosomeEncodedValue, RibosomeEncodingBits},
    json::JsonString,
    validation::EntryValidationData
};
use holochain_wasm_utils::{
    api_serialization::validation::{
        EntryValidationArgs, LinkValidationArgs, LinkValidationPackageArgs,
    },
    holochain_core_types::error::RibosomeErrorCode,
    memory::{
        allocation::AllocationError,
        ribosome::{load_ribosome_encoded_json, return_code_for_allocation_result},
    },
};
use std::{collections::BTreeMap,convert::TryFrom};

trait Ribosome {
    fn define_entry_type(&mut self, name: String, entry_type: ValidatingEntryType);
}

#[derive(Debug, Serialize, Deserialize, DefaultJson, Default)]
struct PartialZome {
    #[serde(serialize_with = "serialize_entry_types")]
    #[serde(deserialize_with = "deserialize_entry_types")]
    entry_types: ZomeEntryTypes,
    traits: ZomeTraits,
    fn_declarations: ZomeFnDeclarations,
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
    fn __list_traits() -> ZomeTraits;
    fn __list_functions() -> ZomeFnDeclarations;
}

#[no_mangle]
pub extern "C" fn __hdk_get_validation_package_for_entry_type(
    encoded_allocation_of_input: RibosomeEncodingBits,
) -> RibosomeEncodingBits {
    let allocation = match ::global_fns::init_global_memory_from_ribosome_encoding(
        encoded_allocation_of_input,
    ) {
        Ok(allocation) => allocation,
        Err(allocation_error) => return allocation_error.as_ribosome_encoding(),
    };

    let mut zd = ZomeDefinition::new();
    unsafe { zome_setup(&mut zd) };

    let name = allocation.read_to_string();
    

    match zd
        .entry_types
        .into_iter()
        .find(|ref validating_entry_type| {
            validating_entry_type.name == EntryType::App(AppEntryType::from(name.clone()))
        }) {
        None => RibosomeEncodedValue::Failure(RibosomeErrorCode::CallbackFailed).into(),
        Some(mut entry_type_definition) => {
            let package = (*entry_type_definition.package_creator)();
            return_code_for_allocation_result(crate::global_fns::write_json(package)).into()
        }
    }
}

#[no_mangle]
pub extern "C" fn __hdk_validate_app_entry(
    encoded_allocation_of_input: RibosomeEncodingBits,
) -> RibosomeEncodingBits {
    if let Err(allocation_error) =
        ::global_fns::init_global_memory_from_ribosome_encoding(encoded_allocation_of_input)
    {
        return allocation_error.as_ribosome_encoding();
    }

    let mut zd = ZomeDefinition::new();
    unsafe { zome_setup(&mut zd) };

    // Deserialize input
    let input: EntryValidationArgs = match load_ribosome_encoded_json(encoded_allocation_of_input) {
        Ok(v) => v,
        Err(e) => return RibosomeEncodedValue::from(e).into(),
    };

    let entry_type_to_app_entry = match entry_validation_to_app_entry_type(input.validation_data.clone().entry_validation)
    {
        Ok(v) => v,
        Err(e) => return RibosomeEncodedValue::from(e).into()
    };
    
    match zd
        .entry_types
        .into_iter()
        .find(|ref validating_entry_type| validating_entry_type.name == entry_type_to_app_entry)
    {
        None => RibosomeErrorCode::CallbackFailed as RibosomeEncodingBits,
        Some(mut entry_type_definition) => {
            let validation_result =
                (*entry_type_definition.validator)(input.validation_data);

            match validation_result {
                Ok(()) => RibosomeEncodedValue::Success.into(),
                Err(fail_string) => {
                    return_code_for_allocation_result(crate::global_fns::write_json(fail_string))
                        .into()
                }
            }
        }
    }
}

fn entry_validation_to_app_entry_type(entry_validation : EntryValidationData) ->Result<EntryType,HolochainError>
{
    match entry_validation
    {
        EntryValidationData::Create(entry) => {
            Ok(EntryType::App(AppEntryType::try_from(entry.entry_type())?))
        },
        EntryValidationData::Delete(_,entry) => Ok(EntryType::App(AppEntryType::try_from(entry.entry_type())?)),
        EntryValidationData::Modify(latest,_) =>{
            Ok(EntryType::App(AppEntryType::try_from(latest.entry_type())?))
        }
    }
}

#[no_mangle]
pub extern "C" fn __hdk_get_validation_package_for_link(
    encoded_allocation_of_input: RibosomeEncodingBits,
) -> RibosomeEncodingBits {
    if let Err(allocation_error) =
        ::global_fns::init_global_memory_from_ribosome_encoding(encoded_allocation_of_input)
    {
        return allocation_error.as_ribosome_encoding();
    };

    let mut zd = ZomeDefinition::new();
    unsafe { zome_setup(&mut zd) };

    let input: LinkValidationPackageArgs =
        match load_ribosome_encoded_json(encoded_allocation_of_input) {
            Ok(v) => v,
            Err(e) => return RibosomeEncodedValue::from(e).into(),
        };

    RibosomeEncodingBits::from(
        zd.entry_types
            .into_iter()
            .find(|ref validation_entry_type| {
                validation_entry_type.name == EntryType::from(input.entry_type.clone())
            })
            .and_then(|entry_type| {
                entry_type.links.into_iter().find(|ref link_definition| {
                    link_definition.tag == input.tag && link_definition.link_type == input.direction
                })
            })
            .and_then(|mut link_definition| {
                let package = (*link_definition.package_creator)();
                Some(return_code_for_allocation_result(::global_fns::write_json(
                    package,
                )))
            })
            .unwrap_or(RibosomeEncodedValue::Failure(
                RibosomeErrorCode::CallbackFailed,
            )),
    )
}

#[no_mangle]
pub extern "C" fn __hdk_validate_link(
    encoded_allocation_of_input: RibosomeEncodingBits,
) -> RibosomeEncodingBits {
    if let Err(allocation_error) =
        ::global_fns::init_global_memory_from_ribosome_encoding(encoded_allocation_of_input)
    {
        return allocation_error.as_ribosome_encoding();
    };

    let mut zd = ZomeDefinition::new();
    unsafe { zome_setup(&mut zd) }

    let input: LinkValidationArgs = match load_ribosome_encoded_json(encoded_allocation_of_input) {
        Ok(v) => v,
        Err(e) => return RibosomeEncodedValue::from(e).into(),
    };

    RibosomeEncodingBits::from(
        zd.entry_types
            .into_iter()
            .find(|ref validation_entry_type| {
                validation_entry_type.name == EntryType::from(input.entry_type.clone())
            })
            .and_then(|entry_type_definition| {
                entry_type_definition
                    .links
                    .into_iter()
                    .find(|link_definition| {
                        link_definition.tag == *input.link.tag()
                            && link_definition.link_type == input.direction
                    })
            })
            .and_then(|mut link_definition| {
                let validation_result = (*link_definition.validator)(
                    input.link.base().clone(),
                    input.link.target().clone(),
                    input.validation_data,
                );
                Some(match validation_result {
                    Ok(()) => RibosomeEncodedValue::Success,
                    Err(fail_string) => {
                        return_code_for_allocation_result(::global_fns::write_json(fail_string))
                    }
                })
            })
            .unwrap_or(RibosomeEncodedValue::Failure(
                RibosomeErrorCode::CallbackFailed,
            )),
    )
}

#[no_mangle]
pub extern "C" fn __hdk_git_hash(
    encoded_allocation_of_input: RibosomeEncodingBits,
) -> RibosomeEncodingBits {
    if let Err(allocation_error) =
        ::global_fns::init_global_memory_from_ribosome_encoding(encoded_allocation_of_input)
    {
        return allocation_error.as_ribosome_encoding();
    }

    let mut mem_stack = unsafe {
        match G_MEM_STACK {
            Some(mem_stack) => mem_stack,
            None => {
                return AllocationError::BadStackAlignment.as_ribosome_encoding();
            }
        }
    };

    return_code_for_allocation_result(mem_stack.write_string(holochain_core_types::GIT_HASH)).into()
}

#[no_mangle]
pub extern "C" fn __hdk_get_json_definition(
    encoded_allocation_of_input: RibosomeEncodingBits,
) -> RibosomeEncodingBits {
    if let Err(allocation_error) =
        ::global_fns::init_global_memory_from_ribosome_encoding(encoded_allocation_of_input)
    {
        return allocation_error.as_ribosome_encoding();
    }

    let mut zd = ZomeDefinition::new();
    unsafe { zome_setup(&mut zd) };

    let mut entry_types = BTreeMap::new();
    for validating_entry_type in zd.entry_types {
        entry_types.insert(
            validating_entry_type.name,
            validating_entry_type.entry_type_definition,
        );
    }

    let traits = unsafe { __list_traits() };
    let fn_declarations = unsafe { __list_functions() };

    let partial_zome = PartialZome {
        entry_types,
        traits,
        fn_declarations,
    };

    let json_string = JsonString::from(partial_zome);

    let mut mem_stack = unsafe {
        match G_MEM_STACK {
            Some(mem_stack) => mem_stack,
            None => {
                return AllocationError::BadStackAlignment.as_ribosome_encoding();
            }
        }
    };

    return_code_for_allocation_result(mem_stack.write_string(&String::from(json_string))).into()
}

#[cfg(test)]
pub mod tests {
    use crate as hdk;
    use crate::ValidationPackageDefinition;
    use holochain_core_types::{
        dna::{
            entry_types::Sharing,
            zome::{ZomeFnDeclarations, ZomeTraits},
        },
        error::HolochainError,
        json::JsonString,
    };
    use meta::PartialZome;
    use std::collections::BTreeMap;

    // Adding empty zome_setup() so that the cfg(test) build can link.
    #[no_mangle]
    pub fn zome_setup(_: &mut super::ZomeDefinition) {}

    #[no_mangle]
    pub fn __list_traits() -> ZomeTraits {
        BTreeMap::new()
    }

    #[no_mangle]
    pub fn __list_functions() -> ZomeFnDeclarations {
        Vec::new()
    }

    #[test]
    fn partial_zome_json() {
        #[derive(Serialize, Deserialize, Debug, DefaultJson)]
        pub struct Post {
            content: String,
            date_created: String,
        }

        let mut entry_types = BTreeMap::new();

        let validating_entry_type = entry!(
            name: "post",
            description: "blog entry post",
            sharing: Sharing::Public,
            native_type: Post,

            validation_package: || {
                ValidationPackageDefinition::Entry
            },

            validation: |_post: Post, _validation_data: hdk::ValidationData| {
                Ok(())
            }

        );
        entry_types.insert(
            validating_entry_type.name,
            validating_entry_type.entry_type_definition,
        );

        let partial_zome = PartialZome {
            entry_types,
            ..Default::default()
        };

        assert_eq!(
            JsonString::from(partial_zome),
            JsonString::from("{\"entry_types\":{\"post\":{\"description\":\"blog entry post\",\"sharing\":\"public\",\"links_to\":[],\"linked_from\":[]}},\"traits\":{},\"fn_declarations\":[]}"),
        );
    }
}
