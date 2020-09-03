use crate::{
    context::Context,
    nucleus::{
        actions::run_validation_callback::run_validation_callback,
        validation::{ValidationContext, ValidationError, ValidationResult},
        CallbackFnCall,
    },
    wasm_engine::callback::links_utils,
};
use boolinator::*;
use holochain_core_types::{
    entry::Entry,
    validation::{LinkValidationData, ValidationData},
};

use holochain_persistence_api::cas::content::AddressableContent;

use holochain_wasm_utils::api_serialization::validation::{LinkDirection, LinkValidationArgs};
use std::sync::Arc;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub async fn validate_link_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
    validation_context: ValidationContext,
) -> ValidationResult {
    let address = entry.address();
    let link = match entry.clone() {
        Entry::LinkAdd(link_add) => link_add.clone(),
        Entry::LinkRemove((link_remove, links_to_remove)) => {
            // if we are validating for holding check to make sure that original add links
            // are already being held
            if let ValidationContext::Holding = validation_context {
                for link in links_to_remove.iter() {
                    debug!(
                        "checking if holding AddLink being requested to remove: {:?}",
                        link
                    );
                    let maybe_entry_with_meta =
                        crate::nucleus::actions::get_entry::get_entry_with_meta(
                            context,
                            link.clone(),
                        )
                        .map_err(|e| {
                            ValidationError::Error(
                                format!("Could not lookup LinkAdd locally: {}", e).into(),
                            )
                        })?;
                    if maybe_entry_with_meta.is_none() {
                        return Err(ValidationError::UnresolvedDependencies(
                            [link.clone()].to_vec(),
                        ));
                    }
                }
            }
            link_remove // return the the link to check for its dependencies
        }
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
