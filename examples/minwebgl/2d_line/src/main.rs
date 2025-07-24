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
  let mut positions = [ [ 0.0; 2 ]; N  ];
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

  let fragment_shader_src = include_str!( "../shaders/main.frag" );

  let projection_matrix = gl::math::mat3x3h::orthographic_rh_gl( -width / 2.0, width / 2.0, -height / 2.0, height / 2.0, 0.0, 1.0 );
  let line_width = 50.0;

  let points = generate_sample_points_interleaved( width, height );

  let mut line = line_tools::d2::Line::default();
  line.join = line_tools::Join::Round( 50 );

  for p in points
  {
    line.points.push( p.into() );
  }

  let mesh = line.to_mesh( &gl, fragment_shader_src )?;
  mesh.upload_matrix( &gl, "u_projection_matrix", &projection_matrix.to_array() )?;
  mesh.upload( &gl, "u_width", &line_width )?;

  mesh.upload_to( &gl, "body", "u_color", &[ 1.0, 1.0, 1.0 ] )?;
  mesh.upload_to( &gl, "join", "u_color", &[ 1.0, 0.0, 0.0 ] )?;
  
  // Define the update and draw logic
  let update_and_draw =
  {
    
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;
      
      mesh.draw( &gl );

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
