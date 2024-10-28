//! Just draw a large point in the middle of the screen.

use std::io::{ BufReader, Cursor };

use minwebgl as gl;
use gl::
{
  GL,
};

use cgmath::{ EuclideanSpace, Rotation3 };

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
  let eye = cgmath::Vector3::new( 0.0, 0.0, 10.0 );
  let up = cgmath::Vector3::< f32 >::unit_y();

  let scale = 0.1;
  let aspect = width / height;
  let perspective = cgmath::PerspectiveFov
  {
    fovy : cgmath::Deg( 70.0 ).into(),
    aspect,
    near : 0.1,
    far : 1000.0
  };
  // let orhogonal = cgmath::Ortho
  // {
  //   left : -1.0 * aspect,
  //   right : 1.0 * aspect,
  //   bottom : -1.0,
  //   top : 1.0,
  //   near : 0.1,
  //   far : 1000.0,
  // };

  let model_trans = cgmath::Decomposed
  {
    scale,
    rot : cgmath::Basis3::from_angle_y::< cgmath::Rad< f32 > >( cgmath::Deg( 180.0 ).into() ),
    disp : cgmath::Vector3::new( 0.0, 0.0, 0.0 ),
  };

  let model_matrix = cgmath::Matrix4::from( model_trans );
  let _projection_matrix = cgmath::Matrix4::from( perspective );
  // let projection_matrix = cgmath::Matrix4::from( orhogonal );

  gl.enable( gl::DEPTH_TEST );

  // Define the update and draw logic
  let update_and_draw =
  {
    let indices_amount = mesh.indices.len();
    move | t : f64 |
    {
      let time = t as f32 / 1000.0;
      let rotation = cgmath::Matrix3::from_angle_y( cgmath::Rad( time ) );
      let eye = rotation * eye;

      let _view_matrix = cgmath::Matrix4::look_at_rh( cgmath::Point3::from_vec( eye ), cgmath::Point3::origin(), up );

      // let projective_view_matrix = projection_matrix * view_matrix * model_matrix;
      let projective_view_matrix = model_matrix;
      let projective_view_matrix = &< cgmath::Matrix4< f32 > as AsRef< [ f32; 16 ] > >::as_ref( &projective_view_matrix )[ .. ];

      gl::uniform::matrix_upload( &gl, projective_view_location.clone(), projective_view_matrix.as_ref(), true ).unwrap();
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
