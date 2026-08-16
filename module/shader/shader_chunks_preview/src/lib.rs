//! Preview utility CLI: the `preview` command. Builds a
//! [`shader_chunks_preview_core::PreviewBundle`] from a bundled chunk name
//! or a local WGSL file, validates the composed WGSL natively with naga
//! ( the same front end wgpu uses — so a broken shader fails here, in the
//! terminal, not in the browser console ), writes the bundle as
//! `-preview.json` into the `shader_chunks_preview_web` runner crate, and
//! — by default — serves that runner in the browser via the repo's shared
//! `action/browser_serve` script. `serve::0` stops after writing and
//! prints a summary instead, which is also what makes the command testable
//! without a browser.

mod private
{
  use core::fmt;
  use std::path::{ Path, PathBuf };
  use unilang::prelude::*;
  use cli_fmt::prelude::*;
  use shader_chunks_cli_core::{ CliApp, CommandSet, arg_bool, arg_string, error_report, named_arg, stdout_print, text_output };
  use shader_chunks_preview_core::{ bundle_build, PreviewBundle, PreviewError };

  /// This utility's standalone binary name.
  pub const BINARY : &str = "shader_chunks_preview";

  /// What to preview: a bundled chunk by name, or a local WGSL chunk file
  /// ( manifest header included ).
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub enum PreviewTarget
  {
    /// A bundled chunk name resolved via [`shader_chunks_core::chunk_get`].
    Name( String ),
    /// A path to a local `.wgsl` chunk file.
    File( String ),
  }

  /// Error returned by the preview command functions.
  #[ derive( Debug ) ]
  pub enum PreviewCliError
  {
    /// `preview <name>` named a chunk not present in
    /// [`shader_chunks_core::CHUNKS`].
    UnknownChunk( String ),
    /// Bundle building rejected the target ( see
    /// [`shader_chunks_preview_core::PreviewError`] ).
    Preview( PreviewError ),
    /// The composed WGSL failed naga parse/validation.
    Validation( String ),
    /// Reading the target file or writing the bundle failed.
    Io( String ),
    /// Launching the browser dev server failed.
    Serve( String ),
  }

  impl fmt::Display for PreviewCliError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::UnknownChunk( name ) => write!( f, "unknown chunk: `{name}` (see `shader_chunks list` for valid names)" ),
        Self::Preview( err ) => write!( f, "{err}" ),
        Self::Validation( msg ) => write!( f, "composed WGSL failed validation:\n{msg}" ),
        Self::Io( msg ) => write!( f, "io error: {msg}" ),
        Self::Serve( msg ) => write!( f, "serve error: {msg}" ),
      }
    }
  }

  impl std::error::Error for PreviewCliError {}

  impl PreviewCliError
  {
    /// Maps this error to a process exit code: `1` for a bad target or a
    /// target whose shader doesn't build ( caller-fixable by picking a
    /// different chunk or fixing the WGSL ), `2` for io/serve failures
    /// ( environmental ).
    #[ must_use ]
    pub fn exit_code( &self ) -> i32
    {
      match self
      {
        Self::UnknownChunk( _ ) | Self::Preview( _ ) | Self::Validation( _ ) => 1,
        Self::Io( _ ) | Self::Serve( _ ) => 2,
      }
    }
  }

  fn preview_cli_error( err : &PreviewCliError ) -> ErrorData
  {
    let code = match err
    {
      PreviewCliError::UnknownChunk( _ )
      | PreviewCliError::Preview( _ )
      | PreviewCliError::Validation( _ ) => ErrorCode::ValidationRuleFailed,
      PreviewCliError::Io( _ ) | PreviewCliError::Serve( _ ) => ErrorCode::InternalError,
    };
    error_report( err.exit_code(), code, err.to_string() )
  }

  /// The sibling `shader_chunks_preview_web` runner crate's directory —
  /// where the bundle is written and the dev server is started.
  #[ must_use ]
  pub fn web_crate_dir() -> PathBuf
  {
    Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "../shader_chunks_preview_web" )
  }

  /// Builds and naga-validates the bundle for `target`.
  ///
  /// # Errors
  ///
  /// Returns [`PreviewCliError::UnknownChunk`] for an unbundled name,
  /// [`PreviewCliError::Io`] for an unreadable file,
  /// [`PreviewCliError::Preview`] when bundle building rejects the target,
  /// or [`PreviewCliError::Validation`] when the composed WGSL fails naga.
  pub fn bundle_prepare( target : &PreviewTarget ) -> Result< PreviewBundle, PreviewCliError >
  {
    let bundle = match target
    {
      PreviewTarget::Name( name ) =>
      {
        let chunk = shader_chunks_core::chunk_get( name )
        .ok_or_else( || PreviewCliError::UnknownChunk( name.clone() ) )?;
        bundle_build( chunk.wgsl ).map_err( PreviewCliError::Preview )?
      }
      PreviewTarget::File( path ) =>
      {
        let wgsl = std::fs::read_to_string( path )
        .map_err( | err | PreviewCliError::Io( format!( "reading `{path}`: {err}" ) ) )?;
        bundle_build( &wgsl ).map_err( PreviewCliError::Preview )?
      }
    };

    let module = naga::front::wgsl::parse_str( &bundle.wgsl )
    .map_err( | err | PreviewCliError::Validation( err.emit_to_string( &bundle.wgsl ) ) )?;
    naga::valid::Validator::new( naga::valid::ValidationFlags::all(), naga::valid::Capabilities::default() )
    .validate( &module )
    .map_err( | err | PreviewCliError::Validation( err.emit_to_string( &bundle.wgsl ) ) )?;

    Ok( bundle )
  }

  /// Writes `bundle` as `-preview.json` into `dir` ( the web runner crate's
  /// directory — trunk copies the file into the served site ), returning
  /// the written path.
  ///
  /// # Errors
  ///
  /// Returns [`PreviewCliError::Io`] when serialization or the write fails.
  pub fn bundle_write( bundle : &PreviewBundle, dir : &Path ) -> Result< PathBuf, PreviewCliError >
  {
    let path = dir.join( "-preview.json" );
    let json = serde_json::to_string_pretty( bundle )
    .map_err( | err | PreviewCliError::Io( format!( "serializing bundle: {err}" ) ) )?;
    std::fs::write( &path, json )
    .map_err( | err | PreviewCliError::Io( format!( "writing `{}`: {err}", path.display() ) ) )?;
    Ok( path )
  }

  /// Human-readable summary of a prepared-and-written bundle: target,
  /// composed size, validation status, sliders, and the written path.
  #[ must_use ]
  pub fn summary( bundle : &PreviewBundle, written_to : &Path ) -> String
  {
    let mut lines = vec!
    [
      format!( "wrote {} ({} bytes wgsl, naga-validated)", written_to.display(), bundle.wgsl.len() ),
      format!( "target: {}", bundle.target ),
      "sliders:".to_string(),
    ];
    for param in &bundle.parameters
    {
      lines.push( format!( "  {}  {}..{}  start {}", param.property, param.min, param.max, param.value ) );
    }
    lines.join( "\n" )
  }

  fn serve() -> Result< (), PreviewCliError >
  {
    let script = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "../../../action/browser_serve" );
    let status = std::process::Command::new( &script )
    .current_dir( web_crate_dir() )
    .status()
    .map_err( | err | PreviewCliError::Serve( format!( "spawning `{}`: {err}", script.display() ) ) )?;
    if !status.success()
    {
      return Err( PreviewCliError::Serve( format!( "`{}` exited with {status}", script.display() ) ) );
    }
    Ok( () )
  }

  /// The whole `preview` command: build, validate, write the bundle into
  /// the web runner crate, and — with `serve_bundle` set — print the
  /// summary and hand off to the browser dev server ( blocks until the
  /// server is stopped ). With it unset, returns the summary instead.
  ///
  /// # Errors
  ///
  /// Every [`PreviewCliError`] variant, per [`bundle_prepare`],
  /// [`bundle_write`], and the serve hand-off.
  pub fn preview( target : &PreviewTarget, serve_bundle : bool ) -> Result< String, PreviewCliError >
  {
    let bundle = bundle_prepare( target )?;
    let path = bundle_write( &bundle, &web_crate_dir() )?;
    let summary = summary( &bundle, &path );
    if serve_bundle
    {
      stdout_print( &summary );
      serve()?;
      Ok( String::new() )
    }
    else
    {
      Ok( summary )
    }
  }

  fn cmd_preview( binary : &str ) -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".preview" )
    .namespace( String::new() )
    .description( "Render a chunk live in the browser: composed, naga-validated, with `//@ param:` uniforms wired to sliders.".to_string() )
    .hint( "live browser preview of one chunk, sliders from its tunable parameters" )
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
      format!( "{binary} preview fbm3" ),
      format!( "{binary} preview file::shader/my_chunk.wgsl" ),
      format!( "{binary} preview fbm3 serve::0" ),
    ])
    .arguments( vec!
    [
      ArgumentDefinition::former()
      .name( "name" )
      .kind( Kind::String )
      .hint( "Bundled chunk name (see `shader_chunks list`); omit when passing `file::`." )
      .attributes( ArgumentAttributes { optional : true, ..ArgumentAttributes::default() } )
      .end(),
      named_arg( "file", Kind::String, "Path to a local `.wgsl` chunk file (manifest header included), instead of a bundled name.", None ),
      named_arg( "serve", Kind::Boolean, "Serve the preview in the browser; 0 = only build, validate, and write the bundle.", Some( "true".to_string() ) ),
    ])
    .end();

    let routine : CommandRoutine = Box::new( | cmd, _ctx |
    {
      let name = arg_string( &cmd, "name" );
      let file = arg_string( &cmd, "file" );
      let target = match ( name, file )
      {
        ( Some( name ), None ) => PreviewTarget::Name( name ),
        ( None, Some( file ) ) => PreviewTarget::File( file ),
        _ => return Err( error_report
        (
          1,
          ErrorCode::ValidationRuleFailed,
          "preview needs exactly one target: a chunk name (see `shader_chunks list`) or `file::<path>`".to_string(),
        )),
      };
      let serve_bundle = arg_bool( &cmd, "serve", true );
      let content = preview( &target, serve_bundle ).map_err( | e | preview_cli_error( &e ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  /// This utility's command set — the single `preview` command — with
  /// example invocations spelled against `binary`.
  #[ must_use ]
  pub fn commands( binary : &str ) -> CommandSet
  {
    vec![ cmd_preview( binary ) ]
  }

  /// This utility's help-screen group: `Preview` — same name and
  /// membership as `docs/cli/command_group/` documents for the aggregator.
  #[ must_use ]
  pub fn help_groups() -> Vec< CommandGroup >
  {
    vec!
    [
      CommandGroup
      {
        name : "Preview".to_string(),
        entries : vec!
        [
          CommandEntry { name : "preview [name]".to_string(), desc : "Render a chunk live in the browser, sliders wired to its tunables.".to_string() },
        ],
      },
    ]
  }

  /// This utility's help-screen example invocations, spelled against
  /// `binary`.
  #[ must_use ]
  pub fn help_examples( binary : &str ) -> Vec< ExampleEntry >
  {
    vec![ ExampleEntry { invocation : format!( "{binary} preview fbm3" ), desc : None } ]
  }

  /// Standalone entry point for the `shader_chunks_preview` binary.
  pub fn run()
  {
    shader_chunks_cli_core::run( CliApp
    {
      binary : BINARY.to_string(),
      tagline : "Render shader chunks live in the browser with tunable sliders.".to_string(),
      groups : help_groups(),
      examples : help_examples( BINARY ),
      commands : commands( BINARY ),
    });
  }
}

::mod_interface::mod_interface!
{
  own use BINARY;
  own use PreviewTarget;
  own use PreviewCliError;
  own use web_crate_dir;
  own use bundle_prepare;
  own use bundle_write;
  own use summary;
  own use preview;
  own use commands;
  own use help_groups;
  own use help_examples;
  own use run;
}
