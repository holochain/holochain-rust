//! Convenience re-export of common members
//!
//! Like the standard library's prelude, this module simplifies importing of
//! common items. Unlike the standard prelude, the contents of this module must
//! be imported manually:
//!
//! ```
//! use logging::prelude::*;
//! # logging::FastLoggerBuilder::new()
//!     .build_test().unwrap();
//! # // Test if the re-export from the prelude works
//! # assert_eq!(Level::Debug, Level::Debug);
//! # info!("Here we go!");
//! ```


// log macro re-export
pub use log::{log, trace, debug, info, warn, error};
pub use log::Level;
pub use crate::{FastLogger, FastLoggerBuilder, rule::{RuleFilter, RuleFilterBuilder}};
