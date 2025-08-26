//! 2d line demo
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::assign_op_pattern ) ]
#![ allow( clippy::semicolon_if_nothing_returned ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::needless_range_loop ) ]
#![ allow( clippy::uninlined_format_args ) ]

use minwebgl as gl;
use std::
{
  cell::RefCell,
  rc::Rc,
};
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;

use crate::events::update;

mod lil_gui;
mod events;
mod impls;

#[ derive( Default, Serialize, Deserialize ) ]
struct Settings
{
  join : String,
  cap : String,
  width : f32
}

fn generate_points( num_points : usize ) -> Vec< impls::Point2D >
{
  let mut points = Vec::with_capacity( num_points );
  for _ in 0..num_points
  {
    let point = gl::F32x2::new( fastrand::f32(), fastrand::f32() );
    points.push( impls::Point2D( point ) );
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

  let point_frag = include_str!( "../shaders/point.frag" );
  let point_vert = include_str!( "../shaders/point.vert" );
  let background_frag = include_str!( "../shaders/background.frag" );
  let background_vert = include_str!( "../shaders/background.vert" );
  let line_frag = include_str!( "../shaders/line.frag" );

  let background_program = gl::ProgramFromSources::new( background_vert, background_frag ).compile_and_link( &gl )?;
  let point_program = gl::ProgramFromSources::new( point_vert, point_frag ).compile_and_link( &gl )?;

  let projection_matrix = gl::math::mat3x3h::orthographic_rh_gl( -1.0, 1.0, -1.0, 1.0, 0.0, 1.0 );
  let world_matrix = gl::F32x3x3::from_scale_rotation_translation( [ 2.0, 2.0 ], 0.0, [ -1.0, -1.0 ] );

  gl::info!( "{:?}", world_matrix );

  const NUM_POINTS : usize = 500;
  let points = generate_points( NUM_POINTS );
  let colors = vec![ gl::F32x3::splat( 0.0 ); NUM_POINTS ];

  let mut kd_tree = spart::kd_tree::KdTree::new();
  kd_tree.insert_bulk( points.clone() ).expect( "Failed to insert bulk" );


  let positions_buffer = gl::buffer::create( &gl )?;
  let colors_buffer = gl::buffer::create( &gl )?;

  gl::buffer::upload( &gl, &positions_buffer, &points.iter().map( | p | p.0.to_array() ).flatten().collect::< Vec< f32 > >(), gl::STATIC_DRAW );
  gl::buffer::upload( &gl, &colors_buffer, &colors.iter().map( | p | p.to_array() ).flatten().collect::< Vec< f32 > >(), gl::STATIC_DRAW );

  let points_vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &points_vao ) );
  gl::BufferDescriptor::new::< [ f32; 2 ] >().offset( 0 ).stride( 2 ).divisor( 0 ).attribute_pointer( &gl, 0, &positions_buffer )?;
  gl::BufferDescriptor::new::< [ f32; 3 ] >().offset( 0 ).stride( 3 ).divisor( 0 ).attribute_pointer( &gl, 1, &colors_buffer )?;

  gl.use_program( Some( &point_program ) );

  gl::uniform::matrix_upload( &gl, gl.get_uniform_location( &point_program, "projectionMatrix" ), &projection_matrix.to_array(), true )?;
  gl::uniform::matrix_upload( &gl, gl.get_uniform_location( &point_program, "worldMatrix" ), &world_matrix.to_array(), true )?;


  let mut n_neighbours = 10;
  let mut lines = Vec::with_capacity( 5 );


  let mut input = browser_input::Input::new
  (
    Some( canvas.clone().dyn_into().unwrap() ),
    browser_input::SCREEN,
  );

  // let settings = Settings
  // {
  //   join : "miter".into(),
  //   cap : "butt".into(),
  //   width : line_width
  // };

  // let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  // let gui = lil_gui::new_gui();

  // // Joins
  // let prop = lil_gui::add_dropdown( &gui, &object, "join", &serde_wasm_bindgen::to_value( &[ "miter", "bevel", "round" ] ).unwrap() );
  // let callback = Closure::new
  // (
  //   {
  //     let line = line.clone();
  //     move | value : String |
  //     {
  //       gl::info!( "{:?}", value );
  //       let mut line = line.borrow_mut();
  //       match value.as_str()
  //       {
  //         "miter" => { line.set_join( line_tools::Join::Miter( 7, 7 ) ); },
  //         "bevel" => { line.set_join( line_tools::Join::Bevel( 7, 7 ) ); },
  //         "round" => { line.set_join( line_tools::Join::Round( 16, 8 ) ); },
  //         _ => {}
  //       }
  //     }
  //   }
  // );
  // lil_gui::on_change_string( &prop, &callback );
  // callback.forget();

  // // Caps
  // let prop = lil_gui::add_dropdown( &gui, &object, "cap", &serde_wasm_bindgen::to_value( &[ "butt", "square", "round" ] ).unwrap() );
  // let callback = Closure::new
  // (
  //   {
  //     let line = line.clone();
  //     move | value : String |
  //     {
  //       gl::info!( "{:?}", value );
  //       let mut line = line.borrow_mut();
  //       match value.as_str()
  //       {
  //         "butt" => { line.set_cap( line_tools::Cap::Butt ); },
  //         "square" => { line.set_cap( line_tools::Cap::Square ); },
  //         "round" => { line.set_cap( line_tools::Cap::Round( 16 ) ); },
  //         _ => {}
  //       }
  //     }
  //   }
  // );
  // lil_gui::on_change_string( &prop, &callback );
  // callback.forget();

  // let prop = lil_gui::add_slider( &gui, &object, "width", 0.0, 500.0, 0.1 );
  // let callback = Closure::new
  // (
  //   {
  //     let line = line.clone();
  //     let gl = gl.clone();
  //     move | value : f32 |
  //     {
  //       line.borrow().get_mesh().upload( &gl, "u_width", &value ).unwrap();
  //     }
  //   }
  // );
  // lil_gui::on_change( &prop, &callback );
  // callback.forget();

  gl.enable( gl::BLEND );
  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      let width = canvas.width() as f32;
      let height = canvas.height() as f32;


      input.update_state();
      let mouse_pos = input.pointer_position();
      let mouse_pos = gl::F32x2::new( mouse_pos.0[ 0 ] as f32, height - mouse_pos.0[ 1 ] as f32 ) / gl::F32x2::new( width, height );

      let neighbours = kd_tree.knn_search::< spart::geometry::EuclideanDistance >( &impls::Point2D( mouse_pos ), n_neighbours );

      for i in 0..neighbours.len()
      {
        if i >= lines.len()
        {
          let mut line = line_tools::d2::Line::default();

          line.set_cap( line_tools::Cap::Round( 16 ) );
          line.create_mesh( &gl, line_frag ).expect( "Failed to create a line" );

          line.get_mesh().upload( &gl, "u_width", &0.01 ).unwrap();
          line.get_mesh().upload_matrix( &gl, "u_projection_matrix", &projection_matrix.to_array() ).unwrap();
          line.get_mesh().upload_matrix( &gl, "u_world_matrix", &world_matrix.to_array() ).unwrap();

          lines.push( line );
        }

        lines[ i ].clear();
        lines[ i ].add_point( mouse_pos );
        lines[ i ].add_point( neighbours[ i ].0 );
      }

      for browser_input::Event { event_type, .. } in input.event_queue().iter()
      {
        // if let browser_input::EventType::MouseMovement( new_pos ) = *event_type
        // {
        //   let mut x = mouse_pos.0[ 0 ] as f32;
        //   let mut y = height - mouse_pos.0[ 1 ] as f32;

        //   x = x * 2.0 - width;
        //   y = y * 2.0 - height;

        //   mouse_pos

        //   x /= 2.0;
        //   y /= 2.0;

        // }
      }

      input.clear_events();



      // Draw background
      gl.use_program( Some( &background_program ) );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      // Draw points
      gl.use_program( Some( &point_program ) );

      gl.bind_vertex_array( Some( &points_vao ) );
      gl.draw_arrays( gl::POINTS, 0, NUM_POINTS as i32 );

      // Draw lines
      for i in 0..neighbours.len()
      {
        lines[ i ].draw( &gl ).unwrap();
      }

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
