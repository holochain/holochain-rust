use crate::{
    context::Context,
    nucleus::{
        ribosome::{
            self,
            callback::{links_utils, CallbackResult},
            runtime::WasmCallData,
        },
        CallbackFnCall,
    },
};
use holochain_core_types::{
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    validation::ValidationPackageDefinition,
};

use holochain_json_api::json::JsonString;

use holochain_wasm_utils::api_serialization::validation::LinkValidationPackageArgs;
use std::{convert::TryFrom, sync::Arc};

#[cfg(not(target_arch = "wasm32"))]
#[flame]
pub fn get_validation_package_definition(
    entry: &Entry,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    let dna = context.get_dna().expect("Callback called without DNA set!");
    let result = match entry.entry_type().clone() {
        EntryType::App(app_entry_type) => {
            let zome_name = dna.get_zome_name_for_app_entry_type(&app_entry_type);
            if zome_name.is_none() {
                return Ok(CallbackResult::NotImplemented(
                    "get_validation_package_definition/1".into(),
                ));
            }

            let call = CallbackFnCall::new(
                zome_name
                    .as_ref()
                    .expect("No zome_name in get_validation_package_defintion"),
                "__hdk_get_validation_package_for_entry_type",
                app_entry_type.clone(),
            );
            ribosome::run_dna(
                Some(app_entry_type.to_string().into_bytes()),
                WasmCallData::new_callback_call(context, call),
            )?
        }
        EntryType::LinkAdd => {
            let link_add = match entry {
                Entry::LinkAdd(link_add) => link_add,
                _ => {
                    return Err(HolochainError::ValidationFailed(
                        "Failed to extract LinkAdd".into(),
                    ));
                }
            };

            let link_definition_path =
                links_utils::find_link_definition_by_type(link_add.link().link_type(), &context)?;

            let params = LinkValidationPackageArgs {
                entry_type: link_definition_path.entry_type_name,
                link_type: link_definition_path.link_type,
                direction: link_definition_path.direction,
            };

            let call = CallbackFnCall::new(
                &link_definition_path.zome_name,
                "__hdk_get_validation_package_for_link",
                params,
            );

            ribosome::run_dna(
                Some(call.parameters.to_bytes()),
                WasmCallData::new_callback_call(context.clone(), call),
            )?
        }
        EntryType::LinkRemove => {
            let link_remove = match entry {
                Entry::LinkRemove((link_remove, _)) => link_remove,
                _ => {
                    return Err(HolochainError::ValidationFailed(
                        "Failed to extract LinkRemove".into(),
                    ));
                }
            };

            let link_definition_path = links_utils::find_link_definition_by_type(
                link_remove.link().link_type(),
                &context,
            )?;

            let params = LinkValidationPackageArgs {
                entry_type: link_definition_path.entry_type_name,
                link_type: link_definition_path.link_type,
                direction: link_definition_path.direction,
            };

            let call = CallbackFnCall::new(
                &link_definition_path.zome_name,
                "__hdk_get_validation_package_for_link",
                params,
            );

            ribosome::run_dna(
                Some(call.parameters.to_bytes()),
                WasmCallData::new_callback_call(context.clone(), call),
            )?
        }
        EntryType::Deletion => JsonString::from(ValidationPackageDefinition::ChainFull),
        EntryType::CapTokenGrant => JsonString::from(ValidationPackageDefinition::Entry),
        EntryType::AgentId => JsonString::from(ValidationPackageDefinition::Entry),
        EntryType::ChainHeader => JsonString::from(ValidationPackageDefinition::Entry),
        _ => Err(HolochainError::NotImplemented(
            "get_validation_package_definition/3".into(),
        ))?,
    };

    if result.is_null() {
        Err(HolochainError::SerializationError(String::from(
            "__hdk_get_validation_package_for_entry_type returned empty result",
        )))
    } else {
        match ValidationPackageDefinition::try_from(result) {
            Ok(package) => Ok(CallbackResult::ValidationPackageDefinition(package)),
            Err(_) => Err(HolochainError::SerializationError(String::from(
                "validation_package result could not be deserialized as ValidationPackage",
            ))),
        }
    }
}
