use minwebgl::{self as gl, IntoArray};
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
  line.join = line_tools::Join::Miter;
  line.cap = line_tools::Cap::Round( 50 );

  for p in points
  {
    line.points.push( p.into() );
  }

  line.create_mesh( &gl, fragment_shader_src )?;
  let mesh = line.get_mesh();
  mesh.upload_matrix( &gl, "u_projection_matrix", &projection_matrix.to_array() )?;
  mesh.upload( &gl, "u_width", &line_width )?;

  mesh.upload( &gl, "u_color", &[ 1.0, 1.0, 1.0 ] )?;

  // mesh.upload_to( &gl, "body", "u_color", &[ 1.0, 1.0, 1.0 ] )?;
  mesh.upload_to( &gl, "join", "u_color", &[ 1.0, 0.0, 0.0 ] )?;
  mesh.upload_to( &gl, "cap", "u_color", &[ 0.0, 1.0, 0.0 ] )?;
  
  // Define the update and draw logic
  let update_and_draw =
  {
    
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      let scale = [ ( ( _time * 2.0 ).sin().abs() + 0.1 ) * 2.0, 1.0 ];
      let rotation = ( _time * 2.0 ).sin();
      let translation = gl::F32x2::default();
      let world_matrix = gl::F32x3x3::from_scale_rotation_translation( scale, rotation, translation.as_array() );
      line.get_mesh().upload_matrix( &gl, "u_world_matrix", &world_matrix.to_array() ).unwrap();
      
      line.draw( &gl ).unwrap();

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
