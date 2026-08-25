//! Native Vulkan backend tests : a real `ash`/Vulkan device via `minvulkan`,
//! no `wgpu` involved ( see `docs/adr/004_native_vulkan_hal_backend.md` ). A
//! software Vulkan ICD ( e.g. lavapipe / mesa-vulkan-drivers ) suffices. The
//! render test draws through the full public HAL surface and asserts on
//! pixels read back from the offscreen surface, mirroring
//! `native_backend_test.rs`'s `triangle_render_readback` exact-equality
//! style — task 202's T02.
#![ cfg( all( feature = "vulkan", not( target_arch = "wasm32" ) ) ) ]
#![ allow( unsafe_code, reason = "as_vulkan_returns_device_usable_through_raw_handle exercises a raw \
`ash` FFI call ( `device_wait_idle` ) against a live Vulkan device to prove the handle `as_vulkan()` \
returns is genuinely usable ; the call site carries its own `// SAFETY:` comment" ) ]

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

/// T01 : `Device::new_vulkan` returns a valid `Device` wrapping the vulkan
/// variant.
#[ test ]
fn device_creation()
{
  let ( device, _queue, surface ) = Device::new_vulkan( 64, 64 )
  .expect
  (
    "no Vulkan device : the vulkan backend needs a Vulkan ICD \
     ( a software one such as lavapipe / mesa-vulkan-drivers suffices )"
  );
  assert_eq!( device.depth_range(), DepthRange::ZeroToOne );
  assert_eq!( surface.format(), TextureFormat::Rgba8Unorm );
}

/// T02 : construct device, load a single solid-color triangle asset, submit
/// one draw, call `Surface::pixels_read` — the readback must return real
/// pixel bytes, with the center pixel matching the configured draw color and
/// a corner pixel matching the clear color ( mirrors task 087's T02
/// exact-equality style ).
#[ test ]
fn triangle_render_readback()
{
  // 100 px wide, matching `native_backend_test.rs`'s own choice for direct
  // comparability — unlike native's `wgpu`-mediated path, Vulkan's
  // `BufferImageCopy::buffer_row_length( 0 )` copies tightly-packed rows
  // with no 256-byte alignment padding, so this width exercises no
  // row-unpacking path here ( see `docs/feature/006_native_pixel_readback.md` ).
  let width = 100u32;
  let height = 100u32;
  let ( device, queue, surface ) = Device::new_vulkan( width, height )
  .expect
  (
    "no Vulkan device : the vulkan backend needs a Vulkan ICD \
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

/// T03 : `device.as_vulkan()` on a non-vulkan-constructed `Device` returns
/// `None` ( non-panicking accessor, per ADR-002 ). Requires the `native`
/// feature too, to construct the non-vulkan `Device` this test needs.
#[ cfg( feature = "native" ) ]
#[ test ]
fn as_vulkan_returns_none_on_native_device()
{
  let ( device, _queue, _surface ) = Device::new_native( 4, 4 )
  .expect( "no native wgpu adapter available" );
  assert!( device.as_vulkan().is_none(), "as_vulkan must return None on a Device::Native handle" );
}

/// `device.as_vulkan()` on a Vulkan-constructed `Device` returns `Some` with a
/// genuinely usable `DeviceVulkan` — the positive counterpart of
/// `as_vulkan_returns_none_on_native_device` above, which only ever exercises
/// the mismatch ( `None` ) branch. Usability is proven by a real driver
/// round-trip through the returned handle's `ash::Device` : `vkDeviceWaitIdle`
/// only succeeds against a genuinely live device.
#[ test ]
fn as_vulkan_returns_device_usable_through_raw_handle()
{
  let ( device, _queue, _surface ) = Device::new_vulkan( 4, 4 )
  .expect
  (
    "no Vulkan device : the vulkan backend needs a Vulkan ICD \
     ( a software one such as lavapipe / mesa-vulkan-drivers suffices )"
  );

  let raw = device.as_vulkan().expect( "as_vulkan must return Some on a Device::Vulkan handle" );

  // SAFETY: `raw.device` is the live `ash::Device` owned by `device`, which is
  // not dropped until after this call returns ; `device_wait_idle` performs no
  // writes through caller-supplied pointers.
  let result = unsafe { raw.device.device_wait_idle() };
  assert!
  (
    result.is_ok(),
    "device_wait_idle should succeed on a freshly-created, idle device drilled down through as_vulkan()"
  );
}

/// Samples one texel from a texture at a fixed interior UV — mirrors
/// `native_backend_test.rs`'s `TEXTURE_WGSL` exactly, exercising
/// `texture_create`, `sampler_create` and `texture_write` under Vulkan
/// specifically ( `triangle_render_readback` above only exercises a
/// `UniformBuffer` binding, never `Texture`/`Sampler` ).
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

/// Resources for `vulkan_texture_write_readback`, kept alive across both writes.
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

/// A 64×64 `Rgba8Unorm` texture ( left empty — filled by the test itself via
/// `texture_write` ), its bind group layout, and a bind group pairing it
/// with a sampler. Texture entry precedes the sampler entry, matching
/// `native_backend_test.rs`'s own load-bearing-on-WebGL entry order.
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
  let ( device, queue, surface ) = Device::new_vulkan( 64, 64 )
  .expect
  (
    "no Vulkan device : the vulkan backend needs a Vulkan ICD \
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

/// T04 : construct a textured quad, upload a texture through `texture_write`,
/// and confirm the sampled center pixel matches — proving `texture_create`,
/// `sampler_create` and `texture_write` all work end-to-end under Vulkan.
/// Then overwrites with a different color to rule out stale/cached data.
#[ test ]
fn vulkan_texture_write_readback()
{
  let scene = textured_scene_setup();
  let texel_count = ( 64 * 64 ) as usize;

  let red : Vec< u8 > = [ 255u8, 0, 0, 255 ].repeat( texel_count );
  scene.queue.texture_write( &scene.texture, &red ).expect( "red texture_write failed" );
  assert_eq!( center_sample( &scene ), [ 255, 0, 0, 255 ], "sampled color should be the uploaded red" );

  let green : Vec< u8 > = [ 0u8, 255, 0, 255 ].repeat( texel_count );
  scene.queue.texture_write( &scene.texture, &green ).expect( "green texture_write failed" );
  assert_eq!( center_sample( &scene ), [ 0, 255, 0, 255 ], "sampled color should be the overwritten green" );
}

// test_kind: bug_reproducer(BUG-430)
/// ## Root Cause
/// `gpu_hal` had a full resource-CREATE API ( `buffer_create`, `texture_create`,
/// `sampler_create`, `shader_module_create`, `bind_group_layout_create`,
/// `bind_group_create`, `render_pipeline_create` ) but no destroy/free counterpart on
/// any backend. This crate's Vulkan backend at least disclosed the leak in its own
/// module doc comment ( `vulkan.rs` ), but the disclosure never reached the public
/// `Device::*_create` doc comments in `device.rs` and no escape-hatch method existed
/// to free anything early — every `vkCreate*`/`vkAllocateMemory` call this backend
/// makes had no `vkDestroy*`/`vkFreeMemory` counterpart reachable from the public API.
/// ## Why Not Caught
/// Every existing test in this crate runs as an isolated, short-lived `cargo nextest`
/// process — one process per test — so a per-resource leak never accumulates across a
/// run and produces no observable failure. No test exercised resource teardown at all,
/// only creation.
/// ## Fix Applied
/// Added `Device::buffer_destroy`/`texture_destroy`/`texture_view_destroy`/
/// `sampler_destroy`/`shader_module_destroy`/`bind_group_layout_destroy`/
/// `bind_group_destroy`/`render_pipeline_destroy`; the Vulkan arm of every one of
/// these issues a real `vkDestroy*`/`vkFreeMemory` call ( `vulkan.rs`'s own new
/// "Resource destruction" section ), dispatched through the same
/// match-self-then-match-owned-resource-by-value pattern this file's own
/// `Queue::submit`/`vulkan_queue_submit` already established.
/// ## Prevention
/// This test creates one resource of every type through the Vulkan backend, destroys
/// each one through its new `Device::*_destroy` method, then confirms the device is
/// still fully usable afterward by running a real render + submit + readback. Unlike
/// the native-backend companion test in `tests/native_backend_test.rs` ( where 6 of 8
/// destroy calls are no-ops, since `wgpu`'s own `Drop` already does the work ), every
/// one of the 8 destroy calls here issues a real `vkDestroy*`/`vkFreeMemory` call —
/// this is the crate's only test giving `TextureView`/`Sampler`/`ShaderModule`/
/// `BindGroupLayout`/`BindGroup`/`RenderPipeline` genuine per-type teardown coverage.
/// A leak or an early free ( use-after-free / double-free ) would either be silently
/// invisible ( leak ; this process-isolated test can't detect that either ) or
/// reliably surface as a validation-layer abort or a corrupted post-destroy render —
/// the second half of this test is what actually catches the latter.
/// ## Pitfall
/// Vulkan's own destroy calls are void-returning and never validate liveness by
/// themselves ( `vkDestroyBuffer` on an already-destroyed handle is undefined
/// behavior, not a returned error ) — a `Device::*_destroy` method that accidentally
/// got called twice on the same handle would not raise any Rust-visible panic at all,
/// only silent memory corruption a validation layer might or might not catch. Taking
/// every resource type by value ( not `&resource` ) is what makes a second call a
/// compile error instead : ownership is consumed the first time.
#[ test ]
fn resource_destroy_methods_do_not_panic()
{
  let ( device, queue, surface ) = Device::new_vulkan( 8, 8 )
  .expect
  (
    "no Vulkan device : the vulkan backend needs a Vulkan ICD \
     ( a software one such as lavapipe / mesa-vulkan-drivers suffices )"
  );

  all_resource_types_create_and_destroy( &device, &surface );
  device_usable_after_destroying_every_resource_type( &device, &queue, &surface );
}

/// Creates one resource of every type through the Vulkan backend, then
/// destroys each one through its new `Device::*_destroy` method. Split out
/// of `resource_destroy_methods_do_not_panic` to keep both halves under
/// this workspace's function-length lint threshold -- see that test's own
/// doc comment for the full `bug_reproducer(BUG-430)` rationale, which
/// applies to this half and `device_usable_after_destroying_every_resource_type`
/// jointly.
fn all_resource_types_create_and_destroy( device : &Device, surface : &Surface )
{
  let shader = device.shader_module_create( &ShaderSource
  {
    wgsl : WGSL,
    glsl_vertex : None,
    glsl_fragment : None
  } )
  .expect( "shader module creation failed" );

  let vertex_buffer = device.buffer_init_create( &as_bytes( &[ -0.5, -0.5, 0.5, -0.5, 0.0, 0.5 ] ), BufferUsage::VERTEX )
  .expect( "vertex buffer creation failed" );

  let texture = device.texture_create( &TextureDesc
  {
    size : [ 4, 4, 1 ],
    format : TextureFormat::Rgba8Unorm,
    usage : TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST
  } )
  .expect( "texture creation failed" );
  let texture_view = texture.view().expect( "texture view creation failed" );
  let sampler = device.sampler_create( SamplerDesc::default() )
  .expect( "sampler creation failed" );

  let layout = device.bind_group_layout_create
  (
    &[ BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::UniformBuffer } ]
  )
  .expect( "bind group layout creation failed" );
  let uniform_buffer = device.buffer_create( 16, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
  .expect( "uniform buffer creation failed" );
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
      attributes : vec! [ VertexAttribute { location : 0, format : VertexFormat::Float32x2, offset : 0 } ]
    } ],
    bind_group_layouts : &[ &layout ],
    color_format : surface.format(),
    depth : None,
    cull_back : false
  } )
  .expect( "pipeline creation failed" );

  // Previously : no method existed to reach any of this early -- the only way to free
  // any of it was dropping the whole `Device`. Dependents destroyed before their
  // dependencies ( pipeline/bind group before the layout/texture/sampler they were
  // built from ), matching `Device::texture_view_destroy`'s documented contract that a
  // view never outlives the texture it was built from.
  device.render_pipeline_destroy( pipeline );
  device.bind_group_destroy( bind_group );
  device.bind_group_layout_destroy( layout );
  device.texture_view_destroy( texture_view );
  device.texture_destroy( texture );
  device.sampler_destroy( sampler );
  device.buffer_destroy( uniform_buffer );
  device.buffer_destroy( vertex_buffer );
  device.shader_module_destroy( shader );
}

/// The device itself must still be usable after every resource type above is
/// gone -- a real render + submit + readback proves destroying all 8
/// resource types left no dangling internal state ( freed-too-early handle,
/// corrupted descriptor pool, etc. ) behind ( `WGSL`'s uniform-colored-triangle
/// shader, reused unchanged from `triangle_render_readback` above ). Split
/// out of `resource_destroy_methods_do_not_panic` -- see that test's own
/// `bug_reproducer(BUG-430)` doc comment for the full rationale.
fn device_usable_after_destroying_every_resource_type( device : &Device, queue : &Queue, surface : &Surface )
{
  let shader = device.shader_module_create( &ShaderSource { wgsl : WGSL, glsl_vertex : None, glsl_fragment : None } )
  .expect( "post-destroy shader module creation failed" );
  let vertex_buffer = device.buffer_init_create( &as_bytes( &[ -0.5, -0.5, 0.5, -0.5, 0.0, 0.5 ] ), BufferUsage::VERTEX )
  .expect( "post-destroy vertex buffer creation failed" );
  let indices : Vec< u8 > = [ 0u32, 1, 2 ].iter().flat_map( | i | i.to_le_bytes() ).collect();
  let index_buffer = device.buffer_init_create( &indices, BufferUsage::INDEX )
  .expect( "post-destroy index buffer creation failed" );
  let uniform_buffer = device.buffer_create( 16, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
  .expect( "post-destroy uniform buffer creation failed" );
  queue.buffer_write( &uniform_buffer, &as_bytes( &[ 0.0, 1.0, 0.0, 1.0 ] ) )
  .expect( "post-destroy uniform write failed" );
  let layout = device.bind_group_layout_create
  (
    &[ BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::UniformBuffer } ]
  )
  .expect( "post-destroy bind group layout creation failed" );
  let bind_group = device.bind_group_create( &layout, &[ BindingResource::Buffer( &uniform_buffer ) ] )
  .expect( "post-destroy bind group creation failed" );
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
  .expect( "post-destroy pipeline creation failed" );

  let view = surface.current_view().expect( "surface view unavailable" );
  let mut encoder = device.command_encoder_create();
  let mut pass = encoder.render_pass_begin
  (
    &ColorAttachmentDesc { view : &view, clear : [ 0.0, 0.0, 0.0, 1.0 ] },
    None
  )
  .expect( "post-destroy render pass failed to begin" );
  pass.pipeline_set( &pipeline );
  pass.bind_group_set( 0, &bind_group );
  pass.vertex_buffer_set( 0, &vertex_buffer );
  pass.index_buffer_set( &index_buffer, IndexFormat::Uint32 );
  pass.draw_indexed( 3 );
  pass.end();
  queue.submit( encoder );

  let pixels = surface.pixels_read( device, queue ).expect( "post-destroy readback failed" );
  assert_eq!( pixels.len(), ( 8 * 8 * 4 ) as usize, "device must still be fully usable after destroying every resource type" );
  let start = ( ( 4u32 * 8 + 4 ) * 4 ) as usize;
  assert_eq!( &pixels[ start..start + 4 ], &[ 0, 255, 0, 255 ], "post-destroy render must still produce correct pixels" );
}

// test_kind: bug_reproducer(BUG-470)
/// ## Root Cause
/// `Queue::submit`'s Vulkan backend ended, submitted, and waited on an
/// encoder's command buffer, but never destroyed the command pool
/// `command_encoder_create` allocated for it, or the render pass/framebuffer
/// pairs `render_pass_begin` created on it during recording -- none of the
/// three had a `vkDestroy*` call anywhere on this path. `CommandEncoderVulkan`
/// itself carried no record of what `render_pass_begin` had created on it, so
/// even adding a destroy call to `submit` would have had nothing to destroy
/// without first tracking those pairs somewhere.
/// ## Why Not Caught
/// Every existing test in this crate ( including `triangle_render_readback`
/// and BUG-430's own `resource_destroy_methods_do_not_panic` above ) submits
/// at most a handful of encoders per `cargo nextest`-isolated process and
/// never inspects `CommandEncoderVulkan`'s internal state before or after
/// `submit` -- a leak that never accumulates past a handful of frames within
/// one short-lived test process produces no observable failure. Only a
/// long-running windowed loop like `examples/gpu_hal/triangle_vulkan_window`
/// submits enough frames for the leak to matter.
/// ## Fix Applied
/// Added `CommandEncoderVulkan::pending_render_passes`, a `Vec` that
/// `render_pass_begin` now pushes every render pass/framebuffer pair it
/// creates onto; `Queue::submit`'s Vulkan backend drains it and destroys
/// every pair, plus the encoder's own command pool, immediately after
/// `vkQueueWaitIdle` confirms the GPU has finished executing everything that
/// referenced them -- the earliest point at which doing so is safe.
/// ## Prevention
/// This test's first half proves the tracking mechanism itself: a fresh
/// encoder starts with zero pending render passes, and each `render_pass_
/// begin`/`end` pair on it increments the count by exactly one, read back
/// through the same `as_vulkan()` accessor BUG-430's own tests established --
/// this assertion would not even compile before the fix, since `pending_
/// render_passes` did not exist. The second half runs many full create-
/// record-submit cycles back to back ( far closer to a real windowed present
/// loop than any single-frame test in this file ) and confirms the device
/// still renders correct pixels afterward, the same "still fully usable"
/// proof BUG-430's reproducer established for its own 8 resource types.
/// ## Pitfall
/// `render_pass_begin` can be called more than once on the same encoder
/// before it is submitted ( `command_encoder_create`'s own doc comment: "any
/// number of render passes can be begun/ended into it" ) -- a fix that
/// destroyed the pending pair inside `render_pass_end`/`RenderPass::end`
/// instead of waiting for `Queue::submit` would free objects a still-
/// recording, not-yet-submitted command buffer continues to reference, which
/// is undefined behavior the instant a later pass on the same encoder
/// records another command.
#[ test ]
fn command_pool_and_render_passes_do_not_leak()
{
  let ( device, queue, surface ) = Device::new_vulkan( 8, 8 )
  .expect
  (
    "no Vulkan device : the vulkan backend needs a Vulkan ICD \
     ( a software one such as lavapipe / mesa-vulkan-drivers suffices )"
  );

  render_passes_tracked_on_encoder_before_submit( &device, &queue, &surface );
  repeated_submit_cycles_leave_device_usable( &device, &queue, &surface );
}

/// Proves `render_pass_begin` accumulates exactly one entry per call onto its
/// encoder's `pending_render_passes`, and that submitting an encoder with
/// several pending pairs at once does not panic. Split out of
/// `command_pool_and_render_passes_do_not_leak` -- see that test's own
/// `bug_reproducer(BUG-470)` doc comment for the full rationale.
fn render_passes_tracked_on_encoder_before_submit( device : &Device, queue : &Queue, surface : &Surface )
{
  let mut encoder = device.command_encoder_create();
  let pending = | encoder : &CommandEncoder |
  {
    encoder.as_vulkan().expect( "as_vulkan must return Some on a Device::Vulkan-backed encoder" )
    .pending_render_passes.len()
  };
  assert_eq!( pending( &encoder ), 0, "a freshly created encoder must start with no pending render passes" );

  for expected in 1..=3usize
  {
    let view = surface.current_view().expect( "surface view unavailable" );
    encoder.render_pass_begin( &ColorAttachmentDesc { view : &view, clear : [ 0.0, 0.0, 0.0, 1.0 ] }, None )
    .expect( "render pass failed to begin" )
    .end();
    assert_eq!( pending( &encoder ), expected, "render_pass_begin must push exactly one pending pair per call" );
  }

  // Submitting with several pending pairs at once exercises `submit`'s
  // destroy loop across more than one iteration, unlike every cycle in
  // `repeated_submit_cycles_leave_device_usable` below, which only ever
  // accumulates one.
  queue.submit( encoder );
}

/// Runs many full create-encoder / begin-pass / draw / submit cycles back to
/// back -- far closer to a real windowed present loop than any single-frame
/// test in this file -- then confirms the device still renders correct
/// pixels afterward. Split out of `command_pool_and_render_passes_do_not_leak`
/// -- see that test's own `bug_reproducer(BUG-470)` doc comment for the full
/// rationale.
fn repeated_submit_cycles_leave_device_usable( device : &Device, queue : &Queue, surface : &Surface )
{
  let shader = device.shader_module_create( &ShaderSource { wgsl : WGSL, glsl_vertex : None, glsl_fragment : None } )
  .expect( "shader module creation failed" );
  let vertex_buffer = device.buffer_init_create( &as_bytes( &[ -0.5, -0.5, 0.5, -0.5, 0.0, 0.5 ] ), BufferUsage::VERTEX )
  .expect( "vertex buffer creation failed" );
  let indices : Vec< u8 > = [ 0u32, 1, 2 ].iter().flat_map( | i | i.to_le_bytes() ).collect();
  let index_buffer = device.buffer_init_create( &indices, BufferUsage::INDEX )
  .expect( "index buffer creation failed" );
  let uniform_buffer = device.buffer_create( 16, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
  .expect( "uniform buffer creation failed" );
  queue.buffer_write( &uniform_buffer, &as_bytes( &[ 0.0, 0.0, 1.0, 1.0 ] ) )
  .expect( "uniform write failed" );
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
      step_mode : StepMode::Vertex,
      attributes : vec! [ VertexAttribute { location : 0, format : VertexFormat::Float32x2, offset : 0 } ]
    } ],
    bind_group_layouts : &[ &layout ],
    color_format : surface.format(),
    depth : None,
    cull_back : false
  } )
  .expect( "pipeline creation failed" );

  // 50 cycles : enough to be a meaningfully repeated loop ( unlike every
  // other test in this file, which submits once or twice ) while staying
  // fast under a software ICD -- not a stand-in for genuine exhaustion
  // detection, which no test in this crate attempts ( see this test's own
  // `bug_reproducer(BUG-470)` doc comment's Why Not Caught section ).
  for _ in 0..50u32
  {
    let view = surface.current_view().expect( "surface view unavailable" );
    let mut encoder = device.command_encoder_create();
    let mut pass = encoder.render_pass_begin
    (
      &ColorAttachmentDesc { view : &view, clear : [ 0.0, 0.0, 0.0, 1.0 ] },
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

  let pixels = surface.pixels_read( device, queue ).expect( "readback failed" );
  assert_eq!( pixels.len(), ( 8 * 8 * 4 ) as usize );
  let start = ( ( 4u32 * 8 + 4 ) * 4 ) as usize;
  assert_eq!
  (
    &pixels[ start..start + 4 ], &[ 0, 0, 255, 255 ],
    "device must still render correctly after many repeated submit cycles"
  );
}
