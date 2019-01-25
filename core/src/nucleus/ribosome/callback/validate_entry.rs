extern crate serde_json;
use crate::{
    context::Context,
    nucleus::{
        ribosome::{
            self,
            callback::{links_utils, CallbackResult},
        },
        ZomeFnCall,
    },
};
use holochain_core_types::{
    dna::wasm::DnaWasm,
    entry::{
        entry_type::{AppEntryType, EntryType},
        Entry,
    },
    error::HolochainError,
    validation::ValidationData,
};
use holochain_wasm_utils::api_serialization::validation::{
    EntryValidationArgs, LinkValidationArgs,
};
use std::sync::Arc;

/// This function determines and runs the appropriate validation callback for the given entry
/// with the given validation data (which includes the validation package).
/// It returns a CallbackResult which would be
/// * CallbackResult::Pass when the entry is valid
/// * CallbackResult::Fail(message) when the entry is invalid, giving the fail string from the
///         validation callback
/// * CallbackResult::NotImplemented if a validation callback is not implemented for the given
///         entry's type.
pub fn validate_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    match entry.entry_type() {
        // DNA entries are not validated currently and always valid
        // TODO: Specify when DNA can be commited as an update and how to implement validation of DNA entries then.
        EntryType::Dna => Ok(CallbackResult::Pass),

        EntryType::App(app_entry_type) => Ok(validate_app_entry(
            entry.clone(),
            app_entry_type.clone(),
            validation_data,
            context,
        )?),

        EntryType::LinkAdd => Ok(validate_link_entry(
            entry.clone(),
            validation_data,
            context,
        )?),

        // Deletion entries are not validated currently and always valid
        // TODO: Specify how Deletion can be commited to chain.
        EntryType::Deletion => Ok(CallbackResult::Pass),

        // a grant should always be private, so it should always pass
        EntryType::CapTokenGrant => Ok(CallbackResult::Pass),

        // TODO: actually check agent against app specific membrane validation rule
        // like for instance: validate_agent_id(
        //                      entry.clone(),
        //                      validation_data,
        //                      context,
        //                    )?
        EntryType::AgentId => Ok(CallbackResult::Pass),

        _ => Ok(CallbackResult::NotImplemented("validate_entry".into())),
    }
}

fn validate_link_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    let link_add = match entry {
        Entry::LinkAdd(link_add) => link_add,
        _ => {
            return Err(HolochainError::ValidationFailed(
                "Could not extract link_add from entry".into(),
            ));
        }
    };
    let link = link_add.link().clone();
    let (base, target) = links_utils::get_link_entries(&link, &context)?;
    let link_definition_path = links_utils::find_link_definition_in_dna(
        &base.entry_type(),
        link.tag(),
        &target.entry_type(),
        &context,
    )
    .map_err(|_| HolochainError::NotImplemented("validate_link_entry".into()))?;

    let wasm = context
        .get_wasm(&link_definition_path.zome_name)
        .expect("Couldn't get WASM for zome");

    let params = LinkValidationArgs {
        entry_type: link_definition_path.entry_type_name,
        link,
        direction: link_definition_path.direction,
        validation_data,
    };
    let call = ZomeFnCall::new(
        &link_definition_path.zome_name,
        None,
        "__hdk_validate_link",
        params,
    );
    Ok(run_validation_callback(
        context.clone(),
        call,
        &wasm,
        context.get_dna().unwrap().name.clone(),
    ))
}

fn validate_app_entry(
    entry: Entry,
    app_entry_type: AppEntryType,
    validation_data: ValidationData,
    context: Arc<Context>,
) -> Result<CallbackResult, HolochainError> {
    let dna = context.get_dna().expect("Callback called without DNA set!");
    let zome_name = dna.get_zome_name_for_app_entry_type(&app_entry_type);
    if zome_name.is_none() {
        return Ok(CallbackResult::NotImplemented(
            "validate_app_entry/1".into(),
        ));
    }

    let zome_name = zome_name.unwrap();
    match context.get_wasm(&zome_name) {
        Some(wasm) => {
            let validation_call = build_validation_call(
                entry,
                EntryType::App(app_entry_type),
                zome_name,
                validation_data,
            )?;
            Ok(run_validation_callback(
                context.clone(),
                validation_call,
                &wasm,
                dna.name.clone(),
            ))
        }
        None => Ok(CallbackResult::NotImplemented(
            "validate_app_entry/2".into(),
        )),
    }
}

fn build_validation_call(
    entry: Entry,
    entry_type: EntryType,
    zome_name: String,
    validation_data: ValidationData,
) -> Result<ZomeFnCall, HolochainError> {
    let params = EntryValidationArgs {
        entry_type,
        entry,
        validation_data,
    };

    Ok(ZomeFnCall::new(
        &zome_name,
        None,
        "__hdk_validate_app_entry",
        params,
    ))
}

fn run_validation_callback(
    context: Arc<Context>,
    fc: ZomeFnCall,
    wasm: &DnaWasm,
    dna_name: String,
) -> CallbackResult {
    match ribosome::run_dna(
        &dna_name,
        context,
        wasm.code.clone(),
        &fc,
        Some(fc.clone().parameters.into_bytes()),
    ) {
        Ok(call_result) => match call_result.is_null() {
            true => CallbackResult::Pass,
            false => CallbackResult::Fail(call_result.to_string()),
        },
        // TODO: have "not matching schema" be its own error
        Err(HolochainError::RibosomeFailed(error_string)) => {
            if error_string == "Argument deserialization failed" {
                CallbackResult::Fail(String::from("JSON object does not match entry schema"))
            } else {
                CallbackResult::Fail(error_string)
            }
        }
        Err(error) => CallbackResult::Fail(error.to_string()),
    }
}
