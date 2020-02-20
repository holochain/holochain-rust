//! This file contains the "secret" functions that get added to Zomes, by the HDK.  These functions match expectations that Holochain has... every Zome technically needs these functions,
//! but not every developer should have to write them. A notable function defined here is
//! __hdk_get_json_definition which allows Holochain to retrieve JSON defining the Zome.

use crate::entry_definition::{AgentValidator, ValidatingEntryType};
use holochain_core_types::{
    dna::{
        entry_types::{deserialize_entry_types, serialize_entry_types},
        zome::{ZomeEntryTypes, ZomeFnDeclarations, ZomeTraits},
    },
    entry::entry_type::{AppEntryType, EntryType},
};
use holochain_json_derive::DefaultJson;
use serde_derive::{Deserialize, Serialize};

use holochain_json_api::{
    error::JsonError,
    json::{JsonString, RawString},
};

use holochain_wasm_utils::{
    api_serialization::validation::{
        AgentIdValidationArgs, EntryValidationArgs, LinkValidationArgs, LinkValidationPackageArgs,
    },
};
use holochain_wasmer_guest::*;
use std::{collections::BTreeMap, convert::TryFrom};

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
#[derive(Default)]
pub struct ZomeDefinition {
    pub entry_types: Vec<ValidatingEntryType>,
    pub agent_entry_validator: Option<AgentValidator>,
}

impl ZomeDefinition {
    pub fn new() -> ZomeDefinition {
        ZomeDefinition::default()
    }

    #[allow(dead_code)]
    pub fn define(&mut self, entry_type: ValidatingEntryType) {
        self.entry_types.push(entry_type);
    }

    pub fn define_agent_validator(&mut self, agent_validator: AgentValidator) {
        self.agent_entry_validator = Some(agent_validator);
    }
}

#[allow(improper_ctypes)]
extern "C" {
    fn zome_setup(zd: &mut ZomeDefinition);
    fn __list_traits() -> ZomeTraits;
    fn __list_functions() -> ZomeFnDeclarations;

    // memory stuff
    fn __import_allocation(guest_allocation_ptr: AllocationPtr, host_allocation_ptr: AllocationPtr);
    fn __import_bytes(host_allocation_ptr: AllocationPtr, guest_bytes_ptr: Ptr);
}

fn zome_definition() -> ZomeDefinition {
    let mut zd = ZomeDefinition::new();
    unsafe { zome_setup(&mut zd) };
    zd
}

#[no_mangle]
pub extern "C" fn __hdk_get_validation_package_for_entry_type(
    host_allocation_ptr: AllocationPtr,
) -> AllocationPtr {
    let name = host_string!(host_allocation_ptr);

    match zome_definition()
        .entry_types
        .into_iter()
        .find(|ref validating_entry_type| {
            validating_entry_type.name == EntryType::App(AppEntryType::from(name.clone()))
        }) {
        Some(mut entry_type_definition) => ret!(WasmResult::Ok(
            (*entry_type_definition.package_creator)().into()
        )),
        None => ret!(WasmResult::Err(WasmError::CallbackFailed)),
    };
}

#[no_mangle]
pub extern "C" fn __hdk_validate_app_entry(host_allocation_ptr: AllocationPtr) -> AllocationPtr {
    // Deserialize input
    let input: EntryValidationArgs = host_args!(host_allocation_ptr);

    let entry_type = try_result!(
        EntryType::try_from(input.validation_data.clone()),
        "Failed to deserialize EntryType"
    );

    match zome_definition()
        .entry_types
        .into_iter()
        .find(|ref validating_entry_type| validating_entry_type.name == entry_type)
    {
        None => ret!(WasmResult::Err(WasmError::CallbackFailed)),
        Some(mut entry_type_definition) => {
            match (*entry_type_definition.validator)(input.validation_data) {
                Ok(()) => ret!(WasmResult::Ok(().into())),
                Err(fail_string) => ret!(WasmResult::Err(WasmError::Zome(fail_string.into()))),
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn __hdk_validate_agent_entry(host_allocation_ptr: AllocationPtr) -> AllocationPtr {
    let input: AgentIdValidationArgs = host_args!(host_allocation_ptr);

    //get the validator code
    let mut validator = try_result!(
        (zome_definition().agent_entry_validator).ok_or(()),
        "No agent validation callback registered for zome."
    );

    ret!(WasmResult::Ok((*validator)(input.validation_data).into()));
}

#[no_mangle]
pub extern "C" fn __hdk_get_validation_package_for_link(
    host_allocation_ptr: AllocationPtr,
) -> AllocationPtr {
    let input: LinkValidationPackageArgs = host_args!(host_allocation_ptr);

    ret!(WasmResult::Ok(
        zome_definition()
            .entry_types
            .into_iter()
            .find(|ref validation_entry_type| {
                validation_entry_type.name == EntryType::from(input.entry_type.clone())
            })
            .and_then(|entry_type| {
                entry_type.links.into_iter().find(|ref link_definition| {
                    link_definition.link_type == input.link_type
                        && link_definition.direction == input.direction
                })
            })
            .and_then(|mut link_definition| { Some((*link_definition.package_creator)()) })
            .into()
    ));
}

#[no_mangle]
pub extern "C" fn __hdk_validate_link(host_allocation_ptr: AllocationPtr) -> AllocationPtr {
    let input: LinkValidationArgs = host_args!(host_allocation_ptr);

    ret!(zome_definition()
        .entry_types
        .into_iter()
        .find(|ref validation_entry_type| {
            validation_entry_type.name == EntryType::from(input.entry_type.clone())
        })
        .and_then(|entry_type_definition| {
            entry_type_definition
                .links
                .into_iter()
                .find(|link_definition| {
                    link_definition.link_type == *input.link.link_type()
                        && link_definition.direction == input.direction
                })
        })
        .and_then(|mut link_definition| {
            let validation_result = (*link_definition.validator)(input.validation_data);
            Some(match validation_result {
                Ok(()) => WasmResult::Ok(().into()),
                Err(fail_string) => WasmResult::Err(WasmError::Zome(fail_string)),
            })
        })
        .unwrap_or(WasmResult::Err(WasmError::CallbackFailed))
    );
}

#[no_mangle]
pub extern "C" fn __hdk_hdk_version(_: AllocationPtr) -> AllocationPtr {
    ret!(WasmResult::Ok(RawString::from(
        holochain_core_types::hdk_version::HDK_VERSION.to_string()
    ).into()))
}

#[no_mangle]
pub extern "C" fn __hdk_get_json_definition(_: AllocationPtr) -> AllocationPtr {
    let mut entry_types = BTreeMap::new();
    for validating_entry_type in zome_definition().entry_types {
        entry_types.insert(
            validating_entry_type.name,
            validating_entry_type.entry_type_definition,
        );
    }

    let traits = unsafe { __list_traits() };
    let fn_declarations = unsafe { __list_functions() };

    ret!(WasmResult::Ok(PartialZome {
        entry_types,
        traits,
        fn_declarations,
    }.into()));
}

#[cfg(test)]
pub mod tests {
    use crate::{meta::PartialZome, prelude::*, ValidationPackageDefinition};
    use holochain_core_types::dna::{
        entry_types::Sharing,
        zome::{ZomeFnDeclarations, ZomeTraits},
    };
    use holochain_json_api::{error::JsonError, json::JsonString};
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
        #[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
        pub struct Post {
            content: String,
            date_created: String,
        }

        let mut entry_types = BTreeMap::new();

        let validating_entry_type = entry!(
            name: "post",
            description: "{\"description\": \"blog entry post\"}",
            sharing: Sharing::Public,


            validation_package: || {
                ValidationPackageDefinition::Entry
            },

            validation: |_validation_data: hdk::EntryValidationData<Post>| {
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
            JsonString::from_json("{\"entry_types\":{\"post\":{\"properties\":\"{\\\"description\\\": \\\"blog entry post\\\"}\",\"sharing\":\"public\",\"links_to\":[],\"linked_from\":[]}},\"traits\":{},\"fn_declarations\":[]}"),
        );
    }
}
