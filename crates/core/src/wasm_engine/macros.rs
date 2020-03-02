/// Macro for creating a WasmError as a RuntimeValue Result-Option on the spot
/// Will panic! if out or memory or other serialization error occured.
#[macro_export]
macro_rules! zome_assert {
    ($stack:ident, $cond:expr) => {
        if !$cond {
            let error_report =
                core_error_generic!(format!(r#"Zome assertion failed: `{}`"#, stringify!($cond)));
            return return_code_for_allocation_result($stack.write_json(error_report)).into();
        }
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
