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

  let shader = device.shader_module_create( &ShaderSource
  {
    wgsl : WGSL,
    glsl_vertex : None,
    glsl_fragment : None
  } )
  .expect( "shader module creation failed" );

  let vertices = as_bytes( &[ -0.5, -0.5, 0.5, -0.5, 0.0, 0.5 ] );
  let vertex_buffer = device.buffer_init_create( &vertices, BufferUsage::VERTEX )
  .expect( "vertex buffer creation failed" );
  let indices : Vec< u8 > = [ 0u32, 1, 2 ].iter().flat_map( | i | i.to_le_bytes() ).collect();
  let index_buffer = device.buffer_init_create( &indices, BufferUsage::INDEX )
  .expect( "index buffer creation failed" );

  // Created empty and filled through the queue, so the readback proves
  // `buffer_write` landed, not just `buffer_init_create`.
  let uniform_buffer = device.buffer_create( 16, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
  .expect( "uniform buffer creation failed" );
  queue.buffer_write( &uniform_buffer, &as_bytes( &[ 1.0, 0.0, 0.0, 1.0 ] ) )
  .expect( "uniform write failed" );

  let layout = device.bind_group_layout_create
  (
    &[ BindGroupLayoutEntry
    {
      visibility : ShaderStages::FRAGMENT,
      ty : BindingType::UniformBuffer
    } ]
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
  let mut encoder = device.command_encoder_create();
  let mut pass = encoder.render_pass_begin
  (
    &ColorAttachmentDesc
    {
      view : &view,
      clear : [ 0.0, 0.0, 0.0, 1.0 ]
    },
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

  let pixels = surface.pixels_read( &device, &queue ).expect( "readback failed" );
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

/// Fullscreen-triangle fragment stage sampling a HAL texture through a
/// sampler at a fixed interior UV — the test exercises upload correctness
/// only, never texture-coordinate or orientation edge cases.
const TEXTURE_WGSL : &str = "
@group( 0 ) @binding( 0 ) var tex : texture_2d< f32 >;
@group( 0 ) @binding( 1 ) var samp : sampler;

@vertex
fn vs_main( @location( 0 ) position : vec2f ) -> @builtin( position ) vec4f
{
  return vec4f( position, 0.0, 1.0 );
}

@fragment
fn fs_main() -> @location( 0 ) vec4f
{
  return textureSample( tex, samp, vec2f( 0.5, 0.5 ) );
}
";

/// Resources for `texture_write_readback`, kept alive across both writes.
struct TexturedScene
{
  device : Device,
  queue : Queue,
  surface : Surface,
  pipeline : RenderPipeline,
  bind_group : BindGroup,
  vertex_buffer : Buffer,
  index_buffer : Buffer,
  texture : Texture
}

/// A single triangle large enough to cover the full clip-space area, plus
/// its index buffer.
fn fullscreen_geometry_create( device : &Device ) -> ( Buffer, Buffer )
{
  let vertices = as_bytes( &[ -1.0, -1.0, 3.0, -1.0, -1.0, 3.0 ] );
  let vertex_buffer = device.buffer_init_create( &vertices, BufferUsage::VERTEX )
  .expect( "vertex buffer creation failed" );
  let indices : Vec< u8 > = [ 0u32, 1, 2 ].iter().flat_map( | i | i.to_le_bytes() ).collect();
  let index_buffer = device.buffer_init_create( &indices, BufferUsage::INDEX )
  .expect( "index buffer creation failed" );
  ( vertex_buffer, index_buffer )
}

/// A 64×64 `Rgba8Unorm` texture ( left empty — filled by the test itself
/// via `texture_write` ), its bind group layout, and a bind group pairing
/// it with a sampler. Texture entry precedes the sampler entry : the
/// WebGL backend pairs a sampler with the nearest preceding texture
/// entry, so this order is load-bearing.
fn textured_bind_group_create( device : &Device ) -> ( Texture, BindGroupLayout, BindGroup )
{
  let texture = device.texture_create( &TextureDesc
  {
    size : [ 64, 64, 1 ],
    format : TextureFormat::Rgba8Unorm,
    usage : TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST
  } )
  .expect( "texture creation failed" );
  let texture_view = texture.view().expect( "texture view creation failed" );
  let sampler = device.sampler_create( SamplerDesc::default() )
  .expect( "sampler creation failed" );

  let layout = device.bind_group_layout_create
  (
    &[
      BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Texture },
      BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::Sampler }
    ]
  )
  .expect( "bind group layout creation failed" );
  let bind_group = device.bind_group_create
  (
    &layout,
    &[ BindingResource::TextureView( &texture_view ), BindingResource::Sampler( &sampler ) ]
  )
  .expect( "bind group creation failed" );

  ( texture, layout, bind_group )
}

/// Builds the full textured-quad scene the test renders and re-renders.
fn textured_scene_setup() -> TexturedScene
{
  let ( device, queue, surface ) = Device::new_native( 64, 64 )
  .expect
  (
    "no native wgpu adapter : the native backend needs a Vulkan ICD \
     ( a software one such as lavapipe / mesa-vulkan-drivers suffices )"
  );

  let shader = device.shader_module_create( &ShaderSource
  {
    wgsl : TEXTURE_WGSL,
    glsl_vertex : None,
    glsl_fragment : None
  } )
  .expect( "shader module creation failed" );

  let ( vertex_buffer, index_buffer ) = fullscreen_geometry_create( &device );
  let ( texture, layout, bind_group ) = textured_bind_group_create( &device );

  let pipeline = device.render_pipeline_create( &RenderPipelineDesc
  {
    shader : &shader,
    vertex_entry : "vs_main",
    fragment_entry : "fs_main",
    vertex_buffers : &[ VertexBufferLayout
    {
      stride : 8,
      attributes : vec! [ VertexAttribute { location : 0, format : VertexFormat::Float32x2, offset : 0 } ]
    } ],
    bind_group_layouts : &[ &layout ],
    color_format : surface.format(),
    depth : None,
    cull_back : false
  } )
  .expect( "pipeline creation failed" );

  TexturedScene { device, queue, surface, pipeline, bind_group, vertex_buffer, index_buffer, texture }
}

/// Renders one frame and reads back the center pixel — called once per
/// `texture_write` call below to prove each upload actually lands.
fn center_sample( scene : &TexturedScene ) -> [ u8 ; 4 ]
{
  let view = scene.surface.current_view().expect( "surface view unavailable" );
  let mut encoder = scene.device.command_encoder_create();
  let mut pass = encoder.render_pass_begin
  (
    &ColorAttachmentDesc { view : &view, clear : [ 0.0, 0.0, 0.0, 1.0 ] },
    None
  )
  .expect( "render pass failed to begin" );
  pass.pipeline_set( &scene.pipeline );
  pass.bind_group_set( 0, &scene.bind_group );
  pass.vertex_buffer_set( 0, &scene.vertex_buffer );
  pass.index_buffer_set( &scene.index_buffer, IndexFormat::Uint32 );
  pass.draw_indexed( 3 );
  pass.end();
  scene.queue.submit( encoder );

  let pixels = scene.surface.pixels_read( &scene.device, &scene.queue ).expect( "readback failed" );
  let start = ( ( 32u32 * 64 + 32 ) * 4 ) as usize;
  [ pixels[ start ], pixels[ start + 1 ], pixels[ start + 2 ], pixels[ start + 3 ] ]
}

#[ test ]
fn texture_write_readback()
{
  let scene = textured_scene_setup();
  let texel_count = ( 64 * 64 ) as usize;

  // T01 : the upload lands and is sampled back correctly.
  let red : Vec< u8 > = [ 255u8, 0, 0, 255 ].repeat( texel_count );
  scene.queue.texture_write( &scene.texture, &red ).expect( "red texture_write failed" );
  assert_eq!( center_sample( &scene ), [ 255, 0, 0, 255 ], "sampled color should be the uploaded red" );

  // T02 : overwrite semantics — a genuinely different color replaces the
  // first, proving this isn't stale or cached data.
  let green : Vec< u8 > = [ 0u8, 255, 0, 255 ].repeat( texel_count );
  scene.queue.texture_write( &scene.texture, &green ).expect( "green texture_write failed" );
  assert_eq!( center_sample( &scene ), [ 0, 255, 0, 255 ], "sampled color should be the overwritten green" );
}
