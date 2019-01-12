//-------------------------------------------------------------------------------------------------
// Raw
//-------------------------------------------------------------------------------------------------


/// Write in wasm memory according to stack state.
fn write_in_wasm_memory(
    stack: &mut SinglePageStack,
    bytes: &[u8],
    len: u16,
) -> Result<WasmAllocation, RibosomeErrorCode> {
    if u32::from(len) + u32::from(stack.top()) > U16_MAX {
        return Err(RibosomeErrorCode::OutOfMemory);
    }
    let ptr = stack.allocate(len) as *mut c_char;
    let ptr_safe = unsafe { slice::from_raw_parts_mut(ptr, len as usize) };
    for (i, byte) in bytes.iter().enumerate() {
        ptr_safe[i] = *byte as i8;
    }
    SinglePageAllocation::new(ptr as u16, len)
}

//-------------------------------------------------------------------------------------------------
// String
//-------------------------------------------------------------------------------------------------

/// Write a string in wasm memory according to stack state.
pub fn store_string(
    stack: &mut SinglePageStack,
    s: &str,
) -> Result<SinglePageAllocation, RibosomeErrorCode> {
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;
    if len > U16_MAX {
        return Err(RibosomeErrorCode::OutOfMemory);
    }

    write_in_wasm_memory(stack, bytes, len as u16)
}

// Sugar
pub fn store_string_into_encoded_allocation(stack: &mut SinglePageStack, s: &str) -> i32 {
    store_string(stack, s).unwrap().encode() as i32
}

//-------------------------------------------------------------------------------------------------
// JSON
//-------------------------------------------------------------------------------------------------

/// Write a data struct as a json string in wasm memory according to stack state.
pub fn write_json<J: TryInto<JsonString>>(
    stack: &mut SinglePageStack,
    jsonable: J,
) -> Result<WasmAllocation, AllocationError> {
    let j: JsonString = jsonable
        .try_into()
        .map_err(|_| RibosomeErrorCode::ArgumentDeserializationFailed)?;

    let json_bytes = j.into_bytes();
    let json_bytes_len = json_bytes.len() as u32;
    if json_bytes_len > U16_MAX {
        return Err(RibosomeErrorCode::OutOfMemory);
    }
    write_in_wasm_memory(stack, &json_bytes, json_bytes_len as u16)
}
