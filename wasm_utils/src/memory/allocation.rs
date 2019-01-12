use holochain_core_types::error::RibosomeErrorCode;
use std::ffi::CStr;
use std::os::raw::c_char;
use serde::Deserialize;
use holochain_core_types::error::HolochainError;
use holochain_core_types::error::CoreError;
use holochain_core_types::error::RibosomeMemoryAllocation;
use holochain_core_types::bits_n_pieces::u32_split_bits;
use memory::MemoryBits;
use memory::MemoryIntMax;
use std::convert::TryFrom;
use memory::MemoryInt;

#[derive(Copy, Clone, Debug)]
pub struct Offset(MemoryInt);
#[derive(Copy, Clone, Debug)]
pub struct Length(MemoryInt);

impl From<Offset> for MemoryInt {
    fn from(offset: Offset) -> Self {
        offset.0
    }
}

impl From<Offset> for MemoryBits {
    fn from(offset: Offset) -> Self {
        offset.0 as MemoryBits
    }
}

impl From<MemoryInt> for Offset {
    fn from(i: MemoryInt) -> Self {
        Offset(i)
    }
}

impl From<Length> for MemoryInt {
    fn from(length: Length) -> Self {
        length.0
    }
}

impl From<Length> for MemoryBits {
    fn from(length: Length) -> Self {
        length.0 as MemoryBits
    }
}

impl From<MemoryInt> for Length {
    fn from(i: MemoryInt) -> Self {
        Length(i)
    }
}

pub enum AllocationError {
    OutOfBounds,
    ZeroLength,
}

#[derive(Copy, Clone, Debug)]
pub struct WasmAllocation {
    offset: Offset,
    length: Length,
}

impl WasmAllocation {
    // represent the max as MemoryBits type to allow gt comparisons
    fn max()-> MemoryBits {
        MemoryIntMax
    }

    pub fn new(offset: Offset, length: Length) -> Result<Self, AllocationError> {
        if (MemoryBits::from(offset) + MemoryBits::from(length)) > WasmAllocation::max() {
            Err(AllocationError::OutOfBounds)
        }
        else if MemoryInt::from(length) == 0 {
            Err(AllocationError::ZeroLength)
        }
        else {
            Ok(WasmAllocation { offset, length })
        }
    }

    pub fn offset(self) -> Offset {
        self.offset
    }

    pub fn length(self) -> Length {
        self.length
    }

    /// Retrieve a stored string from an allocation.
    /// Return error code if encoded_allocation is invalid.
    pub fn read_to_string(&self) -> String {
        let ptr_data = MemoryInt::from(self.offset()) as *mut c_char;
        let ptr_safe_c_str = unsafe {CStr::from_ptr(ptr_data) };
        ptr_safe_c_str.to_str().unwrap().to_string()
    }

    pub fn read_to_json<'s, T: Deserialize<'s>>(&self) -> Result<T, HolochainError> {
        let s = self.read_to_string();
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

impl TryFrom<RibosomeMemoryAllocation> for WasmAllocation {
    type Error = AllocationError;
    fn try_from(ribosome_memory_allocation: RibosomeMemoryAllocation) -> Result<Self, Self::Error> {
        let (offset, length) = u32_split_bits(MemoryBits::from(ribosome_memory_allocation));
        WasmAllocation::new(offset.into(), length.into())
    }
}
