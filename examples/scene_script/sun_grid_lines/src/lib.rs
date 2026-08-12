//! Library surface of the sun-grid-lines WebGPU demo: the native-testable
//! [`scene`] configuration loader and [`shader_source`] assembler, exposed
//! as a library so their tests can live in `tests/`, per the
//! all-tests-in-tests/ convention.

pub mod scene;
pub mod shader_source;
