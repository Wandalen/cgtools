use minwebgl::{ self as gl, IntoArray };
use std::
{
  cell::RefCell,
  rc::Rc,
};
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;

mod lil_gui;

#[ derive( Default, Serialize, Deserialize ) ]
struct Settings
{
  join : String,
  cap : String,
  width : f32
}

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let main_frag = include_str!( "../shaders/main.frag" );
  let background_frag = include_str!( "../shaders/background.frag" );
  let background_vert = include_str!( "../shaders/background.vert" );

  let background_program = gl::ProgramFromSources::new( background_vert, background_frag ).compile_and_link( &gl )?;

  let world_matrix = gl::math::mat3x3::identity();
  let projection_matrix = gl::math::mat3x3h::orthographic_rh_gl( -width / 2.0, width / 2.0, -height / 2.0, height / 2.0, 0.0, 1.0 );
  let line_width = 2.0;

  let radius = 300.0;

  let mut line = line_tools::d2::Line::default();
  line.join = line_tools::Join::Miter;
  line.cap = line_tools::Cap::Square;
  line.create_mesh( &gl, main_frag )?;
  let mesh = line.get_mesh();
  
  mesh.upload_matrix( &gl, "u_projection_matrix", &projection_matrix.to_array() )?;
  mesh.upload_matrix( &gl, "u_world_matrix", &world_matrix.to_array() )?;
  mesh.upload( &gl, "u_width", &line_width )?;

  let line = Rc::new( RefCell::new( line ) );

   let settings = Settings
  {
    join : "miter".into(),
    cap : "square".into(),
    width : line_width
  };

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = lil_gui::new_gui();

  // Joins
  let prop = lil_gui::add_dropdown( &gui, &object, "join", &serde_wasm_bindgen::to_value( &[ "miter", "bevel", "round" ] ).unwrap() );
  let callback = Closure::new
  (
    {
      let line = line.clone();
      move | value : String |
      {
        gl::info!( "{:?}", value );
        let mut line = line.borrow_mut();
        match value.as_str()
        {
          "miter" => { line.set_join( line_tools::Join::Miter ); },
          "bevel" => { line.set_join( line_tools::Join::Bevel ); },
          "round" => { line.set_join( line_tools::Join::Round( 16 ) ); },
          _ => {}
        }
      }
    }
  );
  lil_gui::on_change_string( &prop, &callback );
  callback.forget();

  // Caps
  let prop = lil_gui::add_dropdown( &gui, &object, "cap", &serde_wasm_bindgen::to_value( &[ "butt", "square", "round" ] ).unwrap() );
  let callback = Closure::new
  (
    {
      let line = line.clone();
      move | value : String |
      {
        gl::info!( "{:?}", value );
        let mut line = line.borrow_mut();
        match value.as_str()
        {
          "butt" => { line.set_cap( line_tools::Cap::Butt ); },
          "square" => { line.set_cap( line_tools::Cap::Square ); },
          "round" => { line.set_cap( line_tools::Cap::Round( 16 ) ); },
          _ => {}
        }
      }
    }
  );
  lil_gui::on_change_string( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_slider( &gui, &object, "width", 0.0, 100.0, 0.1 );
  let callback = Closure::new
  (
    {
      let line = line.clone();
      let gl = gl.clone();
      move | value : f32 |
      {
        line.borrow().get_mesh().upload( &gl, "u_width", &value ).unwrap();
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();
  
  // Define the update and draw logic
  let update_and_draw =
  {
    let add_interval = 0.2;
    let mut elapsed_time = 0.0;
    move | t : f64 |
    {
      let time = t as f32 / 1000.0;

      let x_freq = ( time / 10.0 ).sin() * 3.0;
      let y_freq = ( time / 10.0 ).cos() * 3.0;

      let x_offset = 0.0;
      let y_offset = 0.0;

      let x = ( x_freq * time + x_offset ).cos() * radius; 
      let y = ( y_freq * time + y_offset ).sin() * radius; 

      if elapsed_time > add_interval
      {
        line.borrow_mut().add_point( gl::F32x2::new( x, y ) );
        elapsed_time = elapsed_time - add_interval;
      }

      //draw
      gl.use_program( Some( &background_program ) );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );
      line.borrow_mut().draw( &gl ).unwrap();

      elapsed_time += time;

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
