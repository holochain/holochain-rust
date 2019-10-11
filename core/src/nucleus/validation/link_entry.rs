use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        ribosome::callback::links_utils,
        validation::{ValidationError, ValidationResult},
        CallbackFnCall,
    },
};
use boolinator::*;
use holochain_core_types::{
    entry::Entry,
    validation::{LinkValidationData, ValidationData},
};

use holochain_persistence_api::cas::content::AddressableContent;

use holochain_wasm_utils::api_serialization::validation::{LinkDirection, LinkValidationArgs};
use std::sync::Arc;

pub async fn validate_link_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> ValidationResult {
    let address = entry.address().clone();
    let link = match entry.clone() {
        Entry::LinkAdd(link_add) => link_add.clone(),
        Entry::LinkRemove((link_remove, _)) => link_remove,
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

    let link_definition_path = links_utils::find_link_definition_by_type(link.link_type(), context)
        .map_err(|_| ValidationError::NotImplemented)?;

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

    (base.entry_type().to_string() == base_type)
        .ok_or(ValidationError::Fail(format!(
            "Wrong base type for link of type '{}'. Found '{}', but link is defined to link from '{}'s.",
            link.link_type(),
            base_type,
            base.entry_type().to_string(),
        )))?;

    (target.entry_type().to_string() == target_type)
        .ok_or(ValidationError::Fail(format!(
            "Wrong target type for link of type '{}'. Found '{}', but link is defined to link to '{}'s.",
            link.link_type(),
            target_type,
            target.entry_type().to_string(),
        )))?;

    let validation_data = match entry.clone() {
        Entry::LinkAdd(link) => Ok(LinkValidationData::LinkAdd {
            link,
            validation_data,
        }),
        Entry::LinkRemove((link, _)) => Ok(LinkValidationData::LinkRemove {
            link,
            validation_data,
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

    run_validation_callback(address, call, context).await
}
