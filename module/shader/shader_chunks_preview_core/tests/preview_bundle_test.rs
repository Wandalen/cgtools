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
  let target = shader_chunks_core::chunk_get( "value_noise" ).expect( "value_noise is bundled" );
  let bundle = bundle_build( target.wgsl ).expect( "value_noise exports a previewable value function" );

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
  assert_eq!( code_occurrences( &bundle.wgsl, "vec4f( value, 0.5, 1.0 )" ), 1, "the Vec2 shape writes red/green from value with a fixed blue pad, no rescaling" );
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
    "candidate selection must fall back to the first previewable export, since none is named exactly `palette_cosine`"
  );
  assert_eq!( code_occurrences( &bundle.wgsl, "vec4f( value, 1.0 )" ), 1, "the Vec3 shape writes a direct RGB passthrough, no rescaling" );
  naga_validate( &bundle.wgsl );
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
fn value_chunk_declaring_params_is_rejected()
{
  let wgsl = "\
//@ name: local_probe
//@ description: Probe.
//@ tags: category:test
//@ depends_on:
//@ export: fn local_probe(p: vec2f) -> f32
//@ param: octaves argument u32 range(1, 8)

fn local_probe( p : vec2f, octaves : u32 ) -> f32 { return 0.0; }
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
