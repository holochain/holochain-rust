use crate::memory_allocation::{
 SinglePageAllocation, WasmStack,
};
use holochain_core_types::{
    error::{CoreError, HolochainError, RibosomeErrorCode, RibosomeReturnCode},
    json::JsonString,
};
use serde::Deserialize;
use serde_json;
use holochain_core_types::bits_n_pieces::U16_MAX;
use std::{convert::TryInto, ffi::CStr, os::raw::c_char, slice};
