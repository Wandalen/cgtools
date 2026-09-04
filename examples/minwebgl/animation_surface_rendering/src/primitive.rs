
mod private
{
  use kurbo::PathEl;
  use crate::primitive_data::PrimitiveData;

  /// Delegates to `primitive_generation::curve_to_geometry` — the single source of
  /// truth for this geometry math, including TASK-018's zero-length-segment NaN
  /// guard and BUG-217's winding-independent normal fix — then re-wraps the shared
  /// `AttributesData` into this crate's own `PrimitiveData` ( which carries
  /// `Behavior` where `primitive_generation`'s carries `color` ).
  pub fn curve_to_geometry( curve : &[ [ f32; 2 ] ], width : f32 ) -> Option< PrimitiveData >
  {
    primitive_generation::curve_to_geometry( curve, width )
    .map( | pd | PrimitiveData::new( pd.attributes ) )
  }

  /// Delegates to `primitive_generation::contours_to_fill_geometry` for the same
  /// reason as `curve_to_geometry` above — including TASK-018's fix returning
  /// `None` on a failed triangulation instead of silently dropping that body —
  /// re-wrapping the result into this crate's own `PrimitiveData`.
  pub fn contours_to_fill_geometry( contours : &[ Vec< [ f32; 2 ] > ] ) -> Option< PrimitiveData >
  {
    primitive_generation::contours_to_fill_geometry( contours )
    .map( | pd | PrimitiveData::new( pd.attributes ) )
  }

  /// Converts a vector of 2D points into a `Vec<PathEl>`.
  ///
  /// The first point is converted to a `PathEl::MoveTo` and subsequent points
  /// are converted to `PathEl::LineTo`.
  ///
  /// # Arguments
  ///
  /// * `points` - A `Vec` of 2D points `[ f32; 2 ]`.
  ///
  /// # Returns
  ///
  /// A `Vec<PathEl>` representing the path.
  pub fn points_to_path( points : Vec< [ f32; 2 ] > ) -> Vec< PathEl >
  {
    let mut points = points.into_iter()
    .map
    (
      | [ x, y ] |
      {
        PathEl::LineTo( kurbo::Point::new( f64::from(x), f64::from(y) ) )
      }
    )
    .collect::< Vec< _ > >();

    if let Some( el ) = points.get_mut( 0 )
    {
      if let PathEl::LineTo( p ) = el
      {
        *el = PathEl::MoveTo( *p );
      }
    }

    points
  }

  /// Converts a `Vec<PathEl>` into a flattened vector of 2D points.
  ///
  /// This function uses `kurbo::flatten` to convert a path with curves
  /// into a series of straight line segments. The tolerance for flattening is
  /// set to `0.25`.
  ///
  /// No `primitive_generation` equivalent can be used here: `interpoli` (git,
  /// pinned to rev `04ae4a48` in the root `Cargo.toml`) requires `kurbo ^0.11`,
  /// while `primitive_generation` depends on `kurbo` 0.13 — two incompatible
  /// versions of the same crate produce distinct, non-interchangeable `PathEl`
  /// types, so this stays a local reimplementation rather than a thin delegate.
  ///
  /// # Arguments
  ///
  /// * `path` - A `Vec<PathEl>` representing the path to flatten.
  ///
  /// # Returns
  ///
  /// A `Vec<[f32; 2]>` containing the flattened 2D points of the path.
  //
  // Fix(BUG-127): matches `primitive_generation::path_to_points`'s own fix --
  // `kurbo::flatten` also emits `PathEl::ClosePath` whenever the input path
  // closes a subpath, so treating it as unreachable panics on any closed path.
  // See that function's own doc comment
  // (`module/helper/primitive_generation/src/primitive.rs`) for the full
  // root-cause writeup.
  pub fn path_to_points( path : Vec< PathEl > ) -> Vec< [ f32; 2 ] >
  {
    let mut points = vec![];

    kurbo::flatten
    (
      kurbo::BezPath::from_vec( path ),
      0.25,
      | el |
      {
        match el
        {
          PathEl::MoveTo( p ) | PathEl::LineTo( p ) =>
          {
            points.push( [ p.x as f32, p.y as f32 ] );
          },
          PathEl::ClosePath => {}
          _ => unreachable!( "kurbo::flatten can only return MoveTo, LineTo, and ClosePath PathEls" )
        }
      }
    );

    points
  }
}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  orphan use
  {
    curve_to_geometry,
    contours_to_fill_geometry,
    points_to_path,
    path_to_points,
  };
}
