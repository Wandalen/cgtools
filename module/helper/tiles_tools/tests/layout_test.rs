//! Tests for the `layout` module — `RectangularGrid` bounds/center calculation
//! driven purely through the public surface.

#![ cfg( feature = "enabled" ) ]

use tiles_tools::layout::RectangularGrid;
use tiles_tools::coordinates::hexagonal::{ Coordinate, Offset, Odd, Flat };

// test_kind: bug_reproducer(BUG-131)
/// ## Root Cause
/// `RectangularGrid<Parity, Flat>::center()` was copy-pasted from the sibling
/// `Pointy` impl without updating which axis its min/max guard tests. Pointy's
/// pixel-x depends on `r`'s parity, so its guard correctly tests `min.r<max.r`;
/// Flat's pixel-y depends on `q`'s parity (the candidate point already varies
/// `q`, via `min.q + 1`), so the guard must test `min.q<max.q` -- it tested
/// `min.r<max.r` instead, a leftover from the Pointy impl.
///
/// ## Why Not Caught
/// `RectangularGrid` had no dedicated test file before this one; the only real
/// caller (`examples/minwebgl/hexagonal_grid`) only instantiates the `Pointy`
/// orientation, never `Flat`.
///
/// ## Fix Applied
/// `layout.rs`'s `Flat` impl now guards on `min.q<max.q` / `max.q>min.q`,
/// matching the axis the candidate points themselves already vary.
///
/// ## Prevention
/// When two sibling impls share near-identical structure differing only in
/// which axis is parity-dependent, verify every field reference against the
/// orientation's own parity rule after copy-pasting -- a wrong-axis guard
/// compiles cleanly and only shows up as a silently wrong numeric result.
///
/// ## Pitfall
/// Invisible whenever the bounds span only one column (`min.q == max.q`) --
/// both the buggy and fixed guards degenerate to the same `else` branch. Only
/// a multi-column `Flat` bounds range exposes the wrong axis.
#[ expect( clippy::float_cmp, reason = "1.5 == 3/2 is exactly representable in f32; pins the exact stored value" ) ]
#[ test ]
fn flat_center_accounts_for_the_shifted_middle_column()
{
  let bounds =
  [
    Coordinate::< Offset< Odd >, Flat >::new( 0, 0 ),
    Coordinate::< Offset< Odd >, Flat >::new( 2, 0 ),
  ];
  let grid = RectangularGrid::< Odd, Flat >::new( bounds );

  let center = grid.center();

  assert_eq!( center[ 0 ], 1.5, "x is unaffected by this guard -- sanity check only" );
  assert!(
    ( center[ 1 ] - 0.433_012_7 ).abs() < 0.0001,
    "column q=1 (odd, between the two even-parity corners q=0/q=2) sits at a \
     shifted y (~0.866) that the true bounding range must account for -- the \
     center's y must be their true midpoint (~0.433), not q=0's own y alone \
     (0.0, what the buggy r-based guard would produce since min.r==max.r here \
     and never notices the q range spans a different-parity column) \
     -- got {}", center[ 1 ]
  );
}
