//! Just draw a large point in the middle of the screen.

use minwebgpu::
{
  self as gl, 
  AsWeb
};
use uniform::{UniformRaw, UniformState};

mod uniform;

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

fn create_vertex_descriptors() -> [ gl::web_sys::GpuVertexBufferLayout; 3 ]
{
  // Step mode defaults to `Vertex`
  // Vertex Attribute offset defaults to 0.0
  // Vertex Attribute format defaults to Float32x3
  // If stride is not specified, it is computed from the attributes
  let pos_buffer_layout = gl::VertexBufferLayout::new()
  .attribute
  (
    gl::VertexAttribute::new()
    .location( 0 )
  );

  let normal_buffer_layout = gl::VertexBufferLayout::new()
  .attribute
  (
    gl::VertexAttribute::new()
    .location( 1 )
  );

  let uv_buffer_layout = gl::VertexBufferLayout::new()
  .attribute
  (
    gl::VertexAttribute::new()
    .location( 2 )
    .format( gl::GpuVertexFormat::Float32x2 )
  );

  [ pos_buffer_layout.into(), normal_buffer_layout.into(), uv_buffer_layout.into() ]
}

async fn run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( Default::default() );
  //let canvas = gl::canvas::retrieve_or_make()?;
  let canvas = gl::canvas::make()?;
  let context = gl::context::from_canvas( &canvas )?;
  let adapter = gl::context::request_adapter().await;
  let device = gl::context::request_device( &adapter ).await;
  let queue = device.queue();

  let presentation_format = gl::context::preferred_format();
  gl::context::configure( &device, &context, presentation_format )?;

  let width = canvas.width();
  let height = canvas.height();
  
  let gbuffer_shader = gl::ShaderModule::new( include_str!( "../shaders/gbuffer.wgsl" ) ).create( &device );
  let render_shader = gl::ShaderModule::new( include_str!( "../shaders/render.wgsl" ) ).create( &device );

  let [ pos_vertex_desc, normal_vertex_desc, uv_vertex_desc ] = create_vertex_descriptors();
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

  // Load models, create buffer and initialize buffer with the data
  let model = gl::file::load( "bunny.obj" ).await.expect( "Failed to fetch the model" );
  let ( models, _ ) = gl::model::obj::load_model_from_slice( &model, "", &tobj::GPU_LOAD_OPTIONS ).await.unwrap();
  let model = models.first().unwrap();
  let mesh = &model.mesh;

  let pos_buffer = gl::BufferInitDescriptor::new( &mesh.positions, gl::BufferUsage::VERTEX ).create( &device )?;
  let normal_buffer = gl::BufferInitDescriptor::new( &mesh.normals, gl::BufferUsage::VERTEX ).create( &device )?;
  let uv_buffer = gl::BufferInitDescriptor::new( &mesh.texcoords, gl::BufferUsage::VERTEX ).create( &device )?;
  let index_buffer = gl::BufferInitDescriptor::new( &mesh.indices, gl::BufferUsage::INDEX ).create( &device )?;

  let uniform_state = UniformState::new( &device )?;

  let gbuffer_bind_group_layout = gl::layout::bind_group::create
  ( 
    &device, 
    // Sets the visibility `FRAGMENT` to all entries
    // And auto computes binding value for each entry
    &gl::layout::bind_group::desc()
    .fragment()
    .auto_bindings()
    .entry_from_ty( gl::binding_type::texture().sample_unfilterable_float() )
    .entry_from_ty( gl::binding_type::texture().sample_unfilterable_float() )
    .entry_from_ty( gl::binding_type::texture().sample_unfilterable_float() )
    .entry_from_ty( gl::binding_type::texture().sample_depth() )
    .to_web()
  )?;

  // Create pipeline layout for the gbuffer render pipeline
  let gbuffer_pipeline_layout = gl::layout::pipeline::desc()
  .bind_group( &uniform_state.bind_group_layout )
  .create( &device );

  let gbuffer_render_pipeline = gl::render_pipeline::create
  ( 
    &device, 
    &gl::render_pipeline::desc
    ( 
      gl::VertexState::new( &gbuffer_shader )
      .buffer( &pos_vertex_desc )
      .buffer( &normal_vertex_desc )
      .buffer( &uv_vertex_desc )
    )
    .layout( &gbuffer_pipeline_layout )
    .fragment
    ( 
      gl::FragmentState::new( &gbuffer_shader ) 
      .target( gl::ColorTargetState::new() )
      .target( gl::ColorTargetState::new().format( gl::GpuTextureFormat::Rgba16float ) )
      .target( gl::ColorTargetState::new().format( gl::GpuTextureFormat::Rgba16float ) )
    )
    .primitive( gl::PrimitiveState::new().cull_none() )
    .depth_stencil( gl::DepthStencilState::new() )
    .to_web()
  )?;

  // Create pipeline layout for the main render pipeline
  let render_pipeline_layout = gl::layout::pipeline::desc()
  .bind_group( &uniform_state.bind_group_layout )
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

  let eye = gl::math::F32x3::from( [ 20.0, 30.0, 0.0 ] );
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
    let index_len = mesh.indices.len() as u32;
    move | t : f64 |
    {      
      let canvas_texture = gl::context::current_texture( &context ).unwrap();
      let canvas_view = gl::texture::view( &canvas_texture ).unwrap();
      let rot = gl::math::mat3x3::from_angle_y( t as f32 / 1000.0 );
      let eye = rot * eye;

      let view_matrix = gl::math::mat3x3h::loot_at_rh( eye, center, up );
      let uniform_raw = UniformRaw
      {
        view_matrix : view_matrix.to_array(),
        projection_matrix : projection_matrix.to_array(),
        camera_pos : eye.to_array(),
        ..Default::default()
      };

      uniform_state.update( &queue, uniform_raw ).unwrap();

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

        render_pass.set_pipeline( &gbuffer_render_pipeline );
        render_pass.set_bind_group( 0, Some( &uniform_state.bind_group ) );
        render_pass.set_vertex_buffer( 0, Some( &pos_buffer ) );
        render_pass.set_vertex_buffer( 1, Some( &normal_buffer ) );
        render_pass.set_vertex_buffer( 2, Some( &uv_buffer ) );
        render_pass.set_index_buffer( &index_buffer, gl::GpuIndexFormat::Uint32 );
        render_pass.draw_indexed( index_len );
        render_pass.end();
      }

      // Final render pass
      {
        let render_pass = encoder.begin_render_pass
        ( 
          &gl::RenderPassDescriptor::new()
          .color_attachment( gl::ColorAttachment::new( &canvas_view )) 
          .into()
        ).unwrap();

        render_pass.set_pipeline( &render_pipeline );
        render_pass.set_bind_group( 0, Some( &uniform_state.bind_group ) );
        render_pass.set_bind_group( 1, Some( &gbuffer_bind_group ) );
        render_pass.draw( 4 );
        render_pass.end();
      }

      gl::queue::submit( &device.queue(), encoder.finish() );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok(())
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
