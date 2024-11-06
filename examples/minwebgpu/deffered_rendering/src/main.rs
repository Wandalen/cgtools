//! Just draw a large point in the middle of the screen.

use minwebgpu as gl;

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
  // Sets the usage flag to `RENDER_ATTACHMENT`
  .render_attachment()
  // Sets the usage flag to `TEXTURE_BINDING`
  .texture_binding();

  let vector_tex_desc = color_tex_desc.clone().format( gl::GpuTextureFormat::Rgba16float );

  let position_tex = gl::texture::create( &device, &vector_tex_desc )?;
  let albedo_tex = gl::texture::create( &device, color_tex_desc )?;
  let normal_tex = gl::texture::create( &device, &vector_tex_desc )?;

  Ok( [ position_tex, albedo_tex, normal_tex ] )
}

fn create_vertex_descriptors() -> [ gl::VertexBufferLayout; 3 ]
{
  let pos_buffer_layout = gl::VertexBufferLayout::new()
  .vertex()
  .stride::< [ f32; 3 ] >()
  .attribute
  (
    gl::VertexAttribute::new()
    .location( 0 )
  );

  let normal_buffer_layout = gl::VertexBufferLayout::new()
  .vertex()
  .stride::< [ f32; 3 ] >()
  .attribute
  (
    gl::VertexAttribute::new()
    .location( 1 )
  );

  let uv_buffer_layout = gl::VertexBufferLayout::new()
  .vertex()
  .stride::< [ f32; 2 ] >()
  .attribute
  (
    gl::VertexAttribute::new()
    .location( 2 )
    .format( gl::GpuVertexFormat::Float32x2 )
  );

  [ pos_buffer_layout, normal_buffer_layout, uv_buffer_layout ]
}

async fn run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::retrieve_or_make()?;
  let adapter = gl::context::request_adapter().await;
  let device = gl::context::request_device( &adapter ).await;

  let width = canvas.width();
  let height = canvas.height();
  
  let gbuffer_shader = gl::ShaderModule::new( include_str!( "../shaders/gbuffer.wgsl" ) ).create( &device );
  let [ pos_t, albedo_t, normal_t ] = create_textures( &device, [ width, height, 1 ] )?;
  let [ pos_desc, albedo_desc, normal_desc ] = create_vertex_descriptors();



  let model = gl::file::load( "bunny.obj" ).await.expect( "Failed to fetch the model" );
  let ( models, _ ) = gl::model::obj::load_model_from_slice( &model, "", &tobj::GPU_LOAD_OPTIONS ).await.unwrap();
  let model = models.first().unwrap();
  let mesh = &model.mesh;

  let bind_group_layout = gl::layout::bind_group::create
  ( 
    &device, 
    // Sets the visibility `FRAGMENT` to all entries
    // And auto computes binding value for each entry
    gl::layout::bind_group::desc()
    .fragment()
    .auto_bindings()
    .entry_from_ty( gl::binding_type::texture() )
    .entry_from_ty( gl::binding_type::texture() )
    .entry_from_ty( gl::binding_type::texture() )
  )?;

  let pipeline_layout = gl::PipelineLayout::new().bind_group( bind_group_layout ).create( &device );
  let render_pipeline = gl::render_pipeline::create
  ( 
    &device, 
    gl::render_pipeline::desc( gl::VertexState::new( &gbuffer_shader ))
    .layout( &pipeline_layout )
    .fragment
    ( 
      gl::FragmentState::new( &gbuffer_shader ) 
      .target( gl::ColorTargetState::new() )
      .target( gl::ColorTargetState::new().format( gl::GpuTextureFormat::Rgba16float ) )
      .target( gl::ColorTargetState::new().format( gl::GpuTextureFormat::Rgba16float ) )
    )
    .primitive( gl::PrimitiveState::new().cull_back() )
    .depth_stencil( gl::DepthStencilState::new() )
  );

  Ok(())
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
