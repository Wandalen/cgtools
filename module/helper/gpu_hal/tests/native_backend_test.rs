//! Native backend tests : a real wgpu device over the machine's Vulkan
//! driver ( a software rasterizer such as lavapipe suffices ), no browser
//! involved. The render test draws through the full public HAL surface and
//! asserts on pixels read back from the offscreen surface.
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]

use gpu_hal::*;

/// Bytes of a `f32` slice, little-endian — the layout vertex and uniform
/// buffers expect.
fn as_bytes( values : &[ f32 ] ) -> Vec< u8 >
{
  values.iter().flat_map( | v | v.to_le_bytes() ).collect()
}

/// Uniform-colored triangle : position passthrough vertex stage, fragment
/// stage sampling one uniform color.
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

#[ test ]
fn device_creation()
{
  let ( device, _queue, surface ) = Device::new_native( 64, 64 )
  .expect
  (
    "no native wgpu adapter : the native backend needs a Vulkan ICD \
     ( a software one such as lavapipe / mesa-vulkan-drivers suffices )"
  );
  assert_eq!( device.depth_range(), DepthRange::ZeroToOne );
  assert_eq!( surface.format(), TextureFormat::Rgba8Unorm );
}

#[ test ]
fn triangle_render_readback()
{
  // 100 px wide : 400 bytes per row, which the readback pads to 512 —
  // the test exercises the row-unpacking path, not just the aligned case.
  let width = 100u32;
  let height = 100u32;
  let ( device, queue, surface ) = Device::new_native( width, height )
  .expect
  (
    "no native wgpu adapter : the native backend needs a Vulkan ICD \
     ( a software one such as lavapipe / mesa-vulkan-drivers suffices )"
  );

  let shader = device.create_shader_module( &ShaderSource
  {
    wgsl : WGSL,
    glsl_vertex : None,
    glsl_fragment : None
  } )
  .expect( "shader module creation failed" );

  let vertices = as_bytes( &[ -0.5, -0.5, 0.5, -0.5, 0.0, 0.5 ] );
  let vertex_buffer = device.create_buffer_init( &vertices, BufferUsage::VERTEX )
  .expect( "vertex buffer creation failed" );
  let indices : Vec< u8 > = [ 0u32, 1, 2 ].iter().flat_map( | i | i.to_le_bytes() ).collect();
  let index_buffer = device.create_buffer_init( &indices, BufferUsage::INDEX )
  .expect( "index buffer creation failed" );

  // Created empty and filled through the queue, so the readback proves
  // `write_buffer` landed, not just `create_buffer_init`.
  let uniform_buffer = device.create_buffer( 16, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
  .expect( "uniform buffer creation failed" );
  queue.write_buffer( &uniform_buffer, &as_bytes( &[ 1.0, 0.0, 0.0, 1.0 ] ) )
  .expect( "uniform write failed" );

  let layout = device.create_bind_group_layout
  (
    &[ BindGroupLayoutEntry
    {
      visibility : ShaderStages::FRAGMENT,
      ty : BindingType::UniformBuffer
    } ]
  )
  .expect( "bind group layout creation failed" );
  let bind_group = device.create_bind_group( &layout, &[ BindingResource::Buffer( &uniform_buffer ) ] )
  .expect( "bind group creation failed" );

  let pipeline = device.create_render_pipeline( &RenderPipelineDesc
  {
    shader : &shader,
    vertex_entry : "vs_main",
    fragment_entry : "fs_main",
    vertex_buffers : &[ VertexBufferLayout
    {
      stride : 8,
      attributes : vec!
      [
        VertexAttribute
        {
          location : 0,
          format : VertexFormat::Float32x2,
          offset : 0
        }
      ]
    } ],
    bind_group_layouts : &[ &layout ],
    color_format : surface.format(),
    depth : None,
    cull_back : false
  } )
  .expect( "pipeline creation failed" );

  let view = surface.current_view().expect( "surface view unavailable" );
  let mut encoder = device.create_command_encoder();
  let mut pass = encoder.begin_render_pass
  (
    &ColorAttachmentDesc
    {
      view : &view,
      clear : [ 0.0, 0.0, 0.0, 1.0 ]
    },
    None
  )
  .expect( "render pass failed to begin" );
  pass.set_pipeline( &pipeline );
  pass.set_bind_group( 0, &bind_group );
  pass.set_vertex_buffer( 0, &vertex_buffer );
  pass.set_index_buffer( &index_buffer, IndexFormat::Uint32 );
  pass.draw_indexed( 3 );
  pass.end();
  queue.submit( encoder );

  let pixels = surface.read_pixels( &device, &queue ).expect( "readback failed" );
  assert_eq!( pixels.len(), ( width * height * 4 ) as usize );

  // Top row first : pixel ( x, y ) starts at ( y * width + x ) * 4.
  let at = | x : u32, y : u32 |
  {
    let start = ( ( y * width + x ) * 4 ) as usize;
    [ pixels[ start ], pixels[ start + 1 ], pixels[ start + 2 ], pixels[ start + 3 ] ]
  };
  // The triangle covers the center of clip space; the corners stay clear.
  assert_eq!( at( 50, 50 ), [ 255, 0, 0, 255 ], "center pixel should be the uniform's red" );
  assert_eq!( at( 0, 0 ), [ 0, 0, 0, 255 ], "corner pixel should be the clear color" );
}
