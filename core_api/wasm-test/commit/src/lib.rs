extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
//extern crate libc;

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
fn hc_commit(data: *mut c_char, entry_type_name: &str, entry_content : &str) -> Result<String, &'static str>
{
  // change args to struct & serialize data
  // data: *mut c_char;
  // data: &[u8];
  // data: *mut c_char = 0;

  let input = CommitInputStruct {
    entry_type_name: entry_type_name.to_string(),
    entry_content: entry_content.to_string(),
  };
  let data_size =  serialize(data, input);

  // Write data in wasm memory
//  let mem8 = unsafe { slice::from_raw_parts_mut(data, data_size as usize) };
//  for (i, byte) in data.iter().enumerate() {
//    mem8[i] = *byte as i8;
//  }

  // Call WASMI-able commit
  let mut result_len = 0;
  unsafe {
    result_len = commit(data_size);
  }

  if result_len != 0  {
    // return Ok("fail".to_string())
    return Ok(result_len.to_string())
  }

  // Un-WASMI result
//  let mut bytes = "Test".to_string().into_bytes();
//  bytes.push(b"\0");
//  let cchars = bytes.iter_mut().map(|b| b as c_char);
//  let name: *mut c_char = cchars.as_mut_ptr();

  // let mut x : Vec<c_char> = vec![123, 125, 0];
  let mut x : Vec<c_char> = vec![0];
  let slice = x.as_mut_slice();
  let ptr = slice.as_mut_ptr();
  let output : CommitOutputStruct = deserialize(ptr);

  // let output : CommitOutputStruct = deserialize(data);


  // Return value
  let output = CommitOutputStruct { hash :"QmXyZ".to_string()};
  Ok(output.hash.to_string())

  // Ok("QmXyZ".to_string())
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

// Convert a data struct into json memory buffer
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
    //let _input : InputStruct = deserialize(data);
    let output = test(data);
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
fn test(data: *mut c_char) -> OutputStruct
{
  let hash = hc_commit(data, "post", "{content:\"hello\"}");

  //let hash = "QmXyZ";
  if let Ok(hash_str) = hash {
    OutputStruct {
      hash: hash_str,
    }
  }
  else
  {
    OutputStruct {
      hash: "fail".to_string(),
    }
  }
}
