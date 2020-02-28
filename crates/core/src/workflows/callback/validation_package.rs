use crate::{
    context::Context,
    nucleus::CallbackFnCall,
    wasm_engine::{
        self,
        runtime::WasmCallData,
        callback::CallbackResult,
    },
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::{
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    validation::ValidationPackageDefinition,
    validation::ValidationResult,
};
use crate::workflows::callback::links_utils;
use holochain_wasm_types::validation::LinkValidationPackageArgs;
use std::{sync::Arc};

// @TODO fix line number mangling
// #[autotrace]
#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn get_validation_package_definition(
    entry: &Entry,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    let dna = context.get_dna().expect("Callback called without DNA set!");
    let result: ValidationPackageDefinition = match entry.entry_type() {
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
            let call_data = WasmCallData::new_callback_call(context, call);
            holochain_wasmer_host::guest::call(
                &mut wasm_engine::factories::instance_for_call_data(&call_data)?,
                &call_data.fn_name(),
                app_entry_type,
            )?
        }
        EntryType::LinkAdd => {
            let link_add = match entry {
                Entry::LinkAdd(link_add) => link_add,
                _ => {
                    return Err(HolochainError::ValidationFailed(ValidationResult::Fail(
                        "Failed to extract LinkAdd".into(),
                    )));
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

            let call_data = WasmCallData::new_callback_call(context, call);
            holochain_wasmer_host::guest::call(
                &mut wasm_engine::factories::instance_for_call_data(&call_data)?,
                &call_data.fn_name(),
                call.parameters,
            )?
        }
        EntryType::LinkRemove => {
            let link_remove = match entry {
                Entry::LinkRemove((link_remove, _)) => link_remove,
                _ => {
                    return Err(HolochainError::ValidationFailed(ValidationResult::Fail(
                        "Failed to extract LinkRemove".into(),
                    )));
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

            let call_data = WasmCallData::new_callback_call(context, call);
            holochain_wasmer_host::guest::call(
                &mut wasm_engine::factories::instance_for_call_data(&call_data)?,
                &call_data.fn_name(),
                call.parameters,
            )?
        }
        EntryType::Deletion => ValidationPackageDefinition::ChainFull,
        EntryType::CapTokenGrant => ValidationPackageDefinition::Entry,
        EntryType::AgentId => ValidationPackageDefinition::Entry,
        EntryType::ChainHeader => ValidationPackageDefinition::Entry,
        _ => Err(HolochainError::NotImplemented(
            "get_validation_package_definition/3".into(),
        ))?,
    };

    Ok(CallbackResult::ValidationPackageDefinition(result))
}
