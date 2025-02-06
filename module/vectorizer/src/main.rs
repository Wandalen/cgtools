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
