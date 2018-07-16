extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
//extern crate libc;

use serde::{Deserialize, Serialize};
use std::{ffi::CStr, os::raw::c_char, slice};

extern {
  fn commit(mem_offset: i32, mem_len: i32) -> i32;
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
fn hc_commit(ptr_data: *mut c_char, entry_type_name: &str, entry_content : &str)
  -> Result<String, &'static str>
{
  // change args to struct & serialize data
  let input = CommitInputStruct {
    entry_type_name: entry_type_name.to_string(),
    entry_content: entry_content.to_string(),
  };
  let data_size =  serialize(ptr_data, input);

  // Call WASMI-able commit
  let mut result_code = 0;
  unsafe {
    result_code = commit(ptr_data as i32, data_size);
  }
  // Exit if error
  if result_code != 0  {
    return Ok(result_code.to_string())
  }

  // Deserialize complex result stored in memory
  let output : CommitOutputStruct = deserialize(ptr_data);

  // Return hash
  Ok(output.hash.to_string())
}


//-------------------------------------------------------------------------------------------------
// Utils
//-------------------------------------------------------------------------------------------------

// Convert json data in a memory buffer into a meaningful data struct
fn deserialize<'s, T: Deserialize<'s>>(ptr_data: *mut c_char) -> T {
    let ptr_safe_c_str = unsafe { CStr::from_ptr(ptr_data) };
    let actual_str = ptr_safe_c_str.to_str().unwrap();
    serde_json::from_str(actual_str).unwrap()
}

// Write a data struct into a memory buffer as json string
fn serialize<T: Serialize>(ptr_data: *mut c_char, internal: T) -> i32 {
    let json_bytes     = serde_json::to_vec(&internal).unwrap();
    let json_bytes_len = json_bytes.len();
    let ptr_data_safe  = unsafe { slice::from_raw_parts_mut(ptr_data, json_bytes_len) };

    for (i, byte) in json_bytes.iter().enumerate() {
        ptr_data_safe[i] = *byte as i8;
    }

    json_bytes_len as i32
}


//-------------------------------------------------------------------------------------------------
// Zome API
//-------------------------------------------------------------------------------------------------

/// Function called by Instance
/// param_mem : pointer to memory buffer holding complex input parameters
/// params_len : size of complex input parameters in memory buffer
/// returns length of returned data (in number of bytes)
#[no_mangle]
pub extern "C" fn test_dispatch(ptr_data_param: *mut c_char, params_len: usize) -> i32 {
    let ptr_data_commit = params_len as *mut c_char;
    let output = test(ptr_data_commit);
    return serialize(ptr_data_param, output);
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
