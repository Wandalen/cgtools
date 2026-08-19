//! # Simple PBR Material Grid
//!
//! Renders a grid of screen-space impostor spheres -- no mesh, just a fullscreen quad and a
//! per-pixel analytic sphere SDF -- each cell varying metallic (rows) and roughness (columns) at
//! a shared, live-adjustable base color. Lit with a three-point (key/fill/rim) setup plus
//! hemisphere ambient, ACES tonemapped. Base color, light intensity, ambient intensity and
//! exposure are controllable live via the lil-gui panel.

use minwebgl as gl;
use gl::GL;

mod lil_gui;
mod gui_setup;

fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let gl = gl::context::retrieve_or_make()?;

  // Vertex and fragment shader source code
  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;

  let resolution_loc = gl.get_uniform_location( &program, "u_resolution" );
  let base_color_loc = gl.get_uniform_location( &program, "u_base_color" );
  let light_intensity_loc = gl.get_uniform_location( &program, "u_light_intensity" );
  let ambient_intensity_loc = gl.get_uniform_location( &program, "u_ambient_intensity" );
  let exposure_loc = gl.get_uniform_location( &program, "u_exposure" );
  let time_loc = gl.get_uniform_location( &program, "u_time" );

  let settings = gui_setup::setup();

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let settings = settings.borrow();

      gl.uniform2f( resolution_loc.as_ref(), width, height );
      gl.uniform3f( base_color_loc.as_ref(), settings.base_color[ 0 ], settings.base_color[ 1 ], settings.base_color[ 2 ] );
      gl.uniform1f( light_intensity_loc.as_ref(), settings.light_intensity );
      gl.uniform1f( ambient_intensity_loc.as_ref(), settings.ambient_intensity );
      gl.uniform1f( exposure_loc.as_ref(), settings.exposure );
      gl.uniform1f( time_loc.as_ref(), t as f32 );

      // Draw points
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
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
