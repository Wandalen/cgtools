//! Minimal cross-backend triangle draw proving `gpu_hal`'s `webgpu` and `webgl`
//! backends both paint real browser pixels — the browser-side counterpart to
//! `gpu_hal/tests/native_backend_test.rs`'s `triangle_render_readback`, which
//! proves the same render path through an offscreen native readback instead.
//! Reuses that test's WGSL shader and vertex/uniform data; the WebGL backend
//! additionally needs a GLSL ES override pair, since unlike WebGPU/native it
//! cannot consume WGSL directly ( `Device::shader_module_create` ).
//!
//! `Device::new_webgpu` is async and `Device::new_webgl` is not, so one
//! `main()` can only drive one backend per build — pick one via Cargo
//! features:
//! ```bash
//! trunk serve --release                                         # webgpu ( default )
//! trunk serve --release --no-default-features --features webgl  # webgl
//! ```

#[ cfg( target_arch = "wasm32" ) ]
use gpu_hal::*;

/// Bytes of a `f32` slice, little-endian — mirrors
/// `native_backend_test.rs::as_bytes`.
#[ cfg( target_arch = "wasm32" ) ]
fn as_bytes( values : &[ f32 ] ) -> Vec< u8 >
{
  values.iter().flat_map( | v | v.to_le_bytes() ).collect()
}

/// Uniform-colored triangle, the same WGSL `native_backend_test.rs`'s
/// `triangle_render_readback` uses — the WebGPU and native backends consume
/// this directly.
#[ cfg( target_arch = "wasm32" ) ]
const WGSL : &str = "
struct Color
{
  value : vec4f
}

@group( 0 ) @binding( 0 ) var< uniform > color : Color;

@vertex
fn vs_main( @location( 0 ) position : vec2f ) -> @builtin( position ) vec4f
{
  return vec4f( position, 0.0, 1.0 );
}

@fragment
fn fs_main() -> @location( 0 ) vec4f
{
  return color.value;
}
";

/// GLSL ES 300 equivalent of `WGSL`'s vertex stage. Attribute location `0`
/// matches the `VertexAttribute.location` wired in `triangle_draw` below.
#[ cfg( target_arch = "wasm32" ) ]
const GLSL_VERTEX : &str = "#version 300 es
layout( location = 0 ) in vec2 position;
void main()
{
  gl_Position = vec4( position, 0.0, 1.0 );
}
";

/// GLSL ES 300 equivalent of `WGSL`'s fragment stage. The uniform block name
/// `ub_0_0` follows `webgl_bindings_introspect`'s `ub_{group}_{binding}`
/// convention ( `gpu_hal/src/device.rs` ), matching `WGSL`'s
/// `@group(0) @binding(0)`.
#[ cfg( target_arch = "wasm32" ) ]
const GLSL_FRAGMENT : &str = "#version 300 es
precision highp float;
layout( std140 ) uniform ub_0_0
{
  vec4 value;
};
out vec4 out_color;
void main()
{
  out_color = value;
}
";

/// Builds the pipeline and resources shared by both backends and issues one
/// render pass drawing a red triangle over a black clear — device creation
/// is the only step that differs between backends, handled by each of this
/// file's two `app_run` variants before calling here.
#[ cfg( target_arch = "wasm32" ) ]
fn triangle_draw( device : &Device, queue : &Queue, surface : &Surface )
{
  let shader = device.shader_module_create( &ShaderSource
  {
    wgsl : WGSL,
    glsl_vertex : Some( GLSL_VERTEX ),
    glsl_fragment : Some( GLSL_FRAGMENT )
  } )
  .expect( "shader module creation failed" );

  let vertices = as_bytes( &[ -0.5, -0.5, 0.5, -0.5, 0.0, 0.5 ] );
  let vertex_buffer = device.buffer_init_create( &vertices, BufferUsage::VERTEX )
  .expect( "vertex buffer creation failed" );
  let indices : Vec< u8 > = [ 0u32, 1, 2 ].iter().flat_map( | i | i.to_le_bytes() ).collect();
  let index_buffer = device.buffer_init_create( &indices, BufferUsage::INDEX )
  .expect( "index buffer creation failed" );

  let uniform_buffer = device.buffer_create( 16, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
  .expect( "uniform buffer creation failed" );
  queue.buffer_write( &uniform_buffer, &as_bytes( &[ 1.0, 0.0, 0.0, 1.0 ] ) )
  .expect( "uniform write failed" );

  // Fix(BUG-200) verification: an oversized `buffer_write` against a WebGL
  // buffer too small to hold it must return `Err`, not silently no-op --
  // WebGL2's `bufferSubData` has no way to surface the underlying
  // `INVALID_VALUE` itself, so this guard is the only thing standing between
  // silent data corruption and a clean error. A cyan clear below means the
  // guard did NOT fire and this example's own render output can no longer
  // be trusted. WebGPU's `writeBuffer` validates out-of-bounds writes itself
  // ( a different failure mode, not this bug ), so this check only applies
  // to the `webgl` build.
  #[ cfg( feature = "webgl" ) ]
  let clear =
  {
    let small_buffer = device.buffer_create( 4, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
    .expect( "small buffer creation failed" );
    let oversized = as_bytes( &[ 1.0, 2.0, 3.0, 4.0 ] ); // 16 bytes into a 4-byte buffer
    if queue.buffer_write( &small_buffer, &oversized ).is_ok()
    {
      [ 0.0, 1.0, 1.0, 1.0 ] // cyan -- BUG-200 guard missing/regressed
    }
    else
    {
      [ 0.0, 0.0, 0.0, 1.0 ]
    }
  };
  #[ cfg( not( feature = "webgl" ) ) ]
  let clear = [ 0.0, 0.0, 0.0, 1.0 ];

  let layout = device.bind_group_layout_create
  (
    &[ BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::UniformBuffer } ]
  )
  .expect( "bind group layout creation failed" );
  let bind_group = device.bind_group_create( &layout, &[ BindingResource::Buffer( &uniform_buffer ) ] )
  .expect( "bind group creation failed" );

  let pipeline = device.render_pipeline_create( &RenderPipelineDesc
  {
    shader : &shader,
    vertex_entry : "vs_main",
    fragment_entry : "fs_main",
    vertex_buffers : &[ VertexBufferLayout
    {
      stride : 8,
      attributes : vec!
      [
        VertexAttribute { location : 0, format : VertexFormat::Float32x2, offset : 0 }
      ]
    } ],
    bind_group_layouts : &[ &layout ],
    color_format : surface.format(),
    depth : None,
    cull_back : false
  } )
  .expect( "pipeline creation failed" );

  let view = surface.current_view().expect( "surface view unavailable" );
  let mut encoder = device.command_encoder_create();
  let mut pass = encoder.render_pass_begin
  (
    &ColorAttachmentDesc { view : &view, clear },
    None
  )
  .expect( "render pass failed to begin" );
  pass.pipeline_set( &pipeline );
  pass.bind_group_set( 0, &bind_group );
  pass.vertex_buffer_set( 0, &vertex_buffer );
  pass.index_buffer_set( &index_buffer, IndexFormat::Uint32 );
  pass.draw_indexed( 3 );
  pass.end();
  queue.submit( encoder );
}

/// `webgpu` build: async device creation, presented to the canvas
/// automatically once submitted — no explicit present call exists on `Surface`.
#[ cfg( all( target_arch = "wasm32", feature = "webgpu" ) ) ]
async fn app_run()
{
  let canvas = mingl::web::canvas::retrieve_or_make().expect( "canvas retrieval failed" );
  let ( device, queue, surface ) = Device::new_webgpu( &canvas ).await
  .expect( "webgpu device creation failed — does this browser support WebGPU?" );
  triangle_draw( &device, &queue, &surface );
}

/// `webgl` build: synchronous device creation over a WebGL2 context.
#[ cfg( all( target_arch = "wasm32", feature = "webgl" ) ) ]
fn app_run()
{
  let canvas = mingl::web::canvas::retrieve_or_make().expect( "canvas retrieval failed" );
  let ( device, queue, surface ) = Device::new_webgl( &canvas )
  .expect( "webgl device creation failed — does this browser support WebGL2?" );
  triangle_draw( &device, &queue, &surface );
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
  println!( "This gpu_hal example only works on WebAssembly targets." );
  println!( "To run it, compile for wasm32-unknown-unknown with one backend feature:" );
  println!( "  cargo build --target wasm32-unknown-unknown --features webgpu" );
  println!( "  cargo build --target wasm32-unknown-unknown --no-default-features --features webgl" );
}
