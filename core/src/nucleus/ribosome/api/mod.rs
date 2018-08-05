pub mod commit;
pub mod get;
pub mod debug;

use nucleus::ribosome::Runtime;
use wasmi::RuntimeArgs;
use wasmi::Trap;
use std::str::FromStr;
use wasmi::RuntimeValue;
use num_traits::FromPrimitive;
use nucleus::ribosome::Defn;
use nucleus::ribosome::api::debug::invoke_debug;
use nucleus::ribosome::api::commit::invoke_commit;
use nucleus::ribosome::api::get::invoke_get;


// Zome API functions are exposed by HC to zome logic

//--------------------------------------------------------------------------------------------------
// ZOME API FUNCTION DEFINITIONS
//--------------------------------------------------------------------------------------------------

/// Enumeration of all Zome functions known and used by HC Core
/// Enumeration converts to str
#[repr(usize)]
#[derive(FromPrimitive)]
pub enum ZomeAPIFunction {
    /// Error index for unimplemented functions
    MissingNo = 0,

    /// Zome API

    /// send debug information to the log
    /// debug(s : String)
    Debug,

    /// Commit an entry to source chain
    /// commit(entry_type : String, entry_content : String) -> Hash
    Commit,

    /// Get an entry from source chain by key (header hash)
    /// get(key: String) -> Pair
    Get,
}

impl Defn for ZomeAPIFunction {

    fn as_str(&self) -> &'static str {
        match *self {
            ZomeAPIFunction::MissingNo => "",
            ZomeAPIFunction::Debug => "debug",
            ZomeAPIFunction::Commit => "commit",
            ZomeAPIFunction::Get => "get",
        }
    }

    fn str_index(s: &str) -> usize {
        match ZomeAPIFunction::from_str(s) {
            Ok(i) => i as usize,
            Err(_) => ZomeAPIFunction::MissingNo as usize,
        }
    }

    fn from_index(i: usize) -> Self {
        match FromPrimitive::from_usize(i) {
            Some(v) => v,
            None => ZomeAPIFunction::MissingNo,
        }
    }

}

impl FromStr for ZomeAPIFunction {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(ZomeAPIFunction::Debug),
            "commit" => Ok(ZomeAPIFunction::Commit),
            "get" => Ok(ZomeAPIFunction::Get),
            _ => Err("Cannot convert string to ZomeAPIFunction"),
        }
    }
}

impl ZomeAPIFunction {

    pub fn as_fn(&self) -> (fn(&mut Runtime, &RuntimeArgs) -> Result<Option<RuntimeValue>, Trap>) {
        /// does nothing, escape hatch so the compiler can enforce exhaustive matching below
        fn noop(
            _runtime: &mut Runtime,
            _args: &RuntimeArgs,
        ) -> Result<Option<RuntimeValue>, Trap> {
            Ok(Some(RuntimeValue::I32(0 as i32)))
        }

        match *self {
            ZomeAPIFunction::MissingNo => noop,
            ZomeAPIFunction::Debug => invoke_debug,
            ZomeAPIFunction::Commit => invoke_commit,
            ZomeAPIFunction::Get => invoke_get,
        }
    }

}
