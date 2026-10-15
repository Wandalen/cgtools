//! Tests for `ndarray_tools` — runs the full `ndarray_cg` suite against the alias's re-exports.


use ndarray_tools as the_module;

#[ path = "../../../math/ndarray_cg/tests/inc/mod.rs" ]
mod inc;
