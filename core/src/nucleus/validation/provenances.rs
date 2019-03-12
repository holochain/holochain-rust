use crate::nucleus::validation::{ValidationError, ValidationResult};
use boolinator::Boolinator;
use holochain_core_types::validation::ValidationData;
use holochain_dpki::utils::Verify;
use holochain_sodium::{secbuf::SecBuf};

pub fn validate_provenances(validation_data: &ValidationData) -> ValidationResult {
    let header = &validation_data.package.chain_header;
    header
        .provenances()
        .iter()
        .map(|provenance| {
            let maybe_has_authored = provenance.verify(header.entry_address().to_string());
            match maybe_has_authored {
                Err(_) => {
                    Err(ValidationError::Fail(format!(
                        "Signature of entry {} from author {} failed to verify public signing key. Key might be invalid.",
                        header.entry_address(),
                        provenance.0,
                    )))
                },
                Ok(has_authored) => {
                    has_authored.ok_or(ValidationError::Fail(format!(
                        "Signature of entry {} from author {} invalid",
                        header.entry_address(),
                        provenance.0,
                    )))
                },
            }
        })
        .collect::<Result<Vec<()>, ValidationError>>()?;
    Ok(())
}
