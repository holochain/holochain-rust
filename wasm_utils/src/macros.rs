/// Macro for creating a RibosomeErrorCode as a RuntimeValue Result-Option on the spot
/// Will panic! if out or memory or other serialization error occured.
#[macro_export]
macro_rules! zome_assert {
    ($stack:ident, $cond:expr) => {
        if !$cond {
            let error_report =
                core_error_generic!(format!(r#"Zome assertion failed: `{}`"#, stringify!($cond)));
            let res = stack.store_as_json(error_report);
            return res.unwrap().encode();
        }
    };
}

#[macro_export]
macro_rules! ribosome_success {
    () => {
        Ok(Some(RuntimeValue::I32(0 as i32)))
    };
}

/// Macro for creating a RibosomeErrorCode as a RuntimeValue Result-Option on the spot
#[macro_export]
macro_rules! ribosome_error_code {
    ($s:ident) => {
        Ok(Some(RuntimeValue::I32(
            $crate::holochain_core_types::error::RibosomeErrorCode::$s as i32,
        )))
    };
}

/// Macro for creating a CoreError from a HolochainError on the spot with file!() and line!()
#[macro_export]
macro_rules! core_error {
    ($hc_err:expr) => {
        $crate::holochain_core_types::error::CoreError {
            kind: $hc_err,
            file: file!().to_string(),
            line: line!().to_string(),
        }
    };
}

/// Macro for creating a generic CoreError on the spot with file!() and line!()
#[macro_export]
macro_rules! core_error_generic {
    ($msg:expr) => {
        $crate::holochain_core_types::error::CoreError {
            kind: $crate::holochain_core_types::error::HolochainError::ErrorGeneric($msg),
            file: file!().to_string(),
            line: line!().to_string(),
        }
    };
}
