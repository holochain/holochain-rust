use crate::nucleus::validation::{ValidationError, ValidationResult};
use boolinator::Boolinator;
use holochain_core_types::validation::ValidationData;
use holochain_dpki;
use holochain_sodium::secbuf::SecBuf;

pub fn validate_provenances(validation_data: &ValidationData) -> ValidationResult {
    let header = &validation_data.package.chain_header;
    header
        .provenances()
        .iter()
        .map(|provenance| {
            let author_id = &provenance.0;
            let signature = &provenance.1;
            let signature_string: String = signature.clone().into();
            let signature_bytes: Vec<u8> = base64::decode(&signature_string).map_err(|_| {
                ValidationError::Fail("Signature syntactically invalid".to_string())
            })?;

            let mut signature_buf = SecBuf::with_insecure(signature_bytes.len());
            signature_buf
                .write(0, signature_bytes.as_slice())
                .expect("SecBuf must be writeable");

            let mut message_buf =
                SecBuf::with_insecure_from_string(header.entry_address().to_string());

            let maybe_has_authored = holochain_dpki::utils::verify(author_id.to_string(), &mut message_buf, &mut signature_buf);
            match maybe_has_authored {
                Err(_) => {
                    Err(ValidationError::Fail(format!(
                        "Signature of entry {} from author {} failed to verify public signing key. Key might be invalid.",
                        header.entry_address(),
                        author_id,
                    )))
                },
                Ok(has_authored) => {
                    has_authored.ok_or(ValidationError::Fail(format!(
                        "Signature of entry {} from author {} invalid",
                        header.entry_address(),
                        author_id,
                    )))
                },
            }
        })
        .collect::<Result<Vec<()>, ValidationError>>()?;
    Ok(())
}
