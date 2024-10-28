//! Just draw a large point in the middle of the screen.

use minwebgl as gl;

#[global_allocator]
static ALLOC : wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static VERTICES : [ f32; 24 ] =
[
    // Front face
   -1.0, -1.0,  1.0, // Left bottom (0)
    1.0, -1.0,  1.0, // Right bottom (1)
    1.0,  1.0,  1.0, // Right top (2)
   -1.0,  1.0,  1.0, // Left top (3)
    // Back face
   -1.0, -1.0, -1.0, // Left bottom (4)
    1.0, -1.0, -1.0, // Right bottom (5)
    1.0,  1.0, -1.0, // Right top (6)
   -1.0,  1.0, -1.0, // Left top (7)
];

static INDICES : [ u16; 24 ] =
[
    // Front face
    0, 1,  1, 2,  2, 3,  3, 0,
    // Back face
    4, 7,  7, 6,  6, 5,  5, 4,
    // Connecting lines
    0, 4,  1, 5,  2, 6,  3, 7,
];

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let vertices_buffer =  gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &vertices_buffer, &VERTICES, gl::GL::STATIC_DRAW );
  gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &vertices_buffer )?;

  let indeces_buffer = gl::buffer::create( &gl )?;
  gl::index::upload( &gl, &indeces_buffer, &INDICES, gl::GL::STATIC_DRAW );

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;
  let aspect_ratio = width / height;
  let fov_y : f32 = 45.0;
  let near_z = 0.1;
  let far_z = 100.0;
  let projection_matrix = glam::Mat4::perspective_rh_gl
  (
    fov_y.to_radians(),
    aspect_ratio,
    near_z,
    far_z
  );

  let projection_matrix_location = gl.get_uniform_location( &program, "projection_matrix" );
  let angle_location = gl.get_uniform_location( &program, "angle" );
  gl::uniform::matrix_upload( &gl, projection_matrix_location, &projection_matrix.to_cols_array()[ .. ], true ).unwrap();

  gl.enable( gl::DEPTH_TEST );

  let update_and_draw =
  {
    let mut angle : f64 = 0.0;
    let vertices_amount = ( VERTICES.len() / 3 ) as i32;
    let indices_len = INDICES.len() as i32;

    move | mut t : f64 |
    {
      t *= 0.001;
      angle = t;

      gl::uniform::upload( &gl, angle_location.clone(), &( angle as f32 ) ).unwrap();

      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
      gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
      gl.draw_arrays( gl::POINTS, 0, vertices_amount );
      gl.draw_elements_with_i32( gl::LINES, indices_len, gl::UNSIGNED_SHORT, 0 );

      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok(())
}

fn main()
{
  run().unwrap()
}
