extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate libc;

use serde::{Deserialize, Serialize};
use std::{ffi::CStr, os::raw::c_char, slice};

extern {
  // fn commit(data: *mut c_char, params_len: usize) -> *mut c_char;
  fn commit(params_len: i32) -> i32;
}


//-------------------------------------------------------------------------------------------------
// HC API funcs
//-------------------------------------------------------------------------------------------------

#[derive(Serialize, Default)]
struct CommitInputStruct {
  entry_type_name: String,
  entry_content: String,
}

#[derive(Deserialize, Default)]
struct CommitOutputStruct {
  hash: String,
}

/// Commit an entry on source chain and broadcast to dht if entry is public
fn hc_commit(entry_type_name: &str, entry_content : &str) -> Result<String, &'static str>
{
  // change args to struct & serialize data
  // data: *mut c_char;
  data: &[u8];
  let input = CommitInputStruct {entry_type_name, entry_content};
  let data_size =  serialize(data, input);

  // Write data in wasm memory
  // FIXME
  mem: *mut c_char = 0;
  let mem8 = unsafe { slice::from_raw_parts_mut(mem, data_size) };
  for (i, byte) in data.iter().enumerate() {
    mem8[i] = *byte as i8;
  }

  // Call WASMI-able commit
  let bin_result = commit(data_size);

  // Un-WASM result and return
  //let str_result = deserialize(bin_result);
  //str_result

  Ok("0")
}


//-------------------------------------------------------------------------------------------------
// Utils
//-------------------------------------------------------------------------------------------------

// Convert json input into something meaningful
fn deserialize<'s, T: Deserialize<'s>>(data: *mut c_char) -> T {
    let c_str = unsafe { CStr::from_ptr(data) };
    let actual_str = c_str.to_str().unwrap();
    serde_json::from_str(actual_str).unwrap()
}

// Convert output data struct into json as memory buffer
fn serialize<T: Serialize>(data: *mut c_char, internal: T) -> i32 {
    let json = serde_json::to_string(&internal).unwrap();
    let bytes = json.as_bytes();
    let len = bytes.len();
    let mem = unsafe { slice::from_raw_parts_mut(data, len) };

    for (i, byte) in bytes.iter().enumerate() {
        mem[i] = *byte as i8;
    }

    len as i32
}


//-------------------------------------------------------------------------------------------------
// Zome API
//-------------------------------------------------------------------------------------------------

/// Function called by Instance
/// data : parameters received as JSON string?
/// _params_len : ???
/// returns length of returned data (in number of bytes)
#[no_mangle]
pub extern "C" fn test_dispatch(data: *mut c_char, _params_len: usize) -> i32 {
    let _input : InputStruct = deserialize(data);
    let output = test();
    return serialize(data, output);
}

// Input and Output Structures

#[derive(Deserialize, Default)]
struct InputStruct {
}

#[derive(Serialize, Default)]
struct OutputStruct {
    hash: String,
}

// Actual test function code
fn test() -> OutputStruct {
  let mut hash = "".to_string();

  unsafe {
     hash = hc_commit("post", "{content:\"hello\"}");
  };
  //let hash = "QmXyZ";
    OutputStruct {
      hash: hash,
    }
}
