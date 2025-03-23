mod hex_render;

use minwebgl as gl;
use gl::{ math::d2::mat2x2h, JsCast, canvas::HtmlCanvasElement };
use hex_render::LineShader;
use rustc_hash::FxHashMap;
use std::{ collections::hash_map::Iter, marker::PhantomData };

fn main() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;
  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let geometry = hex_render::hex_lines_geometry( &gl )?;
  let line_shader = LineShader::new( &gl )?;

  gl.clear_color( 0.9, 0.9, 0.9, 1.0 );
  gl.clear( gl::COLOR_BUFFER_BIT );

  let aspect = height / width;
  let total_scale = mat2x2h::scale( [ aspect * 0.2, 1.0 * 0.2 ] );

  let rows = 7;
  let columns = 10;
  let size = 0.5;

  let ( total_width, total_height ) = HorizontalOddShifted::total_distances( rows, columns, size );
  let mut hex_map = HexMap::new();

  for row in 0..rows
  {
    for column in 0..columns
    {
      let coord = Offset::< HorizontalOddShifted >::new( row, column );
      let ( x, y ) = HorizontalOddShifted::position( row, column, size );
      let position = [ x - total_width * 0.5, y + total_height * 0.5 ];
      hex_map.insert( coord, position );
    }
  }

  for ( _, position ) in hex_map.iter()
  {
    let translation = mat2x2h::translate( position );
    let rotation = mat2x2h::rot( 30.0f32.to_radians() );
    let scale = mat2x2h::scale( [ size, size ] );
    let mvp = total_scale * translation * rotation * scale;
    line_shader.draw( &gl, gl::LINES, &geometry, mvp.raw_slice(), [ 0.1, 0.1, 0.1, 1.0 ] )?;
  }

  Ok( () )
}

#[ derive( Debug, Clone, Copy ) ]
pub struct HorizontalOddShifted;

impl HorizontalOddShifted
{
  pub fn total_distances( rows : i32, columns : i32, size : f32 ) -> ( f32, f32 )
  {
    // horizontal distance between neighbor hexes
    let spacing = 3.0f32.sqrt() * size;

    let rows = ( rows - 1 ) as f32;
    let columns = ( columns - 1 ) as f32;
    let total_width = ( columns + 0.5 ) * spacing;
    let total_height = rows * ( 1.5 * size );

    ( total_width, total_height )
  }

  pub fn position( row : i32, column : i32, size : f32 ) -> ( f32, f32 )
  {
    // horizontal distance between neighbor hexes
    let spacing = 3.0f32.sqrt() * size;

    let shift = spacing / 2.0 * ( row & 1 ) as f32;

    let column = column as f32;
    let x = shift + column * spacing;

    let row = -row as f32;
    let y = row * 1.5 * size;

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

pub struct HexMap< T >
{
  data : FxHashMap< Axial, T >
}

impl< T > HexMap< T >
{
  pub fn new() -> Self
  {
    Self { data : Default::default() }
  }

  pub fn insert< C : Into< Axial > >( &mut self, coord : C, val : T ) -> Option< T >
  {
    self.data.insert( coord.into(), val )
  }

  pub fn remove< C : Into< Axial > >( &mut self, coord : C ) -> Option< T >
  {
    self.data.remove( &coord.into() )
  }

  pub fn iter( &self ) -> Iter< Axial, T >
  {
    self.data.iter()
  }
}
