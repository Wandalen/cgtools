//!
//! CLI commands of the tool.
//!

#[ cfg( feature = "cli" ) ]
/// Internal namespace.
mod private
{
  use crate::*;
  #[ cfg( feature = "cli" ) ]
  use clap::{ Parser, Subcommand, Args };
  use commands::raster ;
  use std::path::PathBuf;

  /// CLI commands of the tool.
  #[ derive ( Debug, Parser ) ]
  pub struct Cli
  {
    /// Root of the CLI commands.
    #[ command ( subcommand ) ]
    pub command : CliCommand,
  }

  /// Root of the CLI commands.
  ///
  /// This enum defines the root commands available in the CLI tool, each corresponding to a different API or functionality.
  /// Each variant contains the specific subcommands and arguments required for that functionality.
  ///
  /// # Variants
  /// * `Raster` - Commands for vectorizing raster images.
  ///   - Includes subcommands for color and layers vectorization methods.

  #[ derive ( Debug, Subcommand ) ]
  pub enum CliCommand
  {
    /// Raster API commands.
    #[ command ( subcommand, name = "raster" ) ]
    Raster( raster::Command ),
  }
  /// Represents configuration for input and output file paths.
  #[ derive( Debug, Args, Default ) ]
  pub struct InputOutput
  {
    /// Input file
    #[ arg( long, short ) ]
    pub input : PathBuf,

    /// Output file
    #[ arg( long, short ) ]
    pub output : Option< PathBuf >,
  }
}

#[ cfg( feature = "cli" ) ]
crate::mod_interface!
{
  layer raster;

  own use
  {
    Cli,
    CliCommand,
    InputOutput,
  };
}
