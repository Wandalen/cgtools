mod framebuffer;

use minwebgl as gl;
use gl::GL;

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async { run().await.unwrap(); } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );
  let file = gl::file::load( "lowpoly_tree.obj" ).await.unwrap();
  let ( models, _ ) = gl::obj::load_model_from_slice( &file, "", &tobj::GPU_LOAD_OPTIONS ).await.unwrap();

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

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;
  let aspect_ratio = width / height;

  let projection = glam::Mat4::perspective_rh( 45.0_f32.to_radians(), aspect_ratio, 0.1, 1000. );
  let model = glam::Mat4::from_translation( glam::vec3( 0., 0., -100. ) );

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let shader = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl ).unwrap();
  let mvp_location = gl.get_uniform_location( &shader, "mvp" );
  let model_location = gl.get_uniform_location( &shader, "model" );
  gl.use_program( Some( &shader ) );
  gl::uniform::matrix_upload( &gl, mvp_location, ( projection * model ).to_cols_array().as_slice(), true ).unwrap();
  gl::uniform::matrix_upload( &gl, model_location, model.to_cols_array().as_slice(), true ).unwrap();

  for ( vao, count ) in meshes
  {
    gl.bind_vertex_array( Some( &vao ) );
    gl.draw_elements_with_i32( GL::TRIANGLES, count, GL::UNSIGNED_INT, 0 );
  }

  Ok( () )
}
