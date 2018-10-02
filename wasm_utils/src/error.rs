use self::RibosomeReturnCode::*;
use std::fmt;

// Macro for creating a RibosomeReturnCode as a RuntimeValue Result-Option on the spot
#[macro_export]
macro_rules! ribosome_return_code {
    ($s:ident) => {
        Ok(Some(RuntimeValue::I32(RibosomeReturnCode::$s as i32)))
    };
}

// Macro for creating a RibosomeErrorReport on the spot with file!() and line!()
#[macro_export]
macro_rules! ribosome_error_report {
    ($s:expr) => {
        RibosomeErrorReport {
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

/// Enum of all possible return codes that a Zome API Function could return.
/// Represents an encoded allocation of zero length with the return code as offset.
/// @see SinglePageAllocation
#[repr(u32)]
#[derive(Debug, PartialEq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum RibosomeReturnCode {
    Success                         = 0,
    Failure                         = 1 << 16,
    ArgumentDeserializationFailed   = 2 << 16,
    OutOfMemory                     = 3 << 16,
    ReceivedWrongActionResult       = 4 << 16,
    CallbackFailed                  = 5 << 16,
    RecursiveCallForbidden          = 6 << 16,
    ResponseSerializationFailed     = 7 << 16,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl ToString for RibosomeReturnCode {
    fn to_string(&self) -> String {
        match self {
            Success                         => "Success",
            Failure                         => "Failure",
            ArgumentDeserializationFailed   => "Argument deserialization failed",
            OutOfMemory                     => "Out of memory",
            ReceivedWrongActionResult       => "Received wrong action result",
            CallbackFailed                  => "Callback failed",
            RecursiveCallForbidden          => "Recursive call forbidden",
            ResponseSerializationFailed     => "Response serialization failed",
        }.to_string()
    }
}

impl RibosomeReturnCode {
    pub fn from_offset(offset: u16) -> RibosomeReturnCode {
        match offset {
            // @TODO what is a success error?
            // @see https://github.com/holochain/holochain-rust/issues/181
            0 => Success,
            2 => ArgumentDeserializationFailed,
            3 => OutOfMemory,
            4 => ReceivedWrongActionResult,
            5 => CallbackFailed,
            6 => RecursiveCallForbidden,
            7 => ResponseSerializationFailed,
            1 | _ => Failure,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn hc_api_return_code_round_trip() {
        let oom = RibosomeReturnCode::from_offset(
            ((RibosomeReturnCode::OutOfMemory as u32) >> 16) as u16,
        );
        assert_eq!(RibosomeReturnCode::OutOfMemory, oom);
        assert_eq!(RibosomeReturnCode::OutOfMemory.to_string(), oom.to_string());
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
            ribosome_error_report!(description).to_string()
        );
    }
}
