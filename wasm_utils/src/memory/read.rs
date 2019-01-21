use crate::memory::{allocation::WasmAllocation, MemoryInt};
use std::{ffi::CStr, os::raw::c_char};

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
}

#[cfg(test)]
pub mod tests {

    use memory::{
        allocation::{Length, Offset, WasmAllocation},
        MemoryInt,
    };
    use std::{ffi::CString, os::raw::c_char};

    #[test]
    pub fn read_str_raw_test() {
        let s = "foo";
        let c_string = CString::new(s).expect("could not create cstring");
        let ptr = c_string.as_ptr();

        assert_eq!(s, WasmAllocation::read_str_raw(ptr as *mut c_char),);
    }

    // #[test]
    // pub fn read_to_string() {
    //     let s = "foo";
    //     let c_string = CString::new(s).expect("could not create cstring");
    //     let ptr = c_string.as_ptr();
    //
    //     println!("{:?}", ptr);
    //
    //     let allocation = WasmAllocation::new(
    //         Offset::from(ptr as MemoryInt),
    //         Length::from(1),
    //     )
    //     .expect("could not build allocation");
    //
    //     assert_eq!(s.to_string(), allocation.read_to_string(),);
    // }

}
