//! Jewelry 3D configurator library
//!
//! This library provides WebGL-based 3D rendering and configuration
//! for jewelry items including rings, gems, and materials.
//!
//! # Public API
//!
//! The intended public API surface consists of:
//! - [`configurator`] - Main jewelry configuration interface
//! - [`ui`] - JavaScript interop layer (WASM-exported functions)
//!
//! # Internal Modules
//!
//! Other modules are exposed with `pub` visibility for integration testing
//! but are **not part of the stable public API**. They are marked with
//! `#[doc(hidden)]` to exclude them from generated documentation.
//!
//! **Do not use internal modules directly** - they may change without notice.
//! Use the public [`configurator`] and [`ui`] modules instead.
//!
//! # WASM Exports
//!
//! When compiled as WebAssembly (cdylib), only functions annotated with
//! `#[wasm_bindgen]` in the [`ui`] module are exported to JavaScript.
//! No other module functions are accessible from JS.

#![ allow( missing_docs ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::empty_docs ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::redundant_static_lifetimes ) ]
#![ allow( clippy::used_underscore_binding ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::clone_on_copy ) ]
#![ allow( clippy::ref_option ) ]
#![ allow( clippy::unnecessary_cast ) ]
#![ allow( clippy::assigning_clones ) ]
#![ allow( clippy::for_kv_map ) ]
#![ allow( clippy::useless_format ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::needless_bool ) ]
#![ allow( clippy::unnecessary_wraps ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::let_and_return ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::field_reassign_with_default ) ]
#![ allow( clippy::if_not_else ) ]

// Public API modules (always exposed)
pub mod configurator;
pub mod ui;

// Internal modules - conditionally public for testing
// In production builds (cdylib for WASM), these are crate-private to prevent unintended API exposure
// In test builds, they're public to enable integration tests
#[ cfg_attr( not( test ), doc( hidden ) ) ]
pub mod uniform_utils;
#[ cfg_attr( not( test ), doc( hidden ) ) ]
pub mod cube_normal_map_generator;
#[ cfg_attr( not( test ), doc( hidden ) ) ]
pub mod gem;
#[ cfg_attr( not( test ), doc( hidden ) ) ]
pub mod scene_utilities;
#[ cfg_attr( not( test ), doc( hidden ) ) ]
pub mod surface_material;

// Shader source modules (completely private to maintain obfuscation)
// Tests don't need direct access to shader sources
pub(crate) mod gem_frag;
pub(crate) mod gem_vert;
