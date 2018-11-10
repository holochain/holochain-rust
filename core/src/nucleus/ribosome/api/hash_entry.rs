use holochain_core_types::{
    self,
    cas::content::AddressableContent,
    entry::{dna::Dna, Entry, SerializedEntry},
    entry_type::EntryType,
};
use nucleus::ribosome::{api::ZomeApiResult, Runtime};
use std::{convert::TryFrom, str::FromStr};
use wasmi::{RuntimeArgs, RuntimeValue};

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
pub fn invoke_hash_entry(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
    // deserialize args
    let args_str = runtime.load_json_string_from_args(&args);
    let serialized_entry = match SerializedEntry::try_from(args_str) {
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
    let maybe_entry_type = get_entry_type(&dna, &serialized_entry.entry_type());
    if let Err(err) = maybe_entry_type {
        return Ok(err);
    }
    let entry = Entry::from(serialized_entry);

    // Return result
    runtime.store_result(Ok(entry.address()))
}
