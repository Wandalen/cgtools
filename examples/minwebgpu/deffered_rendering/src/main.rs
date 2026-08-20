//! Deferred rendering demo -- renders a grid of models into a G-buffer (albedo, normal,
//! position), then a lighting pass composites the G-buffer with a set of point lights,
//! each drawn with a small visualization mesh.
//!
//! This example only works on WebAssembly (wasm32) targets where WebGPU APIs are available.

// Fix(BUG-306-B): the module doc comment above used to read "Just draw a large point in
// the middle of the screen" -- copy-pasted from an unrelated example and never updated
// to describe this crate's actual deferred G-buffer rendering pipeline.
// Root cause: stale copy-paste doc comment, never cross-checked against this crate's own
// render passes after being carried over.
// Pitfall: a demo crate's own top-of-file doc comment is not exempt from doc/source
// cross-checking just because it's "only an example".

#[cfg(target_arch = "wasm32")]
use light::{LightState, LightVisualizationState, NUM_LIGHTS};

#[cfg(target_arch = "wasm32")]
use minwebgpu::
{
  self as gl,
  AsWeb
};
#[cfg(target_arch = "wasm32")]
use model::{ModelState, NUM_MODELS};
#[cfg(target_arch = "wasm32")]
use uniform::{Uniform, UniformState};

#[cfg(target_arch = "wasm32")]
mod uniform;
#[cfg(target_arch = "wasm32")]
mod light;
#[cfg(target_arch = "wasm32")]
mod model;

#[cfg(target_arch = "wasm32")]
fn textures_create
(
  device : &gl::web_sys::GpuDevice,
  size : [ u32; 3 ]
)
-> Result< [ gl::web_sys::GpuTexture; 3], gl::WebGPUError >
{
  // We create textures for every property we need to calculate lighting in the final pass: position, albedo and normal.
  // We don't need samplers as we can just use textureLoad with position.xy in the fragment to sample needed pixel.
  let color_tex_desc = gl::texture::desc()
  .size( size )
  .render_attachment() // Sets the usage flag to `RENDER_ATTACHMENT`
  .texture_binding() // Sets the usage flag to `TEXTURE_BINDING`
  .to_web();

  let vector_tex_desc = gl::texture::desc()
  .size( size )
  .render_attachment()
  .texture_binding()
  .format( gl::GpuTextureFormat::Rgba16float )
  .to_web();

  let position_tex = gl::texture::create( device, &vector_tex_desc )?;
  let albedo_tex = gl::texture::create( device, &color_tex_desc )?;
  let normal_tex = gl::texture::create( device, &vector_tex_desc )?;

  Ok( [ position_tex, albedo_tex, normal_tex ] )
}

/// Creates the gbuffer color textures and the depth texture, returning a view
/// for each : `[ position, albedo, normal, depth ]`.
#[cfg(target_arch = "wasm32")]
fn texture_views_create
(
  device : &gl::web_sys::GpuDevice,
  size : [ u32; 3 ]
)
-> Result< [ gl::web_sys::GpuTextureView; 4 ], gl::WebGPUError >
{
  let [ pos_tex, albedo_tex, normal_tex ] = textures_create( device, size )?;
  let depth_texture = gl::texture::create
  (
    device,
    &gl::texture::desc()
    .size( size )
    .render_attachment()
    .texture_binding()
    .format( gl::GpuTextureFormat::Depth24plus )
    .into()
  )?;

  let views =
  [
    pos_tex.create_view().unwrap(),
    albedo_tex.create_view().unwrap(),
    normal_tex.create_view().unwrap(),
    depth_texture.create_view().unwrap()
  ];
  Ok( views )
}

/// Creates the shared uniform bind group layout ( frame uniforms + light storage )
/// together with its bind group.
#[cfg(target_arch = "wasm32")]
fn uniform_bind_group_create
(
  device : &gl::web_sys::GpuDevice,
  uniform_buffer : &gl::web_sys::GpuBuffer,
  light_buffer : &gl::web_sys::GpuBuffer
)
-> Result< ( gl::web_sys::GpuBindGroupLayout, gl::web_sys::GpuBindGroup ), gl::WebGPUError >
{
  // First entry - uniform paramters like view_matrix, time
  // Second entry - array of lights
  // Fix(BUG-051): `.entry(..)`/`.entry_from_ty(..)` became fallible (`Result<Self, WebGPUError>`)
  // once the underlying `BindGroupLayoutEntry` conversion stopped panicking on an unset binding
  // type — propagate each with `?` instead of chaining directly.
  // Root cause: written against the old infallible builder signature.
  // Pitfall: a builder chain that becomes fallible mid-chain needs `?` after every call that
  // changed, not just the final one — the compiler flags each one individually.
  let uniform_bind_group_layout = gl::BindGroupLayoutDescriptor::new()
  .fragment()
  .auto_bindings()
  .entry
  (
    gl::BindGroupLayoutEntry::new()
    .vertex()
    .ty( gl::binding_type::buffer_type() )
  )?
  .entry_from_ty( gl::binding_type::buffer_type().storage_readonly() )?
  .create( device )?;

  let uniform_bind_group = gl::BindGroupDescriptor::new( &uniform_bind_group_layout )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( uniform_buffer ) )
  .entry_from_resource( &gl::BufferBinding::new( light_buffer ) )
  .create( device );

  Ok( ( uniform_bind_group_layout, uniform_bind_group ) )
}

/// Creates the gbuffer bind group layout and the two pipelines that fill the
/// gbuffer : the instanced model pipeline and the ground plane pipeline.
#[cfg(target_arch = "wasm32")]
fn gbuffer_pipelines_create
(
  device : &gl::web_sys::GpuDevice,
  gbuffer_shader : &gl::web_sys::GpuShaderModule,
  big_plane_shader : &gl::web_sys::GpuShaderModule,
  uniform_bind_group_layout : &gl::web_sys::GpuBindGroupLayout
)
-> Result< ( gl::web_sys::GpuBindGroupLayout, gl::web_sys::GpuRenderPipeline, gl::web_sys::GpuRenderPipeline ), gl::WebGPUError >
{
  let [ pos_vertex_layout, normal_vertex_layout, uv_vertex_layout ] = ModelState::vertex_layout();
  let model_instance_layout = ModelState::instance_layout();

  // Setup gbuffer related state
  let gbuffer_bind_group_layout = gl::layout::bind_group::create
  (
    device,
    // Sets the visibility `FRAGMENT` to all entries
    // And auto computes binding value for each entry
    // Fix(BUG-051): see the identical note above `uniform_bind_group_create` — each
    // `.entry_from_ty(..)` in this chain now returns `Result<Self, WebGPUError>`.
    &gl::layout::bind_group::desc()
    .fragment()
    .auto_bindings()
    .entry_from_ty( gl::binding_type::texture_type().sample_unfilterable_float() )?
    .entry_from_ty( gl::binding_type::texture_type().sample_unfilterable_float() )?
    .entry_from_ty( gl::binding_type::texture_type().sample_unfilterable_float() )?
    .entry_from_ty( gl::binding_type::texture_type().sample_depth() )?
    .to_web()
  )?;

  // Create pipeline layout for the gbuffer render pipeline
  let gbuffer_pipeline_layout = gl::layout::pipeline::desc()
  .bind_group( uniform_bind_group_layout )
  .create( device );

  let fragment_state = gl::FragmentState::new( gbuffer_shader )
  .target( gl::ColorTargetState::new() )
  .target( gl::ColorTargetState::new().format( gl::GpuTextureFormat::Rgba16float ) )
  .target( gl::ColorTargetState::new().format( gl::GpuTextureFormat::Rgba16float ) )
  .to_web();

  // Pipeline that will render to the gbuffer textures.
  let gbuffer_render_pipeline = gl::render_pipeline::create
  (
    device,
    &gl::render_pipeline::desc
    (
      gl::VertexState::new( gbuffer_shader )
      .buffer( &pos_vertex_layout )
      .buffer( &normal_vertex_layout )
      .buffer( &uv_vertex_layout )
      .buffer( &model_instance_layout )
    )
    .layout( &gbuffer_pipeline_layout )
    .fragment( fragment_state.clone() )
    .primitive( gl::PrimitiveState::new().cull_back() )
    .depth_stencil( gl::DepthStencilState::new() )
    .to_web()
  )?;

  // Pipeline that will render a plane.
  // We reuse the fragment state from gbuffer pipeline because they are the same.
  let big_plane_render_pipeline = gl::render_pipeline::create
  (
    device,
    &gl::render_pipeline::desc( gl::VertexState::new( big_plane_shader ) )
    .layout( &gbuffer_pipeline_layout )
    .fragment( fragment_state.clone() )
    .primitive( gl::PrimitiveState::new() )
    .depth_stencil( gl::DepthStencilState::new() )
    .to_web()
  )?;

  Ok( ( gbuffer_bind_group_layout, gbuffer_render_pipeline, big_plane_render_pipeline ) )
}

/// Binds the gbuffer texture views and the depth view for the lighting pass.
#[cfg(target_arch = "wasm32")]
fn gbuffer_bind_group_create
(
  device : &gl::web_sys::GpuDevice,
  layout : &gl::web_sys::GpuBindGroupLayout,
  albedo_view : &gl::web_sys::GpuTextureView,
  pos_view : &gl::web_sys::GpuTextureView,
  normal_view : &gl::web_sys::GpuTextureView,
  depth_view : &gl::web_sys::GpuTextureView
)
-> gl::web_sys::GpuBindGroup
{
  gl::bind_group::create
  (
    device,
    &gl::bind_group::desc( layout )
    .auto_bindings()
    .entry_from_resource( albedo_view )
    .entry_from_resource( pos_view )
    .entry_from_resource( normal_view )
    .entry_from_resource( depth_view )
    .to_web()
  )
}

/// Creates the fullscreen lighting pipeline that composes the gbuffer onto the canvas.
#[cfg(target_arch = "wasm32")]
fn lighting_pipeline_create
(
  device : &gl::web_sys::GpuDevice,
  render_shader : &gl::web_sys::GpuShaderModule,
  uniform_bind_group_layout : &gl::web_sys::GpuBindGroupLayout,
  gbuffer_bind_group_layout : &gl::web_sys::GpuBindGroupLayout,
  presentation_format : gl::web_sys::GpuTextureFormat
)
-> Result< gl::web_sys::GpuRenderPipeline, gl::WebGPUError >
{
  // The main render pipeline. It will do the lighting calculations based on
  // gbuffer texture we filled in gbuffer pipeline
  let render_pipeline_layout = gl::layout::pipeline::desc()
  .bind_group( uniform_bind_group_layout )
  .bind_group( gbuffer_bind_group_layout )
  .create( device );

  gl::render_pipeline::create
  (
    device,
    &gl::render_pipeline::desc( gl::VertexState::new( render_shader ) )
    .layout( &render_pipeline_layout )
    .fragment
    (
      gl::FragmentState::new( render_shader )
      .target( gl::ColorTargetState::new().format( presentation_format ) )
    )
    .primitive( gl::PrimitiveState::new().triangle_strip() )
    .to_web()
  )
}

/// Creates the light-update compute pipeline and the bind groups for the
/// light update and light visualization passes.
#[cfg(target_arch = "wasm32")]
fn light_bindings_create
(
  device : &gl::web_sys::GpuDevice,
  light_update_shader : &gl::web_sys::GpuShaderModule,
  uniform_buffer : &gl::web_sys::GpuBuffer,
  light_buffer : &gl::web_sys::GpuBuffer,
  light_vis_pipeline : &gl::web_sys::GpuRenderPipeline
)
-> ( gl::web_sys::GpuComputePipeline, gl::web_sys::GpuBindGroup, gl::web_sys::GpuBindGroup )
{
  // We create a compute pipeline to update lights
  // Sicne there is only one `compute` function in the shader,
  // the entry point will default to that function
  let light_compute_pipeline = gl::compute_pipeline::desc
  (
    gl::ProgrammableStage::new( light_update_shader )
  )
  .create( device );

  // We create bindgroup from `auto` layout of our pipeline
  let light_update_bind_group = gl::bind_group::desc
  (
    &light_compute_pipeline.get_bind_group_layout( 0 )
  )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( uniform_buffer ) )
  .entry_from_resource( &gl::BufferBinding::new( light_buffer ) )
  .create( device );

  // Light visualization
  let light_vis_bind_group = gl::bind_group::desc
  (
    &light_vis_pipeline.get_bind_group_layout( 0 )
  )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( uniform_buffer ) )
  .create( device );

  ( light_compute_pipeline, light_update_bind_group, light_vis_bind_group )
}

/// Records the gbuffer pass : draws the instanced models and the ground plane
/// into the albedo, position, and normal attachments.
#[cfg(target_arch = "wasm32")]
fn gbuffer_pass_record
(
  encoder : &gl::web_sys::GpuCommandEncoder,
  color_views : [ &gl::web_sys::GpuTextureView; 3 ],
  depth_view : &gl::web_sys::GpuTextureView,
  gbuffer_pipeline : &gl::web_sys::GpuRenderPipeline,
  big_plane_pipeline : &gl::web_sys::GpuRenderPipeline,
  uniform_bind_group : &gl::web_sys::GpuBindGroup,
  model_state : &ModelState
)
{
  let render_pass = encoder.begin_render_pass
  (
    &gl::RenderPassDescriptor::new()
    .color_attachment( gl::ColorAttachment::new( color_views[ 0 ] ) )
    .color_attachment( gl::ColorAttachment::new( color_views[ 1 ] ) )
    .color_attachment( gl::ColorAttachment::new( color_views[ 2 ] ) )
    .depth_stencil_attachment( gl::DepthStencilAttachment::new( depth_view ) )
    .into()
  ).unwrap();

  // Draw model
  render_pass.set_pipeline( gbuffer_pipeline );
  render_pass.set_bind_group( 0, Some( uniform_bind_group ) );
  render_pass.set_vertex_buffer( 0, Some( &model_state.pos_buffer ) );
  render_pass.set_vertex_buffer( 1, Some( &model_state.normal_buffer ) );
  render_pass.set_vertex_buffer( 2, Some( &model_state.uv_buffer ) );
  render_pass.set_vertex_buffer( 3, Some( &model_state.instance_buffer ) );
  render_pass.set_index_buffer( &model_state.index_buffer, gl::GpuIndexFormat::Uint32 );
  render_pass.draw_indexed_with_instance_count( model_state.index_length, NUM_MODELS as u32 );

  // Draw big plane
  render_pass.set_pipeline( big_plane_pipeline );
  render_pass.draw( 6 );
  render_pass.end();
}

/// Records the fullscreen lighting pass composing the gbuffer onto the canvas.
#[cfg(target_arch = "wasm32")]
fn lighting_pass_record
(
  encoder : &gl::web_sys::GpuCommandEncoder,
  canvas_view : &gl::web_sys::GpuTextureView,
  render_pipeline : &gl::web_sys::GpuRenderPipeline,
  uniform_bind_group : &gl::web_sys::GpuBindGroup,
  gbuffer_bind_group : &gl::web_sys::GpuBindGroup
)
{
  let render_pass = encoder.begin_render_pass
  (
    &gl::RenderPassDescriptor::new()
    .color_attachment
    (
      gl::ColorAttachment::new( canvas_view )
    )
    .into()
  ).unwrap();

  render_pass.set_pipeline( render_pipeline );
  render_pass.set_bind_group( 0, Some( uniform_bind_group ) );
  render_pass.set_bind_group( 1, Some( gbuffer_bind_group ) );
  render_pass.draw( 4 );
  render_pass.end();
}

/// Records the light visualization pass drawing every light source on top of
/// the lit scene, reusing the gbuffer depth.
#[cfg(target_arch = "wasm32")]
fn light_vis_pass_record
(
  encoder : &gl::web_sys::GpuCommandEncoder,
  canvas_view : &gl::web_sys::GpuTextureView,
  depth_view : &gl::web_sys::GpuTextureView,
  light_vis_pipeline : &gl::web_sys::GpuRenderPipeline,
  light_vis_bind_group : &gl::web_sys::GpuBindGroup,
  light_buffer : &gl::web_sys::GpuBuffer
)
{
  let render_pass = encoder.begin_render_pass
  (
    &gl::RenderPassDescriptor::new()
    .color_attachment
    (
      gl::ColorAttachment::new( canvas_view )
      .load_op( gl::GpuLoadOp::Load )
    )
    .depth_stencil_attachment
    (
      gl::DepthStencilAttachment::new( depth_view )
      .depth_load_op( gl::GpuLoadOp::Load )
    )
    .into()
  ).unwrap();

  render_pass.set_pipeline( light_vis_pipeline );
  render_pass.set_bind_group( 0, Some( light_vis_bind_group ) );
  render_pass.set_vertex_buffer( 0, Some( light_buffer ) );
  render_pass.draw_with_instance_count( 14, NUM_LIGHTS as u32 );
  render_pass.end();
}

/// Records the compute pass advancing every light's position.
#[cfg(target_arch = "wasm32")]
fn light_update_pass_record
(
  encoder : &gl::web_sys::GpuCommandEncoder,
  compute_pipeline : &gl::web_sys::GpuComputePipeline,
  bind_group : &gl::web_sys::GpuBindGroup
)
{
  let compute_pass = encoder.begin_compute_pass();

  compute_pass.set_pipeline( compute_pipeline );
  compute_pass.set_bind_group( 0, Some( bind_group ) );
  compute_pass.dispatch_workgroups( NUM_LIGHTS.div_ceil( 64 ) as u32 );
  compute_pass.end();
}

#[cfg(target_arch = "wasm32")]
async fn app_run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::retrieve_or_make()?;
  //let canvas = gl::canvas::make()?;
  let context = gl::context::from_canvas( &canvas )?;
  let adapter = gl::context::adapter_request().await?;
  let device = gl::context::device_request( &adapter ).await?;
  let queue = device.queue();

  let presentation_format = gl::context::preferred_format()?;
  gl::context::configure( &device, &context, presentation_format )?;

  let width = canvas.width();
  let height = canvas.height();

  let light_update_shader = gl::ShaderModule::new( include_str!( "../shaders/light_update.wgsl" ) ).create( &device );
  let big_plane_shader = gl::ShaderModule::new( include_str!( "../shaders/big_plane.wgsl" ) ).create( &device );
  let gbuffer_shader = gl::ShaderModule::new( include_str!( "../shaders/gbuffer.wgsl" ) ).create( &device );
  let render_shader = gl::ShaderModule::new( include_str!( "../shaders/render.wgsl" ) ).create( &device );

  let [ pos_view, albedo_view, normal_view, depth_view ] = texture_views_create( &device, [ width, height, 1 ] )?;

  // Create needed state
  let model_state = ModelState::new( &device ).await?;
  let mut uniform_state = UniformState::new( &device )?;
  let light_state = LightState::new( &device )?;
  let light_vis_state = LightVisualizationState::new( &device, presentation_format )?;

  let ( uniform_bind_group_layout, uniform_bind_group ) =
  uniform_bind_group_create( &device, &uniform_state.buffer, &light_state.buffer )?;

  let ( gbuffer_bind_group_layout, gbuffer_render_pipeline, big_plane_render_pipeline ) =
  gbuffer_pipelines_create( &device, &gbuffer_shader, &big_plane_shader, &uniform_bind_group_layout )?;

  let gbuffer_bind_group =
  gbuffer_bind_group_create( &device, &gbuffer_bind_group_layout, &albedo_view, &pos_view, &normal_view, &depth_view );

  let render_pipeline =
  lighting_pipeline_create( &device, &render_shader, &uniform_bind_group_layout, &gbuffer_bind_group_layout, presentation_format )?;

  let ( light_compute_pipeline, light_update_bind_group, light_vis_bind_group ) =
  light_bindings_create( &device, &light_update_shader, &uniform_state.buffer, &light_state.buffer, &light_vis_state.render_pipeline );

  // Define camera related parameters
  let eye = gl::math::F32x3::from( [ 70.0, 50.0, 0.0 ] );
  let center = gl::math::F32x3::ZERO;
  let up = gl::math::F32x3::Y;

  let fovy = 70f32.to_radians();
  let aspect = width as f32 / height as f32;
  let z_near = 0.1;
  let z_far = 1000.0;

  let projection_matrix = gl::math::mat3x3h::perspective_rh( fovy, aspect, z_near, z_far );

  // Define the update and draw logic
  let update_and_draw =
  {
    let mut prev_time = 0.0;
    move | t : f64 |
    {
      let elapsed_time = ( ( t - prev_time ) / 1000.0 ) as f32;
      prev_time = t;
      let t = ( t / 1000.0 ) as f32;

      let canvas_texture = gl::context::current_texture( &context ).unwrap();
      let canvas_view = gl::texture::view( &canvas_texture ).unwrap();
      // let rot = gl::math::mat3x3::from_angle_y( t );
      // let eye = rot * eye;

      let view_matrix = gl::math::mat3x3h::look_at_rh( eye, center, up );
      uniform_state.uniform = Uniform
      {
        view_matrix,
        projection_matrix,
        camera_pos : eye,
        time : t,
        elapsed_time
      };

      uniform_state.update( &queue ).unwrap();

      let encoder = device.create_command_encoder();

      gbuffer_pass_record
      (
        &encoder,
        [ &albedo_view, &pos_view, &normal_view ],
        &depth_view,
        &gbuffer_render_pipeline,
        &big_plane_render_pipeline,
        &uniform_bind_group,
        &model_state
      );
      lighting_pass_record( &encoder, &canvas_view, &render_pipeline, &uniform_bind_group, &gbuffer_bind_group );
      light_vis_pass_record( &encoder, &canvas_view, &depth_view, &light_vis_state.render_pipeline, &light_vis_bind_group, &light_state.buffer );
      light_update_pass_record( &encoder, &light_compute_pipeline, &light_update_bind_group );

      gl::queue::submit( &device.queue(), encoder.finish() );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}

// Stub main for native targets
#[cfg(not(target_arch = "wasm32"))]
fn main()
{
  println!("This WebGPU deferred rendering example only works on WebAssembly targets.");
  println!("To run this example, compile for wasm32-unknown-unknown target:");
  println!("  cargo build --target wasm32-unknown-unknown");
}
