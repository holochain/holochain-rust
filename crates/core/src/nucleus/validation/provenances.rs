use crate::{
    NEW_RELIC_LICENSE_KEY,
};
use holochain_core_types::validation::{ValidationResult};
use holochain_core_types::validation::ValidationData;
use holochain_dpki::utils::Verify;

#[holochain_tracing_macros::newrelic_autotrace(HOLOCHAIN_CORE)]
pub fn validate_provenances(validation_data: &ValidationData) -> ValidationResult {
    let header = &validation_data.package.chain_header;
    match header
        .provenances()
        .iter()
        .map(|provenance| {
            let maybe_has_authored = provenance.verify(header.entry_address().to_string());
            match maybe_has_authored {
                Err(_) => {
                    Err(ValidationResult::Fail(format!(
                        "Signature of entry {} from author {} failed to verify public signing key. Key might be invalid.",
                        header.entry_address(),
                        provenance.source(),
                    )))
                },
                Ok(has_authored) => {
                    if has_authored {
                        Ok(())
                    } else {
                        Err(ValidationResult::Fail(format!(
                            "Signature of entry {} from author {} invalid",
                            header.entry_address(),
                            provenance.source(),
                        )))
                    }
                },
            }
        })
        .collect::<Result<Vec<()>, ValidationResult>>() {
            Ok(_) => ValidationResult::Ok,
            Err(v) => v,
    }
}
