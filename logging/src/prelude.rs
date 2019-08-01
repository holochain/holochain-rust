//! Convenience re-export of common members
//!
//! Like the standard library's prelude, this module simplifies importing of
//! common items. Unlike the standard prelude, the contents of this module must
//! be imported manually:
//!
//! ```
//! use logging::prelude::*;
//! # logging::FastLoggerBuilder::new().build_test().unwrap();
//! # info!("Here we go!");
//! ```


// log macro re-export
pub use log::{trace, debug, info, warn, error};
pub use crate::{FastLogger, FastLoggerBuilder, rule::{RuleFilter, RuleFilterBuilder}};
