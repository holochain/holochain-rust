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

#[derive(Deserialize, Serialize, Default)]
struct CommitOutputStruct {
  hash: String,
}

/// Commit an entry on source chain and broadcast to dht if entry is public
fn hc_commit(data: *mut c_char, entry_type_name: &str, entry_content : &str)
  -> Result<String, &'static str>
{
  // change args to struct & serialize data
  let input = CommitInputStruct {
    entry_type_name: entry_type_name.to_string(),
    entry_content: entry_content.to_string(),
  };
  let data_size =  serialize(data, input);

  // Call WASMI-able commit
  let mut result_code = 0;
  unsafe {
    result_code = commit(data_size);
  }
  // Exit if error
  if result_code != 0  {
    return Ok(result_code.to_string())
  }

  // Deserialize complex result stored in memory

  // Hardcode test
//  let mut x : Vec<c_char> = vec![123, 34, 104, 97, 115, 104, 34, 58, 34, 81, 109, 88, 121, 90, 34, 125];
//  x.push(0);
//  // let mut x : Vec<c_char> = vec![123, 34, 104, 97, 115, 104, 34, 58, 34, 112, 111, 115, 116, 34, 125];
//  let slice = x.as_mut_slice();
//  let ptr = slice.as_mut_ptr();
//  let output : CommitOutputStruct = deserialize(ptr);

  let output : CommitOutputStruct = deserialize(data);

  // Return hash
  Ok(output.hash.to_string())
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
/// data : pointer to memory buffer to use
/// _params_len : size of parameters used in memory buffer
/// returns length of returned data (in number of bytes)
#[no_mangle]
pub extern "C" fn test_dispatch(data: *mut c_char, _params_len: usize) -> i32 {
    //let _input : InputStruct = deserialize(data);
    let output = test(data);
    return serialize(data, output);
}


/// Actual test function code
fn test(data: *mut c_char) -> CommitOutputStruct
{
  // Call Commit API function
  let hash = hc_commit(data, "post", "hello");

  // Return result in complex format
  if let Ok(hash_str) = hash {
    CommitOutputStruct {
      hash: hash_str,
    }
  }
  else
  {
    CommitOutputStruct {
      hash: "fail".to_string(),
    }
  }
}
