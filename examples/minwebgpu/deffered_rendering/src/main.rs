//! Just draw a large point in the middle of the screen.
//!
//! This example only works on WebAssembly (wasm32) targets where WebGPU APIs are available.

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

fn create_textures
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

  let position_tex = gl::texture::create( &device, &vector_tex_desc )?;
  let albedo_tex = gl::texture::create( &device, &color_tex_desc )?;
  let normal_tex = gl::texture::create( &device, &vector_tex_desc )?;

  Ok( [ position_tex, albedo_tex, normal_tex ] )
}

#[cfg(target_arch = "wasm32")]
async fn run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::retrieve_or_make()?;
  //let canvas = gl::canvas::make()?;
  let context = gl::context::from_canvas( &canvas )?;
  let adapter = gl::context::request_adapter().await;
  let device = gl::context::request_device( &adapter ).await;
  let queue = device.queue();

  let presentation_format = gl::context::preferred_format();
  gl::context::configure( &device, &context, presentation_format )?;

  let width = canvas.width();
  let height = canvas.height();

  let light_update_shader = gl::ShaderModule::new( include_str!( "../shaders/light_update.wgsl" ) ).create( &device );
  let big_plane_shader = gl::ShaderModule::new( include_str!( "../shaders/big_plane.wgsl" ) ).create( &device );
  let gbuffer_shader = gl::ShaderModule::new( include_str!( "../shaders/gbuffer.wgsl" ) ).create( &device );
  let render_shader = gl::ShaderModule::new( include_str!( "../shaders/render.wgsl" ) ).create( &device );

  let [ pos_vertex_layout, normal_vertex_layout, uv_vertex_layout ] = ModelState::vertex_layout();
  let model_instance_layout = ModelState::instance_layout();
  let [ pos_tex, albedo_tex, normal_tex ] = create_textures( &device, [ width, height, 1 ] )?;
  let depth_texture = gl::texture::create
  (
    &device,
    &gl::texture::desc()
    .size( [ width, height, 1 ] )
    .render_attachment()
    .texture_binding()
    .format( gl::GpuTextureFormat::Depth24plus )
    .into()
  )?;

  let depth_view = depth_texture.create_view().unwrap();
  let ( pos_view, albedo_view, normal_view ) =
  (
    pos_tex.create_view().unwrap(),
    albedo_tex.create_view().unwrap(),
    normal_tex.create_view().unwrap()
  );

  // Create needed state
  let model_state = ModelState::new( &device ).await?;
  let mut uniform_state = UniformState::new( &device )?;
  let light_state = LightState::new( &device )?;
  let light_vis_state = LightVisualizationState::new( &device, presentation_format )?;

  // First entry - uniform paramters like view_matrix, time
  // Second entry - array of lights
  let uniform_bind_group_layout = gl::BindGroupLayoutDescriptor::new()
  .fragment()
  .auto_bindings()
  .entry
  (
    gl::BindGroupLayoutEntry::new()
    .vertex()
    .ty( gl::binding_type::buffer_type() )
  )
  .entry_from_ty( gl::binding_type::buffer_type().storage_readonly() )
  .create( &device )?;

  let uniform_bind_group = gl::BindGroupDescriptor::new( &uniform_bind_group_layout )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( &uniform_state.buffer ) )
  .entry_from_resource( &gl::BufferBinding::new( &light_state.buffer ))
  .create( &device );
  ///////////////////

  // Setup gbuffer related state
  let gbuffer_bind_group_layout = gl::layout::bind_group::create
  (
    &device,
    // Sets the visibility `FRAGMENT` to all entries
    // And auto computes binding value for each entry
    &gl::layout::bind_group::desc()
    .fragment()
    .auto_bindings()
    .entry_from_ty( gl::binding_type::texture_type().sample_unfilterable_float() )
    .entry_from_ty( gl::binding_type::texture_type().sample_unfilterable_float() )
    .entry_from_ty( gl::binding_type::texture_type().sample_unfilterable_float() )
    .entry_from_ty( gl::binding_type::texture_type().sample_depth() )
    .to_web()
  )?;

  // Create pipeline layout for the gbuffer render pipeline
  let gbuffer_pipeline_layout = gl::layout::pipeline::desc()
  .bind_group( &uniform_bind_group_layout )
  .create( &device );

  let fragment_state = gl::FragmentState::new( &gbuffer_shader )
  .target( gl::ColorTargetState::new() )
  .target( gl::ColorTargetState::new().format( gl::GpuTextureFormat::Rgba16float ) )
  .target( gl::ColorTargetState::new().format( gl::GpuTextureFormat::Rgba16float ) )
  .to_web();

  // Pipeline that will render to the gbuffer textures.
  let gbuffer_render_pipeline = gl::render_pipeline::create
  (
    &device,
    &gl::render_pipeline::desc
    (
      gl::VertexState::new( &gbuffer_shader )
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
    &device,
    &gl::render_pipeline::desc( gl::VertexState::new( &big_plane_shader ) )
    .layout( &gbuffer_pipeline_layout )
    .fragment( fragment_state.clone() )
    .primitive( gl::PrimitiveState::new() )
    .depth_stencil( gl::DepthStencilState::new() )
    .to_web()
  )?;

  let gbuffer_bind_group = gl::bind_group::create
  (
    &device,
    &gl::bind_group::desc( &gbuffer_bind_group_layout )
    .auto_bindings()
    .entry_from_resource( &albedo_view )
    .entry_from_resource( &pos_view )
    .entry_from_resource( &normal_view )
    .entry_from_resource( &depth_view )
    .to_web()
  );
  ////////////////

  // The main render pipeline. It will do the lighting calculations based on
  // gbuffer texture we filled in gbuffer pipeline
  let render_pipeline_layout = gl::layout::pipeline::desc()
  .bind_group( &uniform_bind_group_layout )
  .bind_group( &gbuffer_bind_group_layout )
  .create( &device );

  let render_pipeline = gl::render_pipeline::create
  (
    &device,
    &gl::render_pipeline::desc( gl::VertexState::new( &render_shader ) )
    .layout( &render_pipeline_layout )
    .fragment
    (
      gl::FragmentState::new( &render_shader )
      .target( gl::ColorTargetState::new().format( presentation_format ) )
    )
    .primitive( gl::PrimitiveState::new().triangle_strip() )
    .to_web()
  )?;

  // We create a compute pipeline to update lights
  // Sicne there is only one `compute` function in the shader,
  // the entry point will default to that function
  let light_compute_pipeline = gl::compute_pipeline::desc
  (
    gl::ProgrammableStage::new( &light_update_shader )
  )
  .create( &device );

  // We create bindgroup from `auto` layout of our pipeline
  let light_update_bind_group = gl::bind_group::desc
  (
    &light_compute_pipeline.get_bind_group_layout( 0 )
  )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( &uniform_state.buffer ) )
  .entry_from_resource( &gl::BufferBinding::new( &light_state.buffer ) )
  .create( &device );


  // Light visualization
  let light_vis_bind_group = gl::bind_group::desc
  (
    &light_vis_state.render_pipeline.get_bind_group_layout( 0 )
  )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( &uniform_state.buffer ) )
  .create( &device );

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
        elapsed_time : elapsed_time
      };

      uniform_state.update( &queue ).unwrap();

      let encoder = device.create_command_encoder();
      // Gbuffer pass
      {
        let render_pass = encoder.begin_render_pass
        (
          &gl::RenderPassDescriptor::new()
          .color_attachment( gl::ColorAttachment::new( &albedo_view ))
          .color_attachment( gl::ColorAttachment::new( &pos_view ))
          .color_attachment( gl::ColorAttachment::new( &normal_view ))
          .depth_stencil_attachment( gl::DepthStencilAttachment::new( &depth_view ) )
          .into()
        ).unwrap();

        // Draw model
        render_pass.set_pipeline( &gbuffer_render_pipeline );
        render_pass.set_bind_group( 0, Some( &uniform_bind_group ) );
        render_pass.set_vertex_buffer( 0, Some( &model_state.pos_buffer ) );
        render_pass.set_vertex_buffer( 1, Some( &model_state.normal_buffer ) );
        render_pass.set_vertex_buffer( 2, Some( &model_state.uv_buffer ) );
        render_pass.set_vertex_buffer( 3, Some( &model_state.instance_buffer ) );
        render_pass.set_index_buffer( &model_state.index_buffer, gl::GpuIndexFormat::Uint32 );
        render_pass.draw_indexed_with_instance_count( model_state.index_length, NUM_MODELS as u32 );

        // Draw big plane
        render_pass.set_pipeline( &big_plane_render_pipeline );
        render_pass.draw( 6 );
        render_pass.end();
      }

      // Main render pass
      {
        let render_pass = encoder.begin_render_pass
        (
          &gl::RenderPassDescriptor::new()
          .color_attachment
          (
            gl::ColorAttachment::new( &canvas_view )
          )
          .into()
        ).unwrap();

        render_pass.set_pipeline( &render_pipeline );
        render_pass.set_bind_group( 0, Some( &uniform_bind_group ) );
        render_pass.set_bind_group( 1, Some( &gbuffer_bind_group ) );
        render_pass.draw( 4 );
        render_pass.end();
      }

      // Visualize light
      {
        let render_pass = encoder.begin_render_pass
        (
          &gl::RenderPassDescriptor::new()
          .color_attachment
          (
            gl::ColorAttachment::new( &canvas_view )
            .load_op( gl::GpuLoadOp::Load )
          )
          .depth_stencil_attachment
          (
            gl::DepthStencilAttachment::new( &depth_view )
            .depth_load_op( gl::GpuLoadOp::Load )
          )
          .into()
        ).unwrap();

        render_pass.set_pipeline( &light_vis_state.render_pipeline );
        render_pass.set_bind_group( 0, Some( &light_vis_bind_group ) );
        render_pass.set_vertex_buffer( 0, Some( &light_state.buffer ) );
        render_pass.draw_with_instance_count( 14, NUM_LIGHTS as u32 );
        render_pass.end();
      }

      // Update light positions
      {
        let compute_pass = encoder.begin_compute_pass();

        compute_pass.set_pipeline( &light_compute_pipeline );
        compute_pass.set_bind_group( 0, Some( &light_update_bind_group ) );
        compute_pass.dispatch_workgroups( NUM_LIGHTS.div_ceil( 64 ) as u32 );
        compute_pass.end();
      }

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
  gl::spawn_local( async move { run().await.unwrap() } );
}

// Stub main for native targets
#[cfg(not(target_arch = "wasm32"))]
fn main()
{
  println!("This WebGPU deferred rendering example only works on WebAssembly targets.");
  println!("To run this example, compile for wasm32-unknown-unknown target:");
  println!("  cargo build --target wasm32-unknown-unknown");
}
