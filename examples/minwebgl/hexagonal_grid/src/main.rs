mod drawing;

use minwebgl as gl;
use gl::{ math::d2::mat2x2h, JsCast, canvas::HtmlCanvasElement };
use drawing::{ hex_geometry, LineShader };
use rustc_hash::FxHashMap;
use std::{ collections::hash_map::Iter, marker::PhantomData };

fn main() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;
  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let geometry = hex_geometry( &gl )?;
  let line_shader = LineShader::new( &gl )?;

  gl.clear_color( 0.9, 0.9, 0.9, 1.0 );
  gl.clear( gl::COLOR_BUFFER_BIT );

  let aspect = height / width;
  let total_scale = mat2x2h::scale( [ aspect * 0.2, 1.0 * 0.2 ] );

  let size = 0.5;
  let spacing = 3.0f32.sqrt() * size;
  let total_width = ( 9.0 + 0.5 ) * spacing;
  let total_height = 9.0 * ( 1.5 * size );

  for y in 0..10
  {
    for x in 0..10
    {
      let odd_r = spacing / 2.0 * ( y & 1 ) as f32;
      let x = x as f32;
      let x = odd_r + x * spacing;
      let y = -y as f32;
      let y = y * 1.5 * size;

      let translation = mat2x2h::translate( [ x - total_width * 0.5, y + total_height * 0.5 ] );
      let rotation = mat2x2h::rot( 30.0f32.to_radians() );
      let scale = mat2x2h::scale( [ size, size ] );
      let mvp = total_scale * translation * rotation * scale;

      line_shader.draw( &gl, &geometry, mvp.raw_slice(), [ 0.1, 0.1, 0.1, 1.0 ] )?;
    }
  }

  Ok( () )
}

#[ derive( Debug, Clone, Copy ) ]
pub struct Horizontal;

#[ derive( Debug, Clone, Copy ) ]
pub struct Vertical;

#[ derive( Debug, Clone, Copy ) ]
pub struct OddShifted;

#[ derive( Debug, Clone, Copy ) ]
pub struct EvenShifted;

#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Offset< Layout, Shift >
{
  pub row : i32,
  pub column : i32,
  pub layout : PhantomData< Layout >,
  pub shift : PhantomData< Shift >,
}

impl< Layout, Shift > Offset< Layout, Shift >
{
  pub fn new( row : i32, column : i32, _ : Layout, _ : Shift ) -> Self
  {
    Self
    {
      row,
      column,
      layout : PhantomData,
      shift : PhantomData,
    }
  }

  pub fn from_coords( row : i32, column : i32 ) -> Self
  {
    Self
    {
      row,
      column,
      layout : PhantomData,
      shift : PhantomData,
    }
  }
}

impl From< Axial > for Offset< Horizontal, OddShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q + ( value.r - value.r & 1 ) / 2;
    let row = value.r;
    Self::from_coords( row, col )
  }
}

impl From< Axial > for Offset< Horizontal, EvenShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q + ( value.r + value.r & 1 ) / 2;
    let row = value.r;
    Self::from_coords( row, col )
  }
}

impl From< Axial > for Offset< Vertical, OddShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q - value.q & 1 ) / 2;
    Self::from_coords( row, col )
  }
}

impl From< Axial > for Offset< Vertical, EvenShifted >
{
  fn from( value : Axial ) -> Self
  {
    let col = value.q;
    let row = value.r + ( value.q + value.q & 1 ) / 2;
    Self::from_coords( row, col )
  }
}

#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Axial
{
  pub q : i32,
  pub r : i32,
}

impl From< Offset< Horizontal, OddShifted > > for Axial
{
  fn from( value : Offset< Horizontal, OddShifted > ) -> Self
  {
    let q = value.column - ( value.row - value.row & 1 ) / 2;
    let r = value.row;
    Self { q, r }
  }
}

impl From< Offset< Horizontal, EvenShifted > > for Axial
{
  fn from( value : Offset< Horizontal, EvenShifted > ) -> Self
  {
    let q = value.column - ( value.row + value.row & 1 ) / 2;
    let r = value.row;
    Self { q, r }
  }
}

impl From< Offset< Vertical, OddShifted > > for Axial
{
  fn from( value : Offset< Vertical, OddShifted > ) -> Self
  {
    let q = value.column;
    let r = value.row - ( value.column - value.column & 1 ) / 2;
    Self { q, r }
  }
}

impl From< Offset< Vertical, EvenShifted > > for Axial
{
  fn from( value : Offset< Vertical, EvenShifted > ) -> Self
  {
    let q = value.column;
    let r = value.row - ( value.column + value.column & 1 ) / 2;
    Self { q, r }
  }
}

pub struct GridMap< T >
{
  data : FxHashMap< Axial, T >
}

impl< T > GridMap< T >
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
