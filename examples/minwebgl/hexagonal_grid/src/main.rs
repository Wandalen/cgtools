mod hex_render;
mod layout;
mod coordinates;

use layout::*;
// use coordinates::*;
use minwebgl as gl;
use gl::{ math::d2::mat2x2h, JsCast, canvas::HtmlCanvasElement };
use web_sys::{ wasm_bindgen::prelude::Closure, MouseEvent };
use hex_render::HexShader;

fn main() -> Result< (), gl::WebglError >
{
  draw_hexes::< FlatEvenShifted >()
}

fn draw_hexes< Layout : HexLayout >() -> Result< (), minwebgl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();

  let width = 1000;
  let height = 800;
  canvas.set_width( width );
  canvas.set_height( height );

  let dpr = web_sys::window().unwrap().device_pixel_ratio();
  let css_width = format!( "{}px", width as f64 / dpr );
  let css_height = format!( "{}px", height as f64 / dpr );
  canvas.style().set_property( "width", &css_width ).unwrap();
  canvas.style().set_property( "height", &css_height ).unwrap();

  gl.viewport( 0, 0, width as i32, height as i32 );
  gl.clear_color( 0.9, 0.9, 0.9, 1.0 );
  gl.clear( gl::COLOR_BUFFER_BIT );
  let line_geometry = hex_render::hex_lines_geometry( &gl )?;
  let hex_shader = HexShader::new( &gl )?;

  let aspect = height as f32 / width as f32;
  let scaling = [ aspect * 0.2, 1.0 * 0.2 ];
  let total_scale = mat2x2h::scale( scaling );

  let rows = 7;
  let columns = 10;
  let size = 0.7;
  let angle = Layout::ROTATION_ANGLE;
  let ( total_width, total_height ) = Layout::total_distances( rows, columns, size );
  let mut hex_grid = vec![];
  //HexMap::default();

  for row in 0..rows
  {
    for column in 0..columns
    {
      // let coord = Offset::< VerticalOddShifted >::new( row, column );
      let ( x, y ) = Layout::position( row, column, size );
      let position = [ x - total_width * 0.5, y + total_height * 0.5 ];
      // hex_map.insert( coord.into(), position );
      hex_grid.push( position );
    }
  }

  let mouse_move =
  {
    let gl = gl.clone();
    let canvas = canvas.clone();
    move | e : MouseEvent |
    {
      let rect = canvas.get_bounding_client_rect();
      let canvas_x = rect.left();
      let canvas_y = rect.top();

      let half_width = ( width as f64 / dpr / 2.0 ) as f32;
      let half_height = ( height as f64 / dpr / 2.0 ) as f32;
      let x = ( e.client_x() as f64 - canvas_x ) as f32;
      let y = ( e.client_y() as f64 - canvas_y ) as f32;
      // normalize then
      // multiply by inverse scaling
      let x = ( x - half_width ) / half_width * ( 1.0 / scaling[ 0 ] );
      let y = -( y - half_height ) / half_height * ( 1.0 / scaling[ 1 ] );

      gl.clear( gl::COLOR_BUFFER_BIT );

      let mut distance = f32::INFINITY;
      let mut closest = None;
      for ( i, position ) in hex_grid.iter().enumerate()
      {
        let squared_distance = ( position[ 0 ] - x ).powi( 2 ) + ( position[ 1 ] - y ).powi( 2 );
        if squared_distance < distance
        {
          distance = squared_distance;
          closest = Some( i );
        }

        let translation = mat2x2h::translate( position );
        let rotation = mat2x2h::rot( angle );
        let scale = mat2x2h::scale( [ size, size ] );
        let mvp = total_scale * translation * rotation * scale;
        hex_shader.draw( &gl, gl::LINES, &line_geometry, mvp.raw_slice(), [ 0.1, 0.1, 0.1, 1.0 ] ).unwrap();
      }

      // render closest hex with different color
      let position = hex_grid.get( closest.unwrap() ).unwrap();
      let translation = mat2x2h::translate( position );
      let rotation = mat2x2h::rot( angle );
      let scale = mat2x2h::scale( [ size, size ] );
      let mvp = total_scale * translation * rotation * scale;
      hex_shader.draw( &gl, gl::LINES, &line_geometry, mvp.raw_slice(), [ 0.3, 0.75, 0.3, 1.0 ] ).unwrap();
    }
  };
  let mouse_move = Closure::< dyn Fn( _ ) >::new( Box::new( mouse_move ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  Ok( () )
}
