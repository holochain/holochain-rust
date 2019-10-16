//! Convenience re-export of common members
//!
//! Like the standard library's prelude, this module simplifies importing of
//! common items. Unlike the standard prelude, the contents of this module must
//! be imported manually:
//!
//! ```
//! use holochain_logging::prelude::*;
//! # logging::FastLoggerBuilder::new()
//!     .build_test().unwrap();
//! # // Test if the re-export from the prelude works
//! # assert_eq!(Level::Debug, Level::Debug);
//! # info!("Here we go!");
//! ```

// log macro re-export
pub use crate::{
    rule::{RuleFilter, RuleFilterBuilder},
    FastLogger, FastLoggerBuilder,
};
pub use log::{debug, error, info, log, trace, warn, Level};
