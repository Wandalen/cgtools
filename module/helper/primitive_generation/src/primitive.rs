//! This module store functions and structures for creating `PrimitiveData`
//! of different abstactions like curves.

mod private
{
  use minwebgl as gl;
  use gl::{ F32x2, F32x4 };
  use std::cell::RefCell;
  use std::rc::Rc;
  use crate::
  {
    AttributesData,
    PrimitiveData,
    Transform
  };

  #[ cfg( feature = "font-processing" ) ]
  use gl::{ geometry::BoundingBox, F32x3 };

  #[ cfg( feature = "text" ) ]
  use kurbo::PathEl;

  /// Converts a `&[ [ f32; 2 ] ]` into `GeometryData` representing its 2D outline
  /// as a series of rectangles, each with a specified `width`.
  ///
  /// This function flattens the Bezier path into many small line segments.
  /// For each segment, it constructs a rectangle of the given `width` centered
  /// on the segment. These rectangles are then triangulated (into two triangles)
  /// and added to the `GeometryData`.
  ///
  /// # Arguments
  ///
  /// * `curve` - The `&[ [ f32; 2 ] ]` to convert into a thick geometry.
  /// * `width` - The desired thickness of the stroked path.
  ///
  /// # Returns
  ///
  /// A `GeometryData` struct containing the 3D vertex positions and triangle indices
  /// that form the rectangular segments of the path. The Z-coordinate is always 0.0.
  #[ must_use ]
  pub fn curve_to_geometry( curve : &[ [ f32; 2 ] ], width : f32 ) -> Option< PrimitiveData >
  {
    let first = *curve.first()?;
    let last = *curve.last()?;
    let mut start_point = F32x2::from_array( first );

    // Fix(TASK-018): reject a curve containing a zero-length segment (two
    // consecutive points -- including the implicit closing segment back to
    // the first point -- that coincide) before any geometry math runs.
    // Root cause: `add_segment` (below) computes
    // `direction = ( end_point - start_point ).normalize()` for every
    // segment, including the closing one, with no check that the two points
    // differ. `F32x2::normalize()` divides each component by the vector's
    // magnitude with no zero-length guard, so a zero-length segment (e.g. a
    // single-point curve, whose only segment closes onto itself) silently
    // computes `0.0 / 0.0`, i.e. `NaN`, instead of failing.
    // Pitfall: any code that normalizes a difference vector must validate
    // that the two inputs actually differ first -- `normalize()` has no
    // zero-length guard anywhere in the underlying vector math stack.
    #[ expect( clippy::float_cmp, reason = "coincident-point detection needs exact equality : normalize() yields NaN precisely when two consecutive points are bit-identical; an epsilon test would wrongly reject valid short segments" ) ]
    if curve.windows( 2 ).any( | pair | pair[ 0 ] == pair[ 1 ] ) || curve.first() == curve.last()
    {
      return None;
    }

    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();

    let half_width = width / 2.0;

    // Fix(BUG-217)
    // Root cause: see `AttributesData::normals`'s doc comment. This
    // function's winding is direction-independent: for any unit direction
    // `(dx,dy)`, the segment quad's face normal ( computed via the cross
    // product of its own edges ) algebraically reduces to
    // `(0, 0, -2 * half_width * segment_length)`, always negative for a
    // non-degenerate segment ( `half_width`, `segment_length` > 0 ) --
    // so every vertex this function emits gets the same constant
    // `(0, 0, -1)` normal.
    let mut add_segment =
    | end_point : &F32x2, start_point : &F32x2 |
    {
      let direction = ( *end_point - *start_point ).normalize();

      let normal = F32x2::new( -direction.y(), direction.x() );

      let p0 = *start_point - normal * half_width;
      let p1 = *start_point + normal * half_width;
      let p2 = *end_point + normal * half_width;
      let p3 = *end_point - normal * half_width;

      let base_idx = positions.len() as u32;

      positions.push( [ p0.x(), p0.y(), 0.0 ] );
      positions.push( [ p1.x(), p1.y(), 0.0 ] );
      positions.push( [ p2.x(), p2.y(), 0.0 ] );
      positions.push( [ p3.x(), p3.y(), 0.0 ] );

      normals.extend( [ [ 0.0, 0.0, -1.0 ]; 4 ] );

      indices.push( base_idx );
      indices.push( base_idx + 1 );
      indices.push( base_idx + 2 );

      indices.push( base_idx );
      indices.push( base_idx + 2 );
      indices.push( base_idx + 3 );
    };

    let mut i = 1;
    while i < curve.len()
    {
      let end_point = F32x2::from_array( curve[ i ] );
      add_segment( &end_point, &start_point );
      start_point = end_point;
      i += 1;
    }

    start_point = last.into();
    let end_point = F32x2::from_array( first );
    add_segment( &end_point, &start_point );

    let attributes = AttributesData
    {
      positions,
      indices,
      normals
    };

    Some(
      PrimitiveData
      {
        name : None,
        parent : None,
        attributes : Some( Rc::new( RefCell::new( attributes ) ) ),
        color : F32x4::default(),
        transform: Transform::default(),
      }
    )
  }

  /// Computes the 2D bounding box of one contour's points.
  #[ cfg( feature = "font-processing" ) ]
  fn contour_bounding_box( contour : &[ [ f32; 2 ] ] ) -> BoundingBox
  {
    BoundingBox::compute2d
    (
      contour
      .iter()
      .flatten()
      .copied()
      .collect::< Vec< _ > >()
      .as_slice()
    )
  }

  /// Returns the index of the contour with the largest bounding-box diagonal.
  #[ cfg( feature = "font-processing" ) ]
  fn largest_contour_index( contours : &[ Vec< [ f32; 2 ] > ] ) -> usize
  {
    let mut body_id = 0;
    let mut max_box_diagonal_size = 0.0;
    for ( i, contour ) in contours.iter().enumerate()
    {
      if contour.is_empty()
      {
        continue;
      }
      let ( mut x1, mut y1, mut x2, mut y2 ) = ( f32::MAX, f32::MAX, f32::MIN, f32::MIN );
      for [ x, y ] in contour
      {
        x1 = x1.min( *x );
        y1 = y1.min( *y );
        x2 = x2.max( *x );
        y2 = y2.max( *y );
      }
      let contour_size = ( ( x2 - x1 ).powi( 2 ) + ( y2 - y1 ).powi( 2 ) ).sqrt();
      if max_box_diagonal_size < contour_size
      {
        max_box_diagonal_size = contour_size;
        body_id = i;
      }
    }
    body_id
  }

  /// Flattens a body's outer contour and holes into earcut-ready interleaved
  /// coordinates, returning the flat positions and hole start indices.
  /// Returns `None` when the outer contour is missing or empty.
  #[ cfg( feature = "font-processing" ) ]
  fn with_holes_flatten( body : &[ Vec< [ f32; 2 ] > ] ) -> Option< ( Vec< f64 >, Vec< usize > ) >
  {
    let mut flat_positions : Vec< f64 > = Vec::new();
    let mut hole_indices : Vec< usize > = Vec::new();

    let outer_contour = body.first()?;
    if outer_contour.is_empty()
    {
      return None;
    }
    for &[ x, y ] in outer_contour
    {
      flat_positions.push( f64::from( x ) );
      flat_positions.push( f64::from( y ) );
    }

    // Holes' winding order must be opposite to the outer ( e.g., CW for holes )
    for hole_contour in body.iter().skip( 1 )
    {
      if hole_contour.is_empty()
      {
        continue;
      }
      hole_indices.push( flat_positions.len() / 2 );
      for &[ x, y ] in hole_contour
      {
        flat_positions.push( f64::from( x ) );
        flat_positions.push( f64::from( y ) );
      }
    }

    Some( ( flat_positions, hole_indices ) )
  }

  /// Converts a set of 2D contours into a solid flat 3D primitive suitable for rendering.
  ///
  /// The function first identifies the largest contour, which it assumes is the
  /// main body of the shape. It then classifies other contours as either holes
  /// (if they are fully contained within the main body's bounding box) or as
  /// separate bodies. Finally, it uses a triangulation algorithm (`earcutr`) to
  /// generate a filled mesh with positions and indices.
  ///
  /// # Arguments
  ///
  /// * `contours`: A slice of vectors, where each inner vector is a contour
  ///   represented by an array of `[f32; 2]` points.
  ///
  /// # Returns
  ///
  /// Returns `Some( PrimitiveData )` on success, containing the generated mesh
  /// data. Returns `None` if the input `contours` is empty or if the
  /// triangulation process fails.
  //
  // Fix(TASK-055): gated behind `font-processing`, matching `path_to_points`'s
  // existing `#[cfg(feature = "text")]` gate below.
  // Root cause: this function unconditionally called `earcutr::earcut(..)`, but
  // `earcutr` is an optional dependency only pulled in by `font-processing` --
  // any build with default features alone (`enabled` only, no `text`/
  // `font-processing`) failed with E0433 "cannot find crate earcutr".
  // Pitfall: a function using an optional-dependency's API must be gated behind
  // the same feature that activates that dependency -- the `use`/`dep:` line
  // being correctly gated is not enough if the call site itself is not.
  #[ cfg( feature = "font-processing" ) ]
  #[ must_use ]
  pub fn contours_to_fill_geometry( contours : &[ Vec< [ f32; 2 ] > ] ) -> Option< PrimitiveData >
  {
    if contours.is_empty()
    {
      return None;
    }

    let body_id = largest_contour_index( contours );
    let body_contour = contours.get( body_id )?;
    let body_bounding_box = contour_bounding_box( body_contour );

    let mut outside_body_list = vec![];
    let mut inside_body_list = vec![];
    for ( i, contour ) in contours.iter().enumerate()
    {
      if body_id == i
      {
        continue;
      }

      let bounding_box = contour_bounding_box( contour );

      let has_part_outside_body = bounding_box.left() < body_bounding_box.left() ||
      bounding_box.right() > body_bounding_box.right() ||
      bounding_box.up() > body_bounding_box.up() ||
      bounding_box.down() < body_bounding_box.down();

      if has_part_outside_body
      {
        outside_body_list.push( contour.clone() );
      }
      else
      {
        inside_body_list.push( contour.clone() );
      }
    }

    let mut base = vec![ body_contour.clone() ];
    base.extend( inside_body_list );

    let mut bodies = vec![ base ];
    bodies.extend( outside_body_list.into_iter().map( | c | vec![ c ] ) );

    let mut positions = vec![];
    let mut indices = vec![];
    let mut normals = vec![];

    for contours in bodies
    {
      let ( flat_positions, hole_indices ) = with_holes_flatten( &contours )?;

      // Fix(TASK-018): return None on triangulation failure instead of
      // silently skipping the failed body and continuing with whatever
      // other bodies happened to succeed.
      // Root cause: the doc comment above promises `Returns None ... if the
      // triangulation process fails`, but this `let-else` only ever
      // `continue`d past a failed body, so the function kept going and
      // returned `Some( PrimitiveData )` -- with that body's geometry
      // silently missing -- regardless of the documented failure contract.
      // Pitfall: a `let-else` failure branch inside a `for` loop reads as
      // "handle this item's failure and move on," which silently downgrades
      // a documented hard failure into a partial success -- always check
      // the doc comment's stated contract before choosing `continue` over
      // `return`.
      // Perform triangulation
      let Ok( body_indices ) = earcutr::earcut( &flat_positions, &hole_indices, 2 )
      else
      {
        return None;
      };

      let body_indices = body_indices.into_iter()
      .map( | i | i as u32 )
      .collect::< Vec< _ > >();

      let body_positions = flat_positions.chunks_exact( 2 )
      .map( | c | [ c[ 0 ] as f32, c[ 1 ] as f32, 0.0 ] )
      .collect::< Vec< _ > >();

      // Fix(BUG-217)
      // Root cause: see `AttributesData::normals`'s doc comment. Unlike
      // `plane_to_geometry`/`curve_to_geometry`, this function's winding
      // depends on the caller-supplied contour's own orientation --
      // `earcutr` preserves, never reorders, the input winding -- so the
      // normal is derived from this body's own first triangle rather than
      // assumed, then broadcast to every vertex in the body since the
      // whole body is coplanar ( Z = 0 ) and `earcutr` keeps one
      // consistent winding throughout a single body. Falls back to
      // `(0,0,1)` both when a body triangulates to zero triangles and
      // when the first triangle is degenerate ( zero-area, e.g. from
      // coincident input points ) -- the latter guards against
      // `normalize` of a zero vector reintroducing the exact NaN defect
      // this fix removes, one call site later.
      let body_normal = body_indices.chunks_exact( 3 ).next().map
      (
        | tri |
        {
          let p0 : F32x3 = body_positions[ tri[ 0 ] as usize ].into();
          let p1 : F32x3 = body_positions[ tri[ 1 ] as usize ].into();
          let p2 : F32x3 = body_positions[ tri[ 2 ] as usize ].into();
          ( p1 - p0 ).cross( p2 - p0 )
        }
      )
      .filter( | raw | *raw != F32x3::new( 0.0, 0.0, 0.0 ) )
      .map_or( F32x3::new( 0.0, 0.0, 1.0 ), F32x3::normalize )
      .0;

      let positions_count = positions.len();
      normals.extend( std::iter::repeat( body_normal ).take( body_positions.len() ) );
      positions.extend( body_positions );
      indices.extend
      (
        body_indices.iter()
        .map( | i | i + positions_count as u32 )
      );
    }

    let attributes = AttributesData
    {
      positions,
      indices,
      normals,
    };

    let primitive_data = PrimitiveData
    {
      name : None,
      parent : None,
      attributes : Some( Rc::new( RefCell::new( attributes ) ) ),
      color : F32x4::default(),
      transform : Transform::default()
    };

    Some( primitive_data )
  }

  /// Creates a unit plane (1×1) centered at the origin.
  ///
  /// Use `primitive.transform.scale = F32x3::new(width, height, 1.0);`
  /// to adjust its size later.
  ///
  /// The plane lies in the XY plane (Z = 0).
  #[ must_use ]
  pub fn plane_to_geometry() -> Option< PrimitiveData >
  {
    let positions =
    vec![
      [ -0.5, -0.5, 0.0 ],
      [ 0.5, -0.5, 0.0 ],
      [ 0.5,  0.5, 0.0 ],
      [ -0.5,  0.5, 0.0 ]
    ];

    let indices =
    vec![
      0, 1, 2,
      0, 2, 3
    ];

    // Fix(BUG-217): see `AttributesData::normals`'s doc comment. This
    // quad's own winding ( `0,1,2` = CCW as seen from +Z, confirmed by
    // direct cross product of its own edges ) gives a face normal of
    // `(0,0,1)` via the right-hand rule.
    let normals = vec![ [ 0.0, 0.0, 1.0 ]; 4 ];

    let attributes = AttributesData
    {
      positions,
      indices,
      normals
    };

    Some
    (
      PrimitiveData
      {
        name : None,
        parent : None,
        attributes : Some( Rc::new( RefCell::new( attributes ) ) ),
        color : F32x4::default(),
        transform : Transform::default(),
      }
    )
  }

  /// Converts a `Vec<PathEl>` into a flattened vector of 2D points.
  ///
  /// This function uses `kurbo::flatten` to convert a path with curves
  /// into a series of straight line segments. The tolerance for flattening is
  /// set to `0.25`.
  ///
  /// # Arguments
  ///
  /// * `path` - A `Vec<PathEl>` representing the path to flatten.
  ///
  /// # Returns
  ///
  /// A `Vec<[f32; 2]>` containing the flattened 2D points of the path.
  // Fix(BUG-127)
  // Root cause: `kurbo::flatten` also emits `PathEl::ClosePath` (confirmed directly
  // against `kurbo` 0.13.1's `flatten` source) whenever the input path closes a
  // subpath, but this closure only matched `MoveTo`/`LineTo` and treated every other
  // variant -- including the always-reachable `ClosePath` -- as `unreachable!()`.
  // Pitfall: a local assumption about what a dependency "can only return" is not
  // proof; verify against the dependency's own source before writing an
  // `unreachable!()` arm over its output.
  #[ cfg( feature = "text" ) ]
  #[ must_use ]
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

crate::mod_interface!
{
  orphan use
  {
    curve_to_geometry,
    plane_to_geometry
  };

  #[ cfg( feature = "font-processing" ) ]
  orphan use
  {
    contours_to_fill_geometry
  };

  #[ cfg( feature = "text" ) ]
  orphan use
  {
    path_to_points
  };
}
