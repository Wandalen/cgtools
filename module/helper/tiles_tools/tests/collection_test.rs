//! Tests for the `collection` module — `Grid2D` insert/get/remove round-trips and
//! out-of-bounds panic behavior (both directions), driven purely through the
//! public surface.

#![ cfg( feature = "enabled" ) ]

use tiles_tools::collection::Grid2D;
use tiles_tools::coordinates::hexagonal::{ Coordinate as HexCoord, Axial, Pointy };

fn small_grid() -> Grid2D< Axial, Pointy, Option< i32 > >
{
  // q in [0, 3), r in [0, 3) -- a 3x3 grid.
  Grid2D::< Axial, Pointy, Option< i32 > >::with_size_and_fn(
    HexCoord::< Axial, Pointy >::new( 0, 0 ),
    HexCoord::< Axial, Pointy >::new( 3, 3 ),
    || None,
  )
}

#[ test ]
fn test_grid2d_insert_get_remove_roundtrip()
{
  let mut grid = small_grid();
  let coord = HexCoord::< Axial, Pointy >::new( 1, 1 );

  assert_eq!( grid.get( coord ), None );

  let previous = grid.insert( coord, 42 );
  assert_eq!( previous, None );
  assert_eq!( grid.get( coord ), Some( &42 ) );
  assert_eq!( grid[ coord ], Some( 42 ) );

  let removed = grid.remove( coord );
  assert_eq!( removed, Some( 42 ) );
  assert_eq!( grid.get( coord ), None );
}

// test_kind: UX/DX -- bounds-check message consistency (no BUG-NNN; see
// task/completed/ sweep report for this crate).
/// ## What changed
/// `Grid2D::insert`/`remove`/`Index`/`IndexMut` used to panic with a custom
/// `"Coordinate out of bound"` message for a *negative*-offset out-of-bounds
/// coordinate (the `i64 -> usize` conversion failing), but with ndarray's own,
/// differently-worded internal message for a *positive*-offset out-of-bounds
/// coordinate (the conversion succeeding, then `ndarray`'s own bounds check
/// panicking later) -- the same logical mistake produced two different
/// panic messages depending on which side of the grid the caller missed.
/// These two tests pin that both directions now panic with the identical,
/// explicit `"Coordinate out of bound"` message via the new shared
/// `Grid2D::grid_index` bounds check.
#[ test ]
#[ should_panic( expected = "Coordinate out of bound" ) ]
fn test_grid2d_index_negative_offset_panics_with_consistent_message()
{
  let grid = small_grid();
  let _ = grid[ HexCoord::< Axial, Pointy >::new( -1, 0 ) ];
}

#[ test ]
#[ should_panic( expected = "Coordinate out of bound" ) ]
fn test_grid2d_index_positive_above_bounds_panics_with_consistent_message()
{
  let grid = small_grid();
  let _ = grid[ HexCoord::< Axial, Pointy >::new( 10, 0 ) ];
}

#[ test ]
#[ should_panic( expected = "Coordinate out of bound" ) ]
fn test_grid2d_insert_positive_above_bounds_panics_with_consistent_message()
{
  let mut grid = small_grid();
  grid.insert( HexCoord::< Axial, Pointy >::new( 0, 10 ), 1 );
}

#[ test ]
fn test_grid2d_get_out_of_bounds_returns_none_both_directions()
{
  let grid = small_grid();
  // `get`/`get_mut` were already consistent (graceful `None` both
  // directions) -- pinned here as a contrast to the panicking APIs above.
  assert_eq!( grid.get( HexCoord::< Axial, Pointy >::new( -1, 0 ) ), None );
  assert_eq!( grid.get( HexCoord::< Axial, Pointy >::new( 10, 0 ) ), None );
}
