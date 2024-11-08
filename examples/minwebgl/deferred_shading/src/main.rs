mod framebuffer;

use minwebgl as gl;
use gl::AsBytes as _;
use gl::GL;
use framebuffer::*;
use web_sys::WebGlVertexArrayObject;

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async { gl::info!( "{:?}", run().await ); } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );
  gl.enable( GL::DEPTH_TEST );
  gl.enable( GL::CULL_FACE );
  _ = gl.get_extension( "EXT_color_buffer_float" ).unwrap().unwrap();

  let file = gl::file::load( "lowpoly_tree.obj" ).await.unwrap();
  let ( models, _ ) = gl::model::load_model_from_slice( &file, "", &tobj::GPU_LOAD_OPTIONS )
  .await
  .unwrap();

  let meshes = load_meshes( &models, &gl )?;

  let width = gl.drawing_buffer_width();
  let height = gl.drawing_buffer_height();
  let aspect_ratio = width as f32 / height as f32;

  let projection = glam::Mat4::perspective_rh( 45.0_f32.to_radians(), aspect_ratio, 0.1, 1000. );
  let model = glam::Mat4::from_translation( glam::vec3( 0., -10., -50. ) );

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let object_shader = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl ).unwrap();
  let mvp_location = gl.get_uniform_location( &object_shader, "mvp" );
  let model_location = gl.get_uniform_location( &object_shader, "model" );

  gl.use_program( Some( &object_shader ) );
  gl::uniform::matrix_upload( &gl, mvp_location, ( projection * model ).to_cols_array().as_slice(), true ).unwrap();
  gl::uniform::matrix_upload( &gl, model_location, model.to_cols_array().as_slice(), true ).unwrap();

  let vert = include_str!( "shaders/rasterize.vert" );
  let frag = include_str!( "shaders/deferred.frag" );
  let deferred_shader = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl ).unwrap();
  let positions_location = gl.get_uniform_location( &deferred_shader, "positions" );
  let normals_location = gl.get_uniform_location( &deferred_shader, "normals" );
  let lights_index = gl.get_uniform_block_index( &deferred_shader, "Lights" );

  gl.use_program( Some( &deferred_shader ) );
  gl::uniform::upload( &gl, positions_location, &0 ).unwrap();
  gl::uniform::upload( &gl, normals_location, &1 ).unwrap();

  const BINDING_POINT : u32 = 0;
  gl.uniform_block_binding( &deferred_shader, lights_index, BINDING_POINT );

  let lights = create_lights();
  let lights_ubo = gl::buffer::create( &gl ).unwrap();
  gl::ubo::upload( &gl, &lights_ubo, BINDING_POINT, lights.as_bytes(), GL::STATIC_DRAW );

  let positionbuffer = gl.create_texture().unwrap();
  gl.active_texture( GL::TEXTURE0 );
  gl.bind_texture( GL::TEXTURE_2D, Some( &positionbuffer ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, GL::RGBA16F, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

  let normalbuffer = gl.create_texture().unwrap();
  gl.active_texture( GL::TEXTURE1 );
  gl.bind_texture( GL::TEXTURE_2D, Some( &normalbuffer ) );
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, GL::RGBA16F, width, height );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

  let depthbuffer = gl.create_renderbuffer().unwrap();
  gl.bind_renderbuffer( GL::RENDERBUFFER, Some( &depthbuffer ) );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT16, width, height );

  let framebuffer = FramebufferBuilder::new()
  .attachment( ColorAttachment::N0, Attachment::Texture( positionbuffer ) )
  .attachment( ColorAttachment::N1, Attachment::Texture( normalbuffer ) )
  .depthbuffer( DepthAttachment::Depth, Attachment::Renderbuffer( depthbuffer ) )
  .build( &gl )?;

  gl.use_program( Some( &object_shader ) );
  framebuffer.bind_draw( &gl );
  draw_meshes( &meshes, &gl );

  let draw_loop = move | t |
  {
    gl.use_program( Some( &deferred_shader ) );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    true
  };

  gl::exec_loop::run( draw_loop );

  Ok( () )
}

fn load_meshes( models: &[ tobj::Model ], gl: &GL )
->
Result< Vec< ( WebGlVertexArrayObject, i32 ) >, minwebgl::WebglError >
{
  let mut meshes = vec![];
  for model in models
  {
    let vao = gl::vao::create( gl )?;
    gl.bind_vertex_array( Some( &vao ) );

    let position_buffer = gl::buffer::create( gl )?;
    let normal_buffer = gl::buffer::create( gl )?;
    let index_buffer = gl::buffer::create( gl )?;

    gl::buffer::upload( gl, &position_buffer, &model.mesh.positions, GL::STATIC_DRAW );
    gl::buffer::upload( gl, &normal_buffer, &model.mesh.normals, GL::STATIC_DRAW );
    gl::index::upload( gl, &index_buffer, &model.mesh.indices, GL::STATIC_DRAW );

    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .offset( 0 )
    .stride( 0 )
    .attribute_pointer( gl, 0, &position_buffer )
    .unwrap();

    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .offset( 0 )
    .stride( 0 )
    .attribute_pointer( gl, 1, &normal_buffer )
    .unwrap();

    meshes.push( ( vao, model.mesh.indices.len() as i32 ) );
  }
  Ok( meshes )
}

fn draw_meshes( meshes : &[ ( WebGlVertexArrayObject, i32 ) ], gl : &GL )
{
  for ( vao, count ) in meshes
  {
    gl.bind_vertex_array( Some( &vao ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, *count, GL::UNSIGNED_INT, 0 );
  }
}

// buffer
fn create_lights() -> Box< [ f32 ] >
{
  let mut ret = vec![];
  for i in 0..50
  {
    let position = glam::vec4( 0., 0., -i as f32, 0. );
    let color = glam::vec4( 1., 1., 1., 0. );

    ret.extend_from_slice( &position.to_array() );
    ret.extend_from_slice( &color.to_array() );
  }
  ret.into()
}
