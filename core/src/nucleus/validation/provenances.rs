use crate::{
    nucleus::{
        state::{ValidationError, ValidationResult},
    },
};
use boolinator::Boolinator;
use holochain_core_types::{
    validation::ValidationData,
};
use holochain_dpki::keypair::Keypair;
use holochain_sodium::secbuf::SecBuf;

pub fn validate_provenances(validation_data: &ValidationData) -> ValidationResult {
    let header = &validation_data.package.chain_header;
    header
        .provenances()
        .iter()
        .map(|provenance| {
            let author = &provenance.0;
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
            let result = Keypair::verify(author.to_string(), &mut signature_buf, &mut message_buf)
                .map_err(|e| ValidationError::Error(e.to_string()))?;

            (result == 0).ok_or(ValidationError::Fail(format!(
                "Signature of entry {} from author {} invalid",
                header.entry_address(),
                author,
            )))
        })
        .collect::<Result<Vec<()>, ValidationError>>()?;
    Ok(())
}
