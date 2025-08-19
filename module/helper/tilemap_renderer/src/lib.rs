//! Agnostic 2D rendering engine.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

// Prevent "unused" warnings when features are disabled
#![ cfg_attr( not( feature = "std" ), allow( unused ) ) ]

// Module declarations - using ultra-granular feature gating
#[ cfg( any( feature = "scene-container", feature = "scene-methods" ) ) ]
pub mod scene;

#[ cfg( any(
  feature = "command-line",
  feature = "command-curve",
  feature = "command-text",
  feature = "command-tilemap",
  feature = "command-particle",
  feature = "commands"
) ) ]
pub mod commands;

#[ cfg( any(
  feature = "traits-renderer",
  feature = "traits-primitive",
  feature = "traits-async",
  feature = "ports"
) ) ]
pub mod ports;

#[ cfg( any(
  feature = "adapter-svg-basic",
  feature = "adapter-svg",
  feature = "adapter-svg-browser",
  feature = "adapter-webgl",
  feature = "adapter-webgpu",
  feature = "adapter-terminal-basic",
  feature = "adapter-terminal"
) ) ]
pub mod adapters;

#[ cfg( any(
  feature = "query-basic",
  feature = "query-by-type",
  feature = "query-predicate",
  feature = "query"
) ) ]
pub mod query;

#[ cfg( any(
  feature = "cli-basic",
  feature = "cli-commands",
  feature = "cli-repl",
  feature = "cli"
) ) ]
pub mod cli;

pub mod wgpu_renderer;
