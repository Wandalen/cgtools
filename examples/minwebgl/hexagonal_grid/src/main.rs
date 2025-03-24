mod hex_render;

use minwebgl as gl;
use gl::{ math::d2::mat2x2h, JsCast, canvas::HtmlCanvasElement };
use std::marker::PhantomData;
use web_sys::{ wasm_bindgen::prelude::Closure, MouseEvent };
use hex_render::HexShader;
use rustc_hash::FxHashMap;

fn main() -> Result< (), gl::WebglError >
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
  canvas.style().set_property("width", &css_width).unwrap();
  canvas.style().set_property("height", &css_height).unwrap();

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
  let size = 0.5;

  let ( total_width, total_height ) = HorizontalOddShifted::total_distances( rows, columns, size );
  let mut hex_map = HexMap::default();

  for row in 0..rows
  {
    for column in 0..columns
    {
      let coord = Offset::< HorizontalOddShifted >::new( row, column );
      let ( x, y ) = HorizontalOddShifted::position( row, column, size );
      let position = [ x - total_width * 0.5, y + total_height * 0.5 ];
      hex_map.insert( coord.into(), position );
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
      for ( coord, position ) in hex_map.iter()
      {
        let squared_distance = ( position[ 0 ] - x ).powi( 2 ) + ( position[ 1 ] - y ).powi( 2 );
        if squared_distance < distance
        {
          distance = squared_distance;
          closest = Some( coord );
        }

        let translation = mat2x2h::translate( position );
        let rotation = mat2x2h::rot( 30.0f32.to_radians() );
        let scale = mat2x2h::scale( [ size, size ] );
        let mvp = total_scale * translation * rotation * scale;
        hex_shader.draw( &gl, gl::LINES, &line_geometry, mvp.raw_slice(), [ 0.1, 0.1, 0.1, 1.0 ] ).unwrap();
      }

      // render closest hex with different color
      let position = hex_map.get( closest.unwrap() ).unwrap();
      let translation = mat2x2h::translate( position );
      let rotation = mat2x2h::rot( 30.0f32.to_radians() );
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

#[ derive( Debug, Clone, Copy ) ]
pub struct HorizontalOddShifted;

impl HorizontalOddShifted
{
  pub fn horizontal_spacing( size : f32 ) -> f32
  {
    3.0f32.sqrt() * size
  }

  pub fn vertical_spacing( size : f32 ) -> f32
  {
    1.5 * size
  }

  pub fn total_distances( rows : i32, columns : i32, size : f32 ) -> ( f32, f32 )
  {
    let horizontal_spacing = Self::horizontal_spacing( size );
    let vertical_spacing = Self::vertical_spacing( size );

    let rows = ( rows - 1 ) as f32;
    let columns = ( columns - 1 ) as f32;
    let total_width = ( columns + 0.5 ) * horizontal_spacing;
    let total_height = rows * vertical_spacing;

    ( total_width, total_height )
  }

  pub fn position( row : i32, column : i32, size : f32 ) -> ( f32, f32 )
  {
    let horizontal_spacing = Self::horizontal_spacing( size );
    let vertical_spacing = Self::vertical_spacing( size );

    let shift = horizontal_spacing / 2.0 * ( row & 1 ) as f32;

    let column = column as f32;
    let x = shift + column * horizontal_spacing;

    let row = -row as f32;
    let y = row * vertical_spacing;

    ( x, y )
  }
}

#[ derive( Debug, Clone, Copy ) ]
pub struct HorizontalEvenShifted;

#[ derive( Debug, Clone, Copy ) ]
pub struct VerticalOddShifted;

#[ derive( Debug, Clone, Copy ) ]
pub struct VerticalEvenShifted;

#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Offset< Layout >
{
  pub row : i32,
  pub column : i32,
  pub layout : PhantomData< Layout >,
}

impl< Layout > Offset< Layout >
{
  pub fn new( row : i32, column : i32 ) -> Self
  {
    Self
    {
      row,
      column,
      layout : PhantomData,
    }
  }
}

impl From< Axial > for Offset< HorizontalOddShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q + ( value.r - value.r & 1 ) / 2;
    let row = value.r;
    Self::new( row, col )
  }
}

impl From< Axial > for Offset< HorizontalEvenShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q + ( value.r + value.r & 1 ) / 2;
    let row = value.r;
    Self::new( row, col )
  }
}

impl From< Axial > for Offset< VerticalOddShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q - value.q & 1 ) / 2;
    Self::new( row, col )
  }
}

impl From< Axial > for Offset< VerticalEvenShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q + value.q & 1 ) / 2;
    Self::new( row, col )
  }
}

#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Axial
{
  pub q : i32,
  pub r : i32,
}

impl From< Offset< HorizontalOddShifted > > for Axial
{
  fn from( value : Offset< HorizontalOddShifted > ) -> Self
  {
    let q = value.column - ( value.row - value.row & 1 ) / 2;
    let r = value.row;
    Self { q, r }
  }
}

impl From< Offset< HorizontalEvenShifted > > for Axial
{
  fn from( value : Offset< HorizontalEvenShifted > ) -> Self
  {
    let q = value.column - ( value.row + value.row & 1 ) / 2;
    let r = value.row;
    Self { q, r }
  }
}

impl From< Offset< VerticalOddShifted > > for Axial
{
  fn from( value : Offset< VerticalOddShifted > ) -> Self
  {
    let q = value.column;
    let r = value.row - ( value.column - value.column & 1 ) / 2;
    Self { q, r }
  }
}

impl From< Offset< VerticalEvenShifted > > for Axial
{
  fn from( value : Offset< VerticalEvenShifted > ) -> Self
  {
    let q = value.column;
    let r = value.row - ( value.column + value.column & 1 ) / 2;
    Self { q, r }
  }
}

pub type HexMap< T > = FxHashMap< Axial, T >;
