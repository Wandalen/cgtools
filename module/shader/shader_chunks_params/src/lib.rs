//! Parameters utility CLI: the `tunables` command rendering
//! `shader_chunks_params_core`'s discovery as a table. Exposes its command
//! set, help group, and help examples as data — parameterized by binary
//! name — so the `shader_chunks` aggregator folds them in unchanged, while
//! [`run`] serves the same command as the standalone `shader_chunks_params`
//! binary.

mod private
{
  use core::fmt;
  use unilang::prelude::*;
  use cli_fmt::prelude::*;
  use data_fmt::{ Format, RowBuilder, TableConfig, TableFormatter };
  use shader_chunks_cli_core::{ CliApp, CommandSet, error_report, text_output };

  /// This utility's standalone binary name.
  pub const BINARY : &str = "shader_chunks_params";

  /// Error returned by the tunables command functions.
  #[ derive( Debug ) ]
  pub enum ParamsCliError
  {
    /// The command named a chunk not present in
    /// [`shader_chunks_core::CHUNKS`].
    UnknownChunk( String ),
    /// A `data_fmt` render call failed.
    Render( String ),
  }

  impl fmt::Display for ParamsCliError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::UnknownChunk( name ) => write!( f, "unknown chunk: `{name}` (see `shader_chunks list` for valid names)" ),
        Self::Render( msg ) => write!( f, "render error: {msg}" ),
      }
    }
  }

  impl std::error::Error for ParamsCliError {}

  impl ParamsCliError
  {
    /// Maps this error to a process exit code: `1` for a bad chunk name
    /// ( validation-style, caller-fixable ), `2` for a render failure
    /// ( internal ).
    #[ must_use ]
    pub fn exit_code( &self ) -> i32
    {
      match self
      {
        Self::UnknownChunk( _ ) => 1,
        Self::Render( _ ) => 2,
      }
    }
  }

  fn params_error( err : &ParamsCliError ) -> ErrorData
  {
    let code = match err
    {
      ParamsCliError::UnknownChunk( _ ) => ErrorCode::ValidationRuleFailed,
      ParamsCliError::Render( _ ) => ErrorCode::InternalError,
    };
    error_report( err.exit_code(), code, err.to_string() )
  }

  /// Table of every tunable parameter
  /// [`shader_chunks_params_core::chunk_discover`] finds in `chunk`'s WGSL
  /// — name, kind, type, range, and range source ( declared vs. inferred ).
  /// Exposed separately from [`tunables`] so tests can exercise a chunk
  /// descriptor carrying `//@ param:` lines without any bundled chunk
  /// needing to declare one. A chunk with none renders an explicit
  /// message instead of a blank table or an error.
  ///
  /// # Errors
  ///
  /// Returns [`ParamsCliError::Render`] if the `data_fmt` table formatter
  /// fails.
  pub fn tunables_of_chunk( chunk : &shader_chunks_core::ChunkDescriptor ) -> Result< String, ParamsCliError >
  {
    let params = shader_chunks_params_core::chunk_discover( chunk );
    if params.is_empty()
    {
      return Ok( format!( "chunk `{}` declares no tunable parameters", chunk.name ) );
    }

    let mut builder = RowBuilder::new( vec!
    [
      "name".to_string(), "kind".to_string(), "type".to_string(),
      "range".to_string(), "source".to_string(),
    ]);
    for param in params
    {
      let ( range, source ) = match param.range
      {
        Some( ( range, source ) ) => ( format!( "{}..{}", range.min, range.max ), format!( "{source:?}" ) ),
        None => ( "-".to_string(), "-".to_string() ),
      };
      builder.add_row_mut( vec!
      [
        param.name.into(), format!( "{:?}", param.kind ).into(), format!( "{:?}", param.value_type ).into(),
        range.into(), source.into(),
      ]);
    }
    let view = builder.build_view();
    Format::format( &TableFormatter::with_config( TableConfig::plain() ), &view )
    .map_err( | e | ParamsCliError::Render( e.to_string() ) )
  }

  /// Table of every tunable parameter
  /// [`shader_chunks_params_core::chunk_discover`] finds declared on
  /// bundled chunk `name`. See [`tunables_of_chunk`] for the rendering
  /// itself.
  ///
  /// # Errors
  ///
  /// Returns [`ParamsCliError::UnknownChunk`] if `name` is not bundled, or
  /// [`ParamsCliError::Render`] if the `data_fmt` table formatter fails.
  pub fn tunables( name : &str ) -> Result< String, ParamsCliError >
  {
    let chunk = shader_chunks_core::chunk_get( name )
    .ok_or_else( || ParamsCliError::UnknownChunk( name.to_string() ) )?;
    tunables_of_chunk( chunk )
  }

  fn cmd_tunables( binary : &str ) -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".tunables" )
    .namespace( String::new() )
    .description( "List every tunable parameter a chunk declares via `//@ param:` lines.".to_string() )
    .hint( "name, kind, type, range, and range source for one chunk's declared tunables" )
    .status( "stable" )
    .version( "1.0.0".to_string() )
    .aliases( vec![] )
    .tags( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( String::new() )
    .examples( vec![ format!( "{binary} tunables fbm3" ) ] )
    .arguments( vec!
    [
      ArgumentDefinition::former()
      .name( "name" )
      .kind( Kind::String )
      .hint( "Chunk name (see `shader_chunks list`)." )
      .attributes( ArgumentAttributes::default() )
      .end(),
    ])
    .end();

    let routine : CommandRoutine = Box::new( | cmd, _ctx |
    {
      let name = match cmd.arguments.get( "name" )
      {
        Some( Value::String( name ) ) => name.clone(),
        _ => unreachable!( "argument 'name' is declared Kind::String and required" ),
      };
      let content = tunables( &name ).map_err( | e | params_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  /// This utility's command set — the single `tunables` command — with
  /// example invocations spelled against `binary`.
  #[ must_use ]
  pub fn commands( binary : &str ) -> CommandSet
  {
    vec![ cmd_tunables( binary ) ]
  }

  /// This utility's help-screen group: `Parameters` — same name and
  /// membership as `docs/cli/command_group/` documents for the aggregator.
  #[ must_use ]
  pub fn help_groups() -> Vec< CommandGroup >
  {
    vec!
    [
      CommandGroup
      {
        name : "Parameters".to_string(),
        entries : vec!
        [
          CommandEntry { name : "tunables <name>".to_string(), desc : "List a chunk's tunable parameters: kind, type, range, source.".to_string() },
        ],
      },
    ]
  }

  /// This utility's help-screen example invocations, spelled against
  /// `binary`.
  #[ must_use ]
  pub fn help_examples( binary : &str ) -> Vec< ExampleEntry >
  {
    vec![ ExampleEntry { invocation : format!( "{binary} tunables fbm3" ), desc : None } ]
  }

  /// Standalone entry point for the `shader_chunks_params` binary.
  pub fn run()
  {
    shader_chunks_cli_core::run( CliApp
    {
      binary : BINARY.to_string(),
      tagline : "Inspect tunable `//@ param:` parameters of shader_chunks_core's bundled WGSL chunks.".to_string(),
      groups : help_groups(),
      examples : help_examples( BINARY ),
      commands : commands( BINARY ),
    });
  }
}

::mod_interface::mod_interface!
{
  own use BINARY;
  own use ParamsCliError;
  own use tunables_of_chunk;
  own use tunables;
  own use commands;
  own use help_groups;
  own use help_examples;
  own use run;
}
