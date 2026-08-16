//! Shared render path reused by every backend feature's `main.rs` binary —
//! the only step that actually differs per backend ( live browser
//! presentation for webgpu/webgl vs. one offscreen frame + PNG save for
//! wgpu/vulkan ) lives there; this crate exposes the one call in common.

pub mod uniforms;

// Mirrors `main.rs`'s own guards : `cargo check`/`build` verifies the `lib`
// target before the `bin` target, so without a copy here a zero-or-multiple
// -feature build fails on the `use gpu_hal::{ .. }` block below ( a confusing
// E0432 "no X in the root" ) before `main.rs`'s clearer compile_error! is
// ever reached.
#[ cfg( not( any(
  feature = "webgl",
  feature = "webgpu",
  feature = "wgpu",
  feature = "vulkan",
) ) ) ]
compile_error!( "orrery_flexible: select exactly one backend feature — webgl, webgpu, wgpu, or vulkan" );

#[ cfg( any(
  all( feature = "webgl", feature = "webgpu" ),
  all( feature = "webgl", feature = "wgpu" ),
  all( feature = "webgl", feature = "vulkan" ),
  all( feature = "webgpu", feature = "wgpu" ),
  all( feature = "webgpu", feature = "vulkan" ),
  all( feature = "wgpu", feature = "vulkan" ),
) ) ]
compile_error!( "orrery_flexible: more than one backend feature is enabled — select exactly one ( webgl, webgpu, wgpu, or vulkan ). `wgpu` is the default feature, so building a different one also needs --no-default-features" );

use gpu_hal::
{
  BindGroupLayoutEntry,
  BindingResource,
  BindingType,
  BufferUsage,
  ColorAttachmentDesc,
  Device,
  Error,
  Queue,
  RenderPipelineDesc,
  ShaderSource,
  ShaderStages,
  Surface,
};

/// Uploads `uniform_bytes` and draws one fullscreen-triangle frame of the
/// shared orrery scene ( `orrery_webgpu::shader_source::assemble()`,
/// `vertex_buffers : &[]` — the shader's vertex stage reads only
/// `@builtin(vertex_index)`, the same technique `orrery_webgpu`'s own
/// WebGPU render loop uses ) into `surface`'s current view.
///
/// `glsl` is `Some( ( vertex, fragment ) )` on the `webgl` feature only —
/// the WebGL backend cannot consume WGSL directly ( see `build.rs`, which
/// translates the shared WGSL at build time ); every other feature passes
/// `None` and the shader module carries WGSL alone.
///
/// # Errors
///
/// Returns whichever backend [`Error`] the failing `gpu_hal` call reports —
/// shader/buffer/bind-group/pipeline creation, or render pass acquisition.
pub fn scene_render
(
  device : &Device,
  queue : &Queue,
  surface : &Surface,
  uniform_bytes : &[ u8 ],
  glsl : Option< ( &str, &str ) >,
) -> Result< (), Error >
{
  let wgsl = orrery_webgpu::shader_source::assemble();
  let ( glsl_vertex, glsl_fragment ) = match glsl
  {
    Some( ( vertex, fragment ) ) => ( Some( vertex ), Some( fragment ) ),
    None => ( None, None ),
  };
  let shader = device.shader_module_create( &ShaderSource { wgsl : &wgsl, glsl_vertex, glsl_fragment } )?;

  let uniform_buffer = device.buffer_create
  (
    uniform_bytes.len() as u64,
    BufferUsage::UNIFORM | BufferUsage::COPY_DST,
  )?;
  queue.buffer_write( &uniform_buffer, uniform_bytes )?;

  let layout = device.bind_group_layout_create
  (
    &[ BindGroupLayoutEntry { visibility : ShaderStages::FRAGMENT, ty : BindingType::UniformBuffer } ]
  )?;
  let bind_group = device.bind_group_create( &layout, &[ BindingResource::Buffer( &uniform_buffer ) ] )?;

  let pipeline = device.render_pipeline_create( &RenderPipelineDesc
  {
    shader : &shader,
    vertex_entry : "vs_main",
    fragment_entry : "fs_main",
    vertex_buffers : &[],
    bind_group_layouts : &[ &layout ],
    color_format : surface.format(),
    depth : None,
    cull_back : false,
  } )?;

  let view = surface.current_view()?;
  let mut encoder = device.command_encoder_create();
  let mut pass = encoder.render_pass_begin
  (
    &ColorAttachmentDesc { view : &view, clear : [ 0.0, 0.0, 0.0, 1.0 ] },
    None,
  )?;
  pass.pipeline_set( &pipeline );
  pass.bind_group_set( 0, &bind_group );
  pass.draw( 3 );
  pass.end();
  queue.submit( encoder );

  Ok( () )
}
