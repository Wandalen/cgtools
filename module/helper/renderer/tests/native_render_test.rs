//! End-to-end render of the canonical opaque path on the native backend :
//! a real wgpu device over the machine's Vulkan driver ( lavapipe
//! suffices ), pixels read back from the offscreen surface and asserted —
//! no browser involved.
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]

use minwebgpu as gl;
use renderer::webgpu::{ Frame, Geometry, GpuContext, Lights, PbrMaterial, WebGpuRenderer };

#[ test ]
fn opaque_path_renders_lit_quad()
{
  let width = 100u32;
  let height = 100u32;
  let context = GpuContext::new_native( width, height )
  .expect
  (
    "no native wgpu adapter : the native backend needs a Vulkan ICD \
     ( a software one such as lavapipe / mesa-vulkan-drivers suffices )"
  );
  assert_eq!( context.size(), [ width, height ] );
  let renderer = WebGpuRenderer::new( &context ).expect( "renderer construction failed" );

  // A unit quad at the origin facing +z, counter-clockwise from the camera
  // at +z — surviving the pipeline's back-face culling.
  let geometry = Geometry::new
  (
    &context.device,
    &[ -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0 ],
    &[ 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0 ],
    &[ 0.0; 8 ],
    &[ 1.0; 16 ],
    Some( vec![ 0, 1, 2, 0, 2, 3 ] )
  )
  .expect( "geometry upload failed" );

  let mut material = PbrMaterial::new();
  material.base_color_factor = [ 1.0, 0.0, 0.0, 1.0 ];
  material.metallic_factor = 0.0;
  material.roughness_factor = 1.0;
  let binding = renderer.material_binding_create( &context, &material )
  .expect( "material binding failed" );
  let item = renderer.item_create( &context, geometry, binding, gl::math::mat4x4::identity() )
  .expect( "item creation failed" );

  let mut lights = Lights::new();
  assert!( lights.direct_push( [ 0.0, 0.0, 1.0 ], [ 1.0, 1.0, 1.0 ], 3.0 ) );

  let eye = gl::math::F32x3::from( [ 0.0, 0.0, 2.5 ] );
  let frame = Frame
  {
    view_matrix : gl::math::mat3x3h::look_at_rh( eye, gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] ), gl::math::F32x3::Y ),
    projection_matrix : gl::math::mat3x3h::perspective_rh( 1.0, 1.0, 0.1, 100.0 ),
    eye,
    exposure : 0.0
  };

  renderer.render( &context, &frame, &lights, &[ item ] ).expect( "render failed" );

  let pixels = context.surface.pixels_read( &context.device, &context.queue )
  .expect( "readback failed" );
  assert_eq!( pixels.len(), ( width * height * 4 ) as usize );

  // Top row first : pixel ( x, y ) starts at ( y * width + x ) * 4.
  let at = | x : u32, y : u32 |
  {
    let start = ( ( y * width + x ) * 4 ) as usize;
    [ pixels[ start ], pixels[ start + 1 ], pixels[ start + 2 ], pixels[ start + 3 ] ]
  };

  // The quad fills the center of the view; a red material under a white
  // light must tone-map to a clearly red-dominant center pixel ( observed
  // [ 251, 39, 32, 255 ] on lavapipe ). Exact values track the lighting
  // math and driver rounding, so the assertions bound rather than pin them.
  let center = at( 50, 50 );
  assert!
  (
    center[ 0 ] > 150 && center[ 1 ] < 80 && center[ 2 ] < 80,
    "center pixel should be lit red, got {center:?}"
  );
  // The quad's corners project inside the frame, so the frame's own corners
  // stay background — black through the tone mapping bypass.
  let corner = at( 0, 0 );
  assert!
  (
    corner[ 0 ] == 0 && corner[ 1 ] == 0 && corner[ 2 ] == 0,
    "corner pixel should be background black, got {corner:?}"
  );
}
