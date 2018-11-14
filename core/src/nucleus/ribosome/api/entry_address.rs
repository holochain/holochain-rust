use holochain_core_types::{
    self,
    cas::content::AddressableContent,
    entry::{Entry, SerializedEntry},
    entry_type::EntryType,
};
use holochain_dna::Dna;
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

/// ZomeApiFunction::entry_address function code
/// args: [0] encoded MemoryAllocation as u32
/// Expected complex argument: entry_type_name and entry_value as JsonString
/// Returns an HcApiReturnCode as I32
pub fn invoke_entry_address(runtime: &mut Runtime, args: &RuntimeArgs) -> ZomeApiResult {
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

#[cfg(test)]
pub mod tests {
    extern crate test_utils;
    extern crate wabt;

    use holochain_core_types::{
        cas::content::Address,
        entry::{test_entry, SerializedEntry},
        error::ZomeApiInternalResult,
        json::JsonString,
    };
    use nucleus::ribosome::{
        api::{tests::test_zome_api_function, ZomeApiFunction},
        Defn,
    };

    /// dummy commit args from standard test entry
    pub fn test_hash_entry_args_bytes() -> Vec<u8> {
        let entry = test_entry();

        let serialized_entry = SerializedEntry::from(entry);
        JsonString::from(serialized_entry).into_bytes()
    }

    #[test]
    /// test that we can round trip bytes through a commit action and get the result from WASM
    fn test_hash_entry_round_trip() {
        let (call_result, _) = test_zome_api_function(
            ZomeApiFunction::HashEntry.as_str(),
            test_hash_entry_args_bytes(),
        );

        assert_eq!(
            call_result,
            JsonString::from(
                String::from(JsonString::from(ZomeApiInternalResult::success(
                    Address::from("QmeoLRiWhXLTQKEAHxd8s6Yt3KktYULatGoMsaXi62e5zT")
                ))) + "\u{0}"
            ),
        );
    }

}
