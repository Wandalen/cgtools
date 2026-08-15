//! Tests for headless bundle rendering: the pure uniform-packing layout
//! ( no GPU ), and real one-frame renders on the headless context —
//! exact pixels for a constant fragment fixture, structural properties
//! for a synthesized value-chunk harness, row-padding-sensitive sizes,
//! and the loud pre-GPU rejection of a zero size.

use shader_chunks_preview_core::{ PreviewBundle, PreviewParameter, bundle_build, resolution_index };
use shader_chunks_render_core::{ RenderError, render, uniform_floats };

fn parameter( property : &str, value : f64 ) -> PreviewParameter
{
  PreviewParameter
  {
    label : property.to_string(),
    property : property.to_string(),
    value,
    min : 0.0,
    max : 1.0,
    step : 0.01,
  }
}

fn bundle_of( parameters : Vec< PreviewParameter > ) -> PreviewBundle
{
  PreviewBundle
  {
    target : "layout_probe".to_string(),
    wgsl : String::new(),
    parameters,
  }
}

#[ test ]
fn uniform_floats_packs_time_then_params_then_aligned_resolution()
{
  // 1 parameter — the value-chunk shape: [ time, p0, 0, 0, w, h, 0, 0 ].
  let floats = uniform_floats( &bundle_of( vec![ parameter( "scale", 8.0 ) ] ), ( 256, 128 ), 1.5 );
  assert_eq!( resolution_index( 1 ), 4 );
  assert_eq!( floats, vec![ 1.5, 8.0, 0.0, 0.0, 256.0, 128.0, 0.0, 0.0 ] );

  // 3 parameters still fit before the same boundary: [ time, p0, p1, p2, w, h, 0, 0 ].
  let three = vec![ parameter( "a", 0.25 ), parameter( "b", 0.5 ), parameter( "c", 0.75 ) ];
  let floats = uniform_floats( &bundle_of( three ), ( 64, 64 ), 0.0 );
  assert_eq!( resolution_index( 3 ), 4 );
  assert_eq!( floats, vec![ 0.0, 0.25, 0.5, 0.75, 64.0, 64.0, 0.0, 0.0 ] );

  // 4 parameters push `resolution` to the next 16-byte boundary, with
  // explicit zero padding between the last parameter and `resolution`.
  let four = vec![ parameter( "a", 0.1 ), parameter( "b", 0.2 ), parameter( "c", 0.3 ), parameter( "d", 0.4 ) ];
  let floats = uniform_floats( &bundle_of( four ), ( 32, 16 ), 2.0 );
  assert_eq!( resolution_index( 4 ), 8 );
  assert_eq!
  (
    floats,
    vec![ 2.0, 0.1_f64 as f32, 0.2_f64 as f32, 0.3_f64 as f32, 0.4_f64 as f32, 0.0, 0.0, 0.0, 32.0, 16.0, 0.0, 0.0 ]
  );

  // No parameters: [ time, 0, 0, 0, w, h, 0, 0 ] — the buffer stays a
  // whole number of 16-byte rows even without any slider.
  let floats = uniform_floats( &bundle_of( vec![] ), ( 8, 8 ), 0.0 );
  assert_eq!( floats.len(), resolution_index( 0 ) + 4 );
  assert_eq!( floats, vec![ 0.0, 0.0, 0.0, 0.0, 8.0, 8.0, 0.0, 0.0 ] );
}

#[ test ]
fn render_rejects_zero_size_before_any_gpu_work()
{
  // The bundle's WGSL is empty and would fail GPU compilation — proving
  // the zero-size check fires first, before any context is created.
  let bundle = bundle_of( vec![] );
  assert_eq!( render( &bundle, ( 0, 64 ), 0.0 ), Err( RenderError::ZeroSize ) );
  assert_eq!( render( &bundle, ( 64, 0 ), 0.0 ), Err( RenderError::ZeroSize ) );
}

/// A fragment-mode fixture painting every pixel one exact constant color —
/// device-independent ground truth for the whole render path. It declares
/// the mandatory `//@ param:` uniform and the convention's `Params` struct
/// but reads neither, so the expected bytes are exact on any conformant
/// GPU: `( 0.2, 0.4, 0.6, 1.0 )` → `( 51, 102, 153, 255 )`.
const CONSTANT_PROBE : &str = "//@ name: constant_probe
//@ description: Test fixture painting every pixel one exact constant color.
//@ tags: category:test
//@ stage: fragment
//@ depends_on: fullscreen_triangle
//@ export: fn fs_main(in: VertexOutput) -> @location(0) vec4f
//@ param: level uniform f32 range(0.0, 1.0)

struct Params
{
  time : f32,
  level : f32,
  resolution : vec4f,
}

@group( 0 ) @binding( 0 ) var< uniform > params : Params;

@fragment
fn fs_main( in : VertexOutput ) -> @location( 0 ) vec4f
{
  return vec4f( 0.2, 0.4, 0.6, 1.0 );
}
";

#[ test ]
fn render_of_a_constant_fragment_chunk_is_exact()
{
  let bundle = bundle_build( CONSTANT_PROBE ).expect( "the constant probe is a valid fragment chunk" );
  let image = render( &bundle, ( 16, 16 ), 0.0 ).expect( "the constant probe renders" );
  assert_eq!( image.size, ( 16, 16 ) );
  assert_eq!( image.pixels.len(), 16 * 16 * 4 );
  for pixel in image.pixels.chunks_exact( 4 )
  {
    assert_eq!( pixel, [ 51, 102, 153, 255 ], "every pixel must be the exact constant color" );
  }
}

#[ test ]
fn render_of_a_value_chunk_matches_the_synthesized_grayscale_harness()
{
  let chunk = shader_chunks_core::chunk_get( "fbm3" ).expect( "fbm3 is bundled" );
  let bundle = bundle_build( chunk.wgsl ).expect( "fbm3 is previewable" );
  let image = render( &bundle, ( 64, 64 ), 0.0 ).expect( "fbm3 renders" );
  assert_eq!( image.size, ( 64, 64 ) );
  assert_eq!( image.pixels.len(), 64 * 64 * 4 );

  let mut distinct = std::collections::BTreeSet::new();
  for pixel in image.pixels.chunks_exact( 4 )
  {
    assert_eq!( pixel[ 0 ], pixel[ 1 ], "the synthesized harness writes grayscale: R == G" );
    assert_eq!( pixel[ 1 ], pixel[ 2 ], "the synthesized harness writes grayscale: G == B" );
    assert_eq!( pixel[ 3 ], 255, "the harness writes opaque alpha" );
    distinct.insert( pixel[ 0 ] );
  }
  assert!( distinct.len() > 8, "an fbm field must vary, not collapse to a flat color: {} distinct values", distinct.len() );
}

#[ test ]
fn render_handles_widths_whose_row_bytes_need_padding()
{
  // 100 * 4 = 400 bytes per row is not a multiple of wgpu's 256-byte
  // row alignment — exercises the readback's padding strip end to end.
  let bundle = bundle_build( CONSTANT_PROBE ).expect( "the constant probe is a valid fragment chunk" );
  let image = render( &bundle, ( 100, 50 ), 0.0 ).expect( "non-aligned widths render" );
  assert_eq!( image.size, ( 100, 50 ) );
  assert_eq!( image.pixels.len(), 100 * 50 * 4 );
  assert_eq!( &image.pixels[ ..4 ], [ 51, 102, 153, 255 ] );
  assert_eq!( &image.pixels[ image.pixels.len() - 4.. ], [ 51, 102, 153, 255 ] );
}

#[ test ]
fn render_time_advances_the_synthesized_drift()
{
  let chunk = shader_chunks_core::chunk_get( "fbm3" ).expect( "fbm3 is bundled" );
  let bundle = bundle_build( chunk.wgsl ).expect( "fbm3 is previewable" );
  let frame_zero = render( &bundle, ( 32, 32 ), 0.0 ).expect( "renders at time 0" );
  let frame_late = render( &bundle, ( 32, 32 ), 10.0 ).expect( "renders at time 10" );
  assert_ne!
  (
    frame_zero.pixels, frame_late.pixels,
    "the synthesized harness drifts the sample plane with time, so distinct times must yield distinct frames"
  );
}
