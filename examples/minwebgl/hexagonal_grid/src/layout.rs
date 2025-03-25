/// A trait that defines the layout and positioning of hexagons in a hexagonal grid.
///
/// This trait provides methods and constants to calculate the spacing, dimensions,
/// and positions of hexagons in various grid layouts. It is implemented by specific
/// layout types, such as `PointyOddShifted`, `PointyEvenShifted`, `FlatOddShifted`,
/// and `FlatEvenShifted`.
pub trait HexLayout
{
  /// The rotation angle of the hexagons in the grid, in radians.
  /// This determines the orientation of the hexagons (e.g., "pointy-topped" or "flat-topped").
  const ROTATION_ANGLE : f32;

  /// Calculates the horizontal spacing between hexagons in the grid.
  ///
  /// # Parameters
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// The horizontal spacing between hexagons.
  fn horizontal_spacing( size : f32 ) -> f32;

  /// Calculates the vertical spacing between hexagons in the grid.
  ///
  /// # Parameters
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// The vertical spacing between hexagons.
  fn vertical_spacing( size : f32 ) -> f32;

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
  fn total_distances( rows : i32, columns : i32, size : f32 ) -> ( f32, f32 );

  /// Calculates the position of a hexagon in the grid based on its row, column, and size.
  ///
  /// # Parameters
  /// - `row`: The row index of the hexagon.
  /// - `column`: The column index of the hexagon.
  /// - `size`: The size of the hexagon.
  ///
  /// # Returns
  /// A tuple containing the x and y coordinates of the hexagon's position.
  fn position( row : i32, column : i32, size : f32 ) -> ( f32, f32 );
}

/// Represents a horizontal hexagonal grid layout with odd-row shifting.
/// This layout has "spiky tops" and alternates the horizontal position of hexes
/// in odd rows to create a staggered effect.
pub struct PointyOddShifted;

impl HexLayout for PointyOddShifted
{
  const ROTATION_ANGLE : f32 = 30.0f32.to_radians();

  fn horizontal_spacing( size : f32 ) -> f32
  {
    pointy_layout_spacings( size ).0
  }

  fn vertical_spacing( size : f32 ) -> f32
  {
    pointy_layout_spacings( size ).1
  }

  fn total_distances( rows : i32, columns : i32, size : f32 ) -> ( f32, f32 )
  {
    pointy_layout_distances( rows, columns, size )
  }

  fn position( row : i32, column : i32, size : f32 ) -> ( f32, f32 )
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
pub struct PointyEvenShifted;

impl HexLayout for PointyEvenShifted
{
  const ROTATION_ANGLE : f32 = 30.0f32.to_radians();

  fn horizontal_spacing( size : f32 ) -> f32
  {
    pointy_layout_spacings( size ).0
  }

  fn vertical_spacing( size : f32 ) -> f32
  {
    pointy_layout_spacings( size ).1
  }

  fn total_distances( rows : i32, columns : i32, size : f32 ) -> ( f32, f32 )
  {
    pointy_layout_distances( rows, columns, size )
  }

  fn position( row : i32, column : i32, size : f32 ) -> ( f32, f32 )
  {
    let horizontal_spacing = Self::horizontal_spacing( size );
    let vertical_spacing = Self::vertical_spacing( size );

    let shift = horizontal_spacing / 2.0 * ( row & 1 ^ 1 ) as f32;

    let column = column as f32;
    let x = shift + column * horizontal_spacing;

    let row = -row as f32;
    let y = row * vertical_spacing;

    ( x, y )
  }
}

/// Represents a vertical hexagonal grid layout with odd-column shifting.
/// This layout has "flat tops" and alternates the vertical position of hexes
/// in odd columns to create a staggered effect.
pub struct FlatOddShifted;

impl HexLayout for FlatOddShifted
{
  const ROTATION_ANGLE : f32 = 0.0;

  fn horizontal_spacing( size : f32 ) -> f32
  {
    flat_layout_spacings( size ).0
  }

  fn vertical_spacing( size : f32 ) -> f32
  {
    flat_layout_spacings( size ).1
  }

  fn total_distances( rows : i32, columns : i32, size : f32 ) -> ( f32, f32 )
  {
    flat_layout_distances( rows, columns, size )
  }

  fn position( row : i32, column : i32, size : f32 ) -> ( f32, f32 )
  {
    let horizontal_spacing = Self::horizontal_spacing( size );
    let vertical_spacing = Self::vertical_spacing( size );

    let shift = -vertical_spacing / 2.0 * ( column & 1 ) as f32;

    let column = column as f32;
    let x = column * horizontal_spacing;

    let row = -row as f32;
    let y = shift + row * vertical_spacing;

    ( x, y )
  }
}

/// Represents a vertical hexagonal grid layout with even-column shifting.
/// Similar to `VerticalOddShifted`, but the vertical position of hexes
/// is staggered in even columns instead of odd columns.
pub struct FlatEvenShifted;

impl HexLayout for FlatEvenShifted
{
  const ROTATION_ANGLE : f32 = 0.0;

  fn horizontal_spacing( size : f32 ) -> f32
  {
    flat_layout_spacings( size ).0
  }

  fn vertical_spacing( size : f32 ) -> f32
  {
    flat_layout_spacings( size ).1
  }

  fn total_distances( rows : i32, columns : i32, size : f32 ) -> ( f32, f32 )
  {
    flat_layout_distances( rows, columns, size )
  }

  fn position( row : i32, column : i32, size : f32 ) -> ( f32, f32 )
  {
    let horizontal_spacing = Self::horizontal_spacing( size );
    let vertical_spacing = Self::vertical_spacing( size );

    let shift = -vertical_spacing / 2.0 * ( column & 1 ^ 1 ) as f32;

    let column = column as f32;
    let x = column * horizontal_spacing;

    let row = -row as f32;
    let y = shift + row * vertical_spacing;

    ( x, y )
  }
}

fn pointy_layout_spacings( size : f32 ) -> ( f32, f32 )
{
  ( 3.0f32.sqrt() * size , 1.5 * size )
}

fn pointy_layout_distances( rows : i32, columns : i32, size : f32 ) -> ( f32, f32 )
{
  let ( horizontal_spacing, vertical_spacing ) = pointy_layout_spacings( size );

  let rows = ( rows - 1 ) as f32;
  let columns = ( columns - 1 ) as f32;
  let total_width = ( columns + 0.5 ) * horizontal_spacing;
  let total_height = rows * vertical_spacing;

  ( total_width, total_height )
}

fn flat_layout_spacings( size : f32 ) -> ( f32, f32 )
{
  ( 1.5 * size, 3.0f32.sqrt() * size )
}

fn flat_layout_distances( rows : i32, columns : i32, size : f32 ) -> ( f32, f32 )
{
  let ( horizontal_spacing, vertical_spacing ) = flat_layout_spacings( size );

  let rows = ( rows - 1 ) as f32;
  let columns = ( columns - 1 ) as f32;
  let total_width = columns * horizontal_spacing;
  let total_height = ( rows + 0.5 ) * vertical_spacing;

  ( total_width, total_height )
}
