use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        CallbackFnCall,
    },
    workflows::callback::links_utils,
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::{
    entry::Entry,
    validation::{ValidationResult},
    validation::{LinkValidationData, ValidationData},
};

use holochain_persistence_api::cas::content::AddressableContent;

use holochain_wasm_types::validation::{LinkDirection, LinkValidationArgs};
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn validate_link_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    let address = entry.address();
    let link = match entry.clone() {
        Entry::LinkAdd(link_add) => link_add.clone(),
        Entry::LinkRemove((link_remove, _)) => link_remove,
        _ => {
            return ValidationResult::Fail(
                "Could not extract link_add from entry".into(),
            );
        }
    };
    let link = link.link().clone();
    let (base, target) = match links_utils::get_link_entries(&link, context) {
        Ok(v) => v,
        Err(_) => {
            return ValidationResult::UnresolvedDependencies(
                [link.base().clone(), link.target().clone()].to_vec(),
            );
        },
    };

    let link_definition_path = match links_utils::find_link_definition_by_type(link.link_type(), context) {
        Ok(v) => v,
        Err(_) => return ValidationResult::NotImplemented,
    };

    let dna = context
        .state()
        .expect("There has to be a state in the context when using it for validation")
        .nucleus()
        .dna()
        .expect("There has to be a DNA in the nucleus when trying to validate entries");

    let entry_def = dna
        .get_entry_type_def(&link_definition_path.entry_type_name)
        .expect("This is Some === find_link_definition_by_type is correct");

    let (base_type, target_type) = if link_definition_path.direction == LinkDirection::To {
        let link_def = entry_def
            .links_to
            .iter()
            .find(|link| link.link_type == link_definition_path.link_type)
            .expect("This is Some === find_link_definition_by_type is correct");
        (
            link_definition_path.entry_type_name.clone(),
            link_def.target_type.clone(),
        )
    } else {
        let link_def = entry_def
            .linked_from
            .iter()
            .find(|link| link.link_type == link_definition_path.link_type)
            .expect("This is Some === find_link_definition_by_type is correct");
        (
            link_def.base_type.clone(),
            link_definition_path.entry_type_name.clone(),
        )
    };

    if base.entry_type().to_string() != base_type {
        return ValidationResult::Fail(format!(
            "Wrong base type for link of type '{}'. Found '{}', but link is defined to link from '{}'s.",
            link.link_type(),
            base_type,
            base.entry_type().to_string(),
        ));
    };

    if target.entry_type().to_string() != target_type {
        return ValidationResult::Fail(format!(
            "Wrong target type for link of type '{}'. Found '{}', but link is defined to link to '{}'s.",
            link.link_type(),
            target_type,
            target.entry_type().to_string(),
        ));
    };

    let validation_data = match entry.clone() {
        Entry::LinkAdd(link) => LinkValidationData::LinkAdd {
            link,
            validation_data,
        },
        Entry::LinkRemove((link, _)) => LinkValidationData::LinkRemove {
            link,
            validation_data,
        },
        _ => return ValidationResult::Fail("Entry is not link".to_string()),
    };

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

    run_validation_callback(address, call, context).await
}
