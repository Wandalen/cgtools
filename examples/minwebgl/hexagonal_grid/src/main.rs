mod webgl_render;
mod layout;
mod coordinates;
mod grid;
mod mesh;
mod patterns;

use layout::*;
use patterns::*;
use minwebgl as gl;
use gl::{ math::d2::mat2x2h, JsCast, canvas::HtmlCanvasElement };
use web_sys::{ wasm_bindgen::prelude::Closure, MouseEvent };
use webgl_render::HexShader;

fn main() -> Result< (), gl::WebglError >
{
  draw_hexes()
}

fn draw_hexes() -> Result< (), minwebgl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();

  // qqq : redundant probably
  let width = 1000;
  let height = 800;
  canvas.set_width( width );
  canvas.set_height( height );

  // qqq : redundant probably
  // remove affection of system scaling on canvas size
  let dpr = web_sys::window().unwrap().device_pixel_ratio();
  let css_width = format!( "{}px", width as f64 / dpr );
  let css_height = format!( "{}px", height as f64 / dpr );
  canvas.style().set_property( "width", &css_width ).unwrap();
  canvas.style().set_property( "height", &css_height ).unwrap();

  gl.viewport( 0, 0, width as i32, height as i32 );
  gl.clear_color( 0.9, 0.9, 0.9, 1.0 );

  // size of a hexagon (from center to vertex)
  let size = 0.9;
  // orientation of the hexagons
  let layout = HexLayout { orientation : Orientation::Pointy, size };
  // how to shift the hexagons to form a rectangle
  let shift_type = ShiftType::Odd;
  // grid size
  let rows = 3;
  let columns = 5;
  // determine the center of the grid
  // to shift it to the center of the canvas
  let ( center_x, center_y ) = layout::grid_center
  (
    ShiftedRectangleIter::new( rows, columns, shift_type, layout ),
    &layout
  );

  let hex_shader = HexShader::new( &gl )?;
  // triangular fan mesh for of a hexagon
  let triangle_geometry = webgl_render::geometry2d( &gl, &mesh::hex_triangle_fan_mesh( &layout ) )?;
  // line loop mesh for the outline of a hexagon
  let line_geometry = webgl_render::geometry2d( &gl, &mesh::hex_line_loop_mesh( &layout ) )?;

  let aspect = height as f32 / width as f32;
  let scaling = [ aspect * 0.2, 1.0 * 0.2 ];
  let total_scale = mat2x2h::scale( scaling );

  let mut selected_hex = None;

  let mouse_move =
  {
    let gl = gl.clone();
    let canvas = canvas.clone();
    move | e : MouseEvent |
    {
      let rect = canvas.get_bounding_client_rect();
      let canvas_x = rect.left();
      let canvas_y = rect.top();

      // transform mouse coordinates from pixels to world coordinates
      // where the center of the canvas is ( 0.0, 0.0 )
      let half_width = ( width as f64 / dpr / 2.0 ) as f32;
      let half_height = ( height as f64 / dpr / 2.0 ) as f32;
      let x = ( e.client_x() as f64 - canvas_x ) as f32;
      let y = ( e.client_y() as f64 - canvas_y ) as f32;
      // normalize then multiply by inverse scaling
      // and offset by center of the grid
      let x = ( x - half_width ) / half_width * ( 1.0 / scaling[ 0 ] ) + center_x;
      let y = ( y - half_height ) / half_height * ( 1.0 / scaling[ 1 ] ) + center_y;

      let cursor_coord = layout.hex_coordinates( x, y );

      // rerender only if the selected hexagon has changed
      if selected_hex.is_some_and( | hex | hex == cursor_coord )
      {
        return;
      }

      selected_hex = Some( cursor_coord );

      gl.clear( gl::COLOR_BUFFER_BIT );

      // draw outline
      // hexagon center in world coords
      let ( x, y ) = layout.hex_2d_position( cursor_coord );
      // offset by center of the grid
      let translation = mat2x2h::translate( [ x - center_x, -y + center_y ] );
      // let scale = mat2x2h::scale( [ size, size ] );
      let mvp = total_scale * translation;
      hex_shader.draw( &gl, gl::LINE_LOOP, &line_geometry, mvp.raw_slice(), [ 0.3, 0.3, 0.3, 1.0 ] ).unwrap();

      // draw hexes
      for coord in ShiftedRectangleIter::new( rows, columns, shift_type, layout )
      {
        // hexagon center in world coords
        let ( x, y ) = layout.hex_2d_position( coord );

        let position = [ x - center_x, -y + center_y ];
        let translation = mat2x2h::translate( position );
        let scale = mat2x2h::scale( [ 0.95, 0.95 ] );
        let mvp = total_scale * translation * scale;
        hex_shader.draw
        (
          &gl,
          gl::TRIANGLE_FAN,
          &triangle_geometry,
          mvp.raw_slice(),
          [ 0.3, 0.75, 0.3, 1.0 ]
        ).unwrap();
      }
    }
  };
  let mouse_move = Closure::< dyn FnMut( _ ) >::new( Box::new( mouse_move ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  Ok( () )
}
