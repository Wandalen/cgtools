//! Render utility CLI: the `render` command. Reuses
//! [`shader_chunks_preview`]'s `bundle_prepare` — the same target
//! resolution ( bundled name or local `file::` ) and the same naga
//! validation the live preview runs — then renders one frame of the
//! bundle on a headless GPU via [`shader_chunks_render_core`] and writes
//! it as a PNG. Every bundle parameter takes its initial ( slider-start )
//! value unless overridden via `set::`, so the written image is exactly
//! what the browser preview shows before anyone touches a slider — or,
//! with overrides applied, what it would show mid-drag — frozen at the
//! requested `time::`. Unlike `.preview`, nothing here needs a browser, a
//! dev server, or the web runner crate — the side effect is one image
//! file at `out::`. `all::1` sweeps every bundled chunk in one pass
//! instead of one target, skipping ( not failing ) chunks whose shape
//! isn't previewable and writing `<out>/<name>.png` per chunk, creating
//! `<out>` first if it doesn't already exist.

mod private
{
  use core::fmt;
  use std::path::{ Path, PathBuf };
  use unilang::prelude::*;
  use cli_fmt::prelude::*;
  use shader_chunks_cli_core::{ CliApp, CommandSet, arg_bool_checked, arg_list, arg_string_checked, error_report, named_arg, stdout_print, text_output };
  use shader_chunks_preview::{ PreviewCliError, PreviewTarget, bundle_prepare };
  use shader_chunks_preview_core::{ PreviewBundle, PreviewError };
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
    /// A `set::` override token has no `:` separator, or its value side
    /// does not parse as a finite number.
    InvalidOverride( String ),
    /// A `set::` override's property name matches none of this bundle's
    /// declared parameters.
    UnknownOverrideParameter
    {
      /// The offending override's property name.
      name : String,
      /// Every property this bundle actually declares.
      valid : Vec< String >,
    },
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
        Self::InvalidOverride( raw ) =>
        write!( f, "invalid `set` override: `{raw}` (allowed: `<property>:<finite number>`)" ),
        Self::UnknownOverrideParameter { name, valid } =>
        write!( f, "unknown parameter: `{name}` (valid parameters: {})", valid.join( ", " ) ),
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
        Self::InvalidSize( _ ) | Self::InvalidOverride( _ ) | Self::UnknownOverrideParameter { .. } => 1,
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
  /// values baked into the frame ( defaults, or `set::`-overridden ).
  #[ must_use ]
  pub fn summary( bundle : &PreviewBundle, size : ( u32, u32 ), time : f32, written_to : &Path ) -> String
  {
    let mut lines = vec!
    [
      format!( "wrote {} ({}x{} px, naga-validated)", written_to.display(), size.0, size.1 ),
      format!( "target: {}", bundle.target ),
      format!( "time: {time}" ),
      "parameters:".to_string(),
    ];
    for param in &bundle.parameters
    {
      lines.push( format!( "  {} = {}", param.property, param.value ) );
    }
    lines.join( "\n" )
  }

  /// Parses `set::` override tokens — each `<property>:<value>` — into
  /// `(property, value)` pairs, preserving order. A later token overriding
  /// the same property as an earlier one is not deduplicated here; whoever
  /// applies the pairs decides how that's resolved ( see
  /// [`overrides_apply`], which applies in order so the last one wins ).
  ///
  /// # Errors
  ///
  /// Returns [`RenderCliError::InvalidOverride`] for a token missing its
  /// `:` separator, or whose value side does not parse as a finite `f64`.
  pub fn overrides_parse( raw : &[ String ] ) -> Result< Vec< ( String, f64 ) >, RenderCliError >
  {
    raw.iter().map( | token |
    {
      let ( property, value_str ) = token.split_once( ':' )
      .ok_or_else( || RenderCliError::InvalidOverride( token.clone() ) )?;
      let value : f64 = value_str.trim().parse().ok().filter( | value : &f64 | value.is_finite() )
      .ok_or_else( || RenderCliError::InvalidOverride( token.clone() ) )?;
      Ok( ( property.to_string(), value ) )
    }).collect()
  }

  /// Applies parsed `set::` overrides onto `bundle.parameters` in place,
  /// matching each override to a parameter by `property` name. Overrides
  /// are applied in order, so a later override of an already-overridden
  /// property wins. Values are baked in as-is — never clamped to the
  /// parameter's `min`/`max`, which describe the browser slider's UI
  /// range, not a hard constraint on the underlying uniform.
  ///
  /// # Errors
  ///
  /// Returns [`RenderCliError::UnknownOverrideParameter`], naming the
  /// offending property and every property this bundle actually declares,
  /// the moment an override's property matches none of them.
  pub fn overrides_apply( bundle : &mut PreviewBundle, overrides : &[ ( String, f64 ) ] ) -> Result< (), RenderCliError >
  {
    for ( property, value ) in overrides
    {
      if let Some( param ) = bundle.parameters.iter_mut().find( | p | &p.property == property )
      {
        param.value = *value;
      }
      else
      {
        let valid = bundle.parameters.iter().map( | p | p.property.clone() ).collect();
        return Err( RenderCliError::UnknownOverrideParameter { name : property.clone(), valid } );
      }
    }
    Ok( () )
  }

  /// The whole `render` command: build and naga-validate the bundle
  /// ( via [`shader_chunks_preview::bundle_prepare`] ), apply `set::`
  /// overrides ( if any — see [`overrides_apply`] ), render one frame
  /// headlessly, write it as a PNG at `out`, and return the summary.
  ///
  /// # Errors
  ///
  /// Every [`RenderCliError`] variant except `InvalidSize` ( the caller
  /// parses `size::` first, via [`size_parse`] ) and `InvalidOverride`
  /// ( the caller parses `set::` first, via [`overrides_parse`] ).
  pub fn render_to_png( target : &PreviewTarget, size : ( u32, u32 ), time : f32, overrides : &[ ( String, f64 ) ], out : &Path )
  -> Result< String, RenderCliError >
  {
    let mut bundle = bundle_prepare( target ).map_err( RenderCliError::Preview )?;
    overrides_apply( &mut bundle, overrides )?;
    let image = shader_chunks_render_core::render( &bundle, size, time ).map_err( RenderCliError::Render )?;
    let ( width, height ) = image.size;
    image::save_buffer( out, &image.pixels, width, height, image::ColorType::Rgba8 )
    .map_err( | err | RenderCliError::Io( format!( "writing `{}`: {err}", out.display() ) ) )?;
    Ok( summary( &bundle, size, time, out ) )
  }

  /// One chunk's outcome from a batch render pass ( see
  /// [`render_all_to_png`] ).
  #[ derive( Debug ) ]
  pub enum BatchOutcome
  {
    /// Rendered and written successfully, to this path.
    Rendered
    {
      /// The chunk's name.
      name : String,
      /// Where its PNG was written.
      path : PathBuf,
    },
    /// Not previewable — see
    /// [`shader_chunks_preview_core::PreviewError::Unpreviewable`]. Not a
    /// failure: this is the expected outcome for a chunk whose exports
    /// don't fit either previewable shape ( e.g. an entry-point struct or
    /// a helper returning something other than `f32`/`vec2f`/`vec3f`/`vec4f` ).
    Skipped
    {
      /// The chunk's name.
      name : String,
      /// Why it isn't previewable.
      reason : String,
    },
    /// Every other [`RenderCliError`] — naga validation, GPU, or io
    /// failure. Does not stop the batch, but flips [`batch_summary`]'s
    /// caller toward a non-zero exit.
    Failed
    {
      /// The chunk's name.
      name : String,
      /// The underlying error.
      error : RenderCliError,
    },
  }

  /// Renders every bundled chunk ( [`shader_chunks_core::CHUNKS`] ), each
  /// to `<out_dir>/<name>.png`, at the given `size`/`time` ( no `set::`
  /// overrides — a single override list can't cleanly apply across
  /// chunks with different declared parameters ). Creates `out_dir` if it
  /// doesn't already exist. A chunk whose shape isn't previewable is
  /// [`BatchOutcome::Skipped`], not a failure; every other error is
  /// [`BatchOutcome::Failed`] and does not stop the batch.
  ///
  /// # Errors
  ///
  /// Returns [`RenderCliError::Io`] if `out_dir` doesn't exist and can't
  /// be created — the one failure that stops the whole batch before it
  /// starts, since no chunk could be written anyway.
  pub fn render_all_to_png( size : ( u32, u32 ), time : f32, out_dir : &Path ) -> Result< Vec< BatchOutcome >, RenderCliError >
  {
    std::fs::create_dir_all( out_dir )
    .map_err( | err | RenderCliError::Io( format!( "creating `{}`: {err}", out_dir.display() ) ) )?;
    Ok( shader_chunks_core::CHUNKS.iter().map( | chunk |
    {
      let name = chunk.name.to_string();
      let target = PreviewTarget::Name( name.clone() );
      let out = out_dir.join( format!( "{name}.png" ) );
      match render_to_png( &target, size, time, &[], &out )
      {
        Ok( _ ) => BatchOutcome::Rendered { name, path : out },
        Err( RenderCliError::Preview( PreviewCliError::Preview( PreviewError::Unpreviewable { reason, .. } ) ) ) =>
        BatchOutcome::Skipped { name, reason },
        Err( error ) => BatchOutcome::Failed { name, error },
      }
    }).collect() )
  }

  /// Human-readable batch report: one line per chunk, then a totals line.
  #[ must_use ]
  pub fn batch_summary( outcomes : &[ BatchOutcome ] ) -> String
  {
    let mut lines : Vec< String > = outcomes.iter().map( | outcome | match outcome
    {
      BatchOutcome::Rendered { name, path } => format!( "{name}: wrote {}", path.display() ),
      BatchOutcome::Skipped { name, reason } => format!( "{name}: skipped ({reason})" ),
      BatchOutcome::Failed { name, error } => format!( "{name}: failed ({error})" ),
    }).collect();
    let rendered = outcomes.iter().filter( | o | matches!( o, BatchOutcome::Rendered { .. } ) ).count();
    let skipped = outcomes.iter().filter( | o | matches!( o, BatchOutcome::Skipped { .. } ) ).count();
    let failed = outcomes.iter().filter( | o | matches!( o, BatchOutcome::Failed { .. } ) ).count();
    lines.push( format!( "{} chunks: {rendered} rendered, {skipped} skipped, {failed} failed", outcomes.len() ) );
    lines.join( "\n" )
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
    .description( "Render a chunk to a static PNG on a headless GPU: one frame of the same naga-validated bundle the browser preview shows, parameters at their defaults unless overridden via `set::`. With `all::1`, renders every previewable chunk instead of one target.".to_string() )
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
      format!( "{binary} render fbm3 set::lacunarity:2.5,gain:0.75" ),
      format!( "{binary} render all::1 out::renders/ size::128" ),
    ])
    .arguments( vec!
    [
      ArgumentDefinition::former()
      .name( "name" )
      .kind( Kind::String )
      .hint( "Bundled chunk name (see `list`); omit when passing `file::` or `all::1`." )
      .attributes( ArgumentAttributes { optional : true, ..ArgumentAttributes::default() } )
      .end(),
      named_arg( "file", Kind::String, "Path to a local `.wgsl` chunk file (manifest header included), instead of a bundled name.", None ),
      named_arg( "out", Kind::String, "Output PNG path; default `<target>.png` in the current directory. With `all::1`, the output DIRECTORY instead (default: current directory), created if it doesn't exist — each chunk writes `<dir>/<name>.png`.", None ),
      named_arg( "size", Kind::String, "Output size in pixels: `<n>` (square) or `<width>x<height>`.", Some( "256".to_string() ) ),
      named_arg( "time", Kind::Float, "Value of the bundle's `time` uniform for this frame.", Some( "0".to_string() ) ),
      named_arg( "set", Kind::List( Box::new( Kind::String ), Some( ',' ) ), "Parameter overrides, comma-separated `property:value` pairs (see `tunables` for a chunk's property names). Not usable with `all::1`.", None ),
      named_arg( "all", Kind::Boolean, "Render every previewable chunk instead of one target; cannot be combined with `name`, `file::`, or `set::`.", Some( "false".to_string() ) ),
    ])
    .end();

    // Fix(BUG-285): every `arg_string`/`arg_bool` call in this routine
    // switched to `arg_string_checked`/`arg_bool_checked`. Root cause: same
    // defect class as BUG-283 (`shader_chunks_cli_core`'s catch-all `Value`
    // match arms cannot tell "argument absent" apart from "argument
    // supplied twice"); BUG-283 fixed `shader_chunks_compose` only. Pitfall:
    // see the matching comment in `shader_chunks_query/src/lib.rs`.
    let routine : CommandRoutine = Box::new( | cmd, _ctx |
    {
      let name = arg_string_checked( &cmd, "name" )?;
      let file = arg_string_checked( &cmd, "file" )?;
      let set_tokens = arg_list( &cmd, "set" );
      if arg_bool_checked( &cmd, "all", false )?
      {
        if name.is_some() || file.is_some() || !set_tokens.is_empty()
        {
          return Err( error_report
          (
            1,
            ErrorCode::ValidationRuleFailed,
            "render `all::1` renders every chunk and cannot be combined with a target (`name`/`file::`) or `set::`".to_string(),
          ));
        }
        let size = size_parse( &arg_string_checked( &cmd, "size" )?.unwrap_or_else( || "256".to_string() ) )
        .map_err( | err | render_cli_error( &err ) )?;
        let time = arg_time( &cmd )?;
        let out_dir = PathBuf::from( arg_string_checked( &cmd, "out" )?.unwrap_or_else( || ".".to_string() ) );
        let outcomes = render_all_to_png( size, time, &out_dir ).map_err( | err | render_cli_error( &err ) )?;
        let failed = outcomes.iter().filter( | o | matches!( o, BatchOutcome::Failed { .. } ) ).count();
        stdout_print( &batch_summary( &outcomes ) );
        if failed > 0
        {
          return Err( error_report
          (
            1,
            ErrorCode::ValidationRuleFailed,
            format!( "{failed} chunk(s) failed to render" ),
          ));
        }
        return Ok( text_output( String::new() ) );
      }
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
      let size = size_parse( &arg_string_checked( &cmd, "size" )?.unwrap_or_else( || "256".to_string() ) )
      .map_err( | err | render_cli_error( &err ) )?;
      let time = arg_time( &cmd )?;
      let overrides = overrides_parse( &set_tokens ).map_err( | err | render_cli_error( &err ) )?;
      let out = out_path_of( &target, arg_string_checked( &cmd, "out" )? );
      let content = render_to_png( &target, size, time, &overrides, &out ).map_err( | err | render_cli_error( &err ) )?;
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
          CommandEntry { name : "render [name]".to_string(), desc : "Render a chunk headlessly to a static PNG (or every chunk with `all::1`), parameters at defaults unless overridden.".to_string() },
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
  own use overrides_parse;
  own use overrides_apply;
  own use render_to_png;
  own use BatchOutcome;
  own use render_all_to_png;
  own use batch_summary;
  own use commands;
  own use help_groups;
  own use help_examples;
  own use run;
}
