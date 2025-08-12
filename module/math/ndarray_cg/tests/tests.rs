//! This module contains tests for the `ndarray_cg` module, specifically focusing on the `inc` module.
#![ allow( unused_imports ) ]
#![ allow( clippy::should_panic_without_expect ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::clone_on_copy ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::multiple_bound_locations ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::excessive_precision ) ]
#![ allow( clippy::approx_constant ) ]
#![ allow( clippy::let_underscore_untyped ) ]
#![ allow( clippy::redundant_closure_for_method_calls ) ]
#![ allow( clippy::needless_borrows_for_generic_args ) ]

use test_tools::exposed::*;

use ndarray_cg as the_module;

// #[ path = "inc/inc.rs" ]
mod inc;
