//! Just draw a large point in the middle of the screen.

use std::io::{ BufReader, Cursor };

use minwebgl as gl;
use gl::
{
  GL,
};

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shaders
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Load model
  let obj_buffer = gl::file::load( "suzanne.obj" ).await.expect( "Failed to load the model" );
  let obj_cursor = Cursor::new( obj_buffer );
  let mut obj_reader = BufReader::new( obj_cursor );

  // qqq : for Yevgen : introduce helper to show detailed diagnostic information, with argumenting verbosity controlling level of detauls and strcuture Report having all the diagnostic information in inside, Report might have 'lifetime if necessary

  // qqq : for Yevgen : introduce helper to load from bytes slice
  let suzanne = tobj::load_obj_buf
  (
    &mut obj_reader,
    &tobj::GPU_LOAD_OPTIONS,
    move | _p |
    {
      // qqq : for Yevgen : why error?
      Err( tobj::LoadError::OpenFileFailed )
    }
  );

  // qqq : for Yevhen : implement a example obj_viewer, which allow upload any 3d model and see very detailed and full diagnostics information

  let ( models, _materials ) = suzanne.expect( "Failed to load OBJ file" );
  gl::log::info!( "# of models : {}", models.len() );
  // gl::log::info!( "# of materials : {}", _materials.expect( "Failed to parse materials" ).len() );
  let model = &models[ 0 ];
  let mesh = &model.mesh;
  // gl::log::info!( "{:?}", &model );

  let pos_buffer =  gl::buffer::create( &gl )?;
  let normal_buffer = gl::buffer::create( &gl )?;
  let uv_buffer = gl::buffer::create( &gl )?;

  gl::buffer::upload( &gl, &pos_buffer, &mesh.positions, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &normal_buffer, &mesh.normals, GL::STATIC_DRAW );
  gl::buffer::upload( &gl, &uv_buffer, &mesh.texcoords, GL::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &pos_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 1, &normal_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 2, &uv_buffer )?;
  //gl.bind_vertex_array( None );

  let index_buffer = gl::buffer::create( &gl )?;
  gl::index::upload( &gl, &index_buffer, &mesh.indices, GL::STATIC_DRAW );

  let projective_view_location = gl.get_uniform_location( &program, "project_view_matrix" );

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  // Camera setup
  let eye = gl::F32x3::new( 0.0, 0.0, 10.0 );
  let up = gl::F32x3::Y;

  let scale = 0.1;
  let aspect = width / height;
  let projection_matrix = gl::math::mat3x3h::perspective_rh_gl( 70.0f32.to_radians(), aspect, 0.1, 1000.0 );
  let model_matrix = gl::F32x4x4::from_scale_rotation_translation
  (
    gl::F32x3::splat( scale ) * 8.0,
    gl::QuatF32::from_angle_y( 180.0f32.to_radians() ),
    gl::F32x3::ZERO
  );

  gl.enable( gl::DEPTH_TEST );

  // Define the update and draw logic
  let update_and_draw =
  {
    let indices_amount = mesh.indices.len();
    move | t : f64 |
    {
      let time = t as f32 / 1000.0;
      let rotation = gl::math::mat3x3::from_angle_y( time.to_radians() * 10.0 );
      let eye = rotation * eye;

      let view_matrix = gl::math::mat3x3h::look_at_rh( eye, gl::F32x3::ZERO, up );
      let projective_view_matrix = projection_matrix * view_matrix * model_matrix;

      gl::uniform::matrix_upload( &gl, projective_view_location.clone(), &projective_view_matrix.to_array(), true ).unwrap();
      // Draw points
      // Vertex and index buffers are already bound
      gl.draw_elements_with_i32( gl::TRIANGLES, indices_amount as i32, gl::UNSIGNED_INT, 0 );
      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
