mod framebuffer;

use minwebgl as gl;
use gl::GL;
use framebuffer::*;

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

  let file = gl::file::load( "Cat.obj" ).await.unwrap();
  let ( models, _ ) = gl::model::load_model_from_slice( &file, "", &tobj::GPU_LOAD_OPTIONS )
  .await
  .unwrap();

  let mut meshes = vec![];
  for model in models
  {
    let vao = gl::vao::create( &gl )?;
    gl.bind_vertex_array( Some( &vao ) );

    let position_buffer = gl::buffer::create( &gl )?;
    let normal_buffer = gl::buffer::create( &gl )?;
    let index_buffer = gl::buffer::create( &gl )?;

    gl::buffer::upload( &gl, &position_buffer, &model.mesh.positions, GL::STATIC_DRAW );
    gl::buffer::upload( &gl, &normal_buffer, &model.mesh.normals, GL::STATIC_DRAW );
    gl::index::upload( &gl, &index_buffer, &model.mesh.indices, GL::STATIC_DRAW );

    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .offset( 0 )
    .stride( 0 )
    .attribute_pointer( &gl, 0, &position_buffer )
    .unwrap();

    gl::BufferDescriptor::new::< [ f32; 3 ] >()
    .offset( 0 )
    .stride( 0 )
    .attribute_pointer( &gl, 1, &normal_buffer )
    .unwrap();

    meshes.push( ( vao, model.mesh.indices.len() as i32 ) );
  }

  let width = gl.drawing_buffer_width();
  let height = gl.drawing_buffer_height();
  let aspect_ratio = width as f32 / height as f32;

  let projection = glam::Mat4::perspective_rh( 45.0_f32.to_radians(), aspect_ratio, 0.1, 1000. );
  let model = glam::Mat4::from_translation( glam::vec3( 0., 0., -150. ) );

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
  gl.use_program( Some( &deferred_shader ) );
  gl::uniform::upload( &gl, positions_location, &0 ).unwrap();
  gl::uniform::upload( &gl, normals_location, &1 ).unwrap();

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

  framebuffer.bind_draw( &gl );
  gl.use_program( Some( &object_shader ) );

  for ( vao, count ) in meshes
  {
    gl.bind_vertex_array( Some( &vao ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, count, GL::UNSIGNED_INT, 0 );
  }

  gl.bind_framebuffer( GL::FRAMEBUFFER, None );

  gl.use_program( Some( &deferred_shader ) );
  gl.draw_arrays( GL::TRIANGLES, 0, 3 );

  Ok( () )
}
