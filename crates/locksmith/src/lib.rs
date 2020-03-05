#[cfg(not(feature = "bypass"))]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[cfg(not(feature = "bypass"))]
mod locksmith;
#[cfg(not(feature = "bypass"))]
pub use locksmith::*;

#[cfg(feature = "bypass")]
mod bypass;
#[cfg(feature = "bypass")]
pub use bypass::*;

mod error;
pub use crate::error::LocksmithError;
