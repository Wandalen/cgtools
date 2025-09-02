//! This module contains tests for the `mdmath_core` module, specifically focusing on vector operations and arithmetic.
// #[ allow( unused_imports ) ]
use super::*;

mod assumptions;
#[ cfg( feature = "arithmetics" ) ]
mod arithmetics;
mod plain_test;
mod vector_test;
