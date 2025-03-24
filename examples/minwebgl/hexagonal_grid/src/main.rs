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

/// Represents a horizontal hexagonal grid layout with odd-row shifting.
/// This layout has "spiky tops" and alternates the horizontal position of hexes
/// in odd rows to create a staggered effect.
pub struct HorizontalOddShifted;

impl HorizontalOddShifted
{
  /// Calculates the horizontal spacing between hexagons in the grid.
  ///
  /// # Parameters
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// The horizontal spacing between hexagons.
  pub fn horizontal_spacing( size : f32 ) -> f32
  {
    3.0f32.sqrt() * size
  }

  /// Calculates the vertical spacing between hexagons in the grid.
  ///
  /// # Parameters
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// The vertical spacing between hexagons.
  pub fn vertical_spacing( size : f32 ) -> f32
  {
    1.5 * size
  }

  /// Calculates the total width and height of the grid based on the number of rows,
  /// columns, and the size of the hexagons.
  ///
  /// # Parameters
  /// - `rows`: The number of rows in the grid.
  /// - `columns`: The number of columns in the grid.
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// A tuple containing the total width and height of the grid.
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

  /// Calculates the position of a hexagon in the grid based on its row, column, and size.
  ///
  /// # Parameters
  /// - `row`: The row index of the hexagon.
  /// - `column`: The column index of the hexagon.
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// A tuple containing the x and y coordinates of the hexagon's position.
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

/// Represents a horizontal hexagonal grid layout with even-row shifting.
/// Similar to `HorizontalOddShifted`, but the horizontal position of hexes
/// is staggered in even rows instead of odd rows.
pub struct HorizontalEvenShifted;

/// Represents a vertical hexagonal grid layout with odd-column shifting.
/// This layout has "flat tops" and alternates the vertical position of hexes
/// in odd columns to create a staggered effect.
pub struct VerticalOddShifted;

/// Represents a vertical hexagonal grid layout with even-column shifting.
/// Similar to `VerticalOddShifted`, but the vertical position of hexes
/// is staggered in even columns instead of odd columns.
pub struct VerticalEvenShifted;

/// Represents an offset coordinate in a hexagonal grid.
/// The `Offset` structure is parameterized by a layout type, which determines
/// the specific hexagonal grid layout (e.g., `HorizontalOddShifted`).
///
/// # Fields
/// - `row`: The row index of the hex.
/// - `column`: The column index of the hex.
/// - `layout`: A marker for the layout type.
#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Offset< Layout >
{
  /// The row index of the hexagon in the grid.
  pub row: i32,
  /// The column index of the hexagon in the grid.
  pub column: i32,
  /// A marker for the layout type of the hexagonal grid.
  pub layout: PhantomData<Layout>,
}

impl< Layout > Offset< Layout >
{
  /// Creates a new `Offset` coordinate with the specified row and column.
  ///
  /// # Parameters
  /// - `row`: The row index of the hexagon.
  /// - `column`: The column index of the hexagon.
  ///
  /// # Returns
  /// A new `Offset` instance.
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

/// Represents an axial coordinate in a hexagonal grid.
/// Axial coordinates use two axes (`q` and `r`) to uniquely identify
/// hexes in a grid.
///
/// # Fields
/// - `q`: The "column" coordinate in the axial system.
/// - `r`: The "row" coordinate in the axial system.
#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Axial
{
  /// The "column" coordinate in the axial coordinate system.
  pub q: i32,
  /// The "row" coordinate in the axial coordinate system.
  pub r: i32,
}

impl Axial
{
  /// Creates a new `Axial` coordinate with the specified `q` and `r` values.
  ///
  /// # Parameters
  /// - `q`: The "column" coordinate in the axial system.
  /// - `r`: The "row" coordinate in the axial system.
  ///
  /// # Returns
  /// A new `Axial` instance.
  pub fn new( q : i32, r : i32 ) -> Self
  {
    Self { q, r }
  }
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

/// A type alias for a hash map that associates axial coordinates with values.
/// This is commonly used to store data for hexagonal grids.
///
/// # Type Parameters
/// - `T`: The type of the values stored in the map.
pub type HexMap< T > = FxHashMap< Axial, T >;
