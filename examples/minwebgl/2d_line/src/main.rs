//! 2d line demo

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

#[ derive( Default, Serialize, Deserialize ) ]
struct Settings
{
  join : String,
  cap : String,
  width : f32
}

fn join_dropdown_setup( gui : &JsValue, object : &JsValue, line : Rc< RefCell< line_tools::d2::Line > > )
{
  let prop = lil_gui::dropdown_add( gui, object, "join", &serde_wasm_bindgen::to_value( &[ "miter", "bevel", "round" ] ).unwrap() );
  let callback = Closure::new
  (
    move | value : String |
    {
      gl::info!( "{value:?}" );
      let mut line = line.borrow_mut();
      match value.as_str()
      {
        "miter" => { line.join_set( line_tools::Join::Miter( 7, 7 ) ); },
        "bevel" => { line.join_set( line_tools::Join::Bevel( 7, 7 ) ); },
        "round" => { line.join_set( line_tools::Join::Round( 16, 8 ) ); },
        _ => {}
      }
    }
  );
  lil_gui::on_change_string( &prop, &callback );
  callback.forget();
}

fn cap_dropdown_setup( gui : &JsValue, object : &JsValue, line : Rc< RefCell< line_tools::d2::Line > > )
{
  let prop = lil_gui::dropdown_add( gui, object, "cap", &serde_wasm_bindgen::to_value( &[ "butt", "square", "round" ] ).unwrap() );
  let callback = Closure::new
  (
    move | value : String |
    {
      gl::info!( "{value:?}" );
      let mut line = line.borrow_mut();
      match value.as_str()
      {
        "butt" => { line.cap_set( line_tools::Cap::Butt ); },
        "square" => { line.cap_set( line_tools::Cap::Square ); },
        "round" => { line.cap_set( line_tools::Cap::Round( 16 ) ); },
        _ => {}
      }
    }
  );
  lil_gui::on_change_string( &prop, &callback );
  callback.forget();
}

fn width_slider_setup( gui : &JsValue, object : &JsValue, line : Rc< RefCell< line_tools::d2::Line > >, gl : gl::GL )
{
  let prop = lil_gui::slider_add( gui, object, "width", 0.0, 500.0, 0.1 );
  let callback = Closure::new
  (
    move | value : f32 |
    {
      line.borrow_mut().mesh_get_mut().upload( &gl, "u_width", &value ).unwrap();
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();
}

fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let main_frag = include_str!( "../shaders/main.frag" );
  let background_frag = include_str!( "../shaders/background.frag" );
  let background_vert = include_str!( "../shaders/background.vert" );

  let background_program = gl::ProgramFromSources::new( background_vert, background_frag ).compile_and_link( &gl )?;

  let world_matrix = gl::math::mat3x3::identity();
  let view_matrix = gl::math::mat3x3::identity();
  let projection_matrix = gl::math::mat3x3h::orthographic_rh_gl( -width / 2.0, width / 2.0, -height / 2.0, height / 2.0, 0.0, 1.0 );
  let line_width = 50.0;

  let mut line = line_tools::d2::Line::default();
  line.cap_set( line_tools::Cap::Butt );
  line.join_set( line_tools::Join::Miter( 7, 7 ) );

  line.mesh_create( &gl, main_frag )?;
  let mesh = line.mesh_get_mut();

  mesh.upload( &gl, "u_projection_matrix", &projection_matrix )?;
  mesh.upload( &gl, "u_world_matrix", &world_matrix )?;
  mesh.upload( &gl, "u_view_matrix", &view_matrix )?;
  mesh.upload( &gl, "u_width", &line_width )?;
  mesh.upload_to( &gl, "body", "u_color", &[ 1.0, 1.0, 1.0 ] )?;
  mesh.upload_to( &gl, "body_terminal", "u_color", &[ 1.0, 1.0, 0.0 ] )?;
  mesh.upload_to( &gl, "join", "u_color", &[ 1.0, 0.0, 0.0 ] )?;
  mesh.upload_to( &gl, "cap", "u_color", &[ 0.0, 1.0, 0.0 ] )?;

  let mut input = browser_input::Input::new
  (
    Some( canvas.clone().dyn_into().unwrap() ),
    browser_input::SCREEN,
  ).expect( "Failed to initialize input" );

  let line = Rc::new( RefCell::new( line ) );

  let settings = Settings
  {
    join : "miter".into(),
    cap : "butt".into(),
    width : line_width
  };

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = lil_gui::new_gui();

  join_dropdown_setup( &gui, &object, line.clone() );
  cap_dropdown_setup( &gui, &object, line.clone() );
  width_slider_setup( &gui, &object, line.clone(), gl.clone() );

  gl.enable( gl::BLEND );
  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let time = t as f32 / 1000.0;

      update( &line, &canvas, &mut input );

      let distance = line.borrow().total_distance_get();

      line.borrow_mut().mesh_get_mut().upload( &gl, "time", &time ).unwrap();
      line.borrow_mut().mesh_get_mut().upload( &gl, "totalDistance", &distance ).unwrap();

      //draw
      gl.use_program( Some( &background_program ) );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );
      line.borrow_mut().draw( &gl ).unwrap();

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );
  Ok( () )
}

fn main()
{
  app_run().unwrap();
}
