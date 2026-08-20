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

/// `device.as_native()` on a native-constructed `Device` returns `Some` with a
/// genuinely usable `wgpu::Device` — proven by querying a real GPU-negotiated
/// limit through the returned reference, not just checking it's non-`None`
/// ( no test anywhere in this crate previously exercised the matching-backend
/// branch of any `as_native`/`as_vulkan`/`as_webgpu`/`as_webgl` accessor ).
#[ test ]
fn as_native_returns_device_usable_through_raw_handle()
{
  let ( device, _queue, _surface ) = Device::new_native( 4, 4 )
  .expect( "no native wgpu adapter available" );

  let raw = device.as_native().expect( "as_native must return Some on a Device::Native handle" );

  // `max_texture_dimension_2d` is a real limit negotiated with the adapter at
  // device creation ; wgpu's own downlevel-compatible floor
  // ( `Limits::downlevel_defaults()` ) never lets this drop below 2048, so
  // this also doubles as a sanity check on the queried value itself.
  let limits = raw.limits();
  assert!
  (
    limits.max_texture_dimension_2d >= 2048,
    "max_texture_dimension_2d should be a real GPU-negotiated limit drilled down through as_native(), got {}",
    limits.max_texture_dimension_2d
  );
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
      step_mode : StepMode::Vertex,
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
      step_mode : StepMode::Vertex,
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

/// ## Root Cause
/// `Device::texture_create` ( `src/device.rs` ) forwarded `desc.size` to every backend
/// unvalidated. The native backend's `wgpu::Device::create_texture` panics outright on a
/// zero-sized `Extent3d` — the same zero-size validation panic already fixed for
/// `Surface::configure` in BUG-165 — while WebGPU raises an uncaught validation error and WebGL's
/// `tex_storage_2d` silently no-ops on `INVALID_VALUE`, returning `Ok` for a texture that was
/// never actually allocated.
/// ## Why Not Caught
/// `texture_create` had no test exercising a zero-sized dimension on any backend prior to this
/// bug.
/// ## Fix Applied
/// `texture_create` now rejects any zero component of `desc.size` with `Error::InvalidInput`
/// before dispatching to any backend.
/// ## Prevention
/// This test constructs a `TextureDesc` with a zero width and asserts `texture_create` returns
/// `Err` instead of panicking — this crate's native backend previously panicked here, which
/// `#[ test ]` alone cannot distinguish from a hang without the process actually aborting.
/// ## Pitfall
/// A live browser canvas can transiently report `width()`/`height()` as `0` ( hidden tab, not yet
/// laid out ) — this is reachable with no malformed caller input at all, not just a theoretical
/// edge case.
#[ test ]
fn texture_create_rejects_zero_width()
{
  let ( device, _queue, _surface ) = Device::new_native( 64, 64 )
  .expect( "no native wgpu adapter available" );

  let result = device.texture_create( &TextureDesc
  {
    size : [ 0, 64, 1 ],
    format : TextureFormat::Rgba8Unorm,
    usage : TextureUsage::TEXTURE_BINDING
  } );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "zero width must be rejected with InvalidInput, got {result:?}" );
}

#[ test ]
fn texture_create_rejects_zero_height()
{
  let ( device, _queue, _surface ) = Device::new_native( 64, 64 )
  .expect( "no native wgpu adapter available" );

  let result = device.texture_create( &TextureDesc
  {
    size : [ 64, 0, 1 ],
    format : TextureFormat::Rgba8Unorm,
    usage : TextureUsage::TEXTURE_BINDING
  } );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "zero height must be rejected with InvalidInput, got {result:?}" );
}

#[ test ]
fn texture_create_rejects_zero_depth_or_array_layers()
{
  let ( device, _queue, _surface ) = Device::new_native( 64, 64 )
  .expect( "no native wgpu adapter available" );

  let result = device.texture_create( &TextureDesc
  {
    size : [ 64, 64, 0 ],
    format : TextureFormat::Rgba8Unorm,
    usage : TextureUsage::TEXTURE_BINDING
  } );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "zero depth/array-layers must be rejected with InvalidInput, got {result:?}" );
}

#[ test ]
fn texture_create_accepts_well_formed_size()
{
  let ( device, _queue, _surface ) = Device::new_native( 64, 64 )
  .expect( "no native wgpu adapter available" );

  let result = device.texture_create( &TextureDesc
  {
    size : [ 64, 64, 1 ],
    format : TextureFormat::Rgba8Unorm,
    usage : TextureUsage::TEXTURE_BINDING
  } );

  assert!( result.is_ok(), "a well-formed size must still succeed — got {result:?}" );
}

/// ## Root Cause
/// `Device::new_native` ( `src/device.rs` ) forwarded `width`/`height` straight into
/// `wgpu::Device::create_texture`'s `Extent3d` unvalidated — the same zero-size validation panic
/// already fixed for `Surface::configure` ( BUG-165 ) and `Device::texture_create` ( BUG-176 ) in
/// this same crate, just missed at this third call site.
/// ## Why Not Caught
/// `new_native` had no test exercising a zero `width`/`height` prior to this bug — every existing
/// call site in this file passes a hardcoded nonzero size.
/// ## Fix Applied
/// `new_native` now rejects a zero `width` or `height` with `Error::InvalidInput` before
/// constructing the offscreen surface texture.
/// ## Prevention
/// This test calls `new_native` with a zero component and asserts it returns `Err` instead of
/// panicking — this crate's native backend previously panicked here, which `#[ test ]` alone
/// cannot distinguish from a hang without the process actually aborting.
/// ## Pitfall
/// `width`/`height` are plain public `u32` parameters with no caller-side guarantee of non-zero —
/// reachable with entirely ordinary caller input ( e.g. a size derived from a not-yet-laid-out
/// viewport or an unloaded image ), not just a theoretical edge case.
#[ test ]
fn new_native_rejects_zero_width()
{
  let result = Device::new_native( 0, 64 );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "zero width must be rejected with InvalidInput, got {result:?}" );
}

#[ test ]
fn new_native_rejects_zero_height()
{
  let result = Device::new_native( 64, 0 );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "zero height must be rejected with InvalidInput, got {result:?}" );
}

/// ## Root Cause
/// `Queue::texture_write` ( `src/device.rs` ) forwarded `data` to `wgpu::Queue::write_texture`
/// unvalidated. That method is documented ( `wgpu-30.0.0/src/api/queue.rs` ) to "fail... if `data`
/// is too short", but its signature returns `()`, not `Result` — the failure can only reach wgpu's
/// own error sink. This crate installs no custom `on_uncaptured_error` handler, so wgpu-core's
/// `default_error_handler` takes over and panics unconditionally ( confirmed by reading
/// `wgpu-core-30.0.0/src/backend/wgpu_core.rs`: "Handling wgpu errors as fatal by default" ) — the
/// same "unguarded native panic on bad input" class already fixed for `Surface::configure`
/// ( BUG-165 ), `texture_create` ( BUG-176 ) and `new_native` ( BUG-199 ) in this file.
/// ## Why Not Caught
/// `texture_write_readback` only ever wrote exactly-sized data ( `64 * 64 * 4` bytes for a
/// `64×64` `Rgba8Unorm` texture ) — no existing test exercised an undersized write.
/// ## Fix Applied
/// `texture_write`'s native arm now computes the region's required byte count from the same
/// `bytes_per_row`/`height`/`depth_or_array_layers` it already derives for the write call itself,
/// and rejects a shorter `data` with `Error::InvalidInput` before calling
/// `wgpu::Queue::write_texture`.
/// ## Prevention
/// This test writes 4 bytes into a `2×2` `Rgba8Unorm` texture ( which needs 16 ) and asserts
/// `texture_write` returns `Err` instead of panicking — this crate's native backend previously
/// panicked here, which `#[ test ]` alone cannot distinguish from a hang without the process
/// actually aborting.
/// ## Pitfall
/// `data`'s length is caller-supplied with no compile-time link to the texture it's written into —
/// e.g. a texture resized after its upload buffer was sized, or a format change ( more bytes/texel )
/// without a matching resize of the source data — is reachable with entirely ordinary caller input,
/// not just a theoretical edge case.
#[ test ]
fn texture_write_rejects_undersized_data()
{
  let ( device, queue, _surface ) = Device::new_native( 64, 64 )
  .expect( "no native wgpu adapter available" );
  let texture = device.texture_create( &TextureDesc
  {
    size : [ 2, 2, 1 ],
    format : TextureFormat::Rgba8Unorm,
    usage : TextureUsage::COPY_DST
  } )
  .expect( "texture creation failed" );

  // 2×2 Rgba8Unorm needs 2 * 2 * 4 = 16 bytes; this is 4.
  let result = queue.texture_write( &texture, &[ 0u8, 0, 0, 255 ] );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "undersized data must be rejected with InvalidInput, got {result:?}" );
}

/// ## Root Cause
/// `Queue::buffer_write` ( `src/device.rs` ) forwarded `data` to `wgpu::Queue::write_buffer`
/// unvalidated. That method is documented ( `wgpu-30.0.0/src/api/queue.rs` ) to require the
/// write stay fully in-bounds and — per wgpu-core's own `validate_write_buffer_impl`
/// ( `wgpu-core-30.0.0/src/device/queue.rs` ) — that `data.len()` be a multiple of
/// `wgpu::COPY_BUFFER_ALIGNMENT` ( 4 bytes ), but the method's signature returns `()`, not
/// `Result` — a violation can only reach wgpu's own error sink. This crate installs no custom
/// `on_uncaptured_error` handler, so wgpu-core's `default_error_handler` takes over and panics
/// unconditionally — the same "unguarded native panic on bad input" class already fixed for
/// `Surface::configure` ( BUG-165 ), `texture_create` ( BUG-176 ), `new_native` ( BUG-199 ) and
/// `texture_write` ( BUG-204 ) in this file, just reached through a 5th call site.
/// ## Why Not Caught
/// Every existing call site writes a hardcoded, correctly-sized/aligned payload ( e.g. the
/// 16-byte uniform write in `triangle_render_readback` ) — no test exercised a misaligned or
/// oversized write.
/// ## Fix Applied
/// `buffer_write`'s native arm now rejects a `data` whose length isn't a multiple of
/// `wgpu::COPY_BUFFER_ALIGNMENT`, or that overruns the destination buffer's own allocated size,
/// with `Error::InvalidInput` before calling `wgpu::Queue::write_buffer`.
/// ## Prevention
/// This test writes 3 bytes ( not a multiple of the 4-byte `COPY_BUFFER_ALIGNMENT` ) into an
/// 8-byte buffer and asserts `buffer_write` returns `Err` instead of panicking — this crate's
/// native backend previously panicked here, which `#[ test ]` alone cannot distinguish from a
/// hang without the process actually aborting.
/// ## Pitfall
/// `data`'s length is caller-supplied with no compile-time link to either wgpu's own alignment
/// requirement or the destination buffer's allocated size — e.g. a caller serializing a
/// non-4-byte-aligned struct, or writing a buffer sized for a since-shrunk resource — is
/// reachable with entirely ordinary caller input, not just a theoretical edge case.
#[ test ]
fn buffer_write_rejects_misaligned_data()
{
  let ( device, queue, _surface ) = Device::new_native( 64, 64 )
  .expect( "no native wgpu adapter available" );
  let buffer = device.buffer_create( 8, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
  .expect( "buffer creation failed" );

  // 3 bytes is not a multiple of wgpu's 4-byte COPY_BUFFER_ALIGNMENT.
  let result = queue.buffer_write( &buffer, &[ 0u8, 0, 0 ] );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "misaligned data must be rejected with InvalidInput, got {result:?}" );
}

#[ test ]
fn buffer_write_rejects_oversized_data()
{
  let ( device, queue, _surface ) = Device::new_native( 64, 64 )
  .expect( "no native wgpu adapter available" );
  let buffer = device.buffer_create( 8, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
  .expect( "buffer creation failed" );

  // 12 bytes ( a valid 4-byte-aligned length ) overruns the 8-byte buffer.
  let result = queue.buffer_write( &buffer, &as_bytes( &[ 1.0, 2.0, 3.0 ] ) );

  assert!( matches!( result, Err( Error::InvalidInput( _ ) ) ), "oversized data must be rejected with InvalidInput, got {result:?}" );
}

/// ## Root Cause
/// `RenderPass::vertex_buffer_set`/`index_buffer_set` ( `src/pass.rs` ) forwarded a zero-size
/// buffer straight into `wgpu::RenderPass::set_vertex_buffer`/`set_index_buffer`, which slice
/// the buffer via `BufferSlice::size_expect_nonzero()` ( `wgpu-30.0.0/src/api/render_pass.rs` ) —
/// documented to panic ( "# Panics ... if the buffer's size is zero" ) whenever the bound
/// buffer's own allocated size is 0. Reachable with ordinary input: an all-empty
/// `renderer::webgpu::Geometry` traces end-to-end through `renderer.rs`'s per-slot
/// `vertex_buffer_set` loop, which applies no `vertex_count > 0` guard before binding.
/// ## Why Not Caught
/// Every existing render test ( e.g. `triangle_render_readback` ) binds a buffer holding real
/// vertex/index data — no test exercised binding a buffer created with size 0.
/// ## Fix Applied
/// `vertex_buffer_set`'s and `index_buffer_set`'s native arms now skip the
/// `wgpu::RenderPass::set_vertex_buffer`/`set_index_buffer` call entirely when the bound
/// buffer's own allocated size is 0 — a zero-size buffer has nothing to read regardless of
/// whether it's bound, so the skip is a safe no-op, mirroring the WebGL arm's own existing
/// no-op convention in the same function.
/// ## Prevention
/// This test binds a buffer created with `device.buffer_create( 0, ... )` as the vertex/index
/// buffer mid-pass and asserts the pass still ends and submits without panicking — this crate's
/// native backend previously panicked here, which `#[ test ]` alone cannot distinguish from a
/// hang without the process actually aborting.
/// ## Pitfall
/// A buffer's size-zero-ness is a runtime property of caller-supplied geometry data ( e.g. an
/// empty mesh ), not something the type system tracks — reachable with entirely ordinary caller
/// input, not just a theoretical edge case.
#[ test ]
fn vertex_buffer_set_accepts_zero_size_buffer()
{
  let ( device, queue, surface ) = Device::new_native( 4, 4 )
  .expect( "no native wgpu adapter available" );
  let buffer = device.buffer_create( 0, BufferUsage::VERTEX )
  .expect( "zero-size buffer creation failed" );

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

  // Previously panicked here via wgpu's BufferSlice::size_expect_nonzero().
  pass.vertex_buffer_set( 0, &buffer );
  pass.end();
  queue.submit( encoder );
}

#[ test ]
fn index_buffer_set_accepts_zero_size_buffer()
{
  let ( device, queue, surface ) = Device::new_native( 4, 4 )
  .expect( "no native wgpu adapter available" );
  let buffer = device.buffer_create( 0, BufferUsage::INDEX )
  .expect( "zero-size buffer creation failed" );

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

  // Previously panicked here via wgpu's BufferSlice::size_expect_nonzero().
  pass.index_buffer_set( &buffer, IndexFormat::Uint32 );
  pass.end();
  queue.submit( encoder );
}
