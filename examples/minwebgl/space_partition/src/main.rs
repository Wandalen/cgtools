//! 2d line demo
use minwebgl as gl;
use std::
{
  cell::RefCell,
  rc::Rc,
};
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;

mod lil_gui;
mod impls;

/// Struct to encapsulate user changeable settings for use wiht lil_gui
#[ derive( Default, Serialize, Deserialize ) ]
struct Settings
{
  #[ serde( rename = "Search type" ) ]
  search : String,
  #[ serde( rename = "K Neighbours" ) ]
  k_neighbours : usize,
  #[ serde( rename = "Range radius" ) ]
  range_radius : f32
}

/// Generates points in 0.0..1.0 range
fn generate_points( num_points : usize ) -> Vec< impls::Point2D >
{
  let mut points = Vec::with_capacity( num_points );
  for i in 0..num_points
  {
    let point = gl::F32x2::new( fastrand::f32(), fastrand::f32() );
    points.push( impls::Point2D( point, i ) );
  }

  points
}

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let aspect = width / height;

  let point_frag = include_str!( "../shaders/point.frag" );
  let point_vert = include_str!( "../shaders/point.vert" );
  let circle_frag = include_str!( "../shaders/circle.frag" );
  let circle_vert = include_str!( "../shaders/circle.vert" );
  let background_frag = include_str!( "../shaders/background.frag" );
  let background_vert = include_str!( "../shaders/background.vert" );
  let line_frag = include_str!( "../shaders/line.frag" );

  let background_program = gl::ProgramFromSources::new( background_vert, background_frag ).compile_and_link( &gl )?;
  let point_program = gl::ProgramFromSources::new( point_vert, point_frag ).compile_and_link( &gl )?;
  let circle_program = gl::ProgramFromSources::new( circle_vert, circle_frag ).compile_and_link( &gl )?;

  let projection_matrix = gl::math::mat3x3h::orthographic_rh_gl( -aspect, aspect, -1.0, 1.0, 0.0, 1.0 );
  let world_matrix = gl::F32x3x3::from_scale_rotation_translation( [ 2.0, 2.0 ], 0.0, [ -1.0, -1.0 ] );
  let view_matrix = gl::math::mat3x3::identity();


  const NUM_POINTS : usize = 500;
  let mut points = generate_points( NUM_POINTS );
  for p in points.iter_mut()
  {
    p.0.0[ 0 ] = p.0.0[ 0 ] * 2.0 - 1.0;
    p.0.0[ 0 ] *= aspect;
    p.0.0[ 0 ] = ( p.0.0[ 0 ] + 1.0 ) / 2.0; 
  }
  let mut colors = vec![ gl::F32x3::splat( 0.0 ); NUM_POINTS ];

  let mut kd_tree = spart::kdtree::KdTree::new();
  kd_tree.insert_bulk( points.clone() ).expect( "Failed to insert bulk" );


  let positions_buffer = gl::buffer::create( &gl )?;
  let colors_buffer = gl::buffer::create( &gl )?;

  gl::buffer::upload( &gl, &positions_buffer, &points.iter().map( | p | p.0.to_array() ).flatten().collect::< Vec< f32 > >(), gl::STATIC_DRAW );
  gl::buffer::upload( &gl, &colors_buffer, &colors.iter().map( | p | p.to_array() ).flatten().collect::< Vec< f32 > >(), gl::DYNAMIC_DRAW );

  let points_vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &points_vao ) );
  gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &positions_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 3 ).divisor( 0 ).attribute_pointer( &gl, 1, &colors_buffer )?;

  gl.use_program( Some( &point_program ) );
  gl::uniform::matrix_upload( &gl, gl.get_uniform_location( &point_program, "projectionMatrix" ), &projection_matrix.to_array(), true )?;
  gl::uniform::matrix_upload( &gl, gl.get_uniform_location( &point_program, "worldMatrix" ), &world_matrix.to_array(), true )?;

  gl.use_program( Some( &circle_program ) );
  gl::uniform::matrix_upload( &gl, gl.get_uniform_location( &circle_program, "projectionMatrix" ), &projection_matrix.to_array(), true )?;
  gl::uniform::matrix_upload( &gl, gl.get_uniform_location( &circle_program, "worldMatrix" ), &world_matrix.to_array(), true )?;

  let mut lines = Vec::with_capacity( 5 );

  let mut input = browser_input::Input::new
  (
    Some( canvas.clone().dyn_into().unwrap() ),
    browser_input::SCREEN,
  );

  let settings = Settings
  {
    search : "KNN".into(),
    k_neighbours : 5,
    range_radius : 0.05
  };

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = lil_gui::new_gui();

  let settings = Rc::new( RefCell::new( settings ) );

  // Search type
  let prop = lil_gui::add_dropdown( &gui, &object, "Search type", &serde_wasm_bindgen::to_value( &[ "KNN", "Range" ] ).unwrap() );
  let callback = Closure::new
  (
    {
      let settings = settings.clone();
      move | value : String |
      {
        gl::info!( "{:?}", value );
        settings.borrow_mut().search = value;
      }
    }
  );
  lil_gui::on_change_string( &prop, &callback );
  callback.forget();

  // K Neighbours
  let prop = lil_gui::add_slider( &gui, &object, "K Neighbours", 0.0, 100.0, 1.0 );
  let callback = Closure::new
  (
    {
      let settings = settings.clone();
      move | value : f32 |
      {
        settings.borrow_mut().k_neighbours = value as usize;
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  // Range radius
  let prop = lil_gui::add_slider( &gui, &object, "Range radius", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let settings = settings.clone();
      move | value : f32 |
      {
        settings.borrow_mut().range_radius = value;
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  gl.enable( gl::BLEND );
  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );

  // Define the update and draw logic
  let update_and_draw =
  {
    let settings = settings.clone();
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      let width = canvas.width() as f32;
      let height = canvas.height() as f32;

      for i in 0..NUM_POINTS
      {
        colors[ i ] = gl::F32x3::splat( 0.0 );
      }

      input.update_state();
      let mouse_pos = input.pointer_position();
      let mut mouse_pos = gl::F32x2::new( mouse_pos.0[ 0 ] as f32, height - mouse_pos.0[ 1 ] as f32 ) / gl::F32x2::new( width, height );
      mouse_pos.0[ 0 ] = mouse_pos.0[ 0 ] * 2.0 - 1.0;
      mouse_pos.0[ 0 ] *= aspect;
      mouse_pos.0[ 0 ] = ( mouse_pos.0[ 0 ] + 1.0) / 2.0; 

      let neighbours = 
      match settings.borrow().search.as_str()
      {
        "KNN" => {  kd_tree.knn_search::< spart::geometry::EuclideanDistance >( &impls::Point2D( mouse_pos, 0 ), settings.borrow().k_neighbours ) },
        "Range" =>  kd_tree.range_search::< spart::geometry::EuclideanDistance >( &impls::Point2D( mouse_pos, 0 ), settings.borrow().range_radius as f64 ),
        _ => { panic!( "Search option does not exist" ) }
      };

      for i in 0..neighbours.len()
      {
        if settings.borrow().search.as_str() == "KNN"
        {
          if i >= lines.len()
          {
            let mut line = line_tools::d2::Line::default();

            line.set_cap( line_tools::Cap::Round( 16 ) );
            line.create_mesh( &gl, line_frag ).expect( "Failed to create a line" );

            line.get_mesh().upload( &gl, "u_width", &0.01 ).unwrap();
            line.get_mesh().upload_matrix( &gl, "u_projection_matrix", &projection_matrix.to_array() ).unwrap();
            line.get_mesh().upload_matrix( &gl, "u_world_matrix", &world_matrix.to_array() ).unwrap();
            line.get_mesh().upload_matrix( &gl, "u_view_matrix", &view_matrix.to_array() ).unwrap();

            lines.push( line );
          }

          lines[ i ].clear();
          lines[ i ].add_point( mouse_pos );
          lines[ i ].add_point( neighbours[ i ].0 );
        }

        colors[ neighbours[ i ].1 ] = gl::F32x3::new( 0.0, 1.0, 0.0 );
      }

      input.clear_events();

      gl::buffer::upload( &gl, &colors_buffer, &colors.iter().map( | p | p.to_array() ).flatten().collect::< Vec< f32 > >(), gl::DYNAMIC_DRAW );

      // Draw background
      gl.use_program( Some( &background_program ) );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      // Draw points
      gl.use_program( Some( &point_program ) );
      gl.bind_vertex_array( Some( &points_vao ) );
      gl.draw_arrays( gl::POINTS, 0, NUM_POINTS as i32 );

      match settings.borrow().search.as_str()
      {
        "KNN" => 
        {
          // Draw lines
          for i in 0..neighbours.len()
          {
            lines[ i ].draw( &gl ).unwrap();
          }
        },
        "Range" =>
        {
          // Draw circle
          gl.use_program( Some( &circle_program ) );
          gl::uniform::upload( &gl, gl.get_uniform_location( &circle_program, "position" ), &mouse_pos.to_array() ).unwrap();
          gl::uniform::upload( &gl, gl.get_uniform_location( &circle_program, "radius" ), &settings.borrow().range_radius ).unwrap();
          gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );
        },
        _ => { panic!( "Search option does not exist" ) }
      };

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
