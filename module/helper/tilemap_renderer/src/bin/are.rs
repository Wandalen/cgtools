//! Agnostic Rendering Engine CLI binary.

use tilemap_renderer::cli::run_cli;

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  run_cli()?;
  Ok( () )
}