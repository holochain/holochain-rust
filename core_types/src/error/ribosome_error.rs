use self::{RibosomeErrorCode::*, RibosomeReturnCode::*};
use std::fmt;

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
    UnknownEntryType                = 10 << 16,
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
            UnknownEntryType                => "Unknown entry type",
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
            10 => UnknownEntryType,
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
