//! Compose utility CLI: the `compose` command over `shader_chunks_core`'s
//! resolution ( [`shader_chunks_core::set_resolve`] ) and composition
//! ( [`shader_chunks_core::set_try_compose`] ) — this utility deliberately
//! has no `_core` crate of its own, because `shader_chunks_core` *is* its
//! core. Exposes its command set, help group, and help examples as data —
//! parameterized by binary name — so the `shader_chunks` aggregator folds
//! them in unchanged, while [`run`] serves the same command as the
//! standalone `shader_chunks_compose` binary.

mod private
{
  use core::fmt;
  use unilang::prelude::*;
  use cli_fmt::prelude::*;
  use shader_chunks_cli_core::{ CliApp, CommandSet, arg_bool, error_report, named_arg, names_flatten, text_output };

  /// This utility's standalone binary name.
  pub const BINARY : &str = "shader_chunks_compose";

  /// Error returned by the compose command functions.
  #[ derive( Debug ) ]
  pub enum ComposeCliError
  {
    /// A name ( or, under `transitive::1`, a reachable dependency name )
    /// not present in [`shader_chunks_core::CHUNKS`].
    UnknownChunk( String ),
    /// Composition failed the dependency resolution
    /// [`shader_chunks_core::set_try_compose`] performs.
    Compose( shader_chunks_core::ComposeError ),
  }

  impl fmt::Display for ComposeCliError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::UnknownChunk( name ) => write!( f, "unknown chunk: `{name}` (see `list` for valid names)" ),
        Self::Compose( err ) => write!( f, "{err}" ),
      }
    }
  }

  impl std::error::Error for ComposeCliError {}

  impl ComposeCliError
  {
    /// Maps this error to a process exit code: both variants are
    /// validation-style, caller-fixable by passing different arguments —
    /// exit `1`.
    #[ must_use ]
    pub fn exit_code( &self ) -> i32
    {
      match self
      {
        Self::UnknownChunk( _ ) | Self::Compose( _ ) => 1,
      }
    }
  }

  fn compose_error( err : &ComposeCliError ) -> ErrorData
  {
    error_report( err.exit_code(), ErrorCode::ValidationRuleFailed, err.to_string() )
  }

  /// Composes already-resolved WGSL chunk bodies via
  /// [`shader_chunks_core::try_compose`]. Exposed separately from
  /// [`chunks_compose`] so tests can exercise cyclic/missing-dependency
  /// failures with synthetic fixtures — the real bundled chunk set is fixed
  /// and acyclic, so it can never produce a `CyclicDependency` through the
  /// name-based [`chunks_compose`] path.
  ///
  /// # Errors
  ///
  /// Returns [`ComposeCliError::Compose`] on a cyclic or unresolved
  /// dependency.
  pub fn wgsl_try_compose( chunks : &[ &str ] ) -> Result< String, ComposeCliError >
  {
    shader_chunks_core::try_compose( chunks ).map_err( ComposeCliError::Compose )
  }

  /// Resolves `names` via [`shader_chunks_core::set_resolve`] and composes
  /// the selection. With `transitive` set, the named set is first widened
  /// to its full dependency closure — `chunks_compose( &[ "fbm3" ], true )`
  /// pulls in `value_noise` and `hash21` unasked — so one root name
  /// suffices instead of spelling out its whole chain; with it unset the
  /// named set must already be dependency-complete or composition fails
  /// loudly. Either way [`shader_chunks_core::set_try_compose`]'s
  /// topological sort orders the output, so the closure of a set and the
  /// same set written out explicitly compose to identical text.
  ///
  /// # Errors
  ///
  /// Returns [`ComposeCliError::UnknownChunk`] if any name ( or, under
  /// `transitive`, any reachable dependency name ) is not bundled, or
  /// [`ComposeCliError::Compose`] on a missing dependency.
  pub fn chunks_compose( names : &[ String ], transitive : bool ) -> Result< String, ComposeCliError >
  {
    let names : Vec< &str > = names.iter().map( String::as_str ).collect();
    let selected = shader_chunks_core::set_resolve( &names, transitive )
    .map_err( | shader_chunks_core::ResolveError::UnknownChunk( name ) | ComposeCliError::UnknownChunk( name ) )?;
    let set : Vec< shader_chunks_core::ChunkDescriptor > = selected.into_iter().copied().collect();
    shader_chunks_core::set_try_compose( &set ).map_err( ComposeCliError::Compose )
  }

  fn cmd_compose( binary : &str ) -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".compose" )
    .namespace( String::new() )
    .description( "Preview WGSL composed from one or more chunks, dependency-ordered.".to_string() )
    .hint( "composed WGSL text for the given chunks, in dependency order" )
    .status( "stable" )
    .version( "1.0.0".to_string() )
    .aliases( vec![] )
    .tags( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( String::new() )
    .examples( vec!
    [
      format!( "{binary} compose hash21 value_noise" ),
      format!( "{binary} compose fbm3 transitive::1" ),
    ])
    .arguments( vec!
    [
      ArgumentDefinition::former()
      .name( "names" )
      .kind( Kind::List( Box::new( Kind::String ), None ) )
      .hint( "One or more chunk names (see `list`)." )
      .attributes( ArgumentAttributes { multiple : true, ..ArgumentAttributes::default() } )
      .end(),
      named_arg( "transitive", Kind::Boolean, "Widen the named set to its full dependency closure.", Some( "false".to_string() ) ),
    ])
    .end();

    let routine : CommandRoutine = Box::new( | cmd, _ctx |
    {
      let names : Vec< String > = match cmd.arguments.get( "names" )
      {
        Some( value ) => names_flatten( value ),
        None => unreachable!( "argument 'names' is declared Kind::List, multiple, and required" ),
      };
      let transitive = arg_bool( &cmd, "transitive", false );
      let content = chunks_compose( &names, transitive ).map_err( | e | compose_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  /// This utility's command set — the single `compose` command — with
  /// example invocations spelled against `binary`.
  #[ must_use ]
  pub fn commands( binary : &str ) -> CommandSet
  {
    vec![ cmd_compose( binary ) ]
  }

  /// This utility's help-screen group: `Compose` — same name and
  /// membership as `docs/cli/command_group/` documents for the aggregator.
  #[ must_use ]
  pub fn help_groups() -> Vec< CommandGroup >
  {
    vec!
    [
      CommandGroup
      {
        name : "Compose".to_string(),
        entries : vec!
        [
          CommandEntry { name : "compose <names...>".to_string(), desc : "Preview composed WGSL for the given chunks.".to_string() },
        ],
      },
    ]
  }

  /// This utility's help-screen example invocations, spelled against
  /// `binary`.
  #[ must_use ]
  pub fn help_examples( binary : &str ) -> Vec< ExampleEntry >
  {
    vec![ ExampleEntry { invocation : format!( "{binary} compose hash21 value_noise" ), desc : None } ]
  }

  /// Standalone entry point for the `shader_chunks_compose` binary.
  pub fn run()
  {
    shader_chunks_cli_core::run( CliApp
    {
      binary : BINARY.to_string(),
      tagline : "Compose shader_chunks_core's bundled WGSL chunks into dependency-ordered WGSL.".to_string(),
      groups : help_groups(),
      examples : help_examples( BINARY ),
      commands : commands( BINARY ),
    });
  }
}

::mod_interface::mod_interface!
{
  own use BINARY;
  own use ComposeCliError;
  own use wgsl_try_compose;
  own use chunks_compose;
  own use commands;
  own use help_groups;
  own use help_examples;
  own use run;
}
