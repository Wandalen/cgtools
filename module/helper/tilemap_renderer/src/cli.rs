//! Command-line interface and REPL functionality using unilang framework.
//!
//! This module provides a comprehensive CLI with interactive REPL mode for
//! scene creation, rendering, and management. Built on the unilang framework
//! with enhanced REPL features including command history and auto-completion.

#[ cfg( feature = "cli-basic" ) ]
pub mod private;

#[ cfg( feature = "cli-basic" ) ]
pub use private::*;