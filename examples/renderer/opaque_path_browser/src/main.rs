//! Renders the canonical opaque path's lit-quad scene through a real browser
//! canvas — the browser-side counterpart to
//! `renderer/tests/native_render_test.rs`'s `opaque_path_renders_lit_quad`,
//! which proves the same scene on the native backend through an offscreen
//! readback instead. Reuses that test's exact geometry, material, light, and
//! camera data.
//!
//! `GpuContext::new_webgpu` is async and `GpuContext::new_webgl` is not, so
//! one `main()` can only drive one backend per build — pick one via Cargo
//! features:
//! ```bash
//! trunk serve --release                                         # webgpu ( default )
//! trunk serve --release --no-default-features --features webgl  # webgl
//! ```

#[ cfg( target_arch = "wasm32" ) ]
use minwebgpu as gl;
#[ cfg( target_arch = "wasm32" ) ]
use renderer::webgpu::{ Frame, Geometry, GpuContext, Lights, PbrMaterial, WebGpuRenderer };

/// Builds the same lit-quad scene `opaque_path_renders_lit_quad` renders
/// natively and issues one render pass — presented to the canvas
/// automatically once submitted, no explicit present call exists on `Surface`.
#[ cfg( target_arch = "wasm32" ) ]
fn opaque_path_draw( context : &GpuContext )
{
  let renderer = WebGpuRenderer::new( context ).expect( "renderer construction failed" );

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
  let binding = renderer.material_binding_create( context, &material )
  .expect( "material binding failed" );
  let item = renderer.item_create( context, geometry, binding, gl::math::mat4x4::identity() )
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

  renderer.render( context, &frame, &lights, &[ item ] ).expect( "render failed" );
}

/// `webgpu` build: async context creation, presented to the canvas
/// automatically once submitted.
#[ cfg( all( target_arch = "wasm32", feature = "webgpu" ) ) ]
async fn app_run()
{
  let canvas = mingl::web::canvas::retrieve_or_make().expect( "canvas retrieval failed" );
  let context = GpuContext::new_webgpu( &canvas ).await
  .expect( "webgpu context creation failed — does this browser support WebGPU?" );
  opaque_path_draw( &context );
}

/// `webgl` build: synchronous context creation over a WebGL2 context.
#[ cfg( all( target_arch = "wasm32", feature = "webgl" ) ) ]
fn app_run()
{
  let canvas = mingl::web::canvas::retrieve_or_make().expect( "canvas retrieval failed" );
  let context = GpuContext::new_webgl( &canvas )
  .expect( "webgl context creation failed — does this browser support WebGL2 + EXT_color_buffer_float?" );
  opaque_path_draw( &context );
}

#[ cfg( all( target_arch = "wasm32", feature = "webgpu" ) ) ]
fn main()
{
  wasm_bindgen_futures::spawn_local( app_run() );
}

#[ cfg( all( target_arch = "wasm32", feature = "webgl" ) ) ]
fn main()
{
  app_run();
}

// Stub main for native targets
#[ cfg( not( target_arch = "wasm32" ) ) ]
fn main()
{
  println!( "This renderer example only works on WebAssembly targets." );
  println!( "To run it, compile for wasm32-unknown-unknown with one backend feature:" );
  println!( "  cargo build --target wasm32-unknown-unknown --features webgpu" );
  println!( "  cargo build --target wasm32-unknown-unknown --no-default-features --features webgl" );
}
