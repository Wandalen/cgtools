//!
//! Collection of Embroidery API commands.
//!

mod private
{
  use clap::Subcommand;

  /// Raster API commands.
  #[ derive ( Debug, Subcommand ) ]
  pub enum Command
  {
    /// Vectorize command
    #[ command( subcommand ) ]
    Vectorize( super::vectorize::Command ),
  }

  /// Execute Raster command.
  pub async fn command
  (
    command : Command,
  )
  {
    match command
    {
      Command::Vectorize( c )  =>
      {
        super::vectorize::command( c ).await;
      }
    }
  }
}

crate::mod_interface!
{
  layer vectorize;
  layer common;

  own use
  {
    Command,
    command
  };
}