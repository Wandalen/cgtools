//! Agnostic 2D rendering engine.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

// Prevent "unused" warnings when the feature is disabled.
#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]

// Module declarations - these exist at crate level
#[ cfg( feature = "scene" ) ]
pub mod scene;

#[ cfg( feature = "commands" ) ]
pub mod commands;

#[ cfg( feature = "ports" ) ]  
pub mod ports;

#[ cfg( feature = "query" ) ]
pub mod query;