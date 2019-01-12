use std::ffi::CStr;
use crate::memory::allocation::WasmAllocation;
use crate::memory::MemoryInt;
use holochain_core_types::error::HolochainError;
use std::os::raw::c_char;
use serde::Deserialize;
use holochain_core_types::error::CoreError;
use holochain_core_types::error::RibosomeErrorCode;

/// reads are always from a WasmAllocation
impl WasmAllocation {

    fn read_str_raw<'a>(ptr_data: *mut c_char) -> &'a str {
        let ptr_safe_c_str = unsafe { CStr::from_ptr(ptr_data) };
        ptr_safe_c_str.to_str().unwrap()
    }

    /// Retrieve a stored string from an allocation.
    /// Return error code if encoded_allocation is invalid.
    pub fn read_to_string(&self) -> String {
        WasmAllocation::read_str_raw(MemoryInt::from(self.offset()) as *mut c_char).to_string()
    }

    pub fn read_to_json<'s, T: Deserialize<'s>>(&self) -> Result<T, HolochainError> {
        let s = WasmAllocation::read_str_raw(MemoryInt::from(self.offset()) as *mut c_char);
        let maybe_obj: Result<T, serde_json::Error> = serde_json::from_str(&s);
        match maybe_obj {
            Ok(obj) => Ok(obj),
            Err(_) => {
                // TODO #394 - In Release, load error_string directly and not a CoreError
                let maybe_hc_err: Result<CoreError, serde_json::Error> =
                    serde_json::from_str(&s);

                Err(match maybe_hc_err {
                    Err(_) => {
                        HolochainError::Ribosome(RibosomeErrorCode::ArgumentDeserializationFailed)
                    }
                    Ok(hc_err) => hc_err.kind,
                })
            }
        }
    }

}
