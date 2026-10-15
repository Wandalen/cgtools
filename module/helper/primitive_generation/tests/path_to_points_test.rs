//! Integration tests for `primitive_generation::path_to_points`.
//!
//! Covers BUG-127: `kurbo::flatten` also emits `PathEl::ClosePath` for any path
//! that closes a subpath, but `path_to_points`'s flatten callback only matched
//! `MoveTo`/`LineTo` and hit `unreachable!()` on everything else.

#[ cfg( test ) ]
mod tests
{
  use primitive_generation::path_to_points;
  use kurbo::{ PathEl, Point };

  // test_kind: bug_reproducer(BUG-127)
  /// ## Root Cause
  /// `path_to_points` builds a `kurbo::BezPath` from the caller's `Vec<PathEl>`
  /// and flattens it via `kurbo::flatten`, whose callback only matched
  /// `PathEl::MoveTo`/`PathEl::LineTo` and treated every other variant as
  /// `unreachable!()`. `kurbo` 0.13.1's `flatten` (confirmed directly against
  /// its source, `src/bezpath.rs`) always re-emits a trailing `PathEl::ClosePath`
  /// for any subpath that closes -- which every real caller's input does (e.g.
  /// `velato::Geometry::evaluate` on a `Rect`/`Ellipse`, whose `path_elements`
  /// always ends in `ClosePath`), so the `unreachable!()` arm was in fact always
  /// reachable and panicked on any closed path.
  ///
  /// ## Why Not Caught
  /// No existing test called `path_to_points` with a `Vec<PathEl>` that includes
  /// a `ClosePath` element -- the only exercised inputs were open, unclosed
  /// point sequences.
  ///
  /// ## Fix Applied
  /// Changed the flatten closure in `path_to_points` (`src/primitive.rs`) to
  /// explicitly match `PathEl::ClosePath` as a no-op (it carries no coordinate
  /// and the function's flat `Vec<[f32; 2]>` output has no subpath-boundary
  /// marker to attach it to), keeping `unreachable!()` only for variants
  /// `kurbo::flatten` genuinely never emits.
  ///
  /// ## Prevention
  /// Before writing an `unreachable!()` arm over a dependency's output enum,
  /// verify against the dependency's own source which variants it actually
  /// emits -- a local comment or assumption is not proof.
  ///
  /// ## Pitfall
  /// `kurbo::flatten`'s callback receives `ClosePath` for every closed subpath;
  /// any caller building a full point list (not just line/curve endpoints) from
  /// its output must handle it explicitly rather than assuming it can't occur.
  #[ test ]
  fn path_to_points_does_not_panic_on_a_closed_path()
  {
    let path = vec!
    [
      PathEl::MoveTo( Point::new( 0.0, 0.0 ) ),
      PathEl::LineTo( Point::new( 10.0, 0.0 ) ),
      PathEl::LineTo( Point::new( 10.0, 10.0 ) ),
      PathEl::LineTo( Point::new( 0.0, 10.0 ) ),
      PathEl::ClosePath,
    ];

    let points = path_to_points( path );

    assert_eq!
    (
      points,
      vec!
      [
        [ 0.0_f32, 0.0_f32 ],
        [ 10.0_f32, 0.0_f32 ],
        [ 10.0_f32, 10.0_f32 ],
        [ 0.0_f32, 10.0_f32 ],
      ],
      "ClosePath must not panic and must not contribute a spurious point"
    );
  }

  /// Regression guard: an open (never-closed) path -- the case the pre-fix code
  /// already handled -- must still flatten to exactly its MoveTo/LineTo points.
  #[ test ]
  fn path_to_points_accepts_an_open_path()
  {
    let path = vec!
    [
      PathEl::MoveTo( Point::new( 1.0, 2.0 ) ),
      PathEl::LineTo( Point::new( 3.0, 4.0 ) ),
    ];

    let points = path_to_points( path );

    assert_eq!( points, vec![ [ 1.0_f32, 2.0_f32 ], [ 3.0_f32, 4.0_f32 ] ] );
  }
}
