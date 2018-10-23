use self::{RibosomeErrorCode::*, RibosomeReturnCode::*};
use holochain_core_types::json::JsonString;
use serde_json;
use std::fmt;

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
            let res = store_as_json(&mut $stack, error_report);
            return res.unwrap().encode();
        }
    };
}

/// Macro for creating a RibosomeErrorCode as a RuntimeValue Result-Option on the spot
#[macro_export]
macro_rules! ribosome_error_code {
    ($s:ident) => {
        Ok(Some(RuntimeValue::I32(
            ::holochain_wasm_utils::error::RibosomeErrorCode::$s as i32,
        )))
    };
}

/// Macro for creating a RibosomeErrorReport on the spot with file!() and line!()
#[macro_export]
macro_rules! ribosome_error_report {
    ($s:expr) => {
        ::holochain_wasm_utils::error::RibosomeErrorReport {
            description: $s.to_string(),
            file_name: file!().to_string(),
            line: line!().to_string(),
        }
    };
}

#[derive(Deserialize, Serialize)]
pub struct RibosomeErrorReport {
    pub description: String,
    pub file_name: String,
    pub line: String,
    // TODO #395 - Add advance error debugging info
    // pub stack_trace: Backtrace
}

impl fmt::Display for RibosomeErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Ribosome error: {}\n  --> {}:{}\n",
            self.description, self.file_name, self.line,
        )
    }
}

impl From<RibosomeErrorReport> for String {
    fn from(ribosome_error_report: RibosomeErrorReport) -> String {
        ribosome_error_report.to_string()
    }
}

impl From<JsonString> for RibosomeErrorReport {
    fn from(json_string: JsonString) -> RibosomeErrorReport {
        serde_json::from_str(&String::from(json_string))
            .expect("could not deserialize RibosomeErrorReport")
    }
}

/// Enum of all possible RETURN codes that a Zome API Function could return.
/// Represents an encoded allocation of zero length with the return code as offset.
/// @see SinglePageAllocation
#[repr(u32)]
#[derive(Clone, Debug, PartialEq)]
pub enum RibosomeReturnCode {
    Success,
    Failure(RibosomeErrorCode),
}

/// Enum of all possible ERROR codes that a Zome API Function could return.
#[repr(u32)]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum RibosomeErrorCode {
    Unspecified                     = 1 << 16,
    ArgumentDeserializationFailed   = 2 << 16,
    OutOfMemory                     = 3 << 16,
    ReceivedWrongActionResult       = 4 << 16,
    CallbackFailed                  = 5 << 16,
    RecursiveCallForbidden          = 6 << 16,
    ResponseSerializationFailed     = 7 << 16,
    NotAnAllocation                 = 8 << 16,
    ZeroSizedAllocation             = 9 << 16,
}

impl ToString for RibosomeReturnCode {
    fn to_string(&self) -> String {
        match self {
            Success => "Success".to_string(),
            Failure(code) => code.to_string(),
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl ToString for RibosomeErrorCode {
    fn to_string(&self) -> String {
        match self {
            Unspecified                     => "Unspecified",
            ArgumentDeserializationFailed   => "Argument deserialization failed",
            OutOfMemory                     => "Out of memory",
            ReceivedWrongActionResult       => "Received wrong action result",
            CallbackFailed                  => "Callback failed",
            RecursiveCallForbidden          => "Recursive call forbidden",
            ResponseSerializationFailed     => "Response serialization failed",
            NotAnAllocation                 => "Not an allocation",
            ZeroSizedAllocation             => "Zero-sized allocation",
        }.to_string()
    }
}

impl RibosomeReturnCode {
    pub fn from_error(err_code: RibosomeErrorCode) -> Self {
        Failure(err_code)
    }

    pub fn from_offset(offset: u16) -> Self {
        match offset {
            0 => Success,
            _ => Failure(RibosomeErrorCode::from_offset(offset)),
        }
    }
}

impl RibosomeErrorCode {
    pub fn from_offset(offset: u16) -> Self {
        match offset {
            0 => unreachable!(),
            2 => ArgumentDeserializationFailed,
            3 => OutOfMemory,
            4 => ReceivedWrongActionResult,
            5 => CallbackFailed,
            6 => RecursiveCallForbidden,
            7 => ResponseSerializationFailed,
            8 => NotAnAllocation,
            9 => ZeroSizedAllocation,
            1 | _ => Unspecified,
        }
    }

    pub fn from_return_code(ret_code: RibosomeReturnCode) -> Self {
        match ret_code {
            Success => unreachable!(),
            Failure(rib_err) => rib_err,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn ribosome_return_code_round_trip() {
        let oom =
            RibosomeReturnCode::from_offset(((RibosomeErrorCode::OutOfMemory as u32) >> 16) as u16);
        assert_eq!(Failure(RibosomeErrorCode::OutOfMemory), oom);
        assert_eq!(RibosomeErrorCode::OutOfMemory.to_string(), oom.to_string());
    }

    #[test]
    fn ribosome_error_code_round_trip() {
        let oom =
            RibosomeErrorCode::from_offset(((RibosomeErrorCode::OutOfMemory as u32) >> 16) as u16);
        assert_eq!(RibosomeErrorCode::OutOfMemory, oom);
        assert_eq!(RibosomeErrorCode::OutOfMemory.to_string(), oom.to_string());
    }

    #[test]
    fn ribosome_error_report_to_string() {
        let description = "This is a unit test error description";
        let report = RibosomeErrorReport {
            description: description.to_string(),
            file_name: file!().to_string(),
            line: line!().to_string(),
        };

        assert_ne!(
            report.to_string(),
            RibosomeErrorReport {
                description: description.to_string(),
                file_name: file!().to_string(),
                line: line!().to_string(),
            }.to_string(),
        );
    }
}
