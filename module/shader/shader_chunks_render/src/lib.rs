//! Render utility CLI: the `render` command. Reuses
//! [`shader_chunks_preview`]'s `bundle_prepare` — the same target
//! resolution ( bundled name or local `file::` ) and the same naga
//! validation the live preview runs — then renders one frame of the
//! bundle on a headless GPU via [`shader_chunks_render_core`] and writes
//! it as a PNG. Every bundle parameter takes its initial ( slider-start )
//! value, so the written image is exactly what the browser preview shows
//! before anyone touches a slider, frozen at the requested `time::`.
//! Unlike `.preview`, nothing here needs a browser, a dev server, or the
//! web runner crate — the side effect is one image file at `out::`.

mod private
{
  use core::fmt;
  use std::path::{ Path, PathBuf };
  use unilang::prelude::*;
  use cli_fmt::prelude::*;
  use shader_chunks_cli_core::{ CliApp, CommandSet, arg_string, error_report, named_arg, text_output };
  use shader_chunks_preview::{ PreviewCliError, PreviewTarget, bundle_prepare };
  use shader_chunks_preview_core::PreviewBundle;
  use shader_chunks_render_core::RenderError;

  /// This utility's standalone binary name.
  pub const BINARY : &str = "shader_chunks_render";

  /// Error returned by the render command functions.
  #[ derive( Debug ) ]
  pub enum RenderCliError
  {
    /// Target resolution, bundle building, or naga validation failed —
    /// see [`shader_chunks_preview::PreviewCliError`], reused verbatim.
    Preview( PreviewCliError ),
    /// The `size::` value is not `<n>` or `<width>x<height>` with both
    /// sides at least 1.
    InvalidSize( String ),
    /// The headless GPU render failed ( see
    /// [`shader_chunks_render_core::RenderError`] ).
    Render( RenderError ),
    /// Writing the PNG failed.
    Io( String ),
  }

  impl fmt::Display for RenderCliError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::Preview( err ) => write!( f, "{err}" ),
        Self::InvalidSize( raw ) =>
        write!( f, "invalid `size` value: `{raw}` (allowed: `<n>` or `<width>x<height>`, each side at least 1)" ),
        Self::Render( err ) => write!( f, "{err}" ),
        Self::Io( msg ) => write!( f, "io error: {msg}" ),
      }
    }
  }

  impl std::error::Error for RenderCliError {}

  impl RenderCliError
  {
    /// Maps this error to a process exit code: `1` for a bad target, a
    /// bad `size::`, or a shader that doesn't build ( caller-fixable ),
    /// `2` for GPU/io failures ( environmental ).
    #[ must_use ]
    pub fn exit_code( &self ) -> i32
    {
      match self
      {
        Self::Preview( err ) => err.exit_code(),
        Self::InvalidSize( _ ) => 1,
        Self::Render( _ ) | Self::Io( _ ) => 2,
      }
    }
  }

  fn render_cli_error( err : &RenderCliError ) -> ErrorData
  {
    let code = if err.exit_code() == 1 { ErrorCode::ValidationRuleFailed } else { ErrorCode::InternalError };
    error_report( err.exit_code(), code, err.to_string() )
  }

  /// Parses a `size::` value: `<n>` renders a square, `<width>x<height>`
  /// renders exactly that size; each side must be at least 1.
  ///
  /// # Errors
  ///
  /// Returns [`RenderCliError::InvalidSize`] for anything else — a zero
  /// side, a missing side, a negative or non-numeric value.
  pub fn size_parse( raw : &str ) -> Result< ( u32, u32 ), RenderCliError >
  {
    let side_parse = | side : &str | side.trim().parse::< u32 >().ok().filter( | &n | n >= 1 );
    let trimmed = raw.trim();
    let parsed = match trimmed.split_once( 'x' )
    {
      Some( ( width, height ) ) => side_parse( width ).zip( side_parse( height ) ),
      None => side_parse( trimmed ).map( | side | ( side, side ) ),
    };
    parsed.ok_or_else( || RenderCliError::InvalidSize( raw.to_string() ) )
  }

  /// The output path for `target` when no `out::` is given: the bundled
  /// chunk's name, or the local file's stem, with a `.png` extension, in
  /// the current directory.
  #[ must_use ]
  pub fn out_path_of( target : &PreviewTarget, out : Option< String > ) -> PathBuf
  {
    match out
    {
      Some( path ) => PathBuf::from( path ),
      None => match target
      {
        PreviewTarget::Name( name ) => PathBuf::from( format!( "{name}.png" ) ),
        PreviewTarget::File( path ) =>
        {
          let stem = Path::new( path ).file_stem()
          .map_or_else( || "render".to_string(), | stem | stem.to_string_lossy().into_owned() );
          PathBuf::from( format!( "{stem}.png" ) )
        }
      },
    }
  }

  /// Human-readable summary of a rendered-and-written frame: the written
  /// path and size, the target, the frozen `time`, and the parameter
  /// values baked into the frame.
  #[ must_use ]
  pub fn summary( bundle : &PreviewBundle, size : ( u32, u32 ), time : f32, written_to : &Path ) -> String
  {
    let mut lines = vec!
    [
      format!( "wrote {} ({}x{} px, naga-validated)", written_to.display(), size.0, size.1 ),
      format!( "target: {}", bundle.target ),
      format!( "time: {time}" ),
      "parameters at defaults:".to_string(),
    ];
    for param in &bundle.parameters
    {
      lines.push( format!( "  {} = {}", param.property, param.value ) );
    }
    lines.join( "\n" )
  }

  /// The whole `render` command: build and naga-validate the bundle
  /// ( via [`shader_chunks_preview::bundle_prepare`] ), render one frame
  /// headlessly, write it as a PNG at `out`, and return the summary.
  ///
  /// # Errors
  ///
  /// Every [`RenderCliError`] variant except `InvalidSize` ( the caller
  /// parses `size::` first, via [`size_parse`] ).
  pub fn render_to_png( target : &PreviewTarget, size : ( u32, u32 ), time : f32, out : &Path )
  -> Result< String, RenderCliError >
  {
    let bundle = bundle_prepare( target ).map_err( RenderCliError::Preview )?;
    let image = shader_chunks_render_core::render( &bundle, size, time ).map_err( RenderCliError::Render )?;
    let ( width, height ) = image.size;
    image::save_buffer( out, &image.pixels, width, height, image::ColorType::Rgba8 )
    .map_err( | err | RenderCliError::Io( format!( "writing `{}`: {err}", out.display() ) ) )?;
    Ok( summary( &bundle, size, time, out ) )
  }

  /// Extracts the `time::` float, rejecting non-finite values loudly.
  fn arg_time( cmd : &VerifiedCommand ) -> Result< f32, ErrorData >
  {
    let value = match cmd.arguments.get( "time" )
    {
      Some( Value::Float( number ) ) => *number,
      Some( Value::Integer( number ) ) => *number as f64,
      _ => 0.0,
    };
    if !value.is_finite()
    {
      return Err( error_report
      (
        1,
        ErrorCode::ValidationRuleFailed,
        format!( "invalid `time` value: `{value}` (allowed: a finite number)" ),
      ));
    }
    Ok( value as f32 )
  }

  fn cmd_render( binary : &str ) -> ( CommandDefinition, CommandRoutine )
  {
    let def = CommandDefinition::former()
    .name( ".render" )
    .namespace( String::new() )
    .description( "Render a chunk to a static PNG on a headless GPU: one frame of the same naga-validated bundle the browser preview shows, parameters at their defaults.".to_string() )
    .hint( "one-frame headless PNG render of one chunk" )
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
      format!( "{binary} render fbm3" ),
      format!( "{binary} render fbm3 out::fbm3_far.png size::512 time::2.5" ),
      format!( "{binary} render file::shader/my_chunk.wgsl size::128x64" ),
    ])
    .arguments( vec!
    [
      ArgumentDefinition::former()
      .name( "name" )
      .kind( Kind::String )
      .hint( "Bundled chunk name (see `list`); omit when passing `file::`." )
      .attributes( ArgumentAttributes { optional : true, ..ArgumentAttributes::default() } )
      .end(),
      named_arg( "file", Kind::String, "Path to a local `.wgsl` chunk file (manifest header included), instead of a bundled name.", None ),
      named_arg( "out", Kind::String, "Output PNG path; default `<target>.png` in the current directory.", None ),
      named_arg( "size", Kind::String, "Output size in pixels: `<n>` (square) or `<width>x<height>`.", Some( "256".to_string() ) ),
      named_arg( "time", Kind::Float, "Value of the bundle's `time` uniform for this frame.", Some( "0".to_string() ) ),
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
          "render needs exactly one target: a chunk name (see `list`) or `file::<path>`".to_string(),
        )),
      };
      let size = size_parse( &arg_string( &cmd, "size" ).unwrap_or_else( || "256".to_string() ) )
      .map_err( | err | render_cli_error( &err ) )?;
      let time = arg_time( &cmd )?;
      let out = out_path_of( &target, arg_string( &cmd, "out" ) );
      let content = render_to_png( &target, size, time, &out ).map_err( | err | render_cli_error( &err ) )?;
      Ok( text_output( content ) )
    });

    ( def, routine )
  }

  /// This utility's command set — the single `render` command — with
  /// example invocations spelled against `binary`.
  #[ must_use ]
  pub fn commands( binary : &str ) -> CommandSet
  {
    vec![ cmd_render( binary ) ]
  }

  /// This utility's help-screen group: `Render` — same name and
  /// membership as `docs/cli/command_group/` documents for the aggregator.
  #[ must_use ]
  pub fn help_groups() -> Vec< CommandGroup >
  {
    vec!
    [
      CommandGroup
      {
        name : "Render".to_string(),
        entries : vec!
        [
          CommandEntry { name : "render [name]".to_string(), desc : "Render a chunk headlessly to a static PNG, parameters at defaults.".to_string() },
        ],
      },
    ]
  }

  /// This utility's help-screen example invocations, spelled against
  /// `binary`.
  #[ must_use ]
  pub fn help_examples( binary : &str ) -> Vec< ExampleEntry >
  {
    vec![ ExampleEntry { invocation : format!( "{binary} render fbm3" ), desc : None } ]
  }

  /// Standalone entry point for the `shader_chunks_render` binary.
  pub fn run()
  {
    shader_chunks_cli_core::run( CliApp
    {
      binary : BINARY.to_string(),
      tagline : "Render shader chunks to static PNG images on a headless GPU.".to_string(),
      groups : help_groups(),
      examples : help_examples( BINARY ),
      commands : commands( BINARY ),
    });
  }
}

::mod_interface::mod_interface!
{
  own use BINARY;
  own use RenderCliError;
  own use size_parse;
  own use out_path_of;
  own use summary;
  own use render_to_png;
  own use commands;
  own use help_groups;
  own use help_examples;
  own use run;
}
