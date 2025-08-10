//! Agnostic Rendering Engine CLI binary.

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::needless_return ) ]

use tilemap_renderer::cli::run_cli;

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  run_cli()?;
  return Ok( () );
}