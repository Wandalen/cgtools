use mingl::CameraOrbitControls;
use minwebgl::{self as gl, IntoArray};
use gl::GL;
use std::
{
  cell::RefCell,
  rc::Rc,
};

mod camera_controls;

fn generate_sample_points( num : usize ) -> Vec< gl::F32x3 >
{
  let mut points = Vec::with_capacity( num );
  let mut pos = gl::F32x3::default();
  let scale = 0.1;
  for i in 0..num
  {
    let k = i % 3;
    if k == 0 
    {
      pos += gl::F32x3::X * scale;
    }
    else if k == 1 
    {
      pos += gl::F32x3::Z * scale;    
    } 
    else 
    {
      pos += gl::F32x3::Y * scale;    
    }

    points.push( pos );
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

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::default();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.0001f32;
  let far = 100.0f32;

  let camera = CameraOrbitControls
  {
    eye : eye,
    up : up,
    center : center,
    window_size : [ width, height ].into(),
    fov,
    ..Default::default()
  };
  let camera = Rc::new( RefCell::new( camera ) );
  camera_controls::setup_controls( &canvas, &camera );

  let projection_matrix = gl::math::mat3x3h::perspective_rh_gl( fov, aspect_ratio, near, far );

  let line_width = 0.02;

  let points = generate_sample_points( 2 );

  let mut line = line_tools::d3::Line::default();
  line.points = points;

  line.create_mesh( &gl, 16, fragment_shader_src )?;
  let mesh = line.get_mesh();
  mesh.upload( &gl, "u_width", &line_width )?;
  mesh.upload( &gl, "u_color", &[ 1.0, 1.0, 1.0 ] )?;
  mesh.upload( &gl, "u_resolution", &[ width as f32, height as f32 ] )?;
  mesh.upload_matrix( &gl, "u_projection_matrix", &projection_matrix.to_array() )?;
  mesh.upload_matrix( &gl, "u_world_matrix", &gl::math::mat4x4::identity().to_array() ).unwrap();
  
  // Define the update and draw logic
  let update_and_draw =
  {
    
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      let scale = [ ( ( _time * 2.0 ).sin().abs() + 0.1 ) * 2.0, 1.0, 1.0 ];
      let rotation = gl::QuatF32::from_angle_x( ( _time * 2.0 ).sin() );
      let translation = gl::F32x3::default();
      let world_matrix = gl::F32x4x4::from_scale_rotation_translation( scale, rotation, translation.as_array() );

      let mesh = line.get_mesh();
      //mesh.upload_matrix( &gl, "u_world_matrix", &world_matrix.to_array() ).unwrap();
      mesh.upload_matrix( &gl, "u_view_matrix", &camera.borrow().view().to_array() ).unwrap();
      
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
