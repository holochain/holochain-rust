use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    nucleus::ribosome::callback::{self, CallbackResult},
};
use boolinator::Boolinator;
use futures::{
    future::Future,
    task::{LocalWaker, Poll},
};
use holochain_core_types::{
    cas::content::AddressableContent,
    chain_header::ChainHeader,
    entry::{entry_type::EntryType, Entry},
    error::HolochainError,
    hash::HashString,
    validation::ValidationData,
};
use holochain_dpki::keypair::Keypair;
use holochain_sodium::secbuf::SecBuf;
use snowflake::{self, ProcessUniqueId};
use std::{pin::Pin, sync::Arc, thread};

fn check_entry_type(entry_type: EntryType, context: &Arc<Context>) -> Result<(), HolochainError> {
    match entry_type {
        EntryType::App(app_entry_type) => {
            // Check if app_entry_type is defined in DNA
            let _ = context
                .state()
                .unwrap()
                .nucleus()
                .dna()
                .unwrap()
                .get_zome_name_for_app_entry_type(&app_entry_type)
                .ok_or(HolochainError::ValidationFailed(format!(
                    "Attempted to validate unknown app entry type {:?}",
                    app_entry_type,
                )))?;
        }

        EntryType::LinkAdd => {}
        EntryType::LinkRemove => {}
        EntryType::Deletion => {}
        EntryType::CapTokenGrant => {}
        EntryType::AgentId => {}

        _ => {
            return Err(HolochainError::ValidationFailed(format!(
                "Attempted to validate system entry type {:?}",
                entry_type,
            )));
        }
    }

    Ok(())
}

fn spawn_validation_ribosome(
    id: ProcessUniqueId,
    entry: Entry,
    validation_data: ValidationData,
    context: Arc<Context>,
) {
    thread::spawn(move || {
        let address = entry.address();
        let maybe_validation_result = callback::validate_entry::validate_entry(
            entry.clone(),
            validation_data.clone(),
            context.clone(),
        );

        let result = match maybe_validation_result {
            Ok(validation_result) => match validation_result {
                CallbackResult::Fail(error_string) => Err(error_string),
                CallbackResult::Pass => Ok(()),
                CallbackResult::NotImplemented(reason) => Err(format!(
                    "Validation callback not implemented for {:?} ({})",
                    entry.entry_type().clone(),
                    reason
                )),
                _ => unreachable!(),
            },
            Err(error) => Err(error.to_string()),
        };

        context
            .action_channel()
            .send(ActionWrapper::new(Action::ReturnValidationResult((
                (id, address),
                result,
            ))))
            .expect("action channel to be open in reducer");
    });
}

fn validate_provenances(validation_data: &ValidationData) -> Result<(), HolochainError> {
    let header = &validation_data.package.chain_header;
    header
        .provenances()
        .iter()
        .map(|provenance| {
            let author = &provenance.0;
            let signature = &provenance.1;
            let signature_string: String = signature.clone().into();
            let signature_bytes: Vec<u8> = base64::decode(&signature_string).map_err(|_| {
                HolochainError::ValidationFailed("Signature syntactically invalid".to_string())
            })?;

            let mut signature_buf = SecBuf::with_insecure(signature_bytes.len());
            signature_buf
                .write(0, signature_bytes.as_slice())
                .expect("SecBuf must be writeable");

            let mut message_buf =
                SecBuf::with_insecure_from_string(header.entry_address().to_string());
            let result = Keypair::verify(author.to_string(), &mut signature_buf, &mut message_buf)?;

            (result == 0).ok_or(HolochainError::ValidationFailed(format!(
                "Signature of entry {} from author {} invalid",
                header.entry_address(),
                author,
            )))
        })
        .collect::<Result<Vec<()>, HolochainError>>()?;
    Ok(())
}

fn validate_header_address(entry: &Entry, header: &ChainHeader) -> Result<(), HolochainError> {
    (entry.address() == *header.entry_address()).ok_or(HolochainError::ValidationFailed(
        "Wrong header for entry".to_string(),
    ))
}

/// ValidateEntry Action Creator
/// This is the high-level validate function that wraps the whole validation process and is what should
/// be called from zome api functions and other contexts that don't care about implementation details.
///
/// 1. Checks if the entry type is either an app entry type defined in the DNA or a system entry
///    type that should be validated. Bails early if not.
/// 2. Checks if the entry's address matches the address in given header provided by
///    the validation package.
/// 3. Validates provenances given in the header by verifying the cryptographic signatures
///    against the source agent addresses.
/// 4. Finally spawns a thread to run the custom validation callback in a Ribosome.
///
/// Returns a future that resolves to an Ok(ActionWrapper) or an Err(error_message:String).
pub async fn validate_entry(
    entry: Entry,
    validation_data: ValidationData,
    context: &Arc<Context>,
) -> Result<HashString, HolochainError> {
    let id = snowflake::ProcessUniqueId::new();
    let address = entry.address();

    check_entry_type(entry.entry_type(), context)?;
    validate_header_address(&entry, &validation_data.package.chain_header)?;
    validate_provenances(&validation_data)?;
    spawn_validation_ribosome(id.clone(), entry, validation_data, context.clone());

    await!(ValidationFuture {
        context: context.clone(),
        key: (id, address),
    })
}

/// ValidationFuture resolves to an Ok(ActionWrapper) or an Err(error_message:String).
/// Tracks the state for ValidationResults.
pub struct ValidationFuture {
    context: Arc<Context>,
    key: (snowflake::ProcessUniqueId, HashString),
}

impl Future for ValidationFuture {
    type Output = Result<HashString, HolochainError>;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        lw.wake();
        if let Some(state) = self.context.state() {
            match state.nucleus().validation_results.get(&self.key) {
                Some(Ok(())) => Poll::Ready(Ok(self.key.1.clone())),
                Some(Err(e)) => Poll::Ready(Err(HolochainError::ValidationFailed(e.clone()))),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}
