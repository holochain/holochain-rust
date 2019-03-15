use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        ribosome::callback::links_utils,
        validation::{ValidationError, ValidationResult},
        CallbackFnCall,
    },
};
use holochain_core_types::{
    cas::content::AddressableContent,
    entry::Entry,
    validation::{LinkValidationData, ValidationData},
};
use holochain_wasm_utils::api_serialization::validation::LinkValidationArgs;
use std::sync::Arc;

pub async fn validate_link_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    let address = entry.address().clone();
    let link = match entry.clone() {
        Entry::LinkAdd(link_add) => link_add.clone(),
        Entry::LinkRemove(link_remove) => link_remove,
        _ => {
            return Err(ValidationError::Error(
                "Could not extract link_add from entry".into(),
            ));
        }
    };
    let link = link.link().clone();
    let (base, target) = links_utils::get_link_entries(&link, context).map_err(|_| {
        ValidationError::UnresolvedDependencies(
            [link.base().clone(), link.target().clone()].to_vec(),
        )
    })?;

    let link_definition_path = links_utils::find_link_definition_in_dna(
        &base.entry_type(),
        link.tag(),
        &target.entry_type(),
        context,
    )
    .map_err(|_| ValidationError::NotImplemented)?;

    let validation_data = match entry.clone() {
        Entry::LinkAdd(link) => Ok(LinkValidationData::LinkAdd {
            link,
            validation_package: validation_data.package.clone(),
        }),
        Entry::LinkRemove(link) => Ok(LinkValidationData::LinkRemove {
            link,
            validation_package: validation_data.package.clone(),
        }),
        _ => Err(ValidationError::Fail("Entry is not link".to_string())),
    }?;

    let params = LinkValidationArgs {
        entry_type: link_definition_path.entry_type_name,
        link,
        direction: link_definition_path.direction,
        validation_data,
    };
    let call = CallbackFnCall::new(
        &link_definition_path.zome_name,
        "__hdk_validate_link",
        params,
    );

    await!(run_validation_callback(address, call, context))
}
