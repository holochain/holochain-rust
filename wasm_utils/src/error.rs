/// Macro for creating a RibosomeErrorCode as a RuntimeValue Result-Option on the spot
/// Will panic! if out or memory or other serialization error occured.
#[macro_export]
macro_rules! zome_assert {
    ($stack:ident, $cond:expr) => {
        if !$cond {
            let error_report = ribosome_error_report!(format!(
                r#"Zome assertion failed: `{}`"#,
                stringify!($cond)
            ));
            let res = store_json(&mut $stack, error_report);
            return res.unwrap().encode();
        }
    };
}

/// Macro for creating a RibosomeErrorCode as a RuntimeValue Result-Option on the spot
#[macro_export]
macro_rules! ribosome_error_code {
    ($s:ident) => {
        Ok(Some(RuntimeValue::I32(
            ::holochain_wasm_utils::holochain_core_types::error::RibosomeErrorCode::$s as i32,
        )))
    };
}

/// Macro for creating a RibosomeErrorReport on the spot with file!() and line!()
#[macro_export]
macro_rules! ribosome_error_report {
    ($s:expr) => {
        ::holochain_wasm_utils::holochain_core_types::error::RibosomeErrorReport {
            description: $s.to_string(),
            file_name: file!().to_string(),
            line: line!().to_string(),
        }
    };
}
