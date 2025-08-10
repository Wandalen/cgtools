//! Command-line interface and REPL functionality using unilang framework.

#[ cfg( feature = "cli-basic" ) ]
pub mod private;

#[ cfg( feature = "cli-basic" ) ]
pub use private::*;