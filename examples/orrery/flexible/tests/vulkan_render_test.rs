//! Native Vulkan backend render test : renders one real frame of the
//! shared orrery scene through `orrery_flexible::scene_render` and asserts
//! on pixels read back from the offscreen surface — the sun disc at center
//! reads a warm, saturated color; a corner reads the dark nebula backdrop.
//! Expected values were sampled directly from a real rendered frame
//! ( `-orrery_vulkan.png`, byte-identical to `-orrery_wgpu.png` — see
//! `native_render_test.rs` ), not guessed.
#![ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]

use gpu_hal::Device;
use orrery_flexible::uniforms::UniformsRaw;
use orrery_webgpu::scene;

/// True when every channel of `actual` is within `tolerance` of `expected`
/// — real GPU floating-point math can differ by a few least-significant
/// bits between backends/drivers; center-of-disc and corner-of-background
/// pixels sit far enough from any edge/antialiasing seam that a small
/// tolerance still catches a genuinely wrong render.
fn color_close( actual : [ u8; 4 ], expected : [ u8; 4 ], tolerance : i32 ) -> bool
{
  actual.iter().zip( expected.iter() ).all( | ( a, e ) | ( i32::from( *a ) - i32::from( *e ) ).abs() <= tolerance )
}

#[ test ]
fn scene_render_produces_expected_landmarks()
{
  let width = 800u32;
  let height = 600u32;
  let ( device, queue, surface ) = Device::new_vulkan( width, height )
  .expect( "no native vulkan device : the vulkan backend needs a Vulkan ICD ( a software one such as lavapipe suffices )" );

  let scene_config = scene::SceneConfig::load();
  let base_uniforms = UniformsRaw::from( &scene_config );
  let raw = base_uniforms.with_frame( 0.0, 0.0, 4, 10.0, ( width, height ) );

  orrery_flexible::scene_render( &device, &queue, &surface, &raw.to_bytes() )
  .expect( "scene render failed" );

  let pixels = surface.pixels_read( &device, &queue ).expect( "pixel readback failed" );
  assert_eq!( pixels.len(), ( width * height * 4 ) as usize );

  let at = | x : u32, y : u32 |
  {
    let start = ( ( y * width + x ) * 4 ) as usize;
    [ pixels[ start ], pixels[ start + 1 ], pixels[ start + 2 ], pixels[ start + 3 ] ]
  };

  let center = at( 400, 300 );
  assert!
  (
    color_close( center, [ 254, 136, 28, 255 ], 8 ),
    "center pixel should be the sun disc's warm orange, got {center:?}"
  );

  let corner = at( 0, 0 );
  assert!
  (
    color_close( corner, [ 9, 19, 29, 255 ], 8 ),
    "corner pixel should be the dark background, got {corner:?}"
  );
}
