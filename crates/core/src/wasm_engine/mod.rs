// pub mod callback;
pub mod runtime;
pub use self::{runtime::*};
pub mod callback;
pub use holochain_wasmer_host::*;

pub const MAX_ZOME_CALLS: usize = 10;
