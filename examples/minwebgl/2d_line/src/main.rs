use minwebgl as gl;
use gl::GL;
use std::
{
  cell::RefCell,
  rc::Rc,
};

fn generate_sample_points_interleaved( width : f32, height : f32 ) -> [ [ f32; 2 ]; 8 ]
{
  let stepx = width / 9.0;
  let stepy = height / 3.0;
  let mut points = [ [ 0.0; 2 ]; 8 ];
  let mut i = 0;
  for x in ( 1..9 ).step_by( 2 )
  {
    points[ i ] = [ ( x as f32 + 0.0 ) * stepx - width / 2.0, 1.0 * stepy - height / 2.0];
    points[ i + 1 ] = [ ( x as f32 + 1.0 ) * stepx - width / 2.0, 2.0 * stepy - height / 2.0];
    i += 2;
  }

  return points;
}

fn circle_geometry< const N : usize >() -> [ [ f32; 2 ]; N ]
{
  let mut positions = [ [ 0.0; 2 ]; N ];
  for wedge in 0..N
  {
    let theta = 2.0 * std::f32::consts::PI * wedge as f32 / N as f32;
    let ( s, c ) = theta.sin_cos();
    positions[ wedge as usize ] = [ 0.5 * c, 0.5 * s ]
  }

  positions
}

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/main.vert" );
  let fragment_shader_src = include_str!( "../shaders/main.frag" );
  let round_join_vertex_shader = include_str!( "../shaders/round_join.vert" );
  let main_program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  let round_join_program = gl::ProgramFromSources::new( round_join_vertex_shader, fragment_shader_src ).compile_and_link( &gl )?;
  

  const INSTANCED_GEOMETRY : [ [ f32; 2 ]; 6 ] = 
  [
    [ 0.0, -0.5 ],
    [ 1.0, -0.5 ],
    [ 1.0,  0.5 ],
    [ 0.0, -0.5 ],
    [ 1.0,  0.5 ],
    [ 0.0,  0.5 ]
  ];

  let projection_matrix = gl::math::mat3x3h::orthographic_rh_gl( -width / 2.0, width / 2.0, -height / 2.0, height / 2.0, 0.0, 1.0 );

  let line_width = 50.0;

  gl.use_program( Some( &main_program ) );
  gl::uniform::matrix_upload( &gl, gl.get_uniform_location( &main_program, "u_projection_matrix" ).clone() , &projection_matrix.to_array(), true )?;
  gl::uniform::upload( &gl, gl.get_uniform_location( &main_program, "u_width" ).clone(), &line_width )?;
  gl::uniform::upload( &gl, gl.get_uniform_location( &main_program, "u_color" ).clone(), &[ 1.0, 1.0, 1.0 ] )?;

  gl.use_program( Some( &round_join_program ) );
  gl::uniform::matrix_upload( &gl, gl.get_uniform_location( &round_join_program, "u_projection_matrix" ).clone() , &projection_matrix.to_array(), true )?;
  gl::uniform::upload( &gl, gl.get_uniform_location( &round_join_program, "u_width" ).clone(), &line_width )?;
  gl::uniform::upload( &gl, gl.get_uniform_location( &round_join_program, "u_color" ).clone(), &[ 1.0, 0.0, 0.0 ] )?;

  let points = generate_sample_points_interleaved( width, height );
  let round_join = circle_geometry::< 16 >();

  // Main body VAO
  let main_vao = gl.create_vertex_array();
  gl.bind_vertex_array( main_vao.as_ref() );

  let instanced_geometry_buffer = gl.create_buffer().unwrap();
  gl.bind_buffer( gl::ARRAY_BUFFER, Some( &instanced_geometry_buffer ) );
  gl::buffer::upload( &gl, &instanced_geometry_buffer, &INSTANCED_GEOMETRY, gl::STATIC_DRAW );

  let positions_buffer = gl.create_buffer().unwrap();
  gl.bind_buffer( gl::ARRAY_BUFFER, Some( &positions_buffer ) );
  gl::buffer::upload( &gl, &positions_buffer, &points, gl::STATIC_DRAW );

  gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &instanced_geometry_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &positions_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 2, &positions_buffer )?;


  // Round join vao
  let round_join_vao = gl.create_vertex_array();
  gl.bind_vertex_array( round_join_vao.as_ref() ); 

  let round_join_instanced_buffer = gl.create_buffer().unwrap();
  gl.bind_buffer( gl::ARRAY_BUFFER, Some( &round_join_instanced_buffer ) );
  gl::buffer::upload( &gl, &round_join_instanced_buffer, &round_join, gl::STATIC_DRAW );

  gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &round_join_instanced_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 2 ).stride( 2 ).divisor( 1 ).attribute_pointer( &gl, 1, &positions_buffer )?;

  
  // Define the update and draw logic
  let update_and_draw =
  {
    
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      gl.use_program( Some( &round_join_program ) );
      gl.bind_vertex_array( round_join_vao.as_ref() );
      gl.draw_arrays_instanced( gl::TRIANGLE_FAN, 0, round_join.len() as i32, points.len() as i32 - 2 );

      gl.use_program( Some( &main_program ) );
      gl.bind_vertex_array( main_vao.as_ref() );
      gl.draw_arrays_instanced( gl::TRIANGLES, 0, INSTANCED_GEOMETRY.len() as i32, points.len() as i32 - 1 );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );
  Ok( () )
}

fn main()
{
  run().unwrap()
}
