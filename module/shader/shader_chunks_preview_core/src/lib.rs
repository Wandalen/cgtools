//! Builds a self-contained *preview bundle* — composed WGSL plus a slider
//! parameter list — from one target shader chunk. The bundle is the whole
//! interface between the native `shader_chunks_preview` CLI ( which builds,
//! validates, and serializes it ) and the `shader_chunks_preview_web`
//! browser runner ( which deserializes it and renders: one slider per
//! parameter, one uniform buffer laid out by the convention below ). Pure
//! text processing over `shader_chunks_core`'s manifests and
//! `shader_chunks_params_core`'s `//@ param:` discovery — no I/O, no
//! graphics API, wasm-clean.
//!
//! Two target modes, selected from the target's own manifest:
//!
//! - **Fragment chunk** ( `//@ stage: fragment` ): used directly as the
//!   preview's fragment stage. Must export entry point `fs_main`, and its
//!   `//@ param:` lines ( each `uniform f32` ) become the sliders. Its own
//!   uniform struct must follow the layout convention.
//! - **Value chunk** ( any chunk exporting `fn NAME(p: vec2f) -> T` for
//!   `T` in `f32`/`vec2f`/`vec3f` ): a fragment harness is synthesized
//!   around the export — aspect-corrected, slowly time-drifting, written
//!   out as grayscale ( `f32` ), blue-padded 2-channel ( `vec2f` ), or
//!   direct RGB ( `vec3f` ) — with one synthesized `preview_scale` slider.
//!   No rescaling is applied regardless of shape: the raw value is written
//!   and clamped to `[0, 1]` by the render target, same as an unbounded
//!   SDF value already is in the `f32` case.
//!
//! **Uniform layout convention** ( what the browser runner writes, and what
//! a fragment-mode chunk's own `struct Params` must therefore declare ):
//! `time : f32` first, then each `//@ param:` uniform as `f32` in
//! declaration order, then `resolution : vec4f` ( `.xy` = physical pixels )
//! — WGSL's own struct rules place `resolution` at the next 16-byte
//! boundary, and the runner pads its written buffer identically.

mod private
{
  use core::fmt;
  use serde::{ Deserialize, Serialize };
  use shader_chunks_core::
  {
    ChunkDescriptor, ComposeError, ResolveError, depends_on_parse, exports_parse, name_parse,
    set_resolve, stage_parse, try_compose,
  };
  use shader_chunks_params_core::{ Parameter, ParameterKind, ValueType, discover };

  /// One slider the browser runner creates, mirroring `controls.js`'s
  /// `addSlider(label, property, value, min, max, step)` signature. Order
  /// within [`PreviewBundle::parameters`] is uniform-struct field order —
  /// the runner writes slider values into the uniform buffer by index.
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub struct PreviewParameter
  {
    /// Human-readable slider label ( `"Noise scale"` ).
    pub label : String,
    /// The parameter's declared name — uniform field name and the key the
    /// runner's change callback receives ( `"noise_scale"` ).
    pub property : String,
    /// Initial slider value.
    pub value : f64,
    /// Slider minimum.
    pub min : f64,
    /// Slider maximum.
    pub max : f64,
    /// Slider step.
    pub step : f64,
  }

  /// A self-contained preview: the composed WGSL text ( vertex stage,
  /// dependencies, and fragment stage — everything the browser compiles )
  /// plus the slider parameters driving its uniform buffer.
  #[ derive( Debug, Clone, PartialEq, Serialize, Deserialize ) ]
  pub struct PreviewBundle
  {
    /// The previewed chunk's manifest name.
    pub target : String,
    /// Composed, dependency-ordered WGSL for the whole render pipeline.
    pub wgsl : String,
    /// Sliders, in uniform-struct field order ( see the layout convention
    /// in the crate docs ).
    pub parameters : Vec< PreviewParameter >,
  }

  /// Error returned by [`bundle_build`].
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub enum PreviewError
  {
    /// A `depends_on` name ( direct or transitive ) is not bundled in
    /// [`shader_chunks_core::CHUNKS`].
    UnknownChunk( String ),
    /// The target chunk offers nothing this preview knows how to render.
    Unpreviewable
    {
      /// The target chunk's name ( or `(unnamed chunk)` when the manifest
      /// itself is missing ).
      chunk : String,
      /// Why no preview can be built from it.
      reason : String,
    },
    /// A declared `//@ param:` cannot be wired into the preview's uniform
    /// convention.
    UnsupportedParam
    {
      /// The target chunk's name.
      chunk : String,
      /// The offending parameter's name.
      param : String,
      /// Why it cannot be wired.
      reason : String,
    },
    /// Composition of the assembled chunk set failed.
    Compose( ComposeError ),
  }

  impl fmt::Display for PreviewError
  {
    fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
    {
      match self
      {
        Self::UnknownChunk( name ) => write!( f, "unknown chunk: `{name}` (see `list` for valid names)" ),
        Self::Unpreviewable { chunk, reason } => write!( f, "chunk `{chunk}` is not previewable: {reason}" ),
        Self::UnsupportedParam { chunk, param, reason } =>
        write!( f, "chunk `{chunk}` parameter `{param}` is not previewable: {reason}" ),
        Self::Compose( err ) => write!( f, "{err}" ),
      }
    }
  }

  impl std::error::Error for PreviewError {}

  /// Which previewable value-function shape an export matches — controls
  /// how [`harness_synthesize`] writes the sampled value to the render
  /// target.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  enum ValueFnKind
  {
    /// `fn NAME(p: vec2f) -> f32` — written as grayscale.
    F32,
    /// `fn NAME(p: vec2f) -> vec2f` — written with a fixed blue pad.
    Vec2,
    /// `fn NAME(p: vec2f) -> vec3f` — written directly as RGB.
    Vec3,
  }

  impl ValueFnKind
  {
    /// The composed harness's final `vec4f` write-out expression for this
    /// shape. No rescaling is applied for any shape — the raw value is
    /// written and clamped to `[0, 1]` by the render target, same
    /// convention the `F32` shape already used for unbounded SDF values.
    fn write_expr( self ) -> &'static str
    {
      match self
      {
        Self::F32 => "vec4f( vec3f( value ), 1.0 )",
        Self::Vec2 => "vec4f( value, 0.5, 1.0 )",
        Self::Vec3 => "vec4f( value, 1.0 )",
      }
    }

    /// Short label for the harness's synthesized `//@ description:` line.
    fn describe( self ) -> &'static str
    {
      match self
      {
        Self::F32 => "grayscale",
        Self::Vec2 => "2-channel (blue-padded)",
        Self::Vec3 => "RGB",
      }
    }
  }

  /// Extracts the exported symbol name from a value-function export
  /// signature of a previewable shape — one `vec2f` argument, and a
  /// return type of `f32`, `vec2f`, or `vec3f` ( see [`ValueFnKind`] ).
  /// Anything else ( other arities, other types, structs, entry points )
  /// returns `None`.
  fn value_fn_of( export : &str ) -> Option< ( &str, ValueFnKind ) >
  {
    let rest = export.trim().strip_prefix( "fn " )?;
    let open = rest.find( '(' )?;
    let close = rest.find( ')' )?;
    if close < open
    {
      return None;
    }
    let name = rest[ ..open ].trim();
    let args = &rest[ open + 1..close ];
    let return_ty = rest[ close + 1.. ].trim().strip_prefix( "->" )?.trim();
    let kind = match return_ty
    {
      "f32" => ValueFnKind::F32,
      "vec2f" => ValueFnKind::Vec2,
      "vec3f" => ValueFnKind::Vec3,
      _ => return None,
    };
    if name.is_empty()
    {
      return None;
    }
    let mut parts = args.split( ',' );
    let first = parts.next()?.trim();
    if parts.next().is_some()
    {
      return None;
    }
    let ( _, arg_type ) = first.rsplit_once( ':' )?;
    if arg_type.trim() != "vec2f"
    {
      return None;
    }
    Some( ( name, kind ) )
  }

  /// `"noise_scale"` → `"Noise scale"`: slider label from a parameter name.
  fn label_of( property : &str ) -> String
  {
    let spaced = property.replace( '_', " " );
    let mut chars = spaced.chars();
    match chars.next()
    {
      Some( first ) => first.to_uppercase().collect::< String >() + chars.as_str(),
      None => spaced,
    }
  }

  /// The synthesized fragment harness for a value chunk: samples
  /// `value_fn` from `target` over an aspect-corrected, slowly-drifting
  /// plane and writes the result per `kind` ( see [`ValueFnKind`] ).
  /// Carries its own `//@` manifest ( so raw-text composition orders it
  /// correctly ) and its own synthesized `preview_scale` parameter.
  fn harness_synthesize( target : &str, value_fn : &str, kind : ValueFnKind ) -> String
  {
    let write_expr = kind.write_expr();
    let shape = kind.describe();
    format!( r"//@ name: preview_harness
//@ description: Synthesized preview harness rendering `{value_fn}` from chunk `{target}` as a time-drifting {shape} field.
//@ tags: category:preview
//@ stage: fragment
//@ depends_on: {target}, fullscreen_triangle
//@ export: fn fs_main(in: VertexOutput) -> @location(0) vec4f
//@ param: preview_scale uniform f32 range(1.0, 32.0)

// Synthesized by shader_chunks_preview_core::bundle_build for a value
// chunk: no hand-written WGSL corresponds to this text.

struct Params
{{
  time : f32,
  preview_scale : f32,
  resolution : vec4f, // .xy = physical pixels, .zw unused
}}

@group( 0 ) @binding( 0 ) var< uniform > params : Params;

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4f
{{
  let aspect = params.resolution.x / max( params.resolution.y, 1.0 );
  let q = ( in.uv - vec2f( 0.5, 0.5 ) ) * vec2f( aspect, 1.0 );
  let p = q * params.preview_scale + vec2f( params.time * 0.05, 0.0 );
  let value = {value_fn}( p );
  return {write_expr};
}}
" )
  }

  /// Index ( in `f32` units ) where `resolution : vec4f` begins in the
  /// uniform buffer for a bundle with `param_count` sliders: `time` and the
  /// params occupy indices `0..=param_count`, then WGSL's struct rules align
  /// `vec4f` to the next 16-byte boundary. The buffer's total length is
  /// `resolution_index + 4` floats — the browser runner writes exactly this
  /// layout, and a fragment-mode chunk's own `struct Params` must match it.
  #[ must_use ]
  pub const fn resolution_index( param_count : usize ) -> usize
  {
    ( param_count + 1 ).div_ceil( 4 ) * 4
  }

  /// The synthesized `preview_scale` slider every value-chunk preview
  /// carries — mirrors `harness_synthesize`'s own `//@ param:` line.
  fn preview_scale_parameter() -> PreviewParameter
  {
    PreviewParameter
    {
      label : "Preview scale".to_string(),
      property : "preview_scale".to_string(),
      value : 8.0,
      min : 1.0,
      max : 32.0,
      step : 0.1,
    }
  }

  /// Converts one discovered `//@ param:` into its slider, enforcing the
  /// preview's uniform convention: kind `uniform`, type `f32`, resolvable
  /// range. Initial value is the range midpoint; step is 1/200 of the span.
  fn slider_of( chunk : &str, param : &Parameter ) -> Result< PreviewParameter, PreviewError >
  {
    if param.kind != ParameterKind::Uniform
    {
      return Err( PreviewError::UnsupportedParam
      {
        chunk : chunk.to_string(),
        param : param.name.clone(),
        reason : format!( "kind `{:?}` cannot back a live slider — only `uniform` parameters are wired into the preview's uniform buffer", param.kind ),
      });
    }
    if param.value_type != ValueType::F32
    {
      return Err( PreviewError::UnsupportedParam
      {
        chunk : chunk.to_string(),
        param : param.name.clone(),
        reason : format!( "type `{:?}` is not supported — the preview's uniform convention packs sliders as consecutive `f32` fields", param.value_type ),
      });
    }
    let Some( ( range, _source ) ) = param.range else
    {
      return Err( PreviewError::UnsupportedParam
      {
        chunk : chunk.to_string(),
        param : param.name.clone(),
        reason : "no declared or inferable range".to_string(),
      });
    };
    Ok( PreviewParameter
    {
      label : label_of( &param.name ),
      property : param.name.clone(),
      value : f64::midpoint( range.min, range.max ),
      min : range.min,
      max : range.max,
      step : ( range.max - range.min ) / 200.0,
    })
  }

  /// Builds a [`PreviewBundle`] from one target chunk's raw WGSL text
  /// ( manifest included ) — a bundled chunk's `.wgsl` field or a local
  /// file's content; both modes ( fragment chunk / value chunk ) and both
  /// sources go through this one path. Dependencies are resolved against
  /// the bundled registry ( transitively ); a vertex stage
  /// ( `fullscreen_triangle` ) is pulled in automatically when the set
  /// doesn't already contain one.
  ///
  /// # Errors
  ///
  /// - [`PreviewError::Unpreviewable`] — missing manifest lines, no
  ///   previewable export, or a fragment chunk without `fs_main` /
  ///   without at least one `//@ param:` uniform.
  /// - [`PreviewError::UnknownChunk`] — a dependency name not bundled.
  /// - [`PreviewError::UnsupportedParam`] — a `//@ param:` outside the
  ///   `uniform f32` convention.
  /// - [`PreviewError::Compose`] — the assembled set fails composition.
  pub fn bundle_build( target_wgsl : &str ) -> Result< PreviewBundle, PreviewError >
  {
    for required in [ "name", "depends_on" ]
    {
      let prefix = format!( "//@ {required}:" );
      if !target_wgsl.lines().any( | line | line.starts_with( prefix.as_str() ) )
      {
        return Err( PreviewError::Unpreviewable
        {
          chunk : "(unnamed chunk)".to_string(),
          reason : format!( "missing required `//@ {required}:` manifest line" ),
        });
      }
    }

    let name = name_parse( target_wgsl );
    let deps = depends_on_parse( target_wgsl );
    let stage = stage_parse( target_wgsl );
    let exports = exports_parse( target_wgsl );

    let resolve = | names : &[ &str ] | set_resolve( names, true )
    .map_err( | ResolveError::UnknownChunk( missing ) | PreviewError::UnknownChunk( missing ) );

    let mut selected : Vec< &'static ChunkDescriptor > = resolve( &deps )?;

    let mut texts : Vec< &str > = Vec::new();
    let harness;
    let parameters;

    if stage == Some( "fragment" )
    {
      if !exports.iter().any( | export | export.contains( "fn fs_main(" ) )
      {
        return Err( PreviewError::Unpreviewable
        {
          chunk : name.to_string(),
          reason : "a fragment chunk must export entry point `fs_main` for the preview pipeline to target it".to_string(),
        });
      }
      let discovered = discover( target_wgsl );
      if discovered.is_empty()
      {
        return Err( PreviewError::Unpreviewable
        {
          chunk : name.to_string(),
          reason : "a fragment chunk must declare at least one `//@ param:` uniform — the preview drives the `time`/params/`resolution` uniform convention and has nothing to wire".to_string(),
        });
      }
      parameters = discovered.iter()
      .map( | param | slider_of( name, param ) )
      .collect::< Result< Vec< _ >, _ > >()?;
      harness = None;
    }
    else
    {
      // A value chunk: prefer the export named like the chunk itself, fall
      // back to the first previewable export ( in file order ), regardless
      // of which ValueFnKind either one is — no shape preference.
      let candidates : Vec< ( &str, ValueFnKind ) > = exports.iter().filter_map( | export | value_fn_of( export ) ).collect();
      let ( value_fn, kind ) = candidates.iter().copied()
      .find( | &( found, _ ) | found == name )
      .or_else( || candidates.first().copied() )
      .ok_or_else( || PreviewError::Unpreviewable
      {
        chunk : name.to_string(),
        reason : format!
        (
          "exports contain neither a fragment entry point nor a `fn NAME(p: vec2f) -> f32|vec2f|vec3f` value function; exports: [{}]",
          exports.join( "; " )
        ),
      })?;
      let discovered = discover( target_wgsl );
      if let Some( param ) = discovered.first()
      {
        return Err( PreviewError::UnsupportedParam
        {
          chunk : name.to_string(),
          param : param.name.clone(),
          reason : "the synthesized harness owns the preview's uniform struct; a value chunk's own `//@ param:` declarations are not wired into it".to_string(),
        });
      }
      harness = Some( harness_synthesize( name, value_fn, kind ) );
      parameters = vec![ preview_scale_parameter() ];
    }

    // Ensure the set carries a vertex stage: value-chunk harnesses always
    // depend on `fullscreen_triangle`; a fragment chunk normally names it in
    // `depends_on` already, but is completed here when it doesn't.
    let target_is_vertex = stage == Some( "vertex" );
    if !target_is_vertex && !selected.iter().any( | chunk | chunk.stage == Some( "vertex" ) )
    {
      for chunk in resolve( &[ "fullscreen_triangle" ] )?
      {
        if !selected.iter().any( | present | present.name == chunk.name )
        {
          selected.push( chunk );
        }
      }
    }

    texts.extend( selected.iter().map( | chunk | chunk.wgsl ) );
    texts.push( target_wgsl );
    if let Some( harness ) = &harness
    {
      texts.push( harness );
    }

    let wgsl = try_compose( &texts ).map_err( PreviewError::Compose )?;

    Ok( PreviewBundle { target : name.to_string(), wgsl, parameters } )
  }
}

::mod_interface::mod_interface!
{
  own use PreviewParameter;
  own use PreviewBundle;
  own use PreviewError;
  own use bundle_build;
  own use resolution_index;
}
