//! Agnostic Rendering Engine CLI binary.
//!
//! The `are` (Agnostic Rendering Engine) CLI provides an interactive REPL
//! for creating 2D scenes, adding primitives, and rendering to multiple
//! backend formats including SVG, Terminal ASCII art, and WebGL.
//!
//! ## Usage
//! 
//! Run interactively: `are`
//! Run single command: `are ".help"`
//!
//! ## Examples
//!
//! Create a scene with a line:
//! ```
//! .scene.new
//! .scene.add line 0 0 100 100  
//! .render svg output.svg
//! ```

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::needless_return ) ]

use tilemap_renderer::cli::run_cli;

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  run_cli()?;
  return Ok( () );
}