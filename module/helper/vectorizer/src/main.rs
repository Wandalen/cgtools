//! Converts raster to vector images.
#![ doc( html_root_url = "https://docs.rs/vectorizer/latest/vectorizer/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Converts raster to vector images" ) ]

use std::error::Error;
use clap::Parser;
use vectorizer::commands::{ Cli, CliCommand, self };


#[ tokio::main ]
async fn main() -> Result< (), Box< dyn Error > >
{
  let cli = Cli::parse();

  match cli.command
  {
    CliCommand::Raster( raster_command ) =>
    {
      commands::raster::command( raster_command ).await;
    },
  }

  Ok( () )
}
