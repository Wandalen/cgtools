//! Tests for `bundle_build`'s two target modes ( value chunk with a
//! synthesized harness; fragment chunk used directly ), its loud rejection
//! paths, and — via naga, the same WGSL front end wgpu itself uses — the
//! language validity of every successfully composed bundle.

use shader_chunks_preview_core::{ bundle_build, resolution_index, PreviewBundle, PreviewError };

/// The example fragment chunk ported from the retired
/// `shader_chunk_preview` browser example: three declared `uniform f32`
/// parameters over `shader_chunks_core`'s noise stack.
const PREVIEW_FRAGMENT_WGSL : &str = include_str!( "fixture/preview_fragment.wgsl" );

/// Parses and validates `wgsl` with naga, panicking with the full error
/// context on failure.
fn naga_validate( wgsl : &str )
{
  let module = naga::front::wgsl::parse_str( wgsl )
  .unwrap_or_else( | err | panic!( "composed bundle WGSL must parse: {}", err.emit_to_string( wgsl ) ) );
  naga::valid::Validator::new( naga::valid::ValidationFlags::all(), naga::valid::Capabilities::default() )
  .validate( &module )
  .expect( "composed bundle WGSL must validate" );
}

fn code_occurrences( source : &str, pattern : &str ) -> usize
{
  source.lines()
  .filter( | line | !line.trim_start().starts_with( "//" ) )
  .filter( | line | line.contains( pattern ) )
  .count()
}

#[ test ]
fn value_chunk_gets_a_synthesized_grayscale_harness()
{
  let target = shader_chunks_core::chunk_get( "fbm3" ).expect( "fbm3 is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "fbm3 exports a previewable value function" );

  assert_eq!( bundle.target, "fbm3" );
  for declaration in [ "fn hash21(", "fn value_noise(", "fn fbm3(", "fn vs_main(", "struct VertexOutput", "fn fs_main(", "struct Params" ]
  {
    assert_eq!
    (
      code_occurrences( &bundle.wgsl, declaration ), 1,
      "bundle must declare `{declaration}` exactly once"
    );
  }
  naga_validate( &bundle.wgsl );
}

#[ test ]
fn value_chunk_bundle_carries_exactly_the_synthesized_preview_scale_slider()
{
  // hash21, not value_noise: this test proves a chunk with zero *own*
  // `//@ param:` lines gets exactly one synthesized slider. value_noise
  // now declares its own `seed` param, so it no longer fits; hash21 is a
  // stable zero-param example -- hash chunks' magic constants are
  // deliberately excluded from ever becoming user-tunable.
  let target = shader_chunks_core::chunk_get( "hash21" ).expect( "hash21 is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "hash21 exports a previewable value function" );

  assert_eq!( bundle.parameters.len(), 1 );
  let param = &bundle.parameters[ 0 ];
  assert_eq!( param.property, "preview_scale" );
  assert_eq!( param.label, "Preview scale" );
  assert_eq!( ( param.min, param.max, param.value ), ( 1.0, 32.0, 8.0 ) );
  naga_validate( &bundle.wgsl );
}

#[ test ]
fn vec2_value_chunk_gets_a_synthesized_harness()
{
  let target = shader_chunks_core::chunk_get( "hash22" ).expect( "hash22 is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "hash22 exports a previewable vec2f value function natively" );

  assert_eq!( bundle.target, "hash22" );
  assert_eq!( code_occurrences( &bundle.wgsl, "hash22( p )" ), 1, "candidate selection must pick hash22 itself by name match, no wrapper needed" );
  assert_eq!( code_occurrences( &bundle.wgsl, "let color = vec3f( value, 0.5 );" ), 1, "the Vec2 shape writes red/green from value with a fixed blue pad, no rescaling" );
  naga_validate( &bundle.wgsl );
}

#[ test ]
fn vec3_value_chunk_gets_a_synthesized_harness()
{
  let target = shader_chunks_core::chunk_get( "palette_cosine" ).expect( "palette_cosine is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "palette_cosine_preview exports a previewable vec3f value function" );

  assert_eq!( bundle.target, "palette_cosine" );
  assert_eq!
  (
    code_occurrences( &bundle.wgsl, "palette_cosine_preview( p )" ), 1,
    "candidate selection must fall back to the first previewable export, since none is named exactly `palette_cosine`; \
    the wrapper takes no tunables of its own -- its canonical rainbow phase spread is fixed, not sliderable ( BUG-286 )"
  );
  assert_eq!( code_occurrences( &bundle.wgsl, "let color = value;" ), 1, "the Vec3 shape writes a direct RGB passthrough, no rescaling" );
  naga_validate( &bundle.wgsl );
}

#[ test ]
fn sdf_tagged_value_chunk_gets_filled_banded_visualization_and_stationary_sampling()
{
  let target = shader_chunks_core::chunk_get( "sdf_op_round" ).expect( "sdf_op_round is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "sdf_op_round exports a previewable value function" );

  assert_eq!( bundle.target, "sdf_op_round" );
  assert_eq!
  (
    code_occurrences( &bundle.wgsl, "select( vec3f( 0.92, 0.93, 0.96 ), vec3f( 0.30, 0.55, 0.95 ), value < 0.0 )" ), 1,
    "a category:sdf chunk must get the filled/banded distance visualization, not the raw clamped grayscale"
  );
  assert_eq!
  (
    code_occurrences( &bundle.wgsl, "let p = q * params.preview_scale;" ), 1,
    "a category:sdf chunk's sample point must stay stationary -- a time-drifting offset eventually pans a finite shape out of frame"
  );

  // sdf_op_round's own hardcoded box-extent/round-radius literals are real
  // sliders now, not baked constants -- see `shader/sdf_op_round/sdf_op_round.wgsl`'s
  // `//@ param: ... argument f32 ...` declarations.
  let properties : Vec< &str > = bundle.parameters.iter().map( | p | p.property.as_str() ).collect();
  assert_eq!
  (
    properties, vec![ "box_half_extent", "round_radius", "preview_scale" ],
    "the chunk's own declared tunables precede the synthesized preview_scale slider, in signature order"
  );
  assert_eq!( ( bundle.parameters[ 0 ].min, bundle.parameters[ 0 ].max ), ( 0.05, 0.4 ) );
  assert_eq!( ( bundle.parameters[ 1 ].min, bundle.parameters[ 1 ].max ), ( 0.0, 0.2 ) );
  assert!
  (
    bundle.wgsl.contains( "let value = sdf_op_round_preview( p, params.box_half_extent, params.round_radius );" ),
    "the harness must pass both uniforms positionally into the chunk's own pure wrapper:\n{}", bundle.wgsl
  );

  naga_validate( &bundle.wgsl );
}

#[ test ]
fn non_sdf_value_chunk_keeps_raw_grayscale_and_time_drift()
{
  let target = shader_chunks_core::chunk_get( "fbm3" ).expect( "fbm3 is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "fbm3 exports a previewable value function" );

  assert_eq!( code_occurrences( &bundle.wgsl, "let color = vec3f( value );" ), 1, "a non-sdf f32 chunk keeps the original raw-clamped grayscale write" );
  assert_eq!
  (
    code_occurrences( &bundle.wgsl, "params.time * 0.05" ), 1,
    "a non-sdf value chunk keeps the time-drifting sample point -- it has no finite footprint to drift out of"
  );
}

#[ test ]
fn every_value_chunk_preview_carries_a_reference_grid()
{
  let target = shader_chunks_core::chunk_get( "sdf_op_round" ).expect( "sdf_op_round is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "previewable" );
  assert_eq!
  (
    code_occurrences( &bundle.wgsl, "let grid = max( minor_grid" ), 1,
    "every value-chunk harness overlays a world-space reference grid so scale/center stay legible"
  );
}

#[ test ]
fn composed_bundle_marks_dependency_target_and_harness_sections()
{
  let target = shader_chunks_core::chunk_get( "sdf_op_round" ).expect( "sdf_op_round is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "previewable" );
  assert!( bundle.wgsl.contains( "// ==== dependency chunk: d2_sdf_box ====" ), "dependency section must be banner-marked" );
  assert!( bundle.wgsl.contains( "// ==== previewing: sdf_op_round" ), "target section must be banner-marked" );
  assert!( bundle.wgsl.contains( "// ==== auto-generated preview harness" ), "synthesized harness section must be banner-marked" );
}

#[ test ]
fn every_bundled_chunk_previews_except_the_denylist()
{
  const NOT_PREVIEWABLE_CHUNKS : &[ &str ] = &[ "fullscreen_triangle" ];

  for chunk in shader_chunks_core::CHUNKS
  {
    let result = bundle_build( chunk.wgsl );
    if NOT_PREVIEWABLE_CHUNKS.contains( &chunk.name )
    {
      assert!( result.is_err(), "`{}` is denylisted but previewed successfully", chunk.name );
    }
    else
    {
      let bundle = result.unwrap_or_else( | err | panic!( "`{}` should be previewable: {err}", chunk.name ) );
      naga_validate( &bundle.wgsl );
    }
  }
}

#[ test ]
fn fragment_chunk_is_used_directly_with_its_declared_sliders_in_order()
{
  let bundle = bundle_build( PREVIEW_FRAGMENT_WGSL ).expect( "the fixture fragment chunk is previewable" );

  assert_eq!( bundle.target, "preview_fragment" );
  assert_eq!( code_occurrences( &bundle.wgsl, "fn fs_main(" ), 1, "no harness must be synthesized around a fragment chunk" );
  assert_eq!( code_occurrences( &bundle.wgsl, "fn vs_main(" ), 1, "the vertex stage comes from the dependency closure" );

  let properties : Vec< &str > = bundle.parameters.iter().map( | p | p.property.as_str() ).collect();
  assert_eq!( properties, vec![ "noise_scale", "warp_strength", "brightness" ], "sliders follow declaration order — the uniform layout convention" );
  let ranges : Vec< ( f64, f64 ) > = bundle.parameters.iter().map( | p | ( p.min, p.max ) ).collect();
  assert_eq!( ranges, vec![ ( 0.5, 20.0 ), ( 0.0, 2.0 ), ( 0.0, 3.0 ) ], "declared `range(min, max)` values drive the sliders" );
  assert_eq!( bundle.parameters[ 0 ].label, "Noise scale" );
  naga_validate( &bundle.wgsl );
}

#[ test ]
fn fragment_chunk_bundle_round_trips_through_json()
{
  let bundle = bundle_build( PREVIEW_FRAGMENT_WGSL ).expect( "the fixture fragment chunk is previewable" );
  let json = serde_json::to_string( &bundle ).expect( "bundle serializes" );
  let back : PreviewBundle = serde_json::from_str( &json ).expect( "bundle deserializes" );
  assert_eq!( back, bundle, "the -preview.json transport must be lossless" );
}

#[ test ]
fn vertex_chunk_is_rejected_as_unpreviewable()
{
  let target = shader_chunks_core::chunk_get( "fullscreen_triangle" ).expect( "fullscreen_triangle is bundled" );
  let err = bundle_build( target.wgsl ).expect_err( "a vertex chunk offers nothing to preview" );
  assert!( matches!( err, PreviewError::Unpreviewable { .. } ), "expected Unpreviewable, got {err:?}" );
}

#[ test ]
fn unknown_dependency_is_rejected_loudly()
{
  let wgsl = "\
//@ name: local_probe
//@ description: Probe.
//@ tags: category:test
//@ depends_on: bogus_chunk
//@ export: fn local_probe(p: vec2f) -> f32

fn local_probe( p : vec2f ) -> f32 { return 0.0; }
";
  let err = bundle_build( wgsl ).expect_err( "should fail" );
  assert_eq!( err, PreviewError::UnknownChunk( "bogus_chunk".to_string() ) );
}

#[ test ]
fn value_chunk_with_matching_argument_param_gets_a_real_slider()
{
  let wgsl = "\
//@ name: local_probe
//@ description: Probe.
//@ tags: category:test
//@ depends_on:
//@ export: fn local_probe(p: vec2f, spread: f32) -> f32
//@ param: spread argument f32 range(1.0, 8.0)

fn local_probe( p : vec2f, spread : f32 ) -> f32 { return spread; }
";
  let bundle = bundle_build( wgsl ).expect( "a trailing f32 argument with a matching `argument` declaration is previewable" );

  let properties : Vec< &str > = bundle.parameters.iter().map( | p | p.property.as_str() ).collect();
  assert_eq!( properties, vec![ "spread", "preview_scale" ], "the chunk's own tunable comes before the synthesized preview_scale slider" );
  assert_eq!( ( bundle.parameters[ 0 ].min, bundle.parameters[ 0 ].max ), ( 1.0, 8.0 ) );
  assert!
  (
    bundle.wgsl.contains( "let value = local_probe( p, params.spread );" ),
    "the harness must call the value function with the uniform passed positionally, not read as a global inside the target's own body:\n{}", bundle.wgsl
  );
  naga_validate( &bundle.wgsl );
}

#[ test ]
fn value_chunk_trailing_argument_without_declaration_is_unpreviewable()
{
  // Mirrors a real, previously-observed regression: a chunk's own raw
  // primitive export ( name-matching the chunk, per Stage 1's tie-break )
  // structurally satisfies Stage 0 once it carries a trailing `f32`
  // argument, even when that argument was never meant to be a preview
  // slider — the primitive is real API surface other chunks call with real
  // values, not a `_preview` wrapper. With no matching `//@ param:`
  // declaration at all, and no other candidate export, the chunk must be
  // reported `Unpreviewable`, not crash with `UnsupportedParam`.
  let wgsl = "\
//@ name: local_probe
//@ description: Probe.
//@ tags: category:test
//@ depends_on:
//@ export: fn local_probe(p: vec2f, strength: f32) -> f32

fn local_probe( p : vec2f, strength : f32 ) -> f32 { return strength; }
";
  let err = bundle_build( wgsl ).expect_err( "an undeclared trailing argument must not be silently treated as previewable" );
  assert!( matches!( &err, PreviewError::Unpreviewable { .. } ), "expected Unpreviewable, got {err:?}" );
}

#[ test ]
fn value_chunk_prefers_dedicated_preview_wrapper_over_same_named_primitive_sharing_an_argument_name()
{
  // Mirrors a real regression in `shader/domain_warp/domain_warp.wgsl`:
  // its raw primitive `domain_warp(p: vec2f, strength: f32) -> vec2f`
  // and its dedicated `domain_warp_preview(p: vec2f, strength: f32) ->
  // f32` wrapper both declare a trailing argument named `strength`, and
  // the chunk's single `//@ param: strength ...` line matches either one
  // by name alone -- `is_viable` doesn't check which export a
  // declaration was written for, so it accepted the primitive too. The
  // same-name tie-break then preferred the primitive ( it matches the
  // chunk's own name ) over the dedicated wrapper, so the harness
  // rendered the raw vec2f warp displacement as a color swatch instead
  // of `domain_warp_preview`'s intended scalar noise value.
  let target = shader_chunks_core::chunk_get( "domain_warp" ).expect( "domain_warp is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "domain_warp_preview exports a previewable f32 value function" );

  assert_eq!( bundle.target, "domain_warp" );
  assert_eq!
  (
    code_occurrences( &bundle.wgsl, "let value = domain_warp_preview( p, params.strength, params.lacunarity, params.gain, params.seed );" ), 1,
    "candidate selection must prefer the dedicated `domain_warp_preview` wrapper over the raw \
    `domain_warp` primitive, even though both are viable under the shared `strength` argument name:\n{}", bundle.wgsl
  );
  assert_eq!
  (
    code_occurrences( &bundle.wgsl, "let color = vec3f( value );" ), 1,
    "the chosen export returns f32, so the harness must use the grayscale write, not the Vec2 blue-padded write"
  );
  naga_validate( &bundle.wgsl );
}

#[ test ]
fn value_chunk_declared_param_with_wrong_type_still_fails_loudly()
{
  let wgsl = "\
//@ name: local_probe
//@ description: Probe.
//@ tags: category:test
//@ depends_on:
//@ export: fn local_probe(p: vec2f, octaves: f32) -> f32
//@ param: octaves argument u32 range(1, 8)

fn local_probe( p : vec2f, octaves : f32 ) -> f32 { return octaves; }
";
  let err = bundle_build( wgsl ).expect_err( "should fail" );
  assert!
  (
    matches!( &err, PreviewError::UnsupportedParam { param, .. } if param == "octaves" ),
    "expected UnsupportedParam for `octaves`, got {err:?}"
  );
}

#[ test ]
fn fragment_chunk_with_non_f32_uniform_is_rejected()
{
  let wgsl = "\
//@ name: local_fragment
//@ description: Probe.
//@ tags: category:test
//@ stage: fragment
//@ depends_on: fullscreen_triangle
//@ export: fn fs_main(in: VertexOutput) -> @location(0) vec4f
//@ param: steps uniform u32 range(1, 8)

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4f { return vec4f( 1.0 ); }
";
  let err = bundle_build( wgsl ).expect_err( "should fail" );
  assert!
  (
    matches!( &err, PreviewError::UnsupportedParam { param, .. } if param == "steps" ),
    "expected UnsupportedParam for `steps`, got {err:?}"
  );
}

#[ test ]
fn fragment_chunk_without_fs_main_export_is_rejected()
{
  let wgsl = "\
//@ name: local_fragment
//@ description: Probe.
//@ tags: category:test
//@ stage: fragment
//@ depends_on: fullscreen_triangle
//@ export: fn shade(in: VertexOutput) -> @location(0) vec4f
//@ param: gain uniform f32 range(0.0, 1.0)

@fragment
fn shade( in : VertexOutput ) -> @location( 0 ) vec4f { return vec4f( 1.0 ); }
";
  let err = bundle_build( wgsl ).expect_err( "should fail" );
  assert!( matches!( err, PreviewError::Unpreviewable { .. } ), "expected Unpreviewable, got {err:?}" );
}

#[ test ]
fn resolution_index_pads_to_the_next_16_byte_boundary()
{
  // time alone → resolution at index 4; time + 3 params fill the first
  // 16 bytes exactly → still 4; a 5th float pushes it to 8.
  assert_eq!( resolution_index( 0 ), 4 );
  assert_eq!( resolution_index( 1 ), 4, "the synthesized harness: time + preview_scale" );
  assert_eq!( resolution_index( 3 ), 4, "the fixture fragment chunk: time + 3 params" );
  assert_eq!( resolution_index( 4 ), 8 );
  assert_eq!( resolution_index( 7 ), 8 );
  assert_eq!( resolution_index( 8 ), 12 );
}

#[ test ]
fn missing_manifest_is_rejected_before_parsing_panics()
{
  let err = bundle_build( "fn naked() -> f32 { return 0.0; }" ).expect_err( "should fail" );
  assert!( matches!( err, PreviewError::Unpreviewable { .. } ), "expected Unpreviewable, got {err:?}" );
}

// test_kind: bug_reproducer(BUG-281)
/// ## Root Cause
/// `bundle_build`'s upfront manifest-completeness check ( the loop over
/// `[ "name", "depends_on" ]` ) rejects only those two missing header lines
/// gracefully before any panicking `shader_chunks_core` parser runs. But
/// `value_chunk_harness_and_parameters` later calls `tags_parse` ( to detect
/// the `category:sdf` tag, for the SDF-specific harness treatment ) once a
/// previewable export has been chosen -- `tags_parse` panics outright via
/// `shader_chunks_core`'s `manifest_field` when no `//@ tags:` line exists.
/// A value-chunk-shaped target with a valid previewable export but no
/// `//@ tags:` line reaches that call and crashes the whole process instead
/// of returning `PreviewError::Unpreviewable`, even though `bundle_build`'s
/// own doc comment promises `Unpreviewable` for "missing manifest lines"
/// generally, not just the two currently checked.
/// ## Why Not Caught
/// Every existing test's inline WGSL fixture in this file always includes a
/// `//@ tags:` line out of habit ( following the real manifest convention ),
/// and every bundled `CHUNKS` entry already carries one too ( required
/// elsewhere across the wider `shader_chunks` CLI ), so `tags_parse`'s panic
/// path inside this crate was never exercised by any test. It is reachable
/// in practice through the CLI's `preview file::<path>` mode on a chunk
/// still being hand-authored, before its manifest header is complete.
/// ## Fix Applied
/// Added `"tags"` to `bundle_build`'s upfront required-manifest-fields loop,
/// alongside the existing `"name"`/`"depends_on"` checks, so a missing
/// `//@ tags:` line is caught and reported as `Unpreviewable` before any
/// panicking parser runs -- the same graceful contract already applied to
/// `name`/`depends_on`.
/// ## Prevention
/// This test constructs a value-chunk-shaped WGSL fixture with every other
/// required manifest line present except `//@ tags:`, proving `bundle_build`
/// now returns `Err( PreviewError::Unpreviewable { .. } )` instead of
/// panicking.
/// ## Pitfall
/// An upfront "reject missing manifest lines before parsing panics" guard is
/// only as complete as the field list it actually checks -- adding a new
/// call to a panicking `shader_chunks_core` parser deeper in the pipeline
/// ( as `tags_parse` was, for `category:sdf` detection ) silently reopens
/// the exact panic class the guard exists to close, unless the guard's own
/// field list is extended alongside it.
#[ test ]
fn value_chunk_missing_tags_line_is_rejected_not_panicked()
{
  let wgsl = "\
//@ name: local_probe
//@ description: Probe.
//@ depends_on:
//@ export: fn local_probe(p: vec2f) -> f32

fn local_probe( p : vec2f ) -> f32 { return 0.0; }
";
  let err = bundle_build( wgsl ).expect_err( "a value chunk with no `//@ tags:` line must be rejected, not panic" );
  assert!( matches!( err, PreviewError::Unpreviewable { .. } ), "expected Unpreviewable, got {err:?}" );
}
