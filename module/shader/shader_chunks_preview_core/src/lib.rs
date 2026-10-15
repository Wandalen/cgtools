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
//! - **Value chunk** ( any chunk exporting `fn NAME(p: vec2f, ...) -> T` for
//!   `T` in `f32`/`vec2f`/`vec3f`, with zero or more trailing `f32`
//!   arguments ): a fragment harness is synthesized around the export. Each
//!   trailing argument becomes a real slider when the chunk also declares a
//!   matching `//@ param: NAME argument f32 range(min, max)` line — the
//!   harness reads it from its own uniform buffer and passes it positionally
//!   into the export, so the target chunk's own code never touches a
//!   uniform directly ( see `harness_synthesize`'s doc comment ). A
//!   synthesized `preview_scale` slider is always added on top. The harness
//!   also overlays a world-space reference grid ( unit-spaced minor lines,
//!   emphasized axes through the origin ) on every shape so the previewed
//!   region's scale and center stay legible. A `category:sdf`-tagged
//!   chunk's `f32` shape gets a filled-inside / distance-banded-outside /
//!   crisp-zero-isoline treatment with a stationary sample point — a raw
//!   value clamped straight to `[0, 1]` blurs out fine geometric detail
//!   ( e.g. corner rounding ), and an unbounded time-driven pan eventually
//!   drifts a finite shape out of frame. Every other shape/tag combination
//!   keeps the original convention: aspect-corrected, slowly time-drifting,
//!   the raw value written and clamped to `[0, 1]` by the render target —
//!   grayscale ( `f32` ), blue-padded 2-channel ( `vec2f` ), or direct RGB
//!   ( `vec3f` ).
//!
//! Composed WGSL is banner-commented per section — `// ==== dependency
//! chunk: NAME ====`, `// ==== previewing: NAME ... ====`, `// ====
//! auto-generated preview harness ... ====` — so the concatenated text
//! ( what both `render` and the live browser editor show ) makes clear
//! which part is a dependency, which part is the chunk under preview, and
//! which part ( if any ) is synthesized scaffolding with no hand-written
//! counterpart.
//!
//! **Uniform layout convention** ( what the browser runner writes, and what
//! a fragment-mode chunk's own `struct Params` must therefore declare ):
//! `time : f32` first, then each `//@ param:` uniform as `f32` in
//! declaration order, then `resolution : vec4f` ( `.xy` = physical pixels )
//! — WGSL's own struct rules place `resolution` at the next 16-byte
//! boundary, and the runner pads its written buffer identically.

mod private
{
  use core::fmt::{ self, Write };
  use serde::{ Deserialize, Serialize };
  use shader_chunks_core::
  {
    ChunkDescriptor, ComposeError, ResolveError, depends_on_parse, exports_parse, name_parse,
    set_resolve, stage_parse, tags_parse, try_compose,
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
  /// signature of a previewable shape — a first `vec2f` argument ( the
  /// sample point ), zero or more trailing `f32` arguments ( the chunk's
  /// own tunables — see [`bundle_build`]'s value-chunk branch ), and a
  /// return type of `f32`, `vec2f`, or `vec3f` ( see [`ValueFnKind`] ).
  /// Anything else ( a non-`vec2f` first argument, a trailing argument
  /// that isn't `f32`, other return types, structs, entry points )
  /// returns `None`. Trailing argument names are returned in signature
  /// order so [`bundle_build`] can pair each one, positionally, with its
  /// declared `//@ param: NAME argument f32 range(min, max)` line.
  fn value_fn_of( export : &str ) -> Option< ( &str, ValueFnKind, Vec< String > ) >
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
    let ( _, first_type ) = first.rsplit_once( ':' )?;
    if first_type.trim() != "vec2f"
    {
      return None;
    }
    let mut extra_args = Vec::new();
    for part in parts
    {
      let ( arg_name, arg_type ) = part.trim().rsplit_once( ':' )?;
      if arg_type.trim() != "f32"
      {
        return None;
      }
      extra_args.push( arg_name.trim().to_string() );
    }
    Some( ( name, kind, extra_args ) )
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

  /// The synthesized fragment harness for a value chunk: samples `value_fn`
  /// from `target` over an aspect-corrected plane and writes the result per
  /// `kind` ( see [`ValueFnKind`] ), then overlays a world-space reference
  /// grid — unit-spaced minor lines plus emphasized axes through the origin
  /// — so the previewed region's scale and center stay legible regardless
  /// of `preview_scale`.
  ///
  /// `is_sdf` ( true when the target chunk carries the `category:sdf` tag )
  /// switches the `F32` shape to a filled-inside / distance-banded-outside
  /// / crisp-zero-isoline treatment instead of the raw clamped value: an
  /// unbounded signed distance clamped straight to `[0, 1]` blurs out
  /// exactly the fine geometric detail ( e.g. corner rounding ) an SDF
  /// preview exists to show. `is_sdf` also holds the sample point
  /// stationary rather than drifting it with time — a finite-footprint
  /// shape panned by an unbounded time-driven offset eventually drifts out
  /// of frame and stays blank forever, unlike a noise/color field that is
  /// defined ( and worth watching pan ) everywhere. Non-`F32` shapes and
  /// non-SDF `F32` chunks are unaffected by `is_sdf` and keep the original
  /// raw-value write with time drift. Carries its own `//@` manifest ( so
  /// raw-text composition orders it correctly ) and its own synthesized
  /// `preview_scale` parameter.
  ///
  /// `own_params` are the target chunk's own tunables — each one declared
  /// in the target's manifest as `//@ param: NAME argument f32 range(min,
  /// max)` *and* present as a trailing `f32` argument on `value_fn`'s own
  /// signature ( [`bundle_build`]'s value-chunk branch resolves and orders
  /// `own_params` to match that signature, erroring via
  /// [`PreviewError::UnsupportedParam`] on any mismatch ). Each becomes one
  /// extra `struct Params` field and one extra synthesized `//@ param:
  /// ... uniform f32 ...` manifest line — from the harness's own
  /// perspective these genuinely are uniform-buffer fields, since the
  /// harness is what the browser's uniform buffer actually binds to — Ordered
  /// before `preview_scale` ( `time`, then `own_params` in signature order,
  /// then `preview_scale`, then `resolution` — matching
  /// [`PreviewBundle::parameters`]'s own order, since the browser runner
  /// writes the uniform buffer positionally over that list ). The harness's
  /// `fs_main` body reads each `params.NAME` itself and passes it as a
  /// positional trailing argument — `{value_fn}( p, params.NAME1,
  /// params.NAME2, ... )` — so the *target chunk's own* value-function stays
  /// a plain pure function with no uniform access of its own; nothing
  /// depends on WGSL's module-scope declaration order.
  fn harness_synthesize( target : &str, value_fn : &str, kind : ValueFnKind, is_sdf : bool, own_params : &[ PreviewParameter ] ) -> String
  {
    let shape = kind.describe();
    let own_param_manifest = own_params.iter().fold( String::new(), | mut acc, p |
    {
      writeln!( acc, "//@ param: {} uniform f32 range({}, {})", p.property, p.min, p.max ).unwrap();
      acc
    });
    let own_param_fields = own_params.iter().fold( String::new(), | mut acc, p |
    {
      writeln!( acc, "  {} : f32,", p.property ).unwrap();
      acc
    });
    let own_param_call_args = own_params.iter().fold( String::new(), | mut acc, p |
    {
      write!( acc, ", params.{}", p.property ).unwrap();
      acc
    });
    let sdf_suffix = if is_sdf { ", filled/banded/outlined as a signed distance, sample point held stationary" } else { "" };
    let p_expr = if is_sdf
    {
      "q * params.preview_scale"
    }
    else
    {
      "q * params.preview_scale + vec2f( params.time * 0.05, 0.0 )"
    };
    let color_block : String = match ( kind, is_sdf )
    {
      ( ValueFnKind::F32, true ) => "\
  let aa = px * 1.5;
  var color = select( vec3f( 0.92, 0.93, 0.96 ), vec3f( 0.30, 0.55, 0.95 ), value < 0.0 );
  color = color * ( 0.85 + 0.15 * cos( value * 40.0 ) );
  color = mix( color, vec3f( 0.05, 0.05, 0.05 ), 1.0 - smoothstep( 0.0, aa, abs( value ) ) );"
      .to_string(),
      ( ValueFnKind::F32, false ) => "  let color = vec3f( value );".to_string(),
      ( ValueFnKind::Vec2, _ ) => "  let color = vec3f( value, 0.5 );".to_string(),
      ( ValueFnKind::Vec3, _ ) => "  let color = value;".to_string(),
    };
    format!( r"//@ name: preview_harness
//@ description: Synthesized preview harness rendering `{value_fn}` from chunk `{target}` as a {shape} field{sdf_suffix}.
//@ tags: category:preview
//@ stage: fragment
//@ depends_on: {target}, fullscreen_triangle
//@ export: fn fs_main(in: VertexOutput) -> @location(0) vec4f
{own_param_manifest}//@ param: preview_scale uniform f32 range(1.0, 32.0)

struct Params
{{
  time : f32,
{own_param_fields}  preview_scale : f32,
  resolution : vec4f, // .xy = physical pixels, .zw unused
}}

@group( 0 ) @binding( 0 ) var< uniform > params : Params;

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4f
{{
  let aspect = params.resolution.x / max( params.resolution.y, 1.0 );
  let q = ( in.uv - vec2f( 0.5, 0.5 ) ) * vec2f( aspect, 1.0 );
  let p = {p_expr};
  let value = {value_fn}( p{own_param_call_args} );
  let px = params.preview_scale / max( params.resolution.y, 1.0 );
{color_block}
  let cell = abs( fract( p - vec2f( 0.5 ) ) - vec2f( 0.5 ) );
  let minor_grid = 1.0 - smoothstep( 0.0, px * 1.5, min( cell.x, cell.y ) );
  let axis_grid = 1.0 - smoothstep( 0.0, px * 2.5, min( abs( p.x ), abs( p.y ) ) );
  let grid = max( minor_grid * 0.15, axis_grid * 0.35 );
  let shaded = mix( color, vec3f( 0.0, 0.0, 0.0 ), grid );
  return vec4f( clamp( shaded, vec3f( 0.0 ), vec3f( 1.0 ) ), 1.0 );
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
  /// preview's uniform convention: kind `expected_kind`, type `f32`,
  /// resolvable range. Initial value is the range midpoint; step is 1/200 of
  /// the span. `expected_kind` is `Uniform` for a fragment chunk's own
  /// directly-read params and `Argument` for a value chunk's own tunables,
  /// which the synthesized harness passes positionally into the value
  /// function rather than the target chunk reading a uniform itself — see
  /// [`harness_synthesize`]'s doc comment.
  fn slider_of( chunk : &str, param : &Parameter, expected_kind : ParameterKind ) -> Result< PreviewParameter, PreviewError >
  {
    if param.kind != expected_kind
    {
      return Err( PreviewError::UnsupportedParam
      {
        chunk : chunk.to_string(),
        param : param.name.clone(),
        reason : format!( "kind `{:?}` cannot back a live slider here — only `{:?}` parameters are wired into the preview's uniform buffer in this position", param.kind, expected_kind ),
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

  /// Resolves a fragment-mode target's own `//@ param:` uniforms into
  /// sliders — the fragment branch of [`bundle_build`].
  fn fragment_chunk_parameters( name : &str, target_wgsl : &str, exports : &[ &str ] ) -> Result< Vec< PreviewParameter >, PreviewError >
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
    discovered.iter()
    .map( | param | slider_of( name, param, ParameterKind::Uniform ) )
    .collect()
  }

  /// Resolves a value-chunk target's chosen preview export, its own
  /// tunables, and the synthesized harness wrapping it — the value-chunk
  /// branch of [`bundle_build`].
  fn value_chunk_harness_and_parameters( name : &str, target_wgsl : &str, exports : &[ &str ] ) -> Result< ( String, Vec< PreviewParameter > ), PreviewError >
  {
    // A value chunk: prefer a dedicated `NAME_preview` export when one is
    // viable, else the export named like the chunk itself, else the first
    // previewable export ( in file order ) — regardless of which
    // ValueFnKind any of them is; no shape preference.
    let candidates : Vec< ( &str, ValueFnKind, Vec< String > ) > = exports.iter()
    .filter_map( | export | value_fn_of( export ) )
    .collect();
    let discovered = discover( target_wgsl );
    // A candidate is previewable only when every trailing `f32` argument
    // in its own signature has a matching `//@ param: NAME argument f32
    // range(min, max)` declaration. An export that merely happens to
    // structurally match Stage 0 ( e.g. a plain SDF primitive's own real
    // parameters — `d2_sdf_circle(p: vec2f, radius: f32) -> f32` is not a
    // preview wrapper, it is the chunk's actual API, called by dependents
    // with real values ) is not a viable candidate at all; this is not an
    // error, it just means that export isn't the one to preview. A chunk
    // that DOES declare a `//@ param:` for a name with no matching
    // signature argument, or with the wrong kind/type/range, still fails
    // loudly — see `own_params` below, which re-validates the export
    // actually chosen.
    let is_viable = | extra_args : &[ String ] | extra_args.iter()
    .all( | arg_name | discovered.iter().any( | p | p.name == *arg_name && p.kind == ParameterKind::Argument ) );
    // Fix(BUG-205): `//@ param:` declarations are not scoped to one export
    // -- a single `//@ param: strength argument f32 range(...)` line makes
    // ANY candidate with a trailing `strength: f32` argument viable,
    // including both a primitive ( `domain_warp(p, strength) -> vec2f` )
    // and its dedicated `NAME_preview(p, strength) -> f32` wrapper when
    // they happen to share an argument name -- a natural pattern, since
    // both conceptually take the same parameter. The tie-break below used
    // to check only "viable candidate named like the chunk itself", which
    // matches the primitive ( it always shares the chunk's own name ) and
    // never reaches the wrapper. Checking for a viable `NAME_preview`
    // candidate first restores the intended fall-through: prefer the
    // dedicated wrapper when one is viable, then the chunk's own primitive
    // export, then the first viable export in file order.
    let preview_name = format!( "{name}_preview" );
    let ( value_fn, kind, extra_args ) = candidates.iter()
    .filter( | ( _, _, extra_args ) | is_viable( extra_args ) )
    .find( | ( found, _, _ ) | *found == preview_name )
    .or_else( || candidates.iter().filter( | ( _, _, extra_args ) | is_viable( extra_args ) ).find( | ( found, _, _ ) | *found == name ) )
    .or_else( || candidates.iter().find( | ( _, _, extra_args ) | is_viable( extra_args ) ) )
    .cloned()
    .ok_or_else( || PreviewError::Unpreviewable
    {
      chunk : name.to_string(),
      reason : format!
      (
        "exports contain neither a fragment entry point nor a `fn NAME(p: vec2f, ...) -> f32|vec2f|vec3f` value function with every trailing argument backed by a matching `//@ param: NAME argument f32 range(min, max)` line; exports: [{}]",
        exports.join( "; " )
      ),
    })?;
    // `value_fn`'s own trailing `f32` arguments ( `extra_args`, in
    // signature order — see `value_fn_of`'s doc comment ) are the chosen
    // export's own tunables. `own_params` is built in signature order
    // ( not manifest declaration order, which need not match ) so the
    // synthesized harness can pass each one positionally into `value_fn`
    // itself. See `harness_synthesize`'s doc comment for why the target
    // chunk's own WGSL never touches a uniform directly. `is_viable`
    // above already confirmed a same-named, same-kinded declaration
    // exists for each name; `slider_of` here re-validates type and range,
    // which `is_viable` deliberately does not check.
    let own_params : Vec< PreviewParameter > = extra_args.iter()
    .map( | arg_name |
    {
      let param = discovered.iter().find( | candidate | &candidate.name == arg_name ).ok_or_else( || PreviewError::UnsupportedParam
      {
        chunk : name.to_string(),
        param : arg_name.clone(),
        reason : format!( "`{value_fn}` declares this as a trailing argument but the chunk has no matching `//@ param: {arg_name} argument f32 range(min, max)` line" ),
      })?;
      slider_of( name, param, ParameterKind::Argument )
    })
    .collect::< Result< Vec< _ >, _ > >()?;
    let is_sdf = tags_parse( target_wgsl ).iter().any( | &( group, tag ) | group == "category" && tag == "sdf" );
    let harness = harness_synthesize( name, value_fn, kind, is_sdf, &own_params );
    let parameters = own_params.into_iter().chain( std::iter::once( preview_scale_parameter() ) ).collect();
    Ok( ( harness, parameters ) )
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
    // Fix(BUG-281): added "tags" to the upfront required-manifest-fields
    // list so a missing `//@ tags:` line is rejected here, gracefully, as
    // `Unpreviewable` instead of panicking later inside `tags_parse`.
    // Root cause: this loop only ever checked "name"/"depends_on" before
    // any panicking `shader_chunks_core` parser ran, but
    // `value_chunk_harness_and_parameters` (below) unconditionally calls
    // `tags_parse` once a previewable export is chosen, which panics via
    // `manifest_field` when no `//@ tags:` line exists -- this function's
    // own doc comment promises `Unpreviewable` for "missing manifest
    // lines" generally, not just the two that were actually guarded.
    // Pitfall: this upfront guard is only as complete as the field list it
    // checks -- any future call to another panicking `shader_chunks_core`/
    // `shader_chunks_params_core` manifest parser deeper in this pipeline
    // must extend this same list, or it silently reopens this panic class.
    for required in [ "name", "depends_on", "tags" ]
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

    let ( harness, parameters ) = if stage == Some( "fragment" )
    {
      ( None, fragment_chunk_parameters( name, target_wgsl, &exports )? )
    }
    else
    {
      let ( harness, parameters ) = value_chunk_harness_and_parameters( name, target_wgsl, &exports )?;
      ( Some( harness ), parameters )
    };

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

    // Banner comments mark which part of the composed text is a dependency,
    // which part is the chunk actually being previewed, and which part ( if
    // any ) is synthesized scaffolding with no hand-written counterpart --
    // without them, the concatenated wall of text gives no visual signal of
    // where one chunk ends and the next begins, or that the harness is not
    // itself a chunk. `//`-prefixed, so `try_compose`'s own `name_parse`
    // ( which scans every line for the `//@ name:` prefix regardless of
    // position ) still finds each entry's real manifest header beneath it.
    let mut owned_texts : Vec< String > = selected.iter()
    .map( | chunk | format!( "// ==== dependency chunk: {} ====\n{}", chunk.name, chunk.wgsl ) )
    .collect();
    owned_texts.push( format!( "// ==== previewing: {name} -- the chunk you opened ====\n{target_wgsl}" ) );
    if let Some( harness ) = &harness
    {
      owned_texts.push( format!( "// ==== auto-generated preview harness -- not part of any chunk ====\n{harness}" ) );
    }
    let texts : Vec< &str > = owned_texts.iter().map( String::as_str ).collect();

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
