use holochain_core_types::{self, entry::Entry, entry_type::EntryType, hash::HashString};
use holochain_dna::Dna;
use holochain_wasm_utils::api_serialization::HashEntryArgs;
use multihash::Hash as Multihash;
use nucleus::ribosome::Runtime;
use serde_json;
use std::str::FromStr;
use wasmi::{RuntimeArgs, RuntimeValue, Trap};

pub fn get_entry_type(dna: &Dna, entry_type_name: &str) -> Result<EntryType, Option<RuntimeValue>> {
    let entry_type = EntryType::from_str(&entry_type_name).map_err(|_| {
        Some(RuntimeValue::I32(
            holochain_core_types::error::RibosomeErrorCode::UnknownEntryType as i32,
        ))
    })?;
    // Check if AppEntry is a valid AppEntryType
    if entry_type.is_app() {
        let result = dna.get_entry_type_def(entry_type_name);
        if result.is_none() {
            return Err(Some(RuntimeValue::I32(
                holochain_core_types::error::RibosomeErrorCode::UnknownEntryType as i32,
            )));
        }
    }
    // Done
    Ok(entry_type)
}

/// ZomeApiFunction::hash_entry function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: entry_type_name and entry_value as JsonString
/// Returns an HcApiReturnCode as I32
pub fn invoke_hash_entry(
    runtime: &mut Runtime,
    args: &RuntimeArgs,
) -> Result<Option<RuntimeValue>, Trap> {
    // deserialize args
    let args_str = runtime.load_utf8_from_args(&args);
    let input: HashEntryArgs = match serde_json::from_str(&args_str) {
        Ok(input) => input,
        Err(_) => return ribosome_error_code!(ArgumentDeserializationFailed),
    };
    // Check if entry_type is valid
    let dna = runtime
        .context
        .state()
        .unwrap()
        .nucleus()
        .dna()
        .expect("Should have DNA");
    let maybe_entry_type = get_entry_type(&dna, &input.entry_type_name);
    if let Err(err) = maybe_entry_type {
        return Ok(err);
    }
    let entry_type = maybe_entry_type.unwrap();
    let entry = Entry::new(&entry_type, &input.entry_value);
    // Perform hash
    let hash = HashString::encode_from_serializable(&entry, Multihash::SHA2256);
    // Return result
    runtime.store_utf8(&String::from(hash))
}
