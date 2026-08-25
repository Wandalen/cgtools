//! Validate utility CLI: the `validate` command rendering
//! `shader_chunks_validate_core`'s registry-wide checks as a human-readable
//! findings report. Exposes its command set, help group, and help examples
//! as data — parameterized by binary name — so the `shader_chunks`
//! aggregator folds them in unchanged, while [`run`] serves the same
//! command as the standalone `shader_chunks_validate` binary.

mod private
{
  use core::fmt;
  use unilang::prelude::*;
  use cli_fmt::prelude::*;
  use shader_chunks_cli_core::{ CliApp, CommandSet, error_report, text_output };

  /// This utility's standalone binary name.
  pub const BINARY : &str = "shader_chunks_validate";

  /// Error returned when [`validate`] finds one or more problems. The sole
  /// failure mode this command has — [`shader_chunks_validate_core`]'s
  /// checks are all non-panicking `Vec`-returning functions over an
  /// always-present compiled-in registry, so there is no chunk name to
  /// mistype and no render step that can fail.
  #[ derive( Debug ) ]
  pub enum ValidateCliError
  {
    /// One or more checks reported a finding — carries the fully rendered,
    /// human-readable findings report.
    FindingsPresent( String ),
  }

  impl fmt::Display for ValidateCliError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::FindingsPresent( report ) => write!( f, "{report}" ),
      }
    }
  }

  impl std::error::Error for ValidateCliError {}

  impl ValidateCliError
  {
    /// Maps this error to a process exit code: `1`, validation-style and
    /// caller-fixable — the same code [`shader_chunks_params::ParamsCliError::UnknownChunk`]
    /// and sibling commands use for "ran fine, found a problem".
    #[ must_use ]
    pub fn exit_code( &self ) -> i32
    {
      match self
      {
        Self::FindingsPresent( _ ) => 1,
      }
    }
  }

  fn validate_error( err : &ValidateCliError ) -> ErrorData
  {
    error_report( err.exit_code(), ErrorCode::ValidationRuleFailed, err.to_string() )
  }

  /// [`shader_chunks_validate_core::validate`] over `chunks`, rendered as a
  /// human-readable findings report: an explicit all-clear message when
  /// there are no findings ( never a blank report ), or `Err` carrying
  /// every finding as one `[chunk] check: message` block per finding,
  /// blank-line separated so a multi-line `wgsl_compile` naga diagnostic
  /// ( source snippet, caret, multiple lines — see
  /// `shader_chunks_preview::PreviewCliError::Validation`'s identical
  /// raw-diagnostic-as-plain-text precedent ) stays readable rather than
  /// being forced into a single table cell. Exposed separately from
  /// [`validate`] so tests can exercise the report's rendering against a
  /// self-contained fixture set without any bundled chunk needing to be
  /// broken — the same split [`shader_chunks_params::tunables_of_chunk`]
  /// makes from [`shader_chunks_params::tunables`], for the same reason.
  ///
  /// # Errors
  ///
  /// Returns [`ValidateCliError::FindingsPresent`] when one or more
  /// findings are present.
  pub fn validate_chunks( chunks : &[ shader_chunks_core::ChunkDescriptor ] ) -> Result< String, ValidateCliError >
  {
    let findings = shader_chunks_validate_core::validate( chunks );
    if findings.is_empty()
    {
      return Ok( format!( "registry is clean: {} chunks, 0 findings", chunks.len() ) );
    }

    let body = findings.iter()
    .map( | finding | format!( "[{}] {}: {}", finding.chunk, finding.check, finding.message ) )
    .collect::< Vec< _ > >()
    .join( "\n\n" );

    Err( ValidateCliError::FindingsPresent( format!( "{} finding(s):\n\n{body}", findings.len() ) ) )
  }

  /// [`validate_chunks`] over the real bundled [`shader_chunks_core::CHUNKS`]
  /// registry — what `shader_chunks validate` actually runs. See
  /// [`validate_chunks`] for the rendering itself.
  ///
  /// # Errors
  ///
  /// Returns [`ValidateCliError::FindingsPresent`] when the bundled
  /// registry has one or more findings.
  pub fn validate() -> Result< String, ValidateCliError >
  {
    validate_chunks( shader_chunks_core::CHUNKS )
  }

  fn cmd_validate( binary : &str ) -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".validate" )
    .namespace( String::new() )
    .description( "Run registry-wide integrity checks over every bundled chunk: manifest drift, duplicate names, missing/cyclic dependencies, and naga WGSL validation.".to_string() )
    .hint( "lint the bundled shader_chunks_core registry; exits non-zero if any check reports a finding" )
    .status( "stable" )
    .version( "1.0.0".to_string() )
    .aliases( vec![] )
    .tags( vec![] )
    .permissions( vec![] )
    .idempotent( true )
    .deprecation_message( String::new() )
    .http_method_hint( String::new() )
    .examples( vec![ format!( "{binary} validate" ) ] )
    .arguments( vec![] )
    .end();

    let routine : CommandRoutine = Box::new( | _cmd, _ctx |
    {
      let content = validate().map_err( | e | validate_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  /// This utility's command set — the single `validate` command — with
  /// example invocations spelled against `binary`.
  #[ must_use ]
  pub fn commands( binary : &str ) -> CommandSet
  {
    vec![ cmd_validate( binary ) ]
  }

  /// This utility's help-screen group: `Validate` — same name and
  /// membership as `docs/cli/command_group/` documents for the aggregator.
  #[ must_use ]
  pub fn help_groups() -> Vec< CommandGroup >
  {
    vec!
    [
      CommandGroup
      {
        name : "Validate".to_string(),
        entries : vec!
        [
          CommandEntry { name : "validate".to_string(), desc : "Lint the bundled registry: drift, duplicates, missing/cyclic deps, WGSL compile.".to_string() },
        ],
      },
    ]
  }

  /// This utility's help-screen example invocations, spelled against
  /// `binary`.
  #[ must_use ]
  pub fn help_examples( binary : &str ) -> Vec< ExampleEntry >
  {
    vec![ ExampleEntry { invocation : format!( "{binary} validate" ), desc : None } ]
  }

  /// Standalone entry point for the `shader_chunks_validate` binary.
  pub fn run()
  {
    shader_chunks_cli_core::run( CliApp
    {
      binary : BINARY.to_string(),
      tagline : "Lint shader_chunks_core's bundled WGSL chunks: manifest drift, duplicates, missing/cyclic dependencies, naga validation.".to_string(),
      groups : help_groups(),
      examples : help_examples( BINARY ),
      commands : commands( BINARY ),
    });
  }
}

::mod_interface::mod_interface!
{
  own use BINARY;
  own use ValidateCliError;
  own use validate_chunks;
  own use validate;
  own use commands;
  own use help_groups;
  own use help_examples;
  own use run;
}
